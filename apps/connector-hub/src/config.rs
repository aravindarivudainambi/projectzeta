use anyhow::Result;

/// Represents runtime configuration for the connector hub service.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
}

impl Config {
    /// Loads connector hub configuration from the environment.
    pub fn from_env() -> Result<Self> {
        Ok(Self { port: 8082 })
    }
}
