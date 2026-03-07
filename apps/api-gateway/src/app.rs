use anyhow::Result;
use axum::Router;

use crate::config::Config;

/// Builds the composed Axum router for the gateway surface.
///
/// Real middleware stacking and route registration should live here once the transport layer
/// is implemented end to end.
pub async fn build_router(_config: &Config) -> Result<Router> {
    Ok(Router::new())
}
