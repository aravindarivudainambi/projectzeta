//! API gateway entry point.

mod app;
mod config;
mod errors;
mod middleware;
mod routes;

use anyhow::Result;
use tokio::net::TcpListener;

/// Boots the API gateway runtime, binds the public listener, and serves routes.
#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry("api-gateway")?;

    let _config = config::Config::from_env()?;
    let router = app::build_router().await?;
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, router).await?;
    Ok(())
}
