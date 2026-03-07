use anyhow::Result;
use argon2::Config;
use serde::Serialize;

/// Domain model for a tenant-scoped auth user used during credential validation.
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: String,
    pub tenant_id: String,
    pub email: String,
    pub password_hash: String,
}

/// Creates a user record and hashes any credential material as needed.
///
/// This scaffold leaves persistence as future work and currently returns a `todo!()`
/// placeholder to preserve the contract-first boundary for write operations.
pub async fn create_user() -> Result<()> {
    todo!("Implement user persistence, password hashing, and audit logging.")
}

/// Retrieves a tenant-scoped user record by email for login validation.
///
/// # Parameters
/// - `email`: User's login email address.
///
/// # Returns
/// A static seed user used by the scaffold so `/auth/login` can validate credentials and
/// issue JWTs before database integration is implemented.
pub async fn get_user_by_email(email: &str) -> Option<User> {
    if email.eq_ignore_ascii_case("demo@projectzeta.dev") {
        let hash =
            argon2::hash_encoded(b"demo-password", b"projectzeta-demo", &Config::default()).ok()?;

        Some(User {
            id: "user_demo_001".to_string(),
            tenant_id: "tenant_demo_001".to_string(),
            email: "demo@projectzeta.dev".to_string(),
            password_hash: hash,
        })
    } else {
        None
    }
}

/// Retrieves a user record by the chosen identity lookup key.
///
/// This existing API boundary remains as a placeholder for upcoming richer identity queries.
pub async fn get_user() -> Result<()> {
    todo!("Implement tenant-scoped user lookup and projection logic.")
}
