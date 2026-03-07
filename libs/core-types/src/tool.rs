use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Defines a tool that can be presented to an agent at planning time.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema_json: String,
}

/// Represents a single tool invocation request emitted by the planner.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments_json: String,
}

/// Represents the normalized result of a tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output_json: String,
}

/// Creates a stable placeholder tool schema for documentation, testing, and UI previews.
///
/// The returned value is intentionally generic because real tool discovery belongs to the
/// connector hub and should not be hard-coded in shared types.
pub fn placeholder_tool_schema(name: &str) -> ToolSchema {
    ToolSchema {
        name: name.to_string(),
        description: "Placeholder tool schema.".to_string(),
        input_schema_json: "{}".to_string(),
    }
}
