//! Connector hub entry point.

mod adapters;
mod config;
mod mcp;
mod oauth;
mod registry;
mod vault;

use anyhow::Result;

/// Boots the connector hub process and initializes shared dependencies.
#[tokio::main]
async fn main() -> Result<()> {
    let _config = config::Config::from_env()?;
    telemetry::init_telemetry("connector-hub")?;
    Ok(())
}
