use anyhow::Result;

/// Produces the next agent step from the current execution context.
///
/// The real implementation should delegate to the shared LLM router and preserve planner traces.
#[allow(dead_code)]
pub async fn plan_next_step() -> Result<()> {
    Ok(())
}
