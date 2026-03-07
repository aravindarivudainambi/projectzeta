//! API gateway entry point.

mod app;
mod config;
mod errors;
mod middleware;
mod routes;

use anyhow::Result;

/// Boots the API gateway runtime and prepares the router scaffold.
///
/// The function intentionally stops short of binding a network listener because the task is to
/// establish layout and contracts, not production behavior.
#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::from_env()?;
    let _router = app::build_router(&config).await?;
    telemetry::init_telemetry("api-gateway")?;
    Ok(())
}
