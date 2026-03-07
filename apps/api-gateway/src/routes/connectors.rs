use anyhow::Result;

/// Lists available connector definitions and their connection status for the current tenant.
pub async fn list_connectors() -> Result<()> {
    todo!("Aggregate connector metadata and user-specific connection state.")
}

/// Starts the OAuth or token-based connection flow for a specific integration.
pub async fn connect_connector() -> Result<()> {
    todo!("Create connector authorization sessions and redirect metadata.")
}
