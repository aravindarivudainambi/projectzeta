use std::pin::Pin;

use anyhow::Result;
use futures_core::Stream;

/// A single message in a chat-style conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Defines the contract that every LLM provider adapter must satisfy.
///
/// Implementations should normalize provider-specific request and response payloads
/// into a stable string-based contract for the rest of the platform.
pub trait LlmProvider: Send + Sync {
    /// Generates a model response for the provided prompt (non-streaming).
    fn generate(&self, prompt: &str) -> Result<String>;

    /// Streams a chat completion token by token.
    ///
    /// Returns a pinned, `'static` stream of `Result<String>` chunks.  Each item
    /// is either a content delta or an error.  The stream ends when the model
    /// finishes.  Implementations must move all required state into the returned
    /// future so the stream is independent of `&self`.
    fn complete_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>>;
}
