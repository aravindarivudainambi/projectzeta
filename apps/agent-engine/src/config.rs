use anyhow::Result;

/// Represents runtime configuration for the agent engine service.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
}

impl Config {
    /// Loads agent engine configuration from the environment.
    pub fn from_env() -> Result<Self> {
        Ok(Self { port: 8081 })
    }
}
