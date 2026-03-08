use std::pin::Pin;

use anyhow::{bail, Result};
use futures_core::Stream;

use crate::provider::{ChatMessage, LlmProvider};

/// Placeholder adapter for Anthropic-compatible models.
pub struct AnthropicProvider;

impl LlmProvider for AnthropicProvider {
    fn generate(&self, _prompt: &str) -> Result<String> {
        bail!("Anthropic generation is not configured in this scaffold yet.")
    }

    fn complete_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
        Box::pin(async_stream::stream! {
            let _ = messages;
            yield Err(anyhow::anyhow!(
                "Anthropic streaming is not configured in this scaffold yet."
            ));
        })
    }
}
