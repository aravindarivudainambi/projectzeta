use crate::adapters::notion::NotionClient;

/// Shared application state for the connector hub.
#[derive(Clone)]
pub struct HubState {
    pub notion_client: NotionClient,
}

impl HubState {
    pub fn new() -> Self {
        let http = reqwest::Client::new();
        Self {
            notion_client: NotionClient::new(http),
        }
    }
}
