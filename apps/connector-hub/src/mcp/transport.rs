/// Returns the supported MCP transport names available to the connector hub.
pub fn supported_transports() -> Vec<&'static str> {
    vec!["stdio", "sse"]
}
