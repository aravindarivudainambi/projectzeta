use anyhow::Result;
use axum::{
    extract::Extension,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

use crate::{
    middleware::auth::{self, TenantId, UserId},
    routes,
};

/// Builds the composed Axum router for the gateway surface.
///
/// The gateway exposes infrastructure health checks, agent token issuance, and
/// a protected `/me` endpoint that demonstrates request authentication context
/// injection.
pub async fn build_router() -> Result<Router> {
    Ok(Router::new()
        .route("/health", get(routes::health::health_check))
        .route(
            "/agents/{id}/token",
            post(routes::agents::issue_agent_token),
        )
        .route(
            "/agents/build",
            post(routes::build::build_agent),
        )
        .route(
            "/me",
            get(me_handler).route_layer(middleware::from_fn(auth::auth_middleware)),
        ))
}

/// Returns the authenticated identity extracted by auth middleware.
///
/// This endpoint is intended as a contract test target for verifying that the
/// auth middleware decodes bearer claims and injects typed extensions.
async fn me_handler(
    Extension(user_id): Extension<UserId>,
    Extension(tenant_id): Extension<TenantId>,
) -> impl IntoResponse {
    Json(json!({
        "user_id": user_id.0,
        "tenant_id": tenant_id.0,
    }))
}
