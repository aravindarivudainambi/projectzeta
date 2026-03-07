use std::collections::BTreeSet;

use axum::{extract::Path, http::StatusCode, Json};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

/// Request payload for issuing an agent-scoped JWT.
#[derive(Debug, Deserialize)]
pub struct IssueAgentTokenRequest {
    pub tenant_id: String,
    pub scopes: Vec<String>,
    pub expires_in_seconds: Option<u64>,
}

/// Response payload returned after issuing an agent token.
#[derive(Debug, Serialize)]
pub struct IssueAgentTokenResponse {
    pub token: String,
    pub token_type: &'static str,
    pub expires_at: usize,
}

/// Canonical JWT claims used for agent execution authorization.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTokenClaims {
    pub agent_id: String,
    pub tenant_id: String,
    pub scopes: Vec<String>,
    pub iat: usize,
    pub exp: usize,
}

/// Issues a tenant-scoped JWT for a specific agent.
pub async fn issue_agent_token(
    Path(agent_id): Path<String>,
    Json(payload): Json<IssueAgentTokenRequest>,
) -> Result<Json<IssueAgentTokenResponse>, (StatusCode, String)> {
    if payload.tenant_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "tenant_id must not be empty".to_string(),
        ));
    }

    let scopes = normalize_scopes(payload.scopes)?;
    let now = unix_timestamp_now()?;
    let expires_in = payload.expires_in_seconds.unwrap_or(3600);
    let expires_in_usize = usize::try_from(expires_in).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "expires_in_seconds is too large".to_string(),
        )
    })?;
    let exp = now.checked_add(expires_in_usize).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "expires_in_seconds is too large".to_string(),
        )
    })?;

    let claims = AgentTokenClaims {
        agent_id,
        tenant_id: payload.tenant_id.trim().to_string(),
        scopes,
        iat: now,
        exp,
    };

    let secret = std::env::var("AGENT_TOKEN_SIGNING_SECRET")
        .unwrap_or_else(|_| "dev-agent-token-signing-secret".to_string());

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(internal_error)?;

    Ok(Json(IssueAgentTokenResponse {
        token,
        token_type: "Bearer",
        expires_at: exp,
    }))
}

fn normalize_scopes(scopes: Vec<String>) -> Result<Vec<String>, (StatusCode, String)> {
    let normalized: BTreeSet<String> = scopes
        .into_iter()
        .map(|scope| scope.trim().to_string())
        .filter(|scope| !scope.is_empty())
        .collect();

    if normalized.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "at least one scope is required".to_string(),
        ));
    }

    Ok(normalized.into_iter().collect())
}

fn unix_timestamp_now() -> Result<usize, (StatusCode, String)> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(internal_error)?
        .as_secs();

    usize::try_from(now).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "current timestamp does not fit platform usize".to_string(),
        )
    })
}

fn internal_error<E: std::fmt::Display>(error: E) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to issue agent token: {error}"),
    )
}
