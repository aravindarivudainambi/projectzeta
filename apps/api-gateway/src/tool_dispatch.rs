use core_types::tool::ToolResult;
use futures_util::StreamExt;
use llm_client::{
    openai::OpenAiProvider,
    pii_scrubber::scrub_pii,
    provider::{ChatMessage, LlmProvider},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const CONNECTOR_HUB_URL: &str = "http://localhost:8082";

#[derive(Debug, Serialize)]
struct HubRequest {
    tool_name: String,
    arguments: Value,
    token: String,
}

#[derive(Debug, Deserialize)]
struct HubResponse {
    success: bool,
    output: Value,
}

fn content_blocks_from_text(text: &str) -> Value {
    Value::Array(
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                json!({
                    "object": "block",
                    "type": "paragraph",
                    "paragraph": {
                        "rich_text": [
                            {
                                "type": "text",
                                "text": { "content": line }
                            }
                        ]
                    }
                })
            })
            .collect(),
    )
}

fn summarize_prompt(prompt: &str, limit: usize) -> String {
    let trimmed = prompt.trim();
    let mut words = trimmed.split_whitespace();
    let summary = words.by_ref().take(limit).collect::<Vec<_>>().join(" ");
    if words.next().is_some() {
        format!("{summary}…")
    } else {
        summary
    }
}

fn default_generated_title(prompt: &str) -> String {
    let summary = summarize_prompt(prompt, 8);
    if summary.is_empty() {
        "Generated Draft".to_string()
    } else {
        summary
    }
}

fn serialize_context(context: &Value) -> String {
    match context {
        Value::String(text) => text.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    }
}

fn normalize_generated_content_output(
    mut output: Value,
    prompt: &str,
    target_tool: Option<&str>,
    provider_name: &str,
) -> Value {
    if !output.is_object() {
        output = json!({ "content": output.to_string() });
    }

    let Some(object) = output.as_object_mut() else {
        return json!({
            "title": default_generated_title(prompt),
            "subject": default_generated_title(prompt),
            "content": prompt,
            "blocks": content_blocks_from_text(prompt),
            "provider": provider_name,
        });
    };

    let content = object
        .get("content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            object
                .get("body")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| prompt.to_string());

    object.insert("content".to_string(), Value::String(content.clone()));

    if object
        .get("title")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        object.insert(
            "title".to_string(),
            Value::String(default_generated_title(prompt)),
        );
    }

    if matches!(target_tool, Some("google_send_gmail"))
        && object
            .get("subject")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
    {
        object.insert(
            "subject".to_string(),
            Value::String(default_generated_title(prompt)),
        );
    }

    if matches!(target_tool, Some(tool) if tool.starts_with("notion_"))
        && object.get("blocks").is_none()
    {
        object.insert("blocks".to_string(), content_blocks_from_text(&content));
    }

    object.insert(
        "provider".to_string(),
        Value::String(provider_name.to_string()),
    );

    Value::Object(object.clone())
}

async fn collect_llm_response(
    mut stream: std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>,
) -> anyhow::Result<String> {
    let mut accumulated = String::new();

        while let Some(chunk) = stream.next().await {
        accumulated.push_str(&chunk?);
    }

    if accumulated.trim().is_empty() {
        anyhow::bail!("model returned an empty response");
    }

    Ok(accumulated)
}

/// Produces a best-effort fallback payload when the LLM is unreachable.
///
/// **Note:** This is intentionally kept but currently unused by the main dispatch
/// path, which now returns `success: false` on LLM failure so that downstream
/// tools (e.g. `notion_create_page`) do not write placeholder text into real
/// pages. Retained for potential future use (e.g. dry-run or preview modes).
#[allow(dead_code)]
fn fallback_generated_content_output(
    prompt: &str,
    context: Option<&Value>,
    target_tool: Option<&str>,
) -> Value {
    let title = default_generated_title(prompt);
    let context_suffix = context
        .map(serialize_context)
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("\n\nContext:\n{value}"))
        .unwrap_or_default();
    let content = format!("Draft requested for: {prompt}.{context_suffix}");

    normalize_generated_content_output(
        json!({
            "title": title,
            "subject": default_generated_title(prompt),
            "content": content,
        }),
        prompt,
        target_tool,
        "fallback",
    )
}

fn parse_generated_content_output(raw: &str, prompt: &str, target_tool: Option<&str>) -> Value {
    let trimmed = raw.trim();
    let value = serde_json::from_str::<Value>(trimmed)
        .unwrap_or_else(|_| json!({ "content": trimmed }));

    normalize_generated_content_output(value, prompt, target_tool, "gpt")
}

/// Extracts a retry wait time from rate-limit error messages.
///
/// Parses patterns like "Please wait 51 seconds" commonly returned by
/// GitHub Models / Azure OpenAI.
fn parse_retry_after_seconds(error_msg: &str) -> Option<u64> {
    let lower = error_msg.to_lowercase();
    if let Some(idx) = lower.find("wait ") {
        let after = &error_msg[idx + 5..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(secs) = digits.parse::<u64>() {
            // Add a small buffer to avoid hitting the limit again immediately.
            return Some(secs.saturating_add(2));
        }
    }
    None
}

/// Maximum number of LLM attempts (initial + retries) before giving up.
const GENERATE_CONTENT_MAX_ATTEMPTS: usize = 2;

async fn dispatch_generate_content(arguments: &Value) -> anyhow::Result<HubResponse> {
    let prompt = arguments
        .get("prompt")
        .and_then(Value::as_str)
        .or_else(|| arguments.get("instructions").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("missing prompt"))?;
    let prompt = scrub_pii(prompt);
    let target_tool = arguments.get("target_tool").and_then(Value::as_str);
    let tone = arguments.get("tone").and_then(Value::as_str).unwrap_or("professional");
    let format_hint = arguments
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("plain_text");
    let context = arguments.get("context").cloned();

    let system_prompt = format!(
        "You generate workflow-ready content for downstream automation tools. Respond ONLY with valid JSON. Required shape: {{\"title\": string, \"subject\": string | null, \"content\": string, \"blocks\": array | null}}. If the target tool is Gmail, make `subject` specific and useful. If the target tool is Notion, include concise paragraph blocks in `blocks`. Tone: {tone}. Format hint: {format_hint}. Target tool: {}.",
        target_tool.unwrap_or("generic")
    );

    let mut user_prompt = format!("Prompt:\n{prompt}");
    if let Some(context) = context.as_ref() {
        let serialized = serialize_context(context);
        if !serialized.trim().is_empty() {
            user_prompt.push_str("\n\nContext:\n");
            user_prompt.push_str(&serialized);
        }
    }

    // Use a dedicated content model (defaults to gpt-4o) which has much
    // higher rate limits than reasoning models like gpt-5.
    let provider = match OpenAiProvider::from_env_for_content() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("[generate_content] LLM provider unavailable: {err}");
            return Ok(HubResponse {
                success: false,
                output: json!({
                    "error": format!(
                        "Content generation failed: LLM provider is not configured ({err}). \
                         Set GITHUB_MODELS_API_KEY in the environment."
                    )
                }),
            });
        }
    };

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let mut last_error = String::new();

    for attempt in 1..=GENERATE_CONTENT_MAX_ATTEMPTS {
        match collect_llm_response(provider.complete_stream(messages.clone())).await {
            Ok(raw) => {
                let output = parse_generated_content_output(&raw, &prompt, target_tool);
                return Ok(HubResponse {
                    success: true,
                    output,
                });
            }
            Err(err) => {
                last_error = err.to_string();
                eprintln!(
                    "[generate_content] LLM attempt {attempt}/{GENERATE_CONTENT_MAX_ATTEMPTS} \
                     failed: {last_error}"
                );
                if attempt < GENERATE_CONTENT_MAX_ATTEMPTS {
                    // Parse wait time from rate-limit errors (e.g. "Please wait 51 seconds").
                    let wait_secs = parse_retry_after_seconds(&last_error).unwrap_or(2);
                    eprintln!("[generate_content] waiting {wait_secs}s before retry");
                    tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
                }
            }
        }
    }

    // All retries exhausted — report failure so downstream steps (e.g.
    // notion_create_page) do not receive placeholder text.
    eprintln!(
        "[generate_content] all {GENERATE_CONTENT_MAX_ATTEMPTS} attempts failed for prompt: {}",
        prompt.chars().take(120).collect::<String>()
    );

    Ok(HubResponse {
        success: false,
        output: json!({
            "error": format!(
                "Content generation failed after {GENERATE_CONTENT_MAX_ATTEMPTS} attempts: {last_error}"
            )
        }),
    })
}

fn truncate_response_body(body: &str, limit: usize) -> String {
    let trimmed = body.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }

    let truncated = trimmed.chars().take(limit).collect::<String>();
    format!("{truncated}…")
}

fn decode_hub_response_body(status: reqwest::StatusCode, body: &str) -> anyhow::Result<HubResponse> {
    serde_json::from_str::<HubResponse>(body).map_err(|error| {
        let preview = truncate_response_body(body, 400);
        if preview.is_empty() {
            anyhow::anyhow!(
                "failed to decode connector-hub response: connector-hub returned an empty response body (HTTP {}): {}",
                status.as_u16(),
                error
            )
        } else {
            anyhow::anyhow!(
                "failed to decode connector-hub response: connector-hub returned an invalid response body (HTTP {}): {} ({})",
                status.as_u16(),
                preview,
                error
            )
        }
    })
}

async fn parse_hub_response(resp: reqwest::Response) -> anyhow::Result<HubResponse> {
    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("failed to read connector-hub response body: {e}"))?;

    decode_hub_response_body(status, &body)
}

/// Dispatches a tool call to the appropriate backend service.
///
/// For `notion_*` tools this forwards to connector-hub's `/notion/execute`.
/// Unrecognized tools return a failure `ToolResult` (never panics).
pub async fn dispatch_tool_call(
    http: &reqwest::Client,
    tool_name: &str,
    arguments: &Value,
    token: &str,
) -> ToolResult {
    if tool_name == "generate_content" {
        match dispatch_generate_content(arguments).await {
            Ok(resp) => ToolResult {
                tool_name: tool_name.to_string(),
                success: resp.success,
                output_json: serde_json::to_string(&resp.output)
                    .unwrap_or_else(|_| "{}".to_string()),
            },
            Err(e) => ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output_json: serde_json::json!({ "error": e.to_string() }).to_string(),
            },
        }
    } else if tool_name.starts_with("notion_") {
        match dispatch_notion(http, tool_name, arguments, token).await {
            Ok(resp) => ToolResult {
                tool_name: tool_name.to_string(),
                success: resp.success,
                output_json: serde_json::to_string(&resp.output)
                    .unwrap_or_else(|_| "{}".to_string()),
            },
            Err(e) => ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output_json: serde_json::json!({ "error": e.to_string() }).to_string(),
            },
        }
    } else if tool_name.starts_with("google_") {
        match dispatch_google(http, tool_name, arguments, token).await {
            Ok(resp) => ToolResult {
                tool_name: tool_name.to_string(),
                success: resp.success,
                output_json: serde_json::to_string(&resp.output)
                    .unwrap_or_else(|_| "{}".to_string()),
            },
            Err(e) => ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output_json: serde_json::json!({ "error": e.to_string() }).to_string(),
            },
        }
    } else {
        ToolResult {
            tool_name: tool_name.to_string(),
            success: false,
            output_json: serde_json::json!({
                "error": format!("no dispatcher registered for tool: {tool_name}")
            })
            .to_string(),
        }
    }
}

async fn dispatch_notion(
    http: &reqwest::Client,
    tool_name: &str,
    arguments: &Value,
    token: &str,
) -> anyhow::Result<HubResponse> {
    let hub_req = HubRequest {
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
        token: token.to_string(),
    };

    let resp = http
        .post(format!("{CONNECTOR_HUB_URL}/notion/execute"))
        .json(&hub_req)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("failed to reach connector-hub: {e}"))?;

    parse_hub_response(resp).await
}

async fn dispatch_google(
    http: &reqwest::Client,
    tool_name: &str,
    arguments: &Value,
    token: &str,
) -> anyhow::Result<HubResponse> {
    let hub_req = HubRequest {
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
        token: token.to_string(),
    };

    let resp = http
        .post(format!("{CONNECTOR_HUB_URL}/google/execute"))
        .json(&hub_req)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("failed to reach connector-hub: {e}"))?;

    parse_hub_response(resp).await
}

#[cfg(test)]
mod tests {
    use super::{decode_hub_response_body, parse_generated_content_output};
    use serde_json::json;

    #[test]
    fn decodes_structured_hub_response_body() {
        let decoded = decode_hub_response_body(
            reqwest::StatusCode::OK,
            r#"{"success":true,"output":{"messages":[]}}"#,
        )
        .expect("hub response should decode");

        assert!(decoded.success);
        assert_eq!(decoded.output, json!({ "messages": [] }));
    }

    #[test]
    fn surfaces_plain_text_hub_errors() {
        let error = decode_hub_response_body(
            reqwest::StatusCode::BAD_REQUEST,
            "No mock token configured for provider 'google_workspace'. Set MOCK_TOKEN_GOOGLE_WORKSPACE in .env or environment.",
        )
        .expect_err("plain text response should fail with raw body in message");

        let message = error.to_string();
        assert!(message.contains("HTTP 400"));
        assert!(message.contains("MOCK_TOKEN_GOOGLE_WORKSPACE"));
    }

    #[test]
    fn surfaces_empty_hub_errors() {
        let error = decode_hub_response_body(reqwest::StatusCode::INTERNAL_SERVER_ERROR, "   ")
            .expect_err("empty response should fail");

        assert!(error.to_string().contains("empty response body"));
    }

    #[test]
    fn normalizes_generated_content_json_output() {
        let parsed = parse_generated_content_output(
            r#"{"content":"Hello there","subject":"Status update"}"#,
            "Write a status update email",
            Some("google_send_gmail"),
        );

        assert_eq!(parsed["content"], "Hello there");
        assert_eq!(parsed["subject"], "Status update");
        assert_eq!(parsed["provider"], "gpt");
    }

    #[test]
    fn wraps_plain_text_generated_content_output() {
        let parsed = parse_generated_content_output(
            "Hello there",
            "Write a status update email",
            Some("google_send_gmail"),
        );

        assert_eq!(parsed["content"], "Hello there");
        assert!(parsed["subject"].as_str().is_some());
    }
}
