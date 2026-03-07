use core_types::tool::ToolResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CONNECTOR_HUB_URL: &str = "http://localhost:8082";

#[derive(Debug, Serialize)]
struct HubRequest {
    tool_name: String,
    arguments: Value,
    token: String,
}

#[derive(Debug, Deserialize)]
struct HubResponse {
    success: bool,
    output: Value,
}

/// Dispatches a tool call to the appropriate backend service.
///
/// For `notion_*` tools this forwards to connector-hub's `/notion/execute`.
/// Unrecognized tools return a failure `ToolResult` (never panics).
pub async fn dispatch_tool_call(
    http: &reqwest::Client,
    tool_name: &str,
    arguments: &Value,
    token: &str,
) -> ToolResult {
    if tool_name.starts_with("notion_") {
        match dispatch_notion(http, tool_name, arguments, token).await {
            Ok(resp) => ToolResult {
                tool_name: tool_name.to_string(),
                success: resp.success,
                output_json: serde_json::to_string(&resp.output)
                    .unwrap_or_else(|_| "{}".to_string()),
            },
            Err(e) => ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output_json: serde_json::json!({ "error": e.to_string() }).to_string(),
            },
        }
    } else if tool_name.starts_with("google_") {
        match dispatch_google(http, tool_name, arguments, token).await {
            Ok(resp) => ToolResult {
                tool_name: tool_name.to_string(),
                success: resp.success,
                output_json: serde_json::to_string(&resp.output)
                    .unwrap_or_else(|_| "{}".to_string()),
            },
            Err(e) => ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output_json: serde_json::json!({ "error": e.to_string() }).to_string(),
            },
        }
    } else {
        ToolResult {
            tool_name: tool_name.to_string(),
            success: false,
            output_json: serde_json::json!({
                "error": format!("no dispatcher registered for tool: {tool_name}")
            })
            .to_string(),
        }
    }
}

async fn dispatch_notion(
    http: &reqwest::Client,
    tool_name: &str,
    arguments: &Value,
    token: &str,
) -> anyhow::Result<HubResponse> {
    let hub_req = HubRequest {
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
        token: token.to_string(),
    };

    let resp = http
        .post(format!("{CONNECTOR_HUB_URL}/notion/execute"))
        .json(&hub_req)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("failed to reach connector-hub: {e}"))?;

    let hub_resp: HubResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("failed to parse connector-hub response: {e}"))?;

    Ok(hub_resp)
}

async fn dispatch_google(
    http: &reqwest::Client,
    tool_name: &str,
    arguments: &Value,
    token: &str,
) -> anyhow::Result<HubResponse> {
    let hub_req = HubRequest {
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
        token: token.to_string(),
    };

    let resp = http
        .post(format!("{CONNECTOR_HUB_URL}/google/execute"))
        .json(&hub_req)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("failed to reach connector-hub: {e}"))?;

    let hub_resp: HubResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("failed to parse connector-hub response: {e}"))?;

    Ok(hub_resp)
}
