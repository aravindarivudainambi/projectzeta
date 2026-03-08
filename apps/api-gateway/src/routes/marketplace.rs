use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use core_types::{
    agent::{AgentConfig, AgentStep},
    marketplace::{sample_marketplace_templates, MarketplaceTemplate},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{errors::AppError, state::AppState};

/// Request payload for forking a curated marketplace template.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForkMarketplaceTemplateRequest {
    pub template_id: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Returns the curated marketplace catalog exposed by the gateway.
///
/// The current scaffold keeps the catalog in typed in-memory sample data so the
/// frontend can integrate against a stable JSON contract before persistence,
/// search, ranking, and tenant-specific publishing rules are implemented.
pub async fn list_marketplace_templates() -> Json<Vec<MarketplaceTemplate>> {
    Json(sample_marketplace_templates())
}

/// Creates a new saved agent by cloning a marketplace template definition.
///
/// The forked agent is persisted into the gateway's in-memory agent store using
/// the same `AgentConfig` contract as the builder save flow, which allows the
/// web client to redirect straight into the existing agent detail surface.
pub async fn fork_marketplace_template(
    State(state): State<AppState>,
    Json(payload): Json<ForkMarketplaceTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let template_id = payload.template_id.trim();
    if template_id.is_empty() {
        return Err(AppError::BadRequest(
            "templateId must not be empty".to_string(),
        ));
    }

    let template = sample_marketplace_templates()
        .into_iter()
        .find(|template| template.id == template_id)
        .ok_or(AppError::NotFound)?;

    let agent_name = match payload.name.as_deref().map(str::trim) {
        Some("") => {
            return Err(AppError::BadRequest(
                "name must not be empty when provided".to_string(),
            ))
        }
        Some(name) => name.to_string(),
        None => template.name.clone(),
    };

    let config = AgentConfig {
        id: Uuid::new_v4(),
        name: agent_name,
        trigger: template.trigger,
        steps: template
            .agent_steps
            .into_iter()
            .map(|step| AgentStep {
                id: Uuid::new_v4(),
                name: step.name,
                tool_name: step.tool_name,
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
