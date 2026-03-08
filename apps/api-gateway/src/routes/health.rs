use axum::{
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
};

/// Returns a shallow liveness response for infrastructure health checks.
///
/// The response is intentionally static so load balancers and orchestrators can
/// verify process availability without touching downstream dependencies.
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        r#"{"status":"ok"}"#,
    )
}

/// Returns a deeper readiness response once dependencies have been verified.
#[allow(dead_code)]
pub async fn readiness_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        r#"{"status":"ready"}"#,
    )
}
