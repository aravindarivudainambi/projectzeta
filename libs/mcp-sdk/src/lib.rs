use anyhow::{ensure, Result};

/// Describes a tool exposed by an MCP-compatible server.
#[derive(Debug, Clone)]
pub struct McpToolDescriptor {
    pub name: String,
    pub description: String,
}

/// Discovers the tools available from a configured MCP endpoint.
///
/// The scaffold keeps the transport unspecified so connector-hub can decide between
/// stdio, SSE, or any future adapter protocol.
pub async fn discover_tools(endpoint: &str) -> Result<Vec<McpToolDescriptor>> {
    ensure!(!endpoint.trim().is_empty(), "MCP endpoint cannot be empty");
    Ok(Vec::new())
}

/// Invokes a discovered MCP tool with a JSON string payload.
///
/// The result is a raw JSON string placeholder until a richer typed contract is chosen.
pub async fn invoke_tool(endpoint: &str, tool_name: &str, _args_json: &str) -> Result<String> {
    ensure!(!endpoint.trim().is_empty(), "MCP endpoint cannot be empty");
    ensure!(!tool_name.trim().is_empty(), "MCP tool name cannot be empty");
    Ok(r#"{"status":"accepted"}"#.to_string())
}
