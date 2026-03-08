use std::{convert::Infallible, pin::Pin};

use anyhow::Result as AnyhowResult;
use axum::{
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use core_types::{
    agent::{AgentConfig, AgentStep, Trigger},
    tool::supported_tool_schemas,
};
use futures_util::{Stream, StreamExt};
use llm_client::{
    openai::OpenAiProvider,
    pii_scrubber::scrub_pii,
    provider::{ChatMessage, LlmProvider},
};
use schemars::schema_for;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolIntent {
    Read,
    Write,
}

/// Request body for the agent builder endpoint.
#[derive(Debug, Deserialize)]
pub struct BuildRequest {
    pub description: String,
}

/// POST /agents/build
///
/// Accepts a plain-English workflow description, scrubs PII, generates a valid
/// `AgentConfig`, normalizes it into smaller tool-aware steps, and streams the
/// final JSON back as SSE events.
pub async fn build_agent(
    Json(payload): Json<BuildRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let scrubbed_description = scrub_pii(&payload.description);

    let schema = schema_for!(AgentConfig);
    let schema_json = serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".to_string());
    let supported_tools_json = serde_json::to_string_pretty(&supported_tool_schemas())
        .unwrap_or_else(|_| "[]".to_string());

    let system_prompt = format!(
        "You are an agent config generator for an internal agent builder. Convert the user's natural-language workflow into an agentic workflow and respond ONLY with valid JSON matching this schema:\n\n{schema_json}\n\nSupported tools for this product slice:\n{supported_tools_json}\n\nRequirements:\n- Always produce a concise agent `name`.\n- Always choose the best `trigger` variant. Use `Manual` when no schedule or event source is implied.\n- Always produce an ordered `steps` array with actionable step names.\n- Set `tool_name` only when a step calls one of the supported tools above.\n- Never invent Slack, GitHub, Jira, Salesforce, Discord, or any unsupported connector tool. The only internal non-connector tool allowed is `generate_content`.\n- Use the exact snake_case tool identifiers from the supported tools catalog.\n- Prefer `generate_content` for drafting or transforming content before a Gmail or Notion write step.\n- If a workflow creates a new Notion page that should contain body text, place `generate_content` immediately before `notion_create_page` so the generated content can be written into the page instead of leaving it empty.\n- Use `notion_append_block_children` when the user clearly wants to add content to an existing Notion page or block rather than create a new page.\n- Set `requires_approval` to true for any risky, human-review, or external-write step.\n- Break the workflow into the smallest useful sequence of 2 to 6 steps whenever possible.\n- Do not collapse data collection, reasoning, and delivery into one step. Prefer separate read -> generate_content -> write steps when drafting output is helpful.\n- If multiple supported tool actions are needed, assign each tool action its own step in execution order.\n- When the final destination is unsupported, keep the plan multi-step and end with a human handoff or approval step without inventing a tool.\n- Never assign a tool unless the user request contains enough information to satisfy that tool's required inputs. If required inputs are missing, use a non-tool planning or human handoff step instead of guessing arguments.\n- Do not add retrieval tools just because a connector is mentioned. Only add a read tool when the user explicitly asks to search, list, retrieve, inspect, review, or collect data from that connector.\n- For `google_send_gmail`, only use the tool when the request includes both a recipient and message content that can be inferred from the prompt. If either is missing, do not emit `google_send_gmail`; emit a non-tool step that asks for or waits on the missing delivery details.\n- Never output a tool name that is not present in the supported tools catalog above. If no supported tool fits, leave `tool_name` unset.\n- Do not include commentary, markdown, or code fences. Return JSON only."
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: scrubbed_description.clone(),
        },
    ];

    let generated_config = match OpenAiProvider::from_env() {
        Ok(provider) => match collect_streamed_response(provider.complete_stream(messages)).await {
            Ok(raw_json) => parse_or_fallback_agent_config(&raw_json, &scrubbed_description),
            Err(_) => fallback_agent_config(&scrubbed_description),
        },
        Err(_) => fallback_agent_config(&scrubbed_description),
    };

    let response_json = serde_json::to_string(&generated_config).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to serialize generated agent config: {error}"),
        )
    })?;

    let sse_stream = async_stream::stream! {
        for token in json_tokens(&response_json) {
            yield Ok::<_, Infallible>(Event::default().data(token));
        }

        yield Ok(Event::default().event("done").data("valid"));
    };

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

/// Collects a streamed model response into one string for validation.
async fn collect_streamed_response(
    mut stream: Pin<Box<dyn Stream<Item = AnyhowResult<String>> + Send>>,
) -> Result<String, String> {
    let mut accumulated = String::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => accumulated.push_str(&chunk),
            Err(error) => return Err(error.to_string()),
        }
    }

    if accumulated.trim().is_empty() {
        return Err("model returned an empty response".to_string());
    }

    Ok(accumulated)
}

/// Parses model output into an `AgentConfig`, falling back when the JSON is
/// invalid or too coarse.
fn parse_or_fallback_agent_config(raw_json: &str, description: &str) -> AgentConfig {
    match serde_json::from_str::<AgentConfig>(raw_json) {
        Ok(config) => normalize_agent_config(config, description),
        Err(_) => fallback_agent_config(description),
    }
}

fn default_missing_delivery_step_name(description: &str) -> Option<String> {
    let lower = description.to_lowercase();

    if (lower.contains("gmail") || lower.contains("email"))
        && (lower.contains("send") || lower.contains("reply") || lower.contains("draft"))
    {
        return Some("Request recipient and delivery details".to_string());
    }

    None
}
/// Constructs a best-effort `AgentConfig` from plain-English workflow text.
fn fallback_agent_config(description: &str) -> AgentConfig {
    AgentConfig {
        id: Uuid::new_v4(),
        name: infer_agent_name(description),
        trigger: infer_trigger(description),
        steps: decompose_agent_steps(description),
    }
}

/// Normalizes model-produced configs so the builder consistently gets smaller,
/// tool-aware steps.
fn normalize_agent_config(mut config: AgentConfig, description: &str) -> AgentConfig {
    if config.name.trim().is_empty() {
        config.name = infer_agent_name(description);
    }

    config.steps = if should_expand_steps(&config) {
        decompose_agent_steps(description)
    } else {
        config.steps.into_iter().map(normalize_existing_step).collect()
    };
    config.steps = ensure_generated_content_handoff(config.steps, description);

    config
}

/// Ensures downstream tools that need drafted text receive it from the
/// immediately preceding `generate_content` step.
fn ensure_generated_content_handoff(
    mut steps: Vec<AgentStep>,
    description: &str,
) -> Vec<AgentStep> {
    let mut index = 0;

    while index < steps.len() {
        let Some(tool_name) = steps[index].tool_name.as_deref() else {
            index += 1;
            continue;
        };

        if !requires_generated_content_handoff(tool_name) {
            index += 1;
            continue;
        }

        let has_generate_content_before = index > 0
            && steps[index - 1].tool_name.as_deref() == Some("generate_content");

        if has_generate_content_before {
            index += 1;
            continue;
        }

        steps.insert(
            index,
            AgentStep {
                id: Uuid::new_v4(),
                name: default_content_generation_step_name(tool_name, description),
                tool_name: Some("generate_content".to_string()),
                requires_approval: false,
            },
        );

        index += 2;
    }

    steps
}

fn requires_generated_content_handoff(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "google_send_gmail"
            | "notion_create_page"
            | "notion_update_page"
            | "notion_append_block_children"
    )
}

/// Extracts the subject clause from a description by scanning for common prepositions.
///
/// For example, "Create a Notion page about classical physics" → "classical physics".
fn extract_description_subject(description: &str) -> Option<&str> {
    let lower = description.to_lowercase();
    for prefix in ["about ", "on ", "regarding ", "for ", "covering "] {
        if let Some(idx) = lower.find(prefix) {
            let raw = description[idx + prefix.len()..].trim().trim_matches('.');
            let subject = raw
                .split(|c: char| c == ',' || c == ';')
                .next()
                .unwrap_or(raw)
                .trim();
            if !subject.is_empty() {
                return Some(subject);
            }
        }
    }
    None
}

/// Derives a topic-aware content-generation step name whose text will serve as the
/// LLM prompt inside `dispatch_generate_content`.
fn draft_step_name_from_description(downstream_tool: Option<&str>, description: &str) -> String {
    let subject = extract_description_subject(description);
    let trimmed = description.trim().trim_matches('.');

    match downstream_tool {
        Some("notion_create_page") => subject
            .map(|s| format!("Draft a Notion page about {s}"))
            .unwrap_or_else(|| format!("Draft Notion page content: {trimmed}")),
        Some("notion_update_page") => subject
            .map(|s| format!("Draft updated Notion content about {s}"))
            .unwrap_or_else(|| format!("Draft updated content: {trimmed}")),
        Some("notion_append_block_children") => subject
            .map(|s| format!("Draft content to append about {s}"))
            .unwrap_or_else(|| format!("Draft content to append: {trimmed}")),
        Some("google_send_gmail") => subject
            .map(|s| format!("Draft the email about {s}"))
            .unwrap_or_else(|| "Draft the email content".to_string()),
        _ => default_analysis_step_name(description),
    }
}

fn default_content_generation_step_name(tool_name: &str, description: &str) -> String {
    draft_step_name_from_description(Some(tool_name), description)
}

/// Determines whether a generated plan should be expanded into smaller steps.
fn should_expand_steps(config: &AgentConfig) -> bool {
    config.steps.len() <= 1
        || config.steps.iter().any(|step| {
            let lower = step.name.to_lowercase();
            lower == "process workflow request"
                || lower.contains("workflow request")
                || lower.contains(" and ")
                || lower.contains(',')
        })
}

/// Normalizes step names and tool identifiers when the model already produced a
/// sufficiently decomposed plan.
fn normalize_existing_step(mut step: AgentStep) -> AgentStep {
    if step.name.trim().is_empty() {
        step.name = "Untitled Step".to_string();
    }

    if let Some(tool_name) = step.tool_name.as_deref() {
        let normalized_tool_name = tool_name.trim();
        step.tool_name = if normalized_tool_name.is_empty() {
            None
        } else {
            Some(normalized_tool_name.to_string())
        };
    } else {
        let lower = step.name.to_lowercase();
        if lower.contains("draft")
            || lower.contains("compose")
            || lower.contains("summary")
            || lower.contains("summarize")
        {
            step.tool_name = Some("generate_content".to_string());
        }
    }

    step
}

fn analysis_tool_name(description: &str, write_tool: Option<&str>) -> Option<String> {
    let lower = description.to_lowercase();
    let looks_like_content_work = write_tool.is_some()
        || lower.contains("summary")
        || lower.contains("summarize")
        || lower.contains("draft")
        || lower.contains("compose")
        || lower.contains("email")
        || lower.contains("page")
        || lower.contains("content");

    if looks_like_content_work {
        Some("generate_content".to_string())
    } else {
        None
    }
}

/// Decomposes a workflow description into smaller steps that reflect available
/// tools and a separate reasoning phase.
fn decompose_agent_steps(description: &str) -> Vec<AgentStep> {
    let clause_steps = build_clause_steps(description);
    if clause_steps.len() >= 2 {
        return ensure_reasoning_step(clause_steps, description);
    }

    build_default_steps(description)
}

/// Converts natural-language clauses into ordered candidate steps.
fn build_clause_steps(description: &str) -> Vec<AgentStep> {
    let mut steps = Vec::new();

    for clause in infer_step_clauses(description) {
        let next_step = AgentStep {
            id: Uuid::new_v4(),
            name: sentence_case(&clause),
            tool_name: infer_tool_name(&clause),
            requires_approval: requires_approval(&clause),
        };

        let is_duplicate = steps.last().is_some_and(|previous: &AgentStep| {
            previous.name.eq_ignore_ascii_case(&next_step.name)
                && previous.tool_name == next_step.tool_name
        });

        if !is_duplicate {
            steps.push(next_step);
        }
    }

    steps
}

/// Ensures there is a non-tool reasoning step before the first write or handoff
/// action when the prompt otherwise maps directly from read to write.
fn ensure_reasoning_step(mut steps: Vec<AgentStep>, description: &str) -> Vec<AgentStep> {
    let has_reasoning_step = steps.iter().any(|step| step.tool_name.is_none());
    let first_write_index = steps.iter().position(|step| {
        step.tool_name.as_deref().is_some_and(is_write_tool)
            || (step.tool_name.is_none() && contains_write_verb(&step.name))
    });

    if !has_reasoning_step {
        if let Some(write_index) = first_write_index {
            if write_index > 0 {
                let write_tool = steps
                    .get(write_index)
                    .and_then(|step| step.tool_name.as_deref());
                let reasoning_tool = analysis_tool_name(description, write_tool);
                let reasoning_name = if reasoning_tool.as_deref() == Some("generate_content") {
                    draft_step_name_from_description(write_tool, description)
                } else {
                    default_analysis_step_name(description)
                };
                steps.insert(
                    write_index,
                    AgentStep {
                        id: Uuid::new_v4(),
                        name: reasoning_name,
                        tool_name: reasoning_tool,
                        requires_approval: false,
                    },
                );
            }
        }
    }

    steps
}

/// Builds a safe default multi-step plan when the prompt has only one clause or
/// does not clearly map to multiple explicit tool actions.
fn build_default_steps(description: &str) -> Vec<AgentStep> {
    let read_tool = infer_tool_name_for_mode(description, ToolIntent::Read);
    let write_tool = infer_tool_name_for_mode(description, ToolIntent::Write);
    let analysis_tool = analysis_tool_name(description, write_tool.as_deref());

    let mut steps = vec![AgentStep {
        id: Uuid::new_v4(),
        name: read_tool
            .as_deref()
            .map(default_read_step_name)
            .unwrap_or_else(|| "Gather required context".to_string()),
        tool_name: read_tool,
        requires_approval: false,
    }];

    // Use a topic-aware name when the analysis step calls `generate_content` so
    // the step name doubles as a meaningful LLM prompt (e.g. "Draft a Notion
    // page about classical physics" instead of "Prepare the final action").
    let analysis_step_name = if analysis_tool.as_deref() == Some("generate_content") {
        draft_step_name_from_description(write_tool.as_deref(), description)
    } else {
        default_analysis_step_name(description)
    };

    steps.push(AgentStep {
        id: Uuid::new_v4(),
        name: analysis_step_name,
        tool_name: analysis_tool,
        requires_approval: false,
    });

    steps.push(match write_tool {
        Some(tool_name) => AgentStep {
            id: Uuid::new_v4(),
            name: default_write_step_name(&tool_name),
            tool_name: Some(tool_name),
            requires_approval: requires_approval(description),
        },
        None => AgentStep {
            id: Uuid::new_v4(),
            name: default_missing_delivery_step_name(description).unwrap_or_else(|| {
                if contains_write_verb(description) {
                    "Request approval and hand off the final output".to_string()
                } else {
                    "Review and finalize the result".to_string()
                }
            }),
            tool_name: None,
            requires_approval: contains_write_verb(description) || requires_approval(description),
        },
    });

    steps
}

/// Splits JSON into small, safe chunks that mimic token-by-token streaming.
fn json_tokens(json: &str) -> Vec<String> {
    json.chars().map(|ch| ch.to_string()).collect()
}

/// Derives a stable display name from the first meaningful words.
fn infer_agent_name(description: &str) -> String {
    let words: Vec<&str> = description
        .split_whitespace()
        .filter(|w| !w.trim().is_empty())
        .take(6)
        .collect();

    if words.is_empty() {
        "Generated Agent".to_string()
    } else {
        format!("{} Agent", words.join(" "))
    }
}

/// Infers an `AgentConfig` trigger from lightweight workflow keywords.
fn infer_trigger(description: &str) -> Trigger {
    let lower = description.to_lowercase();

    if lower.contains("every friday") {
        return Trigger::Schedule {
            cron: "0 9 * * FRI".to_string(),
        };
    }

    if lower.contains("every week") || lower.contains("weekly") {
        return Trigger::Schedule {
            cron: "0 9 * * MON".to_string(),
        };
    }

    if lower.contains("every day") || lower.contains("daily") {
        return Trigger::Schedule {
            cron: "0 9 * * *".to_string(),
        };
    }

    if lower.contains("every hour") || lower.contains("hourly") {
        return Trigger::Schedule {
            cron: "0 * * * *".to_string(),
        };
    }

    if lower.contains("when") || lower.contains("on ") {
        return Trigger::Event {
            source: "workflow-input".to_string(),
            event: "description-submitted".to_string(),
        };
    }

    Trigger::Manual
}

/// Splits a workflow description into ordered execution clauses.
fn infer_step_clauses(description: &str) -> Vec<String> {
    let normalized = description
        .replace(" and then ", ",")
        .replace(" then ", ",")
        .replace(" after that ", ",")
        .replace(" and ", ",");

    normalized
        .split([',', ';'])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

/// Infers a best-effort tool binding from common workflow keywords.
fn infer_tool_name(value: &str) -> Option<String> {
    let lower = value.to_lowercase();
    if lower.contains("generate content")
        || lower.contains("draft content")
        || lower.contains("compose")
    {
        return Some("generate_content".to_string());
    }

    if contains_write_verb(value) {
        infer_tool_name_for_mode(value, ToolIntent::Write)
            .or_else(|| infer_tool_name_for_mode(value, ToolIntent::Read))
    } else {
        infer_tool_name_for_mode(value, ToolIntent::Read)
            .or_else(|| infer_tool_name_for_mode(value, ToolIntent::Write))
    }
}

fn contains_email_address(value: &str) -> bool {
    value
        .split_whitespace()
        .map(|part| {
            part.trim_matches(|ch: char| {
                matches!(ch, '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | '.' | ';' | ':' | '"' | '\'')
            })
        })
        .any(|part| {
            let at_index = part.find('@');
            matches!(at_index, Some(index) if index > 0 && index < part.len() - 1)
                && part[at_index.unwrap_or_default() + 1..].contains('.')
        })
}

fn contains_message_content(value: &str) -> bool {
    let lower = value.to_lowercase();
    [" saying ", " that says ", " with body ", " body ", " message ", " subject "]
        .iter()
        .any(|marker| lower.contains(marker))
        || value.matches('"').count() >= 2
        || value.matches('\'').count() >= 2
}

fn can_send_gmail_from_prompt(value: &str) -> bool {
    contains_email_address(value) && contains_message_content(value)
}

/// Infers a tool name for a specific read or write intent.
fn infer_tool_name_for_mode(value: &str, intent: ToolIntent) -> Option<String> {
    let lower = value.to_lowercase();

    if lower.contains("notion") || lower.contains("database") || lower.contains("page") {
        if matches!(intent, ToolIntent::Write) && (lower.contains("update") || lower.contains("edit")) {
            return Some("notion_update_page".to_string());
        }

        if matches!(intent, ToolIntent::Write)
            && (lower.contains("append") || lower.contains("block") || lower.contains("content"))
        {
            return Some("notion_append_block_children".to_string());
        }

        if matches!(intent, ToolIntent::Read)
            && (lower.contains("query") || lower.contains("filter") || lower.contains("database"))
        {
            return Some("notion_query_database".to_string());
        }

        if matches!(intent, ToolIntent::Read) && (lower.contains("search") || lower.contains("find")) {
            return Some("notion_search".to_string());
        }

        if matches!(intent, ToolIntent::Read)
            && (lower.contains("retrieve") || lower.contains("read") || lower.contains("get"))
        {
            return Some("notion_retrieve_page".to_string());
        }

        if matches!(intent, ToolIntent::Write) {
            return Some("notion_create_page".to_string());
        }
    }

    if lower.contains("google")
        || lower.contains("gmail")
        || lower.contains("calendar")
        || lower.contains("drive")
        || lower.contains("docs")
        || lower.contains("sheets")
        || lower.contains("email")
        || lower.contains("meeting")
    {
        if lower.contains("gmail") || lower.contains("email") || lower.contains("inbox") {
            if matches!(intent, ToolIntent::Write)
                && (lower.contains("send") || lower.contains("reply") || lower.contains("draft"))
            {
                if can_send_gmail_from_prompt(value) {
                    return Some("google_send_gmail".to_string());
                }

                return None;
            }

            if matches!(intent, ToolIntent::Read)
                && (lower.contains("search") || lower.contains("find") || lower.contains("query"))
            {
                return Some("google_search_gmail".to_string());
            }

            if matches!(intent, ToolIntent::Read) {
                let explicitly_reads_gmail = contains_read_verb(value)
                    || lower.contains("inbox")
                    || lower.contains("search")
                    || lower.contains("list")
                    || lower.contains("retrieve")
                    || lower.contains("open")
                    || lower.contains("review")
                    || lower.contains("inspect");

                if !explicitly_reads_gmail {
                    return None;
                }

                if lower.contains("get") || lower.contains("retrieve") || lower.contains("open") {
                    return Some("google_get_gmail_message".to_string());
                }

                return Some("google_list_gmail_messages".to_string());
            }
        }

        if lower.contains("calendar") || lower.contains("meeting") || lower.contains("event") {
            if matches!(intent, ToolIntent::Write)
                && (lower.contains("create")
                    || lower.contains("schedule")
                    || lower.contains("book")
                    || lower.contains("add"))
            {
                return Some("google_create_calendar_event".to_string());
            }

            if matches!(intent, ToolIntent::Read)
                && lower.contains("event")
                && (lower.contains("get") || lower.contains("retrieve"))
            {
                return Some("google_get_calendar_event".to_string());
            }

            if matches!(intent, ToolIntent::Read)
                && lower.contains("calendar")
                && (lower.contains("list") || lower.contains("all"))
            {
                return Some("google_list_calendars".to_string());
            }

            if matches!(intent, ToolIntent::Read) {
                return Some("google_list_calendar_events".to_string());
            }
        }

        if lower.contains("drive")
            || lower.contains("file")
            || lower.contains("docs")
            || lower.contains("sheets")
            || lower.contains("slides")
        {
            if matches!(intent, ToolIntent::Read) && (lower.contains("export") || lower.contains("download")) {
                return Some("google_export_drive_file".to_string());
            }

            if matches!(intent, ToolIntent::Read)
                && (lower.contains("get") || lower.contains("open") || lower.contains("read"))
            {
                return Some("google_get_drive_file".to_string());
            }

            if matches!(intent, ToolIntent::Read)
                && (lower.contains("search") || lower.contains("find") || lower.contains("query"))
            {
                return Some("google_search_drive".to_string());
            }

            if matches!(intent, ToolIntent::Read) {
                return Some("google_list_drive_files".to_string());
            }
        }
    }

    None
}

/// Heuristically identifies read-oriented language.
fn contains_read_verb(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "collect",
        "fetch",
        "find",
        "get",
        "inspect",
        "list",
        "query",
        "read",
        "retrieve",
        "review",
        "search",
    ]
    .iter()
    .any(|keyword| lower.contains(keyword))
}

/// Heuristically identifies write- or delivery-oriented language.
fn contains_write_verb(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "add",
        "append",
        "book",
        "create",
        "draft",
        "email",
        "notify",
        "post",
        "publish",
        "reply",
        "schedule",
        "send",
        "share",
        "update",
        "write",
    ]
    .iter()
    .any(|keyword| lower.contains(keyword))
}

/// Returns whether a supported tool mutates state or sends output externally.
fn is_write_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "notion_create_page"
            | "notion_update_page"
            | "notion_append_block_children"
            | "google_create_calendar_event"
            | "google_send_gmail"
    )
}

/// Provides a readable default step name for read-oriented tools.
fn default_read_step_name(tool_name: &str) -> String {
    match tool_name {
        "google_search_gmail" => "Search Gmail for relevant context".to_string(),
        "google_list_gmail_messages" => "Collect context from Gmail".to_string(),
        "google_get_gmail_message" => "Retrieve the relevant Gmail message".to_string(),
        "google_list_calendar_events" => "Review upcoming calendar events".to_string(),
        "google_get_calendar_event" => "Retrieve the relevant calendar event".to_string(),
        "google_list_calendars" => "Inspect available Google calendars".to_string(),
        "google_search_drive" => "Search Google Drive for source files".to_string(),
        "google_get_drive_file" => "Retrieve the source Drive file".to_string(),
        "google_list_drive_files" => "Collect source files from Google Drive".to_string(),
        "google_export_drive_file" => "Export the source Drive document".to_string(),
        "notion_query_database" => "Query Notion for source records".to_string(),
        "notion_search" => "Search Notion for relevant context".to_string(),
        "notion_retrieve_page" => "Retrieve the relevant Notion page".to_string(),
        _ => format!("Use {}", sentence_case(&tool_name.replace('_', " "))),
    }
}

/// Provides a readable default step name for write-oriented tools.
fn default_write_step_name(tool_name: &str) -> String {
    match tool_name {
        "google_send_gmail" => "Send the final Gmail update".to_string(),
        "google_create_calendar_event" => "Create the calendar event".to_string(),
        "notion_create_page" => "Create the Notion page".to_string(),
        "notion_update_page" => "Update the Notion page".to_string(),
        "notion_append_block_children" => "Append the final content in Notion".to_string(),
        _ => format!("Execute {}", sentence_case(&tool_name.replace('_', " "))),
    }
}

/// Provides a stable reasoning step label between retrieval and execution.
fn default_analysis_step_name(description: &str) -> String {
    let lower = description.to_lowercase();

    if lower.contains("summary") || lower.contains("summarize") || lower.contains("standup") {
        "Draft the summary output".to_string()
    } else if contains_write_verb(description) {
        "Prepare the final action".to_string()
    } else if contains_read_verb(description) {
        "Interpret the collected context".to_string()
    } else {
        "Plan the execution details".to_string()
    }
}

/// Infers whether a step likely needs human review.
fn requires_approval(value: &str) -> bool {
    let lower = value.to_lowercase();
    lower.contains("approve")
        || lower.contains("review")
        || lower.contains("human")
        || lower.contains("production")
        || lower.contains("delete")
        || lower.contains("write")
}

/// Converts a phrase into a clean, sentence-cased step label.
fn sentence_case(input: &str) -> String {
    let cleaned = input.trim().trim_matches('.');
    let mut chars = cleaned.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => "Untitled Step".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{fallback_agent_config, infer_trigger, json_tokens, normalize_agent_config};
    use core_types::agent::{AgentConfig, AgentStep, Trigger};
    use uuid::Uuid;

    #[test]
    fn fallback_config_serializes_and_parses_as_agent_config() {
        let config = fallback_agent_config(
            "When a lead submits a form, enrich with CRM, notify sales, and create a task",
        );

        let json = serde_json::to_string(&config).expect("fallback config should serialize");
        let parsed: core_types::agent::AgentConfig =
            serde_json::from_str(&json).expect("serialized config should parse");

        assert!(!parsed.steps.is_empty());
    }

    #[test]
    fn json_tokens_round_trip_to_original_json() {
        let sample = r#"{"status":"ok"}"#;
        let reconstructed = json_tokens(sample).join("");
        assert_eq!(reconstructed, sample);
    }

    #[test]
    fn fallback_config_expands_single_clause_requests_into_multiple_steps() {
        let config = fallback_agent_config("Post standup to Slack every Friday");

        assert!(config.steps.len() >= 3);
        assert!(config.steps.iter().all(|step| match step.tool_name.as_deref() {
            Some(tool_name) => !tool_name.contains("slack"),
            None => true,
        }));
    }

    #[test]
    fn fallback_config_separates_supported_read_and_write_steps() {
        let config = fallback_agent_config(
            "Search Gmail for customer escalations and create a Notion page with the findings",
        );

        assert!(config.steps.len() >= 3);
        assert_eq!(config.steps[0].tool_name.as_deref(), Some("google_search_gmail"));
        assert!(config
            .steps
            .iter()
            .any(|step| step.tool_name.as_deref() == Some("generate_content")));
        assert_eq!(
            config.steps.last().and_then(|step| step.tool_name.as_deref()),
            Some("notion_create_page")
        );
    }

    #[test]
    fn fallback_config_does_not_bind_gmail_tools_when_send_prompt_lacks_recipient() {
        let config = fallback_agent_config("Send an email that says hello Agent");

        assert!(config
            .steps
            .iter()
            .all(|step| step.tool_name.as_deref() != Some("google_send_gmail")));
        assert!(config
            .steps
            .iter()
            .all(|step| step.tool_name.as_deref() != Some("google_list_gmail_messages")));
        assert_eq!(
            config.steps.last().map(|step| step.name.as_str()),
            Some("Request recipient and delivery details")
        );
    }

    #[test]
    fn infer_trigger_supports_every_friday_schedules() {
        match infer_trigger("Post standup to Slack every Friday") {
            Trigger::Schedule { cron } => assert_eq!(cron, "0 9 * * FRI"),
            other => panic!("expected friday schedule trigger, got {other:?}"),
        }
    }

    #[test]
    fn generate_content_step_name_reflects_topic_for_notion_page_creation() {
        let config = fallback_agent_config("Create a notion page about classical physics");

        let content_step = config
            .steps
            .iter()
            .find(|step| step.tool_name.as_deref() == Some("generate_content"))
            .expect("should have a generate_content step");

        assert!(
            content_step.name.to_lowercase().contains("classical physics"),
            "generate_content step name should include the topic so it becomes a useful prompt; got: {:?}",
            content_step.name
        );
    }

    #[test]
    fn normalize_agent_config_inserts_generate_content_before_notion_page_creation() {
        let config = AgentConfig {
            id: Uuid::new_v4(),
            name: "Escalation Digest".to_string(),
            trigger: Trigger::Manual,
            steps: vec![
                AgentStep {
                    id: Uuid::new_v4(),
                    name: "Search Gmail for escalations".to_string(),
                    tool_name: Some("google_search_gmail".to_string()),
                    requires_approval: false,
                },
                AgentStep {
                    id: Uuid::new_v4(),
                    name: "Review the findings".to_string(),
                    tool_name: None,
                    requires_approval: false,
                },
                AgentStep {
                    id: Uuid::new_v4(),
                    name: "Create the Notion page".to_string(),
                    tool_name: Some("notion_create_page".to_string()),
                    requires_approval: true,
                },
            ],
        };

        let normalized = normalize_agent_config(
            config,
            "Search Gmail for escalations and create a Notion page with the findings",
        );
        let write_index = normalized
            .steps
            .iter()
            .position(|step| step.tool_name.as_deref() == Some("notion_create_page"))
            .expect("normalized config should retain the Notion create step");

        assert!(write_index > 0);
        assert_eq!(
            normalized.steps[write_index - 1].tool_name.as_deref(),
            Some("generate_content")
        );
    }
}
