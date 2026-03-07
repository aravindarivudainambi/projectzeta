use std::pin::Pin;

use anyhow::Result;
use futures_core::Stream;

use crate::provider::{ChatMessage, LlmProvider};

/// Placeholder adapter for Anthropic-compatible models.
pub struct AnthropicProvider;

impl LlmProvider for AnthropicProvider {
    fn generate(&self, _prompt: &str) -> Result<String> {
        todo!("Implement the Anthropic transport, auth, and response normalization.")
    }

    fn complete_stream(
        &self,
        _messages: Vec<ChatMessage>,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
        todo!("Implement the Anthropic streaming transport.")
    }
}
