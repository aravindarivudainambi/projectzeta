use anyhow::{Context, Result};

/// Represents runtime configuration for the auth service.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub jwt_signing_secret: String,
}

impl Config {
    /// Loads auth service configuration from the environment.
    ///
    /// # Environment Variables
    /// - `AUTH_SERVICE_PORT`: Optional port override. Defaults to `8083`.
    /// - `AUTH_JWT_SIGNING_SECRET`: Required HMAC secret used to sign and verify JWTs.
    pub fn from_env() -> Result<Self> {
        let port = std::env::var("AUTH_SERVICE_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8083);

        let jwt_signing_secret = std::env::var("AUTH_JWT_SIGNING_SECRET")
            .context("AUTH_JWT_SIGNING_SECRET must be set for token signing")?;

        Ok(Self {
            port,
            jwt_signing_secret,
        })
    }
}
