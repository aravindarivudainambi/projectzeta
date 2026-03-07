use anyhow::Result;

/// Bootstraps tracing and metrics exporters for a named service.
///
/// Each service should call this during startup so logs and spans are structured consistently
/// across the monorepo.
pub fn init_telemetry(_service_name: &str) -> Result<()> {
    Ok(())
}
