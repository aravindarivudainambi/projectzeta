use anyhow::Result;
use uuid::Uuid;

use crate::pool::DatabasePool;

/// Applies the tenant context to a checked-out database connection or request scope.
///
/// The real implementation should set the server-side RLS context before any query runs.
pub async fn apply_tenant_context(pool: &DatabasePool, tenant_id: Uuid) -> Result<()> {
    let _ = (pool, tenant_id);
    Ok(())
}
