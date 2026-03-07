use anyhow::Result;

/// Represents a placeholder database pool handle.
#[derive(Debug, Clone, Default)]
pub struct DatabasePool;

/// Creates the shared database pool used across services.
///
/// This placeholder returns an empty pool handle so service bootstrap code can be wired
/// before a concrete PostgreSQL driver is selected and configured.
pub async fn create_pool(_database_url: &str) -> Result<DatabasePool> {
    Ok(DatabasePool)
}
