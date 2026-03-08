use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::Trigger;

/// Describes the complexity label shown for a marketplace template.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MarketplaceTemplateComplexity {
    Low,
    Medium,
    High,
}

/// Captures an executable step embedded inside a marketplace template.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename_all = "camelCase")]
pub struct MarketplaceTemplateStep {
    pub name: String,
    /// Optional tool binding used when the template is forked into a real agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Whether a human must approve the step before execution continues.
    #[serde(default)]
    pub requires_approval: bool,
}

/// Represents a curated marketplace template that can be previewed and forked.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename_all = "camelCase")]
pub struct MarketplaceTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub full_description: String,
    pub tool_badges: Vec<String>,
    pub run_count: u64,
    pub creator_name: String,
    pub creator_avatar: String,
    pub department: String,
    pub complexity: MarketplaceTemplateComplexity,
    pub example_output: String,
    pub steps: Vec<String>,
    pub required_connectors: Vec<String>,
    pub trigger: Trigger,
    pub agent_steps: Vec<MarketplaceTemplateStep>,
}

/// Returns a stable in-memory marketplace catalog until persistent storage is approved.
///
/// The scaffold intentionally keeps this list deterministic so the frontend and gateway can
/// share a typed preview catalog without introducing database or search dependencies yet.
pub fn sample_marketplace_templates() -> Vec<MarketplaceTemplate> {
    vec![
        MarketplaceTemplate {
            id: "agent-1".to_string(),
            name: "Inbox to Notion Digest".to_string(),
            description: "Summarizes Gmail threads and writes the digest into Notion.".to_string(),
            full_description: "This agent searches Gmail for important messages, groups them into a structured summary, and creates a Notion page for the team knowledge base every Friday afternoon.".to_string(),
            tool_badges: vec!["Google Workspace".to_string(), "Notion".to_string()],
            run_count: 1_420,
            creator_name: "Sasha Engineer".to_string(),
            creator_avatar: "S".to_string(),
            department: "Engineering".to_string(),
            complexity: MarketplaceTemplateComplexity::Medium,
            example_output: "Created Notion page 'Engineering Weekly Digest' with sections for Decisions, Risks, and Follow-ups based on 8 Gmail threads.".to_string(),
            steps: vec![
                "Triggered weekly on Fridays at 4:00 PM".to_string(),
                "Search Gmail for important engineering updates".to_string(),
                "Analyze the messages to extract key decisions and blockers".to_string(),
                "Create a Notion page in the weekly digest database".to_string(),
                "Append a short executive summary to the page".to_string(),
            ],
            required_connectors: vec!["Google Workspace".to_string(), "Notion".to_string()],
            trigger: Trigger::Schedule {
                cron: "0 16 * * 5".to_string(),
            },
            agent_steps: vec![
                MarketplaceTemplateStep {
                    name: "Search Gmail for important engineering updates".to_string(),
                    tool_name: Some("google_search_gmail".to_string()),
                    requires_approval: false,
                },
                MarketplaceTemplateStep {
                    name: "Create a Notion page in the weekly digest database".to_string(),
                    tool_name: Some("notion_create_page".to_string()),
                    requires_approval: false,
                },
                MarketplaceTemplateStep {
                    name: "Append a short executive summary to the page".to_string(),
                    tool_name: Some("notion_append_block_children".to_string()),
                    requires_approval: true,
                },
            ],
        },
        MarketplaceTemplate {
            id: "agent-2".to_string(),
            name: "Meeting Scheduler from Notion".to_string(),
            description: "Creates Google Calendar events from approved Notion tasks.".to_string(),
            full_description: "When a Notion task is marked Ready for Review, this agent gathers the page details and schedules a Google Calendar event with the right attendees and timing.".to_string(),
            tool_badges: vec!["Notion".to_string(), "Google Workspace".to_string()],
            run_count: 5_310,
            creator_name: "DevOps Team".to_string(),
            creator_avatar: "D".to_string(),
            department: "Engineering".to_string(),
            complexity: MarketplaceTemplateComplexity::High,
            example_output: "Created calendar event 'Architecture Review — Agent Builder' for Mar 8, 2:00 PM with notes linked back to the source Notion page.".to_string(),
            steps: vec![
                "Triggered when a Notion page status becomes 'Ready'".to_string(),
                "Read the task page and extract title, owner, and deadline".to_string(),
                "Draft an appropriate review meeting agenda".to_string(),
                "Create a Google Calendar event on the primary calendar".to_string(),
            ],
            required_connectors: vec!["Notion".to_string(), "Google Workspace".to_string()],
            trigger: Trigger::Event {
                source: "notion".to_string(),
                event: "page_status.ready".to_string(),
            },
            agent_steps: vec![
                MarketplaceTemplateStep {
                    name: "Read the task page and extract title, owner, and deadline".to_string(),
                    tool_name: Some("notion_retrieve_page".to_string()),
                    requires_approval: false,
                },
                MarketplaceTemplateStep {
                    name: "Draft an appropriate review meeting agenda".to_string(),
                    tool_name: None,
                    requires_approval: false,
                },
                MarketplaceTemplateStep {
                    name: "Create a Google Calendar event on the primary calendar".to_string(),
                    tool_name: Some("google_create_calendar_event".to_string()),
                    requires_approval: true,
                },
            ],
        },
        MarketplaceTemplate {
            id: "agent-3".to_string(),
            name: "Drive Research Collector".to_string(),
            description: "Finds Google Drive research docs and stores the summary in Notion.".to_string(),
            full_description: "This agent searches Google Drive for files related to a topic, reads the most relevant document metadata, and creates a structured Notion page with links and a synthesized summary.".to_string(),
            tool_badges: vec!["Google Workspace".to_string(), "Notion".to_string()],
            run_count: 380,
            creator_name: "Alex Product".to_string(),
            creator_avatar: "A".to_string(),
            department: "Product".to_string(),
            complexity: MarketplaceTemplateComplexity::Low,
            example_output: "Created Notion page 'Q1 Launch Research' with 5 Drive file links and a concise synthesis of the findings.".to_string(),
            steps: vec![
                "Triggered manually from the builder".to_string(),
                "Search Google Drive for documents matching the topic".to_string(),
                "Summarize the matched files".to_string(),
                "Create a Notion page with the references and summary".to_string(),
            ],
            required_connectors: vec!["Google Workspace".to_string(), "Notion".to_string()],
            trigger: Trigger::Manual,
            agent_steps: vec![
                MarketplaceTemplateStep {
                    name: "Search Google Drive for documents matching the topic".to_string(),
                    tool_name: Some("google_search_drive".to_string()),
                    requires_approval: false,
                },
                MarketplaceTemplateStep {
                    name: "Create a Notion page with the references and summary".to_string(),
                    tool_name: Some("notion_create_page".to_string()),
                    requires_approval: false,
                },
            ],
        },
        MarketplaceTemplate {
            id: "agent-4".to_string(),
            name: "Calendar Agenda Briefing".to_string(),
            description: "Builds a daily agenda from Google Calendar and stores it in Notion.".to_string(),
            full_description: "Each morning, this agent reads the day's Google Calendar events, summarizes the meetings and prep items, and publishes a planning note into Notion.".to_string(),
            tool_badges: vec!["Google Workspace".to_string(), "Notion".to_string()],
            run_count: 89,
            creator_name: "Finance Ops".to_string(),
            creator_avatar: "F".to_string(),
            department: "Finance".to_string(),
            complexity: MarketplaceTemplateComplexity::High,
            example_output: "Created Notion page 'Daily Agenda — March 7' with 6 meetings, owners, and key preparation bullets.".to_string(),
            steps: vec![
                "Triggered daily at 7:30 AM".to_string(),
                "List Google Calendar events for today".to_string(),
                "Summarize priorities and prep notes".to_string(),
                "Create a Notion briefing page for the day".to_string(),
            ],
            required_connectors: vec!["Google Workspace".to_string(), "Notion".to_string()],
            trigger: Trigger::Schedule {
                cron: "30 7 * * *".to_string(),
            },
            agent_steps: vec![
                MarketplaceTemplateStep {
                    name: "List Google Calendar events for today".to_string(),
                    tool_name: Some("google_list_calendar_events".to_string()),
                    requires_approval: false,
                },
                MarketplaceTemplateStep {
                    name: "Create a Notion briefing page for the day".to_string(),
                    tool_name: Some("notion_create_page".to_string()),
                    requires_approval: false,
                },
            ],
        },
    ]
}