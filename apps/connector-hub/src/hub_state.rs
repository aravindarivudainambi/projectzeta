use crate::adapters::google_workspace::GoogleWorkspaceClient;
use crate::adapters::notion::NotionClient;
use crate::config::Config;
use secret_vault::SecretVault;

/// Shared application state for the connector hub.
#[derive(Clone)]
pub struct HubState {
    pub notion_client: NotionClient,
    pub google_client: GoogleWorkspaceClient,
    pub vault: SecretVault,
    pub config: Config,
}

impl HubState {
    pub fn new(config: Config) -> Self {
        let http = reqwest::Client::new();
        Self {
            notion_client: NotionClient::new(http.clone()),
            google_client: GoogleWorkspaceClient::new(http),
            vault: SecretVault::from_env(),
            config,
        }
    }
}
