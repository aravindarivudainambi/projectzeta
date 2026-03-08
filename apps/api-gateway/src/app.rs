use anyhow::Result;
use axum::{
    extract::Extension,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    middleware::auth::{self, TenantId, UserId},
    routes,
    state::AppState,
};

/// Builds the composed Axum router for the gateway surface.
///
/// The gateway exposes infrastructure health checks, agent token issuance,
/// run management with SSE streaming, human approval endpoints, and a
/// protected `/me` endpoint that demonstrates request authentication.
pub async fn build_router() -> Result<(Router, AppState)> {
    let state = AppState::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/health", get(routes::health::health_check))
        .route(
            "/agents",
            get(routes::agents::list_agents).post(routes::agents::create_agent),
        )
        .route("/agents/{id}", get(routes::agents::get_agent))
        .route(
            "/agents/{id}/runs",
            get(routes::agents::list_agent_runs),
        )
        .route(
            "/agents/{id}/token",
            post(routes::agents::issue_agent_token),
        )
        .route("/auth/dev-login", post(routes::auth::dev_login))
        .route("/agents/build", post(routes::build::build_agent))
        .route(
            "/marketplace",
            get(routes::marketplace::list_marketplace_templates),
        )
        .route(
            "/marketplace/fork",
            post(routes::marketplace::fork_marketplace_template),
        )
        .route("/runs", post(routes::runs::create_run))
        .route("/runs/{id}/stream", get(routes::runs::stream_run))
        .route("/runs/{id}/approve", post(routes::runs::approve_run))
        .route("/runs/{id}/reject", post(routes::runs::reject_run))
        .route(
            "/connectors",
            get(routes::connectors::list_connectors),
        )
        .route(
            "/connectors/notion/oauth-url",
            get(routes::connectors::notion_oauth_start),
        )
        .route(
            "/connectors/notion/callback",
            post(routes::connectors::notion_oauth_callback),
        )
        .route(
            "/connectors/google/oauth-url",
            get(routes::connectors::google_oauth_start),
        )
        .route(
            "/connectors/google/callback",
            post(routes::connectors::google_oauth_callback),
        )
        .route(
            "/me",
            get(me_handler).route_layer(middleware::from_fn(auth::auth_middleware)),
        )
        .with_state(state.clone())
        .layer(cors);

    Ok((router, state))
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
