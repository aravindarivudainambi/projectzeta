use anyhow::Result;

/// Returns marketplace templates that can be browsed or forked by users.
#[allow(dead_code)]
pub async fn list_marketplace_templates() -> Result<()> {
    todo!("Load curated templates with tenant-safe visibility rules.")
}

/// Creates a new agent by forking a marketplace template into the current tenant.
#[allow(dead_code)]
pub async fn fork_marketplace_template() -> Result<()> {
    todo!("Copy template metadata into a tenant-owned agent draft.")
}
