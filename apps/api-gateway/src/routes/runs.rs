use std::collections::HashMap;
use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use chrono::Utc;
use core_types::agent::AgentStep;
use core_types::events::AgentEvent;
use core_types::run::{AgentRun, ApprovalStatus, RunHistoryEntry, RunStatus, StepResultEntry};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::errors::AppError;
use crate::state::{AppState, ApprovalRecord, RunRecord};

// ---------------------------------------------------------------------------
// POST /runs
// ---------------------------------------------------------------------------

/// Payload accepted when creating a new agent run.
#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub agent_id: Uuid,
    pub steps: Vec<StepDefinition>,
}

/// A single step definition provided by the caller.
#[derive(Debug, Deserialize)]
pub struct StepDefinition {
    pub name: String,
    #[serde(default)]
    pub requires_approval: bool,
    pub tool_name: Option<String>,
    pub tool_arguments: Option<Value>,
}

/// Response returned after a run is created.
#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub run_id: Uuid,
    pub status: String,
}

fn extract_email_address(step_name: &str) -> Option<String> {
    step_name
        .split_whitespace()
        .map(|part| part.trim_matches(|ch: char| {
            matches!(ch, '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | '.' | ';' | ':' | '"' | '\'')
        }))
        .find(|part| {
            let at_index = part.find('@');
            matches!(at_index, Some(index) if index > 0 && index < part.len() - 1)
                && part[at_index.unwrap_or_default() + 1..].contains('.')
        })
        .map(str::to_string)
}

fn extract_message_body(step_name: &str) -> Option<String> {
    let lower = step_name.to_lowercase();
    let markers = [" saying ", " that says ", " with body ", " body ", " message "];

    markers.iter().find_map(|marker| {
        lower.find(marker).and_then(|idx| {
            let start = idx + marker.len();
            let body = step_name.get(start..)?.trim().trim_matches('.').trim();
            if body.is_empty() {
                None
            } else {
                Some(body.to_string())
            }
        })
    })
}

fn default_gmail_subject(step_name: &str, body: Option<&str>) -> String {
    let lower = step_name.to_lowercase();
    if let Some(idx) = lower.find("subject ") {
        let subject = step_name[idx + "subject ".len()..]
            .trim()
            .trim_matches('.')
            .trim_matches('"')
            .trim_matches('\'')
            .trim();
        if !subject.is_empty() {
            return subject.to_string();
        }
    }

    if let Some(body) = body {
        if !body.is_empty() {
            let preview = body.chars().take(40).collect::<String>();
            return format!("Automated message: {preview}");
        }
    }

    "Automated message".to_string()
}

fn enrich_tool_arguments(step_name: &str, tool_name: &str, args: Option<Value>) -> Value {
    let mut args = args.unwrap_or_else(|| Value::Object(Default::default()));

    if tool_name != "google_send_gmail" {
        return args;
    }

    let Some(object) = args.as_object_mut() else {
        return args;
    };

    let label_text = object
        .get("label")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let description_text = object
        .get("description")
        .and_then(|value| value.as_str())
        .map(str::to_string);

    if object
        .get("to")
        .and_then(|value| value.as_str())
        .is_none_or(str::is_empty)
    {
        if let Some(recipient) = object
            .get("recipient")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
        {
            object.insert("to".to_string(), Value::String(recipient.to_string()));
        } else if let Some(email) = object
            .get("email")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
        {
            object.insert("to".to_string(), Value::String(email.to_string()));
        }
    }

    let missing_to = object
        .get("to")
        .and_then(|value| value.as_str())
        .is_none_or(str::is_empty);
    if missing_to {
        let inferred_to = extract_email_address(step_name)
            .or_else(|| label_text.as_deref().and_then(extract_email_address))
            .or_else(|| description_text.as_deref().and_then(extract_email_address));
        if let Some(to) = inferred_to {
            object.insert("to".to_string(), Value::String(to));
        }
    }

    let missing_body = object
        .get("body")
        .and_then(|value| value.as_str())
        .is_none_or(str::is_empty);
    if missing_body {
        let inferred_body = extract_message_body(step_name)
            .or_else(|| label_text.as_deref().and_then(extract_message_body))
            .or_else(|| description_text.as_deref().and_then(extract_message_body));
        if let Some(body) = inferred_body {
            object.insert("body".to_string(), Value::String(body));
        }
    }

    let missing_subject = object
        .get("subject")
        .and_then(|value| value.as_str())
        .is_none_or(str::is_empty);
    if missing_subject {
        let body = object.get("body").and_then(|value| value.as_str());
        object.insert(
            "subject".to_string(),
            Value::String(default_gmail_subject(step_name, body)),
        );
    }

    args
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

fn generated_content_payload(previous_output: &Value) -> Option<&Value> {
    if previous_output
        .get("tool")
        .and_then(|value| value.as_str())
        == Some("generate_content")
    {
        return previous_output.get("output");
    }

    let nested = previous_output.get("output")?;
    if nested.get("content").is_some() || nested.get("blocks").is_some() {
        Some(nested)
    } else {
        None
    }
}

fn apply_runtime_tool_context(
    step_name: &str,
    tool_name: &str,
    args: Value,
    previous_output: Option<&Value>,
    next_tool_name: Option<&str>,
) -> Value {
    let mut args = enrich_tool_arguments(step_name, tool_name, Some(args));
    let Some(object) = args.as_object_mut() else {
        return args;
    };

    match tool_name {
        "generate_content" => {
            if object
                .get("prompt")
                .and_then(|value| value.as_str())
                .is_none_or(str::is_empty)
            {
                let prompt_source = object
                    .get("description")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.is_empty())
                    .or_else(|| {
                        object
                            .get("label")
                            .and_then(|value| value.as_str())
                            .filter(|value| !value.is_empty())
                    })
                    .unwrap_or(step_name);
                object.insert("prompt".to_string(), Value::String(prompt_source.to_string()));
            }

            if object.get("context").is_none() {
                if let Some(previous_output) = previous_output {
                    let context = previous_output
                        .get("output")
                        .cloned()
                        .unwrap_or_else(|| previous_output.clone());
                    object.insert("context".to_string(), context);
                }
            }

            if object
                .get("target_tool")
                .and_then(|value| value.as_str())
                .is_none_or(str::is_empty)
            {
                if let Some(next_tool_name) = next_tool_name {
                    object.insert(
                        "target_tool".to_string(),
                        Value::String(next_tool_name.to_string()),
                    );
                }
            }
        }
        "google_send_gmail" => {
            if let Some(generated) = previous_output.and_then(generated_content_payload) {
                if object
                    .get("body")
                    .and_then(|value| value.as_str())
                    .is_none_or(str::is_empty)
                {
                    if let Some(content) = generated.get("content").and_then(|value| value.as_str()) {
                        object.insert("body".to_string(), Value::String(content.to_string()));
                    }
                }

                let current_body = object.get("body").and_then(|value| value.as_str());
                let default_subject = default_gmail_subject(step_name, current_body);
                let should_replace_subject = object
                    .get("subject")
                    .and_then(|value| value.as_str())
                    .is_none_or(|value| {
                        value.is_empty()
                            || value == "Automated message"
                            || value == default_subject
                            || value.starts_with("Automated message:")
                    });
                if should_replace_subject {
                    if let Some(subject) = generated
                        .get("subject")
                        .and_then(|value| value.as_str())
                        .or_else(|| generated.get("title").and_then(|value| value.as_str()))
                    {
                        object.insert("subject".to_string(), Value::String(subject.to_string()));
                    }
                }
            }
        }
        "notion_append_block_children" | "notion_create_page" => {
            if let Some(generated) = previous_output.and_then(generated_content_payload) {
                if object.get("children").is_none() {
                    if let Some(blocks) = generated.get("blocks") {
                        object.insert("children".to_string(), blocks.clone());
                    } else if let Some(content) = generated.get("content").and_then(|value| value.as_str()) {
                        object.insert("children".to_string(), content_blocks_from_text(content));
                    }
                }
            }
        }
        _ => {}
    }

    args
}

fn summarize_step_output(step_result: &Value) -> String {
    if let Some(error) = step_result
        .get("output")
        .and_then(|value| value.get("error"))
        .and_then(|value| value.as_str())
    {
        return error.chars().take(200).collect();
    }

    if let Some(error) = step_result
        .get("error")
        .and_then(|value| value.as_str())
    {
        return error.chars().take(200).collect();
    }

    if let Some(output) = step_result.get("output") {
        if let Some(text) = output.as_str() {
            return text.chars().take(200).collect();
        }

        return output.to_string().chars().take(200).collect();
    }

    step_result.to_string().chars().take(200).collect()
}

/// Creates a new agent run and stores it in memory.
///
/// The run is created in `Pending` status and must be streamed via
/// `GET /runs/{id}/stream` to begin execution.
pub async fn create_run(
    State(state): State<AppState>,
    Json(payload): Json<CreateRunRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.steps.is_empty() {
        return Err(AppError::BadRequest(
            "at least one step is required".to_string(),
        ));
    }

    let run_id = Uuid::new_v4();
    let mut tool_bindings = HashMap::new();
    let steps: Vec<AgentStep> = payload
        .steps
        .into_iter()
        .map(|s| {
            let StepDefinition {
                name,
                requires_approval,
                tool_name,
                tool_arguments,
            } = s;
            let step = AgentStep {
                id: Uuid::new_v4(),
                name: name.clone(),
                tool_name: tool_name.clone(),
                requires_approval,
            };
            if let Some(tool_name) = tool_name {
                let args = enrich_tool_arguments(&name, &tool_name, tool_arguments);
                tool_bindings.insert(step.id, (tool_name, args));
            }
            step
        })
        .collect();

    let record = RunRecord {
        run: AgentRun {
            id: run_id,
            agent_id: payload.agent_id,
            status: RunStatus::Pending,
        },
        steps,
        tool_bindings,
    };

    {
        let mut runs = state.runs.write().await;
        runs.insert(run_id, record);
    }

    Ok((
        StatusCode::CREATED,
        Json(CreateRunResponse {
            run_id,
            status: "Pending".to_string(),
        }),
    ))
}

// ---------------------------------------------------------------------------
// GET /runs/:id/stream
// ---------------------------------------------------------------------------

/// Opens an SSE stream that executes the agent run and emits events as they occur.
///
/// Creates a `tokio::sync::mpsc` channel, spawns the run loop in a background
/// Tokio task (producer), and returns an `Sse` response consuming the receiver
/// (consumer). The connection closes automatically when the run finishes.
pub async fn stream_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let record = {
        let runs = state.runs.read().await;
        runs.get(&run_id).cloned().ok_or(AppError::NotFound)?
    };

    match record.run.status {
        RunStatus::Succeeded | RunStatus::Failed => {
            return Err(AppError::BadRequest("run already completed".to_string()));
        }
        RunStatus::Running | RunStatus::WaitingForApproval => {
            return Err(AppError::BadRequest("run already in progress".to_string()));
        }
        _ => {}
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AgentEvent>(32);

    // Mark the run as Running.
    {
        let mut runs = state.runs.write().await;
        if let Some(r) = runs.get_mut(&run_id) {
            r.run.status = RunStatus::Running;
        }
    }

    let steps = record.steps.clone();
    let tool_bindings = record.tool_bindings.clone();
    let agent_id = record.run.agent_id;
    let loop_state = state.clone();
    tokio::spawn(async move {
        execute_run(run_id, agent_id, steps, tool_bindings, tx, loop_state).await;
    });

    let sse_stream = async_stream::stream! {
        while let Some(event) = rx.recv().await {
            let json = serde_json::to_string(&event).unwrap_or_default();
            yield Ok::<_, Infallible>(Event::default().data(json));
        }
    };

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

// ---------------------------------------------------------------------------
// Run loop (spawned Tokio task)
// ---------------------------------------------------------------------------

/// Resolves the correct auth token for a tool based on its prefix.
fn resolve_token(_state: &AppState, _tool_name: &str) -> String {
    String::new()
}

/// Executes an agent run step-by-step, sending events through the channel.
///
/// For each step the loop emits `StepStarted`, dispatches the bound tool (or
/// simulates work if no binding exists), emits `ToolCalled`, and then
/// `StepCompleted`. Steps tagged `requires_approval` pause execution until
/// a human decision is recorded via the approve/reject endpoints.
///
/// On completion (success or failure), a `RunHistoryEntry` is written to state.
async fn execute_run(
    run_id: Uuid,
    agent_id: Uuid,
    steps: Vec<AgentStep>,
    tool_bindings: HashMap<Uuid, (String, Value)>,
    tx: tokio::sync::mpsc::Sender<AgentEvent>,
    state: AppState,
) {
    let started_at = Utc::now();
    let mut step_results: Vec<StepResultEntry> = Vec::new();
    let mut run_failed = false;
    let mut previous_step_output: Option<Value> = None;

    for (index, step) in steps.iter().enumerate() {
        let step_start = std::time::Instant::now();

        // Emit StepStarted.
        let _ = tx
            .send(AgentEvent::StepStarted {
                step_id: step.id,
                step_name: step.name.clone(),
            })
            .await;

        // Dispatch real tool or simulate.
        let (step_result, tool_success) =
            if let Some((tool_name, tool_args)) = tool_bindings.get(&step.id) {
                let token = resolve_token(&state, tool_name);
                let next_tool_name = steps
                    .get(index + 1)
                    .and_then(|next_step| tool_bindings.get(&next_step.id))
                    .map(|(tool_name, _)| tool_name.as_str());
                let runtime_args = apply_runtime_tool_context(
                    &step.name,
                    tool_name,
                    tool_args.clone(),
                    previous_step_output.as_ref(),
                    next_tool_name,
                );

                // Emit ToolCalled before execution so the UI shows the invocation.
                let _ = tx
                    .send(AgentEvent::ToolCalled {
                        tool: tool_name.clone(),
                        args: runtime_args.clone(),
                    })
                    .await;

                let result = crate::tool_dispatch::dispatch_tool_call(
                    &state.http_client,
                    tool_name,
                    &runtime_args,
                    &token,
                )
                .await;

                let success = result.success;
                let parsed_output = serde_json::from_str::<Value>(&result.output_json)
                    .unwrap_or_else(|_| Value::String(result.output_json.clone()));
                let result_json = json!({
                    "tool": result.tool_name,
                    "success": result.success,
                    "output": parsed_output,
                });
                (result_json, success)
            } else {
                // No tool binding — simulate as before.
                tokio::time::sleep(Duration::from_secs(1)).await;

                let _ = tx
                    .send(AgentEvent::ToolCalled {
                        tool: format!(
                            "tool_for_{}",
                            step.name.to_lowercase().replace(' ', "_")
                        ),
                        args: json!({ "step": step.name }),
                    })
                    .await;

                (
                    json!({ "output": format!("{} completed successfully", step.name) }),
                    true,
                )
            };

        // Human approval gate.
        if step.requires_approval {
            let approval_id = Uuid::new_v4();

            // Write approval request to in-memory store.
            {
                let mut approvals = state.approvals.write().await;
                approvals.insert(
                    approval_id,
                    ApprovalRecord {
                        id: approval_id,
                        run_id,
                        step_id: step.id,
                        action: format!("Approve execution of step: {}", step.name),
                        status: ApprovalStatus::Pending,
                    },
                );
            }

            // Update run status to WaitingForApproval.
            {
                let mut runs = state.runs.write().await;
                if let Some(r) = runs.get_mut(&run_id) {
                    r.run.status = RunStatus::WaitingForApproval;
                }
            }

            // Notify the SSE client that approval is required.
            let _ = tx
                .send(AgentEvent::HumanApprovalRequired {
                    action: format!("Approve execution of step: {}", step.name),
                })
                .await;

            // Poll every 500 ms for a decision.
            let decision = loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let approvals = state.approvals.read().await;
                if let Some(record) = approvals.get(&approval_id) {
                    match record.status {
                        ApprovalStatus::Approved => break ApprovalStatus::Approved,
                        ApprovalStatus::Rejected => break ApprovalStatus::Rejected,
                        ApprovalStatus::Pending => continue,
                    }
                } else {
                    break ApprovalStatus::Rejected;
                }
            };

            if decision == ApprovalStatus::Rejected {
                // Record step result and cancel run.
                let latency = step_start.elapsed().as_millis() as u64;
                step_results.push(StepResultEntry {
                    step_name: step.name.clone(),
                    tool_name: step.tool_name.clone(),
                    success: false,
                    output_summary: "Rejected by human".to_string(),
                    latency_ms: latency,
                });

                {
                    let mut runs = state.runs.write().await;
                    if let Some(r) = runs.get_mut(&run_id) {
                        r.run.status = RunStatus::Failed;
                    }
                }
                let _ = tx.send(AgentEvent::RunFinished { cost_usd: 0.0 }).await;

                // Record history before returning.
                let entry = RunHistoryEntry {
                    run_id,
                    agent_id,
                    status: RunStatus::Failed,
                    started_at: started_at.to_rfc3339(),
                    finished_at: Some(Utc::now().to_rfc3339()),
                    step_results,
                };
                state.run_history.write().await.push(entry);
                return;
            }

            // Approved — resume.
            {
                let mut runs = state.runs.write().await;
                if let Some(r) = runs.get_mut(&run_id) {
                    r.run.status = RunStatus::Running;
                }
            }
        }

        let latency = step_start.elapsed().as_millis() as u64;

        // Record step result for history.
        let output_summary = summarize_step_output(&step_result);
        step_results.push(StepResultEntry {
            step_name: step.name.clone(),
            tool_name: step.tool_name.clone(),
            success: tool_success,
            output_summary,
            latency_ms: latency,
        });

        // Emit step completion with actual result.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let _ = tx
            .send(AgentEvent::StepCompleted {
                result: step_result.clone(),
                latency_ms: latency,
            })
            .await;

        previous_step_output = Some(step_result.clone());

        if !tool_success {
            run_failed = true;
            break;
        }
    }

    // All steps finished.
    let final_status = if run_failed {
        RunStatus::Failed
    } else {
        RunStatus::Succeeded
    };
    {
        let mut runs = state.runs.write().await;
        if let Some(r) = runs.get_mut(&run_id) {
            r.run.status = final_status.clone();
        }
    }

    let _ = tx.send(AgentEvent::RunFinished { cost_usd: 0.042 }).await;

    // Persist run history.
    let entry = RunHistoryEntry {
        run_id,
        agent_id,
        status: final_status,
        started_at: started_at.to_rfc3339(),
        finished_at: Some(Utc::now().to_rfc3339()),
        step_results,
    };
    state.run_history.write().await.push(entry);
}

// ---------------------------------------------------------------------------
// POST /runs/:id/approve
// ---------------------------------------------------------------------------

/// Approves a pending human approval checkpoint for the given run.
///
/// The polling loop in `execute_run` picks up the status change within 500 ms
/// and resumes execution.
pub async fn approve_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let mut approvals = state.approvals.write().await;

    let approval = approvals
        .values_mut()
        .find(|a| a.run_id == run_id && a.status == ApprovalStatus::Pending)
        .ok_or(AppError::NotFound)?;

    approval.status = ApprovalStatus::Approved;
    let approval_id = approval.id;

    Ok(Json(json!({
        "status": "approved",
        "run_id": run_id,
        "approval_id": approval_id,
    })))
}

// ---------------------------------------------------------------------------
// POST /runs/:id/reject
// ---------------------------------------------------------------------------

/// Rejects a pending human approval checkpoint for the given run.
///
/// The polling loop in `execute_run` picks up the rejection within 500 ms,
/// marks the run as failed, and closes the SSE connection.
pub async fn reject_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let mut approvals = state.approvals.write().await;

    let approval = approvals
        .values_mut()
        .find(|a| a.run_id == run_id && a.status == ApprovalStatus::Pending)
        .ok_or(AppError::NotFound)?;

    approval.status = ApprovalStatus::Rejected;
    let approval_id = approval.id;

    Ok(Json(json!({
        "status": "rejected",
        "run_id": run_id,
        "approval_id": approval_id,
    })))
}

// ---------------------------------------------------------------------------
// Headless execution (used by the scheduler)
// ---------------------------------------------------------------------------

/// Executes an agent run without an SSE connection.
///
/// Creates a dummy channel, spawns the drain task, and runs the standard
/// `execute_run` loop. Used by the cron scheduler for automated runs.
pub async fn execute_run_headless(
    run_id: Uuid,
    agent_id: Uuid,
    steps: Vec<AgentStep>,
    tool_bindings: HashMap<Uuid, (String, Value)>,
    state: AppState,
) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    // Drain the receiver so the sender doesn't block.
    tokio::spawn(async move {
        while rx.recv().await.is_some() {}
    });
    execute_run(run_id, agent_id, steps, tool_bindings, tx, state).await;
}

#[cfg(test)]
mod tests {
    use super::{
        apply_runtime_tool_context, enrich_tool_arguments, extract_email_address,
        extract_message_body, summarize_step_output,
    };
    use serde_json::json;

    #[test]
    fn extracts_email_address_from_step_name() {
        let email = extract_email_address("Send an email to dev@example.com every 10 minutes saying hello")
            .expect("email should be inferred");
        assert_eq!(email, "dev@example.com");
    }

    #[test]
    fn extracts_message_body_from_saying_clause() {
        let body = extract_message_body("Send an email to dev@example.com saying hello from the agent")
            .expect("body should be inferred");
        assert_eq!(body, "hello from the agent");
    }

    #[test]
    fn enriches_google_send_gmail_arguments_from_step_name() {
        let args = enrich_tool_arguments(
            "Send an email to dev@example.com every 10 minutes saying hello",
            "google_send_gmail",
            None,
        );

        assert_eq!(args["to"], json!("dev@example.com"));
        assert_eq!(args["body"], json!("hello"));
        assert_eq!(args["subject"], json!("Automated message: hello"));
    }

    #[test]
    fn enriches_google_send_gmail_arguments_from_metadata_fields() {
        let args = enrich_tool_arguments(
            "Send the final Gmail update",
            "google_send_gmail",
            Some(json!({
                "label": "Final delivery",
                "description": "Email qa@example.com saying build passed all checks"
            })),
        );

        assert_eq!(args["to"], json!("qa@example.com"));
        assert_eq!(args["body"], json!("build passed all checks"));
        assert_eq!(args["subject"], json!("Automated message: build passed all checks"));
    }

    #[test]
    fn summarize_step_output_prefers_nested_error_message() {
        let summary = summarize_step_output(&json!({
            "tool": "google_send_gmail",
            "success": false,
            "output": { "error": "missing subject" }
        }));

        assert_eq!(summary, "missing subject");
    }

    #[test]
    fn runtime_context_populates_generate_content_prompt_and_target_tool() {
        let args = apply_runtime_tool_context(
            "Draft the customer follow-up email",
            "generate_content",
            json!({}),
            Some(&json!({
                "tool": "google_search_gmail",
                "output": { "messages": [{ "snippet": "Customer asked for an update" }] }
            })),
            Some("google_send_gmail"),
        );

        assert_eq!(args["prompt"], json!("Draft the customer follow-up email"));
        assert_eq!(args["target_tool"], json!("google_send_gmail"));
        assert!(args.get("context").is_some());
    }

    #[test]
    fn runtime_context_hands_generated_content_to_gmail() {
        let args = apply_runtime_tool_context(
            "Send the final Gmail update",
            "google_send_gmail",
            json!({ "to": "qa@example.com" }),
            Some(&json!({
                "tool": "generate_content",
                "output": {
                    "subject": "Weekly status",
                    "content": "Everything shipped on time."
                }
            })),
            None,
        );

        assert_eq!(args["to"], json!("qa@example.com"));
        assert_eq!(args["subject"], json!("Weekly status"));
        assert_eq!(args["body"], json!("Everything shipped on time."));
    }
}
