use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::adapters::google_workspace::CreateEventRequest;
use crate::hub_state::HubState;

/// Returns which connectors currently have a token stored in the vault.
#[derive(Debug, Serialize)]
pub struct ConnectorStatus {
    pub notion: bool,
    pub slack: bool,
    pub google_workspace: bool,
    pub github: bool,
    pub discord: bool,
}

/// Reports live connection status by checking the vault for each provider.
pub async fn connector_status(
    State(state): State<HubState>,
) -> Json<ConnectorStatus> {
    Json(ConnectorStatus {
        notion: state.vault.has_token("notion"),
        slack: state.vault.has_token("slack"),
        google_workspace: state.vault.has_token("google_workspace"),
        github: state.vault.has_token("github"),
        discord: state.vault.has_token("discord"),
    })
}

/// Request payload for the unified Notion tool dispatch endpoint.
#[derive(Debug, Deserialize)]
pub struct NotionExecuteRequest {
    pub tool_name: String,
    pub arguments: Value,
    pub token: String,
}

/// Response returned from every Notion tool invocation.
#[derive(Debug, Serialize)]
pub struct NotionExecuteResponse {
    pub success: bool,
    pub output: Value,
}

/// Dispatches a Notion tool invocation to the appropriate `NotionClient` method.
///
/// Accepts a JSON body with `tool_name`, `arguments`, and `token`. Routes to
/// the matching operation based on the tool name and returns a structured
/// success/failure response.
pub async fn execute_notion_tool(
    State(state): State<HubState>,
    Json(req): Json<NotionExecuteRequest>,
) -> Result<Json<NotionExecuteResponse>, (StatusCode, String)> {
    let client = &state.notion_client;

    // If the caller provided a token, use it. Otherwise fall back to the vault.
    let token = if req.token.is_empty() {
        state
            .vault
            .get_token(uuid::Uuid::nil(), "notion")
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        req.token.clone()
    };

    let result = match req.tool_name.as_str() {
        "notion_query_database" => {
            let db_id = req.arguments["database_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing database_id".to_string()))?;
            let filter = req.arguments.get("filter").cloned();
            let sorts = req.arguments.get("sorts").cloned();
            client.query_database(&token, db_id, filter, sorts).await
        }
        "notion_create_page" => client.create_page(&token, req.arguments).await,
        "notion_retrieve_page" => {
            let page_id = req.arguments["page_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing page_id".to_string()))?;
            client.retrieve_page(&token, page_id).await
        }
        "notion_update_page" => {
            let page_id = req.arguments["page_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing page_id".to_string()))?;
            let properties = req
                .arguments
                .get("properties")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));
            client.update_page(&token, page_id, properties).await
        }
        "notion_search" => {
            let query = req.arguments.get("query").and_then(|v| v.as_str());
            let filter_obj = req
                .arguments
                .get("filter_object")
                .and_then(|v| v.as_str());
            client.search(&token, query, filter_obj).await
        }
        "notion_retrieve_block" => {
            let block_id = req.arguments["block_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing block_id".to_string()))?;
            client.retrieve_block(&token, block_id).await
        }
        "notion_get_block_children" => {
            let block_id = req.arguments["block_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing block_id".to_string()))?;
            client.get_block_children(&token, block_id).await
        }
        "notion_append_block_children" => {
            let block_id = req.arguments["block_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing block_id".to_string()))?;
            let children = req.arguments.get("children").cloned().ok_or((
                StatusCode::BAD_REQUEST,
                "missing children".to_string(),
            ))?;
            client
                .append_block_children(&token, block_id, children)
                .await
        }
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("unknown notion tool: {other}"),
            ));
        }
    };

    match result {
        Ok(output) => Ok(Json(NotionExecuteResponse {
            success: true,
            output,
        })),
        Err(e) => Ok(Json(NotionExecuteResponse {
            success: false,
            output: serde_json::json!({ "error": e.to_string() }),
        })),
    }
}

// ---------------------------------------------------------------------------
// Google Workspace tool dispatch
// ---------------------------------------------------------------------------

/// Request payload for the unified Google Workspace tool dispatch endpoint.
#[derive(Debug, Deserialize)]
pub struct GoogleExecuteRequest {
    pub tool_name: String,
    pub arguments: Value,
    pub token: String,
}

/// Response returned from every Google Workspace tool invocation.
#[derive(Debug, Serialize)]
pub struct GoogleExecuteResponse {
    pub success: bool,
    pub output: Value,
}

/// Dispatches a Google Workspace tool invocation to the appropriate
/// `GoogleWorkspaceClient` method.
pub async fn execute_google_tool(
    State(state): State<HubState>,
    Json(req): Json<GoogleExecuteRequest>,
) -> Result<Json<GoogleExecuteResponse>, (StatusCode, String)> {
    let client = &state.google_client;

    let token = if req.token.is_empty() {
        state
            .vault
            .get_token(uuid::Uuid::nil(), "google_workspace")
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        req.token.clone()
    };

    let result: Result<Value, anyhow::Error> = match req.tool_name.as_str() {
        "google_list_calendar_events" => {
            let calendar_id = req.arguments["calendar_id"]
                .as_str()
                .unwrap_or("primary");
            let max_results = req.arguments["max_results"].as_u64().map(|n| n as u32);
            client
                .list_calendar_events(&token, calendar_id, max_results)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_get_calendar_event" => {
            let calendar_id = req.arguments["calendar_id"]
                .as_str()
                .unwrap_or("primary");
            let event_id = req.arguments["event_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing event_id".to_string()))?;
            client
                .get_calendar_event(&token, calendar_id, event_id)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_list_calendars" => {
            client
                .list_calendars(&token)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_create_calendar_event" => {
            let calendar_id = req.arguments["calendar_id"]
                .as_str()
                .unwrap_or("primary");
            let event: CreateEventRequest = serde_json::from_value(
                req.arguments.get("event").cloned().ok_or((
                    StatusCode::BAD_REQUEST,
                    "missing event".to_string(),
                ))?,
            )
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid event: {e}")))?;
            client
                .create_calendar_event(&token, calendar_id, event)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_list_gmail_messages" => {
            let max_results = req.arguments["max_results"].as_u64().map(|n| n as u32);
            client
                .list_gmail_messages(&token, max_results)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_get_gmail_message" => {
            let message_id = req.arguments["message_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing message_id".to_string()))?;
            let format = req.arguments.get("format").and_then(|v| v.as_str());
            client
                .get_gmail_message(&token, message_id, format)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_search_gmail" => {
            let query = req.arguments["query"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing query".to_string()))?;
            let max_results = req.arguments["max_results"].as_u64().map(|n| n as u32);
            client
                .search_gmail_messages(&token, query, max_results)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_list_drive_files" => {
            let page_size = req.arguments["page_size"].as_u64().map(|n| n as u32);
            client
                .list_drive_files(&token, page_size)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_get_drive_file" => {
            let file_id = req.arguments["file_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing file_id".to_string()))?;
            client
                .get_drive_file(&token, file_id)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_search_drive" => {
            let query = req.arguments["query"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing query".to_string()))?;
            let page_size = req.arguments["page_size"].as_u64().map(|n| n as u32);
            client
                .search_drive_files(&token, query, page_size)
                .await
                .and_then(|r| Ok(serde_json::to_value(r)?))
        }
        "google_export_drive_file" => {
            let file_id = req.arguments["file_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing file_id".to_string()))?;
            let mime_type = req.arguments["mime_type"]
                .as_str()
                .unwrap_or("text/plain");
            client
                .export_drive_file(&token, file_id, mime_type)
                .await
                .map(|content| serde_json::json!({ "content": content }))
        }
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("unknown google tool: {other}"),
            ));
        }
    };

    match result {
        Ok(output) => Ok(Json(GoogleExecuteResponse {
            success: true,
            output,
        })),
        Err(e) => Ok(Json(GoogleExecuteResponse {
            success: false,
            output: serde_json::json!({ "error": e.to_string() }),
        })),
    }
}
