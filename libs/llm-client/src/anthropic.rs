use anyhow::Result;

use crate::provider::LlmProvider;

/// Placeholder adapter for Anthropic-compatible models.
pub struct AnthropicProvider;

impl LlmProvider for AnthropicProvider {
    /// Generates a placeholder response without performing a network call.
    fn generate(&self, _prompt: &str) -> Result<String> {
        todo!("Implement the Anthropic transport, auth, and response normalization.")
    }
}
