use anyhow::Result;

/// Represents runtime configuration for the API gateway service.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
}

impl Config {
    /// Loads gateway configuration from the process environment.
    ///
    /// This placeholder keeps configuration intentionally small so startup paths can be wired
    /// before full deployment settings are finalized.
    pub fn from_env() -> Result<Self> {
        Ok(Self { port: 8080 })
    }
}
