use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Enumerates the typed events that can be streamed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AgentEvent {
    StepStarted { label: String },
    ToolCalled { tool_name: String },
    HumanApprovalRequired { reason: String },
    Finished { status: String },
}

/// Serializes a placeholder event payload for transport-layer scaffolding.
///
/// Real event formatting should be centralized later, but a stable placeholder is useful
/// while the SSE contract is still taking shape.
pub fn placeholder_event() -> AgentEvent {
    AgentEvent::Finished {
        status: "placeholder".to_string(),
    }
}
