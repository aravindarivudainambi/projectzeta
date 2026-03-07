use anyhow::Result;

/// Defines the contract that every LLM provider adapter must satisfy.
pub trait LlmProvider {
    /// Generates a model response for the provided prompt.
    ///
    /// Implementations should normalize provider-specific request and response payloads
    /// into a stable string-based contract for the rest of the platform.
    fn generate(&self, prompt: &str) -> Result<String>;
}
