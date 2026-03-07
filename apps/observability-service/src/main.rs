//! Observability service entry point.

mod aggregates;
mod collector;
mod config;
mod cost;

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

/// Boots the observability service process and initializes telemetry.
#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry("observability-service")?;
    let config = config::Config::from_env()?;

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    let app = Router::new().route("/health", get(health_check));

    axum::serve(listener, app).await?;
    Ok(())
}
