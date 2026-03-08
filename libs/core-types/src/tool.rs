use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const GOOGLE_WORKSPACE_CONNECTOR: &str = "google_workspace";
pub const NOTION_CONNECTOR: &str = "notion";

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

fn defined_tool_schema(
    name: &str,
    description: &str,
    input_schema_json: serde_json::Value,
) -> ToolSchema {
    ToolSchema {
        name: name.to_string(),
        description: description.to_string(),
        input_schema_json: input_schema_json.to_string(),
    }
}

/// Returns the connector identifiers supported by the current product slice.
pub fn supported_connector_names() -> &'static [&'static str] {
    &[GOOGLE_WORKSPACE_CONNECTOR, NOTION_CONNECTOR]
}

/// Returns the currently supported Google Workspace and Notion tool catalog.
pub fn supported_tool_schemas() -> Vec<ToolSchema> {
    vec![
        defined_tool_schema(
            "generate_content",
            "Use GPT to generate structured content that can be handed off to downstream tools like Gmail or Notion.",
            json!({
                "type": "object",
                "required": ["prompt"],
                "properties": {
                    "prompt": { "type": "string" },
                    "context": { "type": ["string", "object", "array", "null"] },
                    "target_tool": { "type": ["string", "null"] },
                    "tone": { "type": ["string", "null"] },
                    "format": { "type": ["string", "null"] }
                }
            }),
        ),
        defined_tool_schema(
            "notion_query_database",
            "Query a Notion database with optional filter and sort criteria.",
            json!({
                "type": "object",
                "required": ["database_id"],
                "properties": {
                    "database_id": { "type": "string" },
                    "filter": { "type": ["object", "null"] },
                    "sorts": { "type": ["array", "null"] }
                }
            }),
        ),
        defined_tool_schema(
            "notion_create_page",
            "Create a new Notion page.",
            json!({
                "type": "object",
                "required": ["parent", "properties"],
                "properties": {
                    "parent": { "type": "object" },
                    "properties": { "type": "object" },
                    "children": { "type": ["array", "null"] }
                }
            }),
        ),
        defined_tool_schema(
            "notion_retrieve_page",
            "Retrieve a Notion page by page ID.",
            json!({
                "type": "object",
                "required": ["page_id"],
                "properties": {
                    "page_id": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "notion_update_page",
            "Update the properties on an existing Notion page.",
            json!({
                "type": "object",
                "required": ["page_id", "properties"],
                "properties": {
                    "page_id": { "type": "string" },
                    "properties": { "type": "object" }
                }
            }),
        ),
        defined_tool_schema(
            "notion_search",
            "Search Notion pages and databases.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "filter_object": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "notion_retrieve_block",
            "Retrieve a Notion block by block ID.",
            json!({
                "type": "object",
                "required": ["block_id"],
                "properties": {
                    "block_id": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "notion_get_block_children",
            "List child blocks for a Notion block.",
            json!({
                "type": "object",
                "required": ["block_id"],
                "properties": {
                    "block_id": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "notion_append_block_children",
            "Append child blocks to a Notion block.",
            json!({
                "type": "object",
                "required": ["block_id", "children"],
                "properties": {
                    "block_id": { "type": "string" },
                    "children": { "type": "array" }
                }
            }),
        ),
        defined_tool_schema(
            "google_list_calendar_events",
            "List events from a Google Calendar.",
            json!({
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string" },
                    "max_results": { "type": "integer", "minimum": 1 }
                }
            }),
        ),
        defined_tool_schema(
            "google_get_calendar_event",
            "Retrieve a single Google Calendar event by ID.",
            json!({
                "type": "object",
                "required": ["event_id"],
                "properties": {
                    "calendar_id": { "type": "string" },
                    "event_id": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "google_list_calendars",
            "List the authenticated user's Google calendars.",
            json!({
                "type": "object",
                "properties": {}
            }),
        ),
        defined_tool_schema(
            "google_create_calendar_event",
            "Create a Google Calendar event.",
            json!({
                "type": "object",
                "required": ["event"],
                "properties": {
                    "calendar_id": { "type": "string" },
                    "event": { "type": "object" }
                }
            }),
        ),
        defined_tool_schema(
            "google_list_gmail_messages",
            "List Gmail messages from the authenticated mailbox.",
            json!({
                "type": "object",
                "properties": {
                    "max_results": { "type": "integer", "minimum": 1 }
                }
            }),
        ),
        defined_tool_schema(
            "google_get_gmail_message",
            "Retrieve a Gmail message by ID.",
            json!({
                "type": "object",
                "required": ["message_id"],
                "properties": {
                    "message_id": { "type": "string" },
                    "format": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "google_search_gmail",
            "Search Gmail using Gmail query syntax.",
            json!({
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": { "type": "string" },
                    "max_results": { "type": "integer", "minimum": 1 }
                }
            }),
        ),
        defined_tool_schema(
            "google_send_gmail",
            "Send an email through Gmail.",
            json!({
                "type": "object",
                "required": ["to", "subject", "body"],
                "properties": {
                    "to": { "type": "string" },
                    "subject": { "type": "string" },
                    "body": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "google_list_drive_files",
            "List files from Google Drive.",
            json!({
                "type": "object",
                "properties": {
                    "page_size": { "type": "integer", "minimum": 1 }
                }
            }),
        ),
        defined_tool_schema(
            "google_get_drive_file",
            "Retrieve Google Drive file metadata by file ID.",
            json!({
                "type": "object",
                "required": ["file_id"],
                "properties": {
                    "file_id": { "type": "string" }
                }
            }),
        ),
        defined_tool_schema(
            "google_search_drive",
            "Search Google Drive files using Drive query syntax.",
            json!({
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": { "type": "string" },
                    "page_size": { "type": "integer", "minimum": 1 }
                }
            }),
        ),
        defined_tool_schema(
            "google_export_drive_file",
            "Export a Google Docs, Sheets, or Slides file to a MIME type.",
            json!({
                "type": "object",
                "required": ["file_id"],
                "properties": {
                    "file_id": { "type": "string" },
                    "mime_type": { "type": "string" }
                }
            }),
        ),
    ]
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
