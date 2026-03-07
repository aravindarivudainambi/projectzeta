use anyhow::Result;

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
pub async fn discover_tools(_endpoint: &str) -> Result<Vec<McpToolDescriptor>> {
    todo!("Implement MCP discovery transport and response decoding.")
}

/// Invokes a discovered MCP tool with a JSON string payload.
///
/// The result is a raw JSON string placeholder until a richer typed contract is chosen.
pub async fn invoke_tool(_endpoint: &str, _tool_name: &str, _args_json: &str) -> Result<String> {
    todo!("Implement MCP tool invocation and result normalization.")
}
