use std::collections::BTreeSet;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use core_types::agent::{AgentConfig, AgentStep, Trigger};
use core_types::run::RunHistoryEntry;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::AppError, state::AppState};

/// Request payload for creating and persisting an agent configuration.
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub trigger: Trigger,
    pub steps: Vec<CreateAgentStepRequest>,
}

/// Step payload accepted by the create-agent endpoint.
#[derive(Debug, Deserialize)]
pub struct CreateAgentStepRequest {
    pub name: String,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub requires_approval: bool,
}

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

/// Creates a new agent and stores it in the gateway's in-memory state.
pub async fn create_agent(
    State(state): State<AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "agent name must not be empty".to_string(),
        ));
    }

    if payload.steps.is_empty() {
        return Err(AppError::BadRequest(
            "at least one agent step is required".to_string(),
        ));
    }

    let config = AgentConfig {
        id: Uuid::new_v4(),
        name: payload.name.trim().to_string(),
        trigger: payload.trigger,
        steps: payload
            .steps
            .into_iter()
            .map(|step| AgentStep {
                id: Uuid::new_v4(),
                name: step.name,
                tool_name: step.tool_name.filter(|value| !value.trim().is_empty()),
                requires_approval: step.requires_approval,
            })
            .collect(),
    };

    {
        let mut agents = state.agents.write().await;
        agents.insert(config.id, config.clone());
    }

    Ok((StatusCode::CREATED, Json(config)))
}

/// Lists all persisted agent configurations.
pub async fn list_agents(
    State(state): State<AppState>,
) -> Json<Vec<AgentConfig>> {
    let agents = state.agents.read().await;
    Json(agents.values().cloned().collect())
}

/// Fetches a previously saved agent configuration.
pub async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentConfig>, AppError> {
    let agent = {
        let agents = state.agents.read().await;
        agents.get(&agent_id).cloned().ok_or(AppError::NotFound)?
    };

    Ok(Json(agent))
}

/// Lists run history entries for a specific agent, most recent first.
pub async fn list_agent_runs(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Json<Vec<RunHistoryEntry>> {
    let history = state.run_history.read().await;
    let mut agent_runs: Vec<_> = history
        .iter()
        .filter(|r| r.agent_id == agent_id)
        .cloned()
        .collect();
    agent_runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Json(agent_runs)
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
