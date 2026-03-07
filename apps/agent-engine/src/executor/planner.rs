use anyhow::Result;

/// Produces the next agent step from the current execution context.
///
/// The real implementation should delegate to the shared LLM router and preserve planner traces.
pub async fn plan_next_step() -> Result<()> {
    todo!("Implement planner prompting and typed step generation.")
}
