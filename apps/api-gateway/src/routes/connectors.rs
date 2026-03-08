use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state::AppState;

const CONNECTOR_HUB_URL: &str = "http://localhost:8082";

/// Information about a single connector integration.
#[derive(Debug, Serialize)]
pub struct ConnectorInfo {
    pub name: String,
    pub display_name: String,
    pub connected: bool,
}

/// Mirror of the connector hub's vault-based status response.
#[derive(Debug, Deserialize)]
struct HubConnectorStatus {
    #[serde(default)]
    notion: bool,
    #[serde(default)]
    google_workspace: bool,
}

/// Returns the list of available connectors with live connection status
/// sourced from the connector hub's vault.
pub async fn list_connectors(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConnectorInfo>>, (StatusCode, String)> {
    // Query the connector hub for real-time vault status.
    // Fall back to the gateway's static env var if the hub is unreachable.
    let fallback = HubConnectorStatus {
        notion: state.mock_notion_token.is_some(),
        google_workspace: state.mock_google_token.is_some(),
    };

    let status = match state
        .http_client
        .get(format!("{CONNECTOR_HUB_URL}/connectors/status"))
        .send()
        .await
    {
        Ok(resp) => resp.json::<HubConnectorStatus>().await.unwrap_or(fallback),
        Err(_) => fallback,
    };

    Ok(Json(vec![
        ConnectorInfo {
            name: "notion".into(),
            display_name: "Notion".into(),
            connected: status.notion,
        },
        ConnectorInfo {
            name: "google_workspace".into(),
            display_name: "Google Workspace".into(),
            connected: status.google_workspace,
        },
    ]))
}

/// Proxies to connector hub's Notion OAuth start endpoint.
pub async fn notion_oauth_start(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let resp = state
        .http_client
        .get(format!("{CONNECTOR_HUB_URL}/oauth/notion/start"))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(body))
}

/// Proxies the OAuth callback code exchange to connector hub.
pub async fn notion_oauth_callback(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let resp = state
        .http_client
        .post(format!("{CONNECTOR_HUB_URL}/oauth/notion/callback"))
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(body))
}

/// Proxies to connector hub's Google OAuth start endpoint.
pub async fn google_oauth_start(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let resp = state
        .http_client
        .get(format!("{CONNECTOR_HUB_URL}/oauth/google/start"))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(body))
}

/// Proxies the Google OAuth callback code exchange to connector hub.
pub async fn google_oauth_callback(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let resp = state
        .http_client
        .post(format!("{CONNECTOR_HUB_URL}/oauth/google/callback"))
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(body))
}
