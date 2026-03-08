use anyhow::{ensure, Result};

/// Enforces per-user or per-tenant rate limits for inbound requests.
///
/// The placeholder captures the intended boundary without selecting a concrete storage-backed
/// token bucket strategy yet.
#[allow(dead_code)]
pub async fn enforce_rate_limit(subject_key: &str) -> Result<()> {
    ensure!(
        !subject_key.trim().is_empty(),
        "rate limit subject key cannot be empty"
    );
    Ok(())
}
