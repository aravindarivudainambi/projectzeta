use anyhow::Result;

/// Discovers tools from a configured MCP server.
pub async fn discover_remote_tools() -> Result<()> {
    todo!("Implement MCP discovery orchestration and typed result mapping.")
}

/// Invokes a remote MCP tool on behalf of a user-scoped agent run.
pub async fn invoke_remote_tool() -> Result<()> {
    todo!("Implement MCP invocation, auth propagation, and error normalization.")
}
