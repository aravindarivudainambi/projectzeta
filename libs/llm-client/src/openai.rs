use anyhow::Result;

use crate::provider::LlmProvider;

/// Placeholder adapter for OpenAI-compatible models.
pub struct OpenAiProvider;

impl LlmProvider for OpenAiProvider {
    /// Generates a placeholder response without performing a network call.
    fn generate(&self, _prompt: &str) -> Result<String> {
        todo!("Implement the OpenAI transport, auth, and response normalization.")
    }
}
