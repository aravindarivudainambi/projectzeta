use anyhow::Result;

/// Represents runtime configuration for the connector hub service.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub notion_client_id: Option<String>,
    pub notion_client_secret: Option<String>,
    pub notion_redirect_uri: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: Option<String>,
}

impl Config {
    /// Loads connector hub configuration from the environment.
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();
        Ok(Self {
            port: 8082,
            notion_client_id: std::env::var("NOTION_CLIENT_ID").ok(),
            notion_client_secret: std::env::var("NOTION_CLIENT_SECRET").ok(),
            notion_redirect_uri: std::env::var("NOTION_REDIRECT_URI").ok(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            google_redirect_uri: std::env::var("GOOGLE_REDIRECT_URI").ok(),
        })
    }
}
