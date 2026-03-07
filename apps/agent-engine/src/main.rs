//! Agent engine entry point.

mod config;
mod events;
mod executor;
mod human_loop;
mod memory;
mod versioning;

use anyhow::Result;

/// Boots the agent engine runtime and prepares the execution loop dependencies.
#[tokio::main]
async fn main() -> Result<()> {
    let _config = config::Config::from_env()?;
    telemetry::init_telemetry("agent-engine")?;
    Ok(())
}
