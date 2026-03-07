use anyhow::Result;
use uuid::Uuid;

use crate::pool::DatabasePool;

/// Applies the tenant context to a checked-out database connection or request scope.
///
/// The real implementation should set the server-side RLS context before any query runs.
pub async fn apply_tenant_context(_pool: &DatabasePool, _tenant_id: Uuid) -> Result<()> {
    todo!("Attach tenant-scoped database context for row-level security enforcement.")
}
