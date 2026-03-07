use anyhow::Result;
use jsonwebtoken::{decode, DecodingKey, Validation};

use super::issue::AccessTokenClaims;

/// Validates and decodes a previously issued token.
///
/// # Parameters
/// - `token`: JWT to decode and verify.
/// - `jwt_signing_secret`: HMAC secret used for signature verification.
///
/// # Returns
/// The decoded claims when signature and expiry are valid.
pub async fn validate_token(token: &str, jwt_signing_secret: &str) -> Result<AccessTokenClaims> {
    let decoded = decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(jwt_signing_secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(decoded.claims)
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{encode, EncodingKey, Header};

    use crate::tokens::issue::{issue_token, AccessTokenClaims};

    use super::validate_token;

    #[tokio::test]
    async fn accepts_fresh_signed_token() {
        let response = issue_token("user_123", "tenant_123", "test-secret")
            .await
            .expect("token issuance must succeed");

        let claims = validate_token(&response.access_token, "test-secret")
            .await
            .expect("token must validate");

        assert_eq!(claims.sub, "user_123");
        assert_eq!(claims.tenant_id, "tenant_123");
    }

    #[tokio::test]
    async fn rejects_tampered_token() {
        let response = issue_token("user_123", "tenant_123", "test-secret")
            .await
            .expect("token issuance must succeed");

        let tampered = format!("{}x", response.access_token);

        let result = validate_token(&tampered, "test-secret").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_expired_token() {
        let claims = AccessTokenClaims {
            sub: "user_123".to_string(),
            tenant_id: "tenant_123".to_string(),
            exp: (chrono::Utc::now() - chrono::Duration::minutes(2)).timestamp() as usize,
        };

        let expired_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test-secret".as_bytes()),
        )
        .expect("encoding should succeed");

        let result = validate_token(&expired_token, "test-secret").await;
        assert!(result.is_err());
    }
}
