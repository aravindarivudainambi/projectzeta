use anyhow::Result;
use axum::{extract::Extension, middleware, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::{
    middleware::auth::{self, TenantId, UserId},
    routes,
};

/// Builds the composed Axum router for the gateway surface.
///
/// The gateway exposes infrastructure health checks and a protected `/me`
/// endpoint that demonstrates request authentication context injection.
pub async fn build_router() -> Result<Router> {
    Ok(Router::new()
        .route("/health", get(routes::health::health_check))
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use super::build_router;

    fn create_token(user_id: &str, tenant_id: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(
            json!({
                "sub": user_id,
                "tenant_id": tenant_id,
            })
            .to_string(),
        );

        format!("{header}.{payload}.signature")
    }

    /// Verifies that `/me` returns the injected user and tenant IDs for a valid bearer token.
    #[tokio::test]
    async fn me_returns_identity_for_valid_token() {
        let app = build_router().await.expect("router should build");

        let user_id = "550e8400-e29b-41d4-a716-446655440000";
        let tenant_id = "2d35b702-5f53-4a3f-9ea4-0c6e770f6571";
        let token = create_token(user_id, tenant_id);

        let request = Request::builder()
            .uri("/me")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(request).await.expect("request should succeed");
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let body: Value = serde_json::from_slice(&bytes).expect("body should be valid JSON");

        assert_eq!(body.get("user_id").and_then(Value::as_str), Some(user_id));
        assert_eq!(
            body.get("tenant_id").and_then(Value::as_str),
            Some(tenant_id)
        );
    }

    /// Verifies that `/me` returns 401 with the expected error body when auth is missing.
    #[tokio::test]
    async fn me_returns_unauthorized_without_token() {
        let app = build_router().await.expect("router should build");

        let request = Request::builder()
            .uri("/me")
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(request).await.expect("request should succeed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let body: Value = serde_json::from_slice(&bytes).expect("body should be valid JSON");

        assert_eq!(body, json!({ "error": "unauthorized" }));
    }
}
