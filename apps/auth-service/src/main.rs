//! Auth service entry point.

mod config;
mod rbac;
mod tokens;
mod users;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::RwLock};
use uuid::Uuid;

use crate::{tokens::issue::AccessTokenClaims, users::UserRecord};

#[derive(Clone)]
struct AppState {
    users: Arc<RwLock<HashMap<String, UserRecord>>>,
    jwt_signing_secret: String,
    default_tenant_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
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

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<users::RegisterRequest>,
) -> Response {
    let email = users::normalize_email(&payload.email);
    if email.is_empty() || payload.password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "email and password are required",
            }),
        )
            .into_response();
    }

    let mut store = state.users.write().await;
    if store.contains_key(&email) {
        return (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "user already exists",
            }),
        )
            .into_response();
    }

    let password_hash = match users::hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "password hashing failed",
                }),
            )
                .into_response();
        }
    };

    let user = UserRecord {
        id: Uuid::new_v4(),
        tenant_id: state.default_tenant_id,
        email: email.clone(),
        password_hash,
    };

    store.insert(email.clone(), user.clone());

    (
        StatusCode::CREATED,
        Json(users::RegisterResponse {
            user_id: user.id,
            email,
        }),
    )
        .into_response()
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    let email = users::normalize_email(&payload.email);

    let store = state.users.read().await;
    let Some(user) = store.get(&email) else {
        return unauthorized_response();
    };

    if !users::verify_password(&user.password_hash, &payload.password) {
        return unauthorized_response();
    }

    match tokens::issue::issue_token(
        &user.id.to_string(),
        &user.tenant_id.to_string(),
        &state.jwt_signing_secret,
    )
    .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "token issuance failed",
            }),
        )
            .into_response(),
    }
}

async fn protected_route(Extension(claims): Extension<AccessTokenClaims>) -> impl IntoResponse {
    Json(serde_json::json!({
        "user_id": claims.sub,
        "tenant_id": claims.tenant_id,
    }))
}

async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let Some(token) = token else {
        return unauthorized_response();
    };

    let claims = match tokens::validate::validate_token(token, &state.jwt_signing_secret).await {
        Ok(claims) => claims,
        Err(_) => return unauthorized_response(),
    };

    request.extensions_mut().insert(claims);
    next.run(request).await
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "unauthorized",
        }),
    )
        .into_response()
}

/// Boots the auth service process and initializes shared instrumentation.
#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry("auth-service")?;

    let config = config::Config::from_env()?;
    let state = AppState {
        users: Arc::new(RwLock::new(HashMap::new())),
        jwt_signing_secret: config.jwt_signing_secret,
        default_tenant_id: Uuid::new_v4(),
    };

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route(
            "/auth/protected",
            get(protected_route).route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            )),
        )
        .with_state(state);

    axum::serve(listener, app).await?;
    Ok(())
}
