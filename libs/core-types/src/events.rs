use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Enumerates the typed events that can be streamed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AgentEvent {
    StepStarted { step_id: Uuid, step_name: String },
    ToolCalled { tool: String, args: Value },
    HumanApprovalRequired { action: String },
    StepCompleted { result: Value, latency_ms: u64 },
    RunFinished { cost_usd: f64 },
}

/// Serializes a placeholder event payload for transport-layer scaffolding.
///
/// Real event formatting should be centralized later, but a stable placeholder is useful
/// while the SSE contract is still taking shape.
pub fn placeholder_event() -> AgentEvent {
    AgentEvent::RunFinished { cost_usd: 0.0 }
}
