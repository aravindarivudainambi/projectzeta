//! Auth service entry point.

mod config;
mod rbac;
mod tokens;
mod users;

use anyhow::Result;
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    jwt_signing_secret: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

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

async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let Some(user) = users::get_user_by_email(&payload.email).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let password_valid =
        argon2::verify_encoded(&user.password_hash, payload.password.as_bytes()).unwrap_or(false);

    if !password_valid {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match tokens::issue::issue_token(&user.id, &user.tenant_id, &app_state.jwt_signing_secret).await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Boots the auth service process and initializes shared instrumentation.
#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry("auth-service")?;
    let config = config::Config::from_env()?;

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    let app_state = AppState {
        jwt_signing_secret: config.jwt_signing_secret,
    };
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", post(login))
        .with_state(app_state);

    axum::serve(listener, app).await?;
    Ok(())
}
