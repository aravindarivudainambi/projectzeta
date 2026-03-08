//! API gateway entry point.

mod app;
mod config;
mod errors;
mod middleware;
mod routes;
mod scheduler;
mod state;
mod tool_dispatch;

use anyhow::Result;
use tokio::net::TcpListener;

/// Boots the API gateway runtime, binds the public listener, and serves routes.
#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry("api-gateway")?;

    let config = config::Config::from_env()?;
    let (router, app_state) = app::build_router().await?;

    // Start the cron scheduler in the background.
    scheduler::spawn_scheduler(app_state);

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;

    axum::serve(listener, router).await?;
    Ok(())
}
