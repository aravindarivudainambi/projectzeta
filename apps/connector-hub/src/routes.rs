use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::hub_state::HubState;

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
    let token = &req.token;

    let result = match req.tool_name.as_str() {
        "notion_query_database" => {
            let db_id = req.arguments["database_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing database_id".to_string()))?;
            let filter = req.arguments.get("filter").cloned();
            let sorts = req.arguments.get("sorts").cloned();
            client.query_database(token, db_id, filter, sorts).await
        }
        "notion_create_page" => client.create_page(token, req.arguments).await,
        "notion_retrieve_page" => {
            let page_id = req.arguments["page_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing page_id".to_string()))?;
            client.retrieve_page(token, page_id).await
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
            client.update_page(token, page_id, properties).await
        }
        "notion_search" => {
            let query = req.arguments.get("query").and_then(|v| v.as_str());
            let filter_obj = req
                .arguments
                .get("filter_object")
                .and_then(|v| v.as_str());
            client.search(token, query, filter_obj).await
        }
        "notion_retrieve_block" => {
            let block_id = req.arguments["block_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing block_id".to_string()))?;
            client.retrieve_block(token, block_id).await
        }
        "notion_get_block_children" => {
            let block_id = req.arguments["block_id"]
                .as_str()
                .ok_or((StatusCode::BAD_REQUEST, "missing block_id".to_string()))?;
            client.get_block_children(token, block_id).await
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
                .append_block_children(token, block_id, children)
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
