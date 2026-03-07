use anyhow::Result;

/// Returns a shallow liveness response for infrastructure health checks.
pub async fn health_check() -> Result<()> {
    Ok(())
}

/// Returns a deeper readiness response once dependencies have been verified.
pub async fn readiness_check() -> Result<()> {
    todo!("Probe downstream dependencies before reporting readiness.")
}
