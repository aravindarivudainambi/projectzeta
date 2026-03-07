//! Connector hub entry point.

mod adapters;
mod config;
mod hub_state;
mod mcp;
mod oauth;
mod registry;
mod routes;
mod vault;

use anyhow::Result;
use axum::{
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
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

    let state = hub_state::HubState::new(config.clone());

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/connectors/status", get(routes::connector_status))
        .route("/notion/execute", post(routes::execute_notion_tool))
        .route(
            "/oauth/notion/start",
            get(oauth::flow::start_notion_oauth),
        )
        .route(
            "/oauth/notion/callback",
            post(oauth::flow::notion_oauth_callback),
        )
        .with_state(state);

    axum::serve(listener, app).await?;
    Ok(())
}
