use axum::{http::StatusCode, Json};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request body for dev login.
#[derive(Debug, Deserialize)]
pub struct DevLoginRequest {
    pub email: Option<String>,
    pub name: Option<String>,
}

/// JWT claims matching the structure expected by `auth_middleware`.
#[derive(Debug, Serialize, Deserialize)]
struct DevLoginClaims {
    sub: String,
    tenant_id: String,
    email: Option<String>,
    name: Option<String>,
    iat: usize,
    exp: usize,
}

/// Response body with the issued JWT.
#[derive(Debug, Serialize)]
pub struct DevLoginResponse {
    pub token: String,
    pub user_id: String,
    pub tenant_id: String,
}

/// Issues a dev JWT for local development and demos.
///
/// Accepts `{ email }` or `{ name }` and returns a signed JWT whose `sub` and
/// `tenant_id` claims are deterministic UUIDs derived from the identifier.
/// The token is valid for 24 hours and uses the same HS256 signing secret as
/// agent token issuance.
pub async fn dev_login(
    Json(payload): Json<DevLoginRequest>,
) -> Result<Json<DevLoginResponse>, (StatusCode, String)> {
    let identifier = payload
        .email
        .as_deref()
        .or(payload.name.as_deref())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "email or name required".to_string(),
        ))?;

    if identifier.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "identifier must not be empty".to_string(),
        ));
    }

    let user_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, identifier.as_bytes());
    let tenant_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, identifier.as_bytes());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .as_secs() as usize;

    let claims = DevLoginClaims {
        sub: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        email: payload.email,
        name: payload.name,
        iat: now,
        exp: now + 86400,
    };

    let secret = std::env::var("AGENT_TOKEN_SIGNING_SECRET")
        .unwrap_or_else(|_| "dev-agent-token-signing-secret".to_string());

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to sign token: {e}"),
        )
    })?;

    Ok(Json(DevLoginResponse {
        token,
        user_id: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
    }))
}
