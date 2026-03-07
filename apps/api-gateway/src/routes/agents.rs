use std::collections::BTreeSet;

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

/// Request payload for issuing an agent-scoped JWT.
///
/// The caller must provide the tenant that owns the agent as well as the
/// exact connector scopes that should be embedded in the issued token.
#[derive(Debug, Deserialize)]
pub struct IssueAgentTokenRequest {
    /// Tenant identifier that scopes all access made with this token.
    pub tenant_id: String,
    /// Explicit allow-list of tool scopes granted to the token.
    pub scopes: Vec<String>,
    /// Optional token lifetime in seconds; defaults to one hour.
    pub expires_in_seconds: Option<u64>,
}

/// Response payload returned after issuing an agent token.
#[derive(Debug, Serialize)]
pub struct IssueAgentTokenResponse {
    /// Signed JWT that downstream services can present as bearer auth.
    pub token: String,
    /// Type marker for HTTP authorization headers.
    pub token_type: &'static str,
    /// Absolute UNIX epoch timestamp when the token expires.
    pub expires_at: usize,
}

/// Canonical JWT claims used for agent execution authorization.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTokenClaims {
    /// Stable agent identifier from the route path.
    pub agent_id: String,
    /// Tenant identifier bound to the token.
    pub tenant_id: String,
    /// Scope set granted to the token.
    pub scopes: Vec<String>,
    /// Issued-at UNIX epoch timestamp.
    pub iat: usize,
    /// Expiration UNIX epoch timestamp.
    pub exp: usize,
}

/// Handles agent creation requests coming from the frontend or API clients.
pub async fn create_agent() -> anyhow::Result<()> {
    todo!("Validate payloads, persist the agent definition, and emit audit events.")
}

/// Handles retrieval of a single agent record and its current version metadata.
pub async fn get_agent() -> anyhow::Result<()> {
    todo!("Load the agent and related state from tenant-scoped storage.")
}

/// Issues a tenant-scoped JWT for a specific agent.
///
/// The generated token includes `agent_id`, `tenant_id`, and the exact `scopes`
/// supplied in the request payload. Scopes are normalized (trimmed, deduplicated,
/// and sorted) before token issuance.
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
    let now_u64 = u64::try_from(now).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "current timestamp does not fit in u64".to_string(),
        )
    })?;
    let exp_u64 = now_u64.checked_add(expires_in).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "expires_in_seconds is too large".to_string(),
        )
    })?;
    let exp = usize::try_from(exp_u64).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "expires_at does not fit platform usize".to_string(),
        )
    })?;

    let claims = AgentTokenClaims {
        agent_id,
        tenant_id: payload.tenant_id.trim().to_string(),
        scopes,
        iat: now,
        exp,
    };

    let secret = signing_secret_from_env()?;

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

/// Produces a normalized scope set from raw user input.
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

/// Returns the current UNIX timestamp in seconds.
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

/// Maps internal errors into a generic HTTP 500 response shape.
fn internal_error<E: std::fmt::Display>(error: E) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to issue agent token: {error}"),
    )
}

/// Loads and validates the configured agent token signing secret.
fn signing_secret_from_env() -> Result<String, (StatusCode, String)> {
    let secret = std::env::var("AGENT_TOKEN_SIGNING_SECRET").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "AGENT_TOKEN_SIGNING_SECRET must be configured".to_string(),
        )
    })?;

    if secret.len() < 32 {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "AGENT_TOKEN_SIGNING_SECRET must be at least 32 characters".to_string(),
        ));
    }

    Ok(secret)
}

impl IntoResponse for IssueAgentTokenResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;
    use jsonwebtoken::{decode, DecodingKey, Validation};

    const TEST_SIGNING_SECRET: &str = "integration-secret-with-32-chars";

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn normalize_scopes_deduplicates_and_sorts() {
        let scopes = vec![
            "tool:github.read".to_string(),
            "tool:slack.post".to_string(),
            " tool:github.read ".to_string(),
        ];

        let normalized = normalize_scopes(scopes).expect("expected valid scopes");

        assert_eq!(
            normalized,
            vec![
                "tool:github.read".to_string(),
                "tool:slack.post".to_string()
            ]
        );
    }

    #[test]
    fn normalize_scopes_rejects_empty_input() {
        let error = normalize_scopes(vec!["   ".to_string()]).expect_err("expected invalid scopes");

        assert_eq!(error.0, StatusCode::BAD_REQUEST);
        assert_eq!(error.1, "at least one scope is required");
    }

    #[tokio::test]
    async fn issued_token_binds_to_path_agent_id() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        std::env::set_var("AGENT_TOKEN_SIGNING_SECRET", TEST_SIGNING_SECRET);

        let response = issue_agent_token(
            Path("agent-a".to_string()),
            Json(IssueAgentTokenRequest {
                tenant_id: "tenant-1".to_string(),
                scopes: vec![
                    "tool:github.read".to_string(),
                    "tool:slack.post".to_string(),
                ],
                expires_in_seconds: Some(600),
            }),
        )
        .await
        .expect("expected token issuance to succeed");

        let token = response.0.token;
        let mut validation = Validation::new(Algorithm::HS256);
        // Expiration is intentionally not validated so the test only verifies claim contents.
        validation.validate_exp = false;

        let decoded = decode::<AgentTokenClaims>(
            &token,
            &DecodingKey::from_secret(TEST_SIGNING_SECRET.as_bytes()),
            &validation,
        )
        .expect("expected token to decode");

        assert_eq!(decoded.claims.agent_id, "agent-a");
        assert_eq!(decoded.claims.tenant_id, "tenant-1");
        assert_eq!(
            decoded.claims.scopes,
            vec![
                "tool:github.read".to_string(),
                "tool:slack.post".to_string()
            ]
        );

        std::env::remove_var("AGENT_TOKEN_SIGNING_SECRET");
    }

    #[tokio::test]
    async fn issue_agent_token_rejects_blank_tenant_id() {
        let response = issue_agent_token(
            Path("agent-a".to_string()),
            Json(IssueAgentTokenRequest {
                tenant_id: "   ".to_string(),
                scopes: vec!["tool:github.read".to_string()],
                expires_in_seconds: Some(600),
            }),
        )
        .await
        .expect_err("expected tenant validation failure");

        assert_eq!(response.0, StatusCode::BAD_REQUEST);
        assert_eq!(response.1, "tenant_id must not be empty");
    }

    #[tokio::test]
    async fn issue_agent_token_rejects_short_signing_secret() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        std::env::set_var("AGENT_TOKEN_SIGNING_SECRET", "short-test-secret");

        let response = issue_agent_token(
            Path("agent-a".to_string()),
            Json(IssueAgentTokenRequest {
                tenant_id: "tenant-1".to_string(),
                scopes: vec!["tool:github.read".to_string()],
                expires_in_seconds: Some(600),
            }),
        )
        .await
        .expect_err("expected signing secret validation failure");

        assert_eq!(response.0, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.1,
            "AGENT_TOKEN_SIGNING_SECRET must be at least 32 characters"
        );

        std::env::remove_var("AGENT_TOKEN_SIGNING_SECRET");
    }

    #[test]
    fn token_round_trip_contains_expected_claims() {
        let claims = AgentTokenClaims {
            agent_id: "agent-a".to_string(),
            tenant_id: "tenant-1".to_string(),
            scopes: vec![
                "tool:github.read".to_string(),
                "tool:slack.post".to_string(),
            ],
            iat: 1,
            exp: 3601,
        };

        let secret = "test-secret-with-32-characters!!";
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("expected token to encode");

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;

        let decoded = decode::<AgentTokenClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .expect("expected token to decode");

        assert_eq!(decoded.claims.agent_id, "agent-a");
        assert_eq!(decoded.claims.tenant_id, "tenant-1");
        assert_eq!(
            decoded.claims.scopes,
            vec![
                "tool:github.read".to_string(),
                "tool:slack.post".to_string()
            ]
        );
    }
}
