use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};

use crate::routes;

/// Builds the composed Axum router for the gateway surface.
///
/// The gateway currently exposes only a health endpoint while broader route
/// composition remains under construction.
pub async fn build_router() -> Result<Router> {
    Ok(Router::new()
        .route("/health", get(routes::health::health_check))
        .route(
            "/agents/{id}/token",
            post(routes::agents::issue_agent_token),
        ))
}
