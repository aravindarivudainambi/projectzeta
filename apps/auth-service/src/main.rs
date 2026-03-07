//! Auth service entry point.

mod config;
mod rbac;
mod tokens;
mod users;

use anyhow::Result;

/// Boots the auth service process and initializes shared instrumentation.
#[tokio::main]
async fn main() -> Result<()> {
    let _config = config::Config::from_env()?;
    telemetry::init_telemetry("auth-service")?;
    Ok(())
}
