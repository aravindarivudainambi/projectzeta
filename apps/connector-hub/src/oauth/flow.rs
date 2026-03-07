use axum::{extract::State, http::StatusCode, Json};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

use crate::hub_state::HubState;

/// Response containing the Notion OAuth authorization URL.
#[derive(Debug, Serialize)]
pub struct OAuthStartResponse {
    pub redirect_url: String,
}

/// Payload sent from the frontend after receiving the authorization code.
#[derive(Debug, Deserialize)]
pub struct NotionCallbackPayload {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NotionTokenResponse {
    access_token: String,
    workspace_name: Option<String>,
    bot_id: Option<String>,
}

/// Returns the Notion OAuth authorization URL for the frontend to redirect to.
pub async fn start_notion_oauth(
    State(state): State<HubState>,
) -> Result<Json<OAuthStartResponse>, (StatusCode, String)> {
    let client_id = state.config.notion_client_id.as_deref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "NOTION_CLIENT_ID not set".to_string(),
    ))?;
    let redirect_uri = state.config.notion_redirect_uri.as_deref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "NOTION_REDIRECT_URI not set".to_string(),
    ))?;

    let url = format!(
        "https://api.notion.com/v1/oauth/authorize?client_id={}&response_type=code&owner=user&redirect_uri={}",
        client_id,
        urlencoding::encode(redirect_uri),
    );

    Ok(Json(OAuthStartResponse { redirect_url: url }))
}

/// Exchanges the authorization code for an access token and stores it in the vault.
pub async fn notion_oauth_callback(
    State(state): State<HubState>,
    Json(payload): Json<NotionCallbackPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let client_id = state.config.notion_client_id.as_deref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "NOTION_CLIENT_ID not set".to_string(),
    ))?;
    let client_secret = state.config.notion_client_secret.as_deref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "NOTION_CLIENT_SECRET not set".to_string(),
    ))?;
    let redirect_uri = state.config.notion_redirect_uri.as_deref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "NOTION_REDIRECT_URI not set".to_string(),
    ))?;

    // Notion uses HTTP Basic Auth with client_id:client_secret for token exchange.
    let basic_auth = STANDARD.encode(format!("{client_id}:{client_secret}"));

    let http = reqwest::Client::new();
    let resp = http
        .post("https://api.notion.com/v1/oauth/token")
        .header("Authorization", format!("Basic {basic_auth}"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "code": payload.code,
            "redirect_uri": redirect_uri,
        }))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("Notion token exchange failed: {body}"),
        ));
    }

    let token_resp: NotionTokenResponse = resp
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // Store the access token in the in-memory vault.
    let user_id = payload
        .state
        .and_then(|s| uuid::Uuid::parse_str(&s).ok())
        .unwrap_or(uuid::Uuid::nil());
    state
        .vault
        .set_token(user_id, "notion", &token_resp.access_token);

    Ok(Json(serde_json::json!({
        "success": true,
        "workspace_name": token_resp.workspace_name,
        "bot_id": token_resp.bot_id,
    })))
}
