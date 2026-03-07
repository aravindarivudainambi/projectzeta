use anyhow::Result;
use secret_vault::SecretVault;
use uuid::Uuid;

/// Retrieves the active credential for a named connector on behalf of a user.
///
/// This is a thin bridge between the connector-hub layer and the `secret-vault`
/// library. It is intentionally stateless: no persistence backend is wired yet.
/// A future implementation would integrate with HashiCorp Vault, an encrypted
/// database secrets column, or an HSM-backed KMS.
///
/// # Parameters
/// - `vault`: Reference to the in-memory vault populated via [`SecretVault::from_env`].
/// - `user_id`: The tenant-scoped user requesting credentials. Falls back to the
///   shared nil-UUID entry when no user-specific credential exists.
/// - `connector`: Lowercase connector identifier, e.g. `"google_workspace"`, `"slack"`.
///
/// # Errors
/// Returns an [`anyhow::Error`] whose message names the missing env var when no
/// token is configured, e.g. `"Set MOCK_TOKEN_GOOGLE_WORKSPACE in .env"`.
pub fn get_connector_secret(vault: &SecretVault, user_id: Uuid, connector: &str) -> Result<String> {
    vault.get_token(user_id, connector)
}

/// Persists connector credentials in the selected secret backend.
///
/// # Scaffolding note
/// This placeholder is intentionally left unimplemented. Encrypted persistence,
/// rotation scheduling, and per-tenant scoping are deferred until the Vault
/// integration milestone.
pub async fn store_connector_secret() -> Result<()> {
    todo!("Implement encrypted secret persistence and rotation support.")
}

#[cfg(test)]
mod tests {
    use super::get_connector_secret;
    use secret_vault::SecretVault;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn returns_token_for_known_connector() {
        let mut tokens = HashMap::new();
        tokens.insert(
            (Uuid::nil(), "google_workspace".to_string()),
            "test-token-abc".to_string(),
        );
        let vault = SecretVault::from_tokens(tokens);

        let result = get_connector_secret(&vault, Uuid::new_v4(), "google_workspace")
            .expect("known connector should resolve via nil-UUID fallback");
        assert_eq!(result, "test-token-abc");
    }

    #[test]
    fn returns_descriptive_error_for_unknown_connector() {
        let vault = SecretVault::from_tokens(HashMap::new());

        let err = get_connector_secret(&vault, Uuid::new_v4(), "google_workspace")
            .expect_err("missing token should produce an error");
        let msg = err.to_string();
        assert!(
            msg.contains("google_workspace"),
            "error should name the provider; got: {msg}"
        );
        assert!(
            msg.contains("MOCK_TOKEN_GOOGLE_WORKSPACE"),
            "error should name the expected env var; got: {msg}"
        );
    }
}
