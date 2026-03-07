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
use core_types::agent::AgentStep;
use core_types::events::AgentEvent;
use core_types::run::{AgentRun, ApprovalStatus, RunStatus};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
}

/// Response returned after a run is created.
#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub run_id: Uuid,
    pub status: String,
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
    let steps: Vec<AgentStep> = payload
        .steps
        .into_iter()
        .map(|s| AgentStep {
            id: Uuid::new_v4(),
            name: s.name,
            requires_approval: s.requires_approval,
        })
        .collect();

    let record = RunRecord {
        run: AgentRun {
            id: run_id,
            agent_id: payload.agent_id,
            status: RunStatus::Pending,
        },
        steps,
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
    let loop_state = state.clone();
    tokio::spawn(async move {
        execute_run(run_id, steps, tx, loop_state).await;
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

/// Executes an agent run step-by-step, sending events through the channel.
///
/// For each step the loop emits `StepStarted`, simulates work, emits `ToolCalled`,
/// and then `StepCompleted`. Steps tagged `requires_approval` pause execution until
/// a human decision is recorded via the approve/reject endpoints.
async fn execute_run(
    run_id: Uuid,
    steps: Vec<AgentStep>,
    tx: tokio::sync::mpsc::Sender<AgentEvent>,
    state: AppState,
) {
    for step in &steps {
        // Emit StepStarted.
        let _ = tx
            .send(AgentEvent::StepStarted {
                step_id: step.id,
                step_name: step.name.clone(),
            })
            .await;

        // Simulate tool execution.
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
                // Cancel the run.
                {
                    let mut runs = state.runs.write().await;
                    if let Some(r) = runs.get_mut(&run_id) {
                        r.run.status = RunStatus::Failed;
                    }
                }
                let _ = tx.send(AgentEvent::RunFinished { cost_usd: 0.0 }).await;
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

        // Simulate step completion.
        tokio::time::sleep(Duration::from_millis(500)).await;

        let _ = tx
            .send(AgentEvent::StepCompleted {
                result: json!({ "output": format!("{} completed successfully", step.name) }),
                latency_ms: 1500,
            })
            .await;
    }

    // All steps finished.
    {
        let mut runs = state.runs.write().await;
        if let Some(r) = runs.get_mut(&run_id) {
            r.run.status = RunStatus::Succeeded;
        }
    }

    let _ = tx.send(AgentEvent::RunFinished { cost_usd: 0.042 }).await;
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
