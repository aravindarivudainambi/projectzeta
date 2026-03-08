use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use anyhow::{bail, Result};
use uuid::Uuid;

pub type UserId = Uuid;

/// Represents a just-in-time token returned from the vault layer.
#[derive(Debug, Clone)]
pub struct ScopedToken {
    pub value: String,
    pub scope: String,
}

/// Represents the credential storage facade used by the connector hub.
///
/// This mock implementation reads static provider tokens from environment
/// variables (for example `MOCK_TOKEN_NOTION=secret_...`) and keeps them in-memory.
/// Keys are stored as `(UserId, provider)` for forward compatibility, while
/// lookups currently ignore user scoping by design.
///
/// The inner map is wrapped in `Arc<RwLock>` so that OAuth callbacks can store
/// freshly acquired tokens at runtime without restarting the service.
#[derive(Debug, Clone, Default)]
pub struct SecretVault {
    tokens: Arc<RwLock<HashMap<(UserId, String), String>>>,
}

impl SecretVault {
    /// Builds a mock vault by loading any `MOCK_TOKEN_*` variables from `.env`
    /// and process environment.
    ///
    /// Example:
    /// - `MOCK_TOKEN_NOTION=secret_123`
    /// - `MOCK_TOKEN_GOOGLE_WORKSPACE=ya29.a0...`
    ///
    /// Tokens are normalized to lowercase provider names (`notion`, `google_workspace`).
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let mut tokens = HashMap::new();
        for (key, value) in std::env::vars() {
            if let Some(provider) = key.strip_prefix("MOCK_TOKEN_") {
                let normalized_provider = provider.to_lowercase();
                tokens.insert((Uuid::nil(), normalized_provider), value);
            }
        }
        Self {
            tokens: Arc::new(RwLock::new(tokens)),
        }
    }

    /// Builds a vault from explicit entries.
    ///
    /// This helper keeps unit tests deterministic while preserving the same
    /// in-memory map structure used by the environment-backed constructor.
    pub fn from_tokens(tokens: HashMap<(UserId, String), String>) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(tokens)),
        }
    }

    /// Returns a mock token for a provider.
    ///
    /// User scoping is intentionally ignored for now: the lookup first checks
    /// the exact user key, then falls back to a shared `(Uuid::nil(), provider)`
    /// entry.
    pub fn get_token(&self, user_id: UserId, provider: &str) -> Result<String> {
        let normalized_provider = provider.to_lowercase();

        let tokens = self
            .tokens
            .read()
            .map_err(|e| anyhow::anyhow!("vault lock poisoned: {e}"))?;

        if let Some(token) = tokens.get(&(user_id, normalized_provider.clone())) {
            return Ok(token.clone());
        }

        if let Some(token) = tokens.get(&(Uuid::nil(), normalized_provider.clone())) {
            return Ok(token.clone());
        }

        let expected_env_var = format!("MOCK_TOKEN_{}", normalized_provider.to_uppercase());
        bail!(
            "No mock token configured for provider '{provider}'. Set {expected_env_var} in .env or environment."
        )
    }

    /// Stores (or overwrites) a token for a user+provider pair at runtime.
    ///
    /// Used by OAuth callbacks to persist freshly acquired access tokens
    /// without restarting the service.
    pub fn set_token(&self, user_id: UserId, provider: &str, value: &str) {
        let normalized_provider = provider.to_lowercase();
        let mut tokens = self.tokens.write().expect("vault lock poisoned");
        tokens.insert((user_id, normalized_provider), value.to_string());
    }

    /// Returns `true` if a token exists for the given provider (any user).
    pub fn has_token(&self, provider: &str) -> bool {
        let normalized = provider.to_lowercase();
        let tokens = self.tokens.read().expect("vault lock poisoned");
        tokens.keys().any(|(_, p)| *p == normalized)
    }
}

/// Fetches a scoped credential for a user and tool combination.
///
/// This mock path maps directly to `SecretVault::get_token` and wraps the value
/// with a caller-provided scope string.
pub async fn get_tool_credentials(
    vault: &SecretVault,
    user_id: Uuid,
    tool: &str,
    scope: &str,
) -> Result<ScopedToken> {
    let value = vault.get_token(user_id, tool)?;
    Ok(ScopedToken {
        value,
        scope: scope.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::SecretVault;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn get_token_returns_provider_token_regardless_of_user_id() {
        let mut tokens = HashMap::new();
        tokens.insert(
            (Uuid::nil(), "google_workspace".to_string()),
            "ya29.test-token".to_string(),
        );
        let vault = SecretVault::from_tokens(tokens);

        let token = vault
            .get_token(Uuid::new_v4(), "google_workspace")
            .expect("google token should resolve");
        assert_eq!(token, "ya29.test-token");
    }

    #[test]
    fn get_token_returns_descriptive_error_for_missing_provider() {
        let vault = SecretVault::from_tokens(HashMap::new());
        let error = vault
            .get_token(Uuid::new_v4(), "notion")
            .expect_err("missing notion token should produce an error");

        let message = error.to_string();
        assert!(message.contains("provider 'notion'"));
        assert!(message.contains("MOCK_TOKEN_NOTION"));
    }

    #[test]
    fn set_token_stores_and_retrieves_successfully() {
        let vault = SecretVault::from_tokens(HashMap::new());
        let user_id = Uuid::new_v4();

        vault.set_token(user_id, "notion", "ntn_test_123");
        let token = vault.get_token(user_id, "notion").expect("should exist");
        assert_eq!(token, "ntn_test_123");
    }

    #[test]
    fn has_token_reflects_stored_state() {
        let vault = SecretVault::from_tokens(HashMap::new());
        assert!(!vault.has_token("notion"));

        vault.set_token(Uuid::nil(), "notion", "ntn_test");
        assert!(vault.has_token("notion"));
    }
}
