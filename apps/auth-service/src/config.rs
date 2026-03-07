use anyhow::Result;

/// Represents runtime configuration for the auth service.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
}

impl Config {
    /// Loads auth service configuration from the environment.
    pub fn from_env() -> Result<Self> {
        let port = std::env::var("AUTH_SERVICE_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8083);

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/agent_builder".to_string()
        });

        Ok(Self { port, database_url })
    }
}
