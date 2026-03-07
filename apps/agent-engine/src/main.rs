//! Agent engine entry point.

mod config;
mod events;
mod executor;
mod human_loop;
mod memory;
mod versioning;

use anyhow::Result;
use axum::{
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::net::TcpListener;

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        r#"{"status":"ok"}"#,
    )
}

/// Boots the agent engine runtime and prepares the execution loop dependencies.
#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry("agent-engine")?;
    let config = config::Config::from_env()?;

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    let app = Router::new().route("/health", get(health_check));

    axum::serve(listener, app).await?;
    Ok(())
}
