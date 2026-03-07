use anyhow::Result;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

/// JWT claim set emitted by the auth service for authenticated users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub tenant_id: String,
    pub exp: usize,
}

/// Response contract returned to clients after successful login.
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: u64,
}

/// Issues a short-lived token for a user identity.
///
/// # Parameters
/// - `sub`: Canonical user identifier to place in the `sub` JWT claim.
/// - `tenant_id`: Tenant scope identifier for authorization boundaries.
/// - `jwt_signing_secret`: HMAC signing secret used with HS256.
///
/// # Returns
/// A signed JWT plus a stable `expires_in` value of `900` seconds.
pub async fn issue_token(
    sub: &str,
    tenant_id: &str,
    jwt_signing_secret: &str,
) -> Result<LoginResponse> {
    let expires_in = 900_u64;
    let exp = (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize;

    let claims = AccessTokenClaims {
        sub: sub.to_string(),
        tenant_id: tenant_id.to_string(),
        exp,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_signing_secret.as_bytes()),
    )?;

    Ok(LoginResponse {
        access_token,
        expires_in,
    })
}
