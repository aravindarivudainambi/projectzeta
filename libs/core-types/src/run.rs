use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Describes an agent execution run at a coarse-grained level.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentRun {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub status: RunStatus,
}

/// Describes an individual step inside an agent run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunStep {
    pub id: Uuid,
    pub label: String,
    pub status: RunStatus,
}

/// Represents the lifecycle state shared by runs and steps.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RunStatus {
    Pending,
    Running,
    WaitingForApproval,
    Succeeded,
    Failed,
}

/// Represents the decision state of a human approval checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

/// Returns a placeholder run object so API contracts can be exercised before execution logic exists.
pub fn sample_run(agent_id: Uuid) -> AgentRun {
    AgentRun {
        id: Uuid::nil(),
        agent_id,
        status: RunStatus::Pending,
    }
}
