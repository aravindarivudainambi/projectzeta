use anyhow::Result;
use argon2::{self, Config as ArgonConfig};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain model for a tenant-scoped auth user used during credential validation.
#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub password_hash: String,
}

/// JSON payload accepted by `POST /auth/register`.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// JSON response returned by successful registration.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub email: String,
}

/// Produces a canonical, case-insensitive key for user emails.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

/// Hashes a plaintext password with a random salt using Argon2.
pub fn hash_password(password: &str) -> Result<String> {
    let mut salt = [0_u8; 32];
    OsRng.fill_bytes(&mut salt);

    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &ArgonConfig::default())?;
    Ok(hash)
}

/// Validates a plaintext password against an encoded Argon2 hash.
pub fn verify_password(password_hash: &str, password: &str) -> bool {
    argon2::verify_encoded(password_hash, password.as_bytes()).unwrap_or(false)
}
