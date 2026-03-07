//! Observability service entry point.

mod aggregates;
mod collector;
mod config;
mod cost;

use anyhow::Result;

/// Boots the observability service process and initializes telemetry.
#[tokio::main]
async fn main() -> Result<()> {
    let _config = config::Config::from_env()?;
    telemetry::init_telemetry("observability-service")?;
    Ok(())
}
