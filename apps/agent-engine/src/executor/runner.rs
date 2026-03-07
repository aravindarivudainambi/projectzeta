use anyhow::Result;

/// Runs the high-level agent loop from planning through completion.
///
/// This function should eventually coordinate planner output, permission checks, tool calls,
/// approvals, and event emission in a deterministic sequence.
pub async fn run_agent() -> Result<()> {
    todo!("Implement the end-to-end agent orchestration loop.")
}
