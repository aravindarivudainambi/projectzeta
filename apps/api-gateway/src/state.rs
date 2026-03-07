use std::collections::HashMap;
use std::sync::Arc;

use core_types::agent::AgentStep;
use core_types::run::{AgentRun, ApprovalStatus};
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory record for a run, pairing the run metadata with its step definitions.
#[derive(Debug, Clone)]
pub struct RunRecord {
    pub run: AgentRun,
    pub steps: Vec<AgentStep>,
}

/// In-memory record for a single approval checkpoint.
#[derive(Debug, Clone)]
pub struct ApprovalRecord {
    pub id: Uuid,
    pub run_id: Uuid,
    pub step_id: Uuid,
    pub action: String,
    pub status: ApprovalStatus,
}

/// Shared application state for the API gateway.
///
/// Uses in-memory storage behind `Arc<RwLock<HashMap>>` following the
/// auth-service pattern. Cloneable for injection via `Router::with_state`.
#[derive(Clone)]
pub struct AppState {
    pub runs: Arc<RwLock<HashMap<Uuid, RunRecord>>>,
    pub approvals: Arc<RwLock<HashMap<Uuid, ApprovalRecord>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
            approvals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
