//! Connector hub entry point.

mod adapters;
mod config;
mod mcp;
mod oauth;
mod registry;
mod vault;

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

/// Boots the connector hub process and initializes shared dependencies.
#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry("connector-hub")?;
    let config = config::Config::from_env()?;

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    let app = Router::new().route("/health", get(health_check));

    axum::serve(listener, app).await?;
    Ok(())
}
