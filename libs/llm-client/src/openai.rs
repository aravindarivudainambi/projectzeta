use std::pin::Pin;

use anyhow::{Context, Result};
use futures_core::Stream;
use reqwest::Client;
use serde_json::Value;
use tokio_stream::StreamExt;

use crate::provider::{ChatMessage, LlmProvider};

/// Adapter for OpenAI-compatible models served by GitHub Models.
///
/// Reads `GITHUB_MODELS_API_KEY` for authentication and sends requests to
/// `GITHUB_MODELS_BASE_URL` (defaults to `https://models.inference.ai.azure.com`).
pub struct OpenAiProvider {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    client: Client,
}

impl OpenAiProvider {
    /// Creates a new provider from explicit values.
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
            client: Client::new(),
        }
    }

    /// Creates a provider by reading `GITHUB_MODELS_API_KEY` and
    /// `GITHUB_MODELS_BASE_URL` from the environment.
    pub fn from_env() -> Result<Self> {
        let api_key =
            std::env::var("GITHUB_MODELS_API_KEY").context("GITHUB_MODELS_API_KEY not set")?;
        let base_url = std::env::var("GITHUB_MODELS_BASE_URL")
            .unwrap_or_else(|_| "https://models.inference.ai.azure.com".to_string());
        let model =
            std::env::var("GITHUB_MODELS_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
        Ok(Self::new(api_key, base_url, model))
    }
}

impl LlmProvider for OpenAiProvider {
    fn generate(&self, _prompt: &str) -> Result<String> {
        todo!("Use complete_stream for all calls; non-streaming generate is not needed yet.")
    }

    /// Streams a chat completion from the GitHub Models (OpenAI-compatible) API.
    ///
    /// Sends a POST to `{base_url}/chat/completions` with `stream: true` and
    /// parses the resulting SSE byte stream, yielding content deltas one by one.
    fn complete_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
        // Clone everything the stream needs so it is `'static`.
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let url = format!("{}/chat/completions", self.base_url);

        let messages_payload: Vec<Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages_payload,
            "stream": true,
        });

        Box::pin(async_stream::try_stream! {
            let response = client
                .post(&url)
                .bearer_auth(&api_key)
                .json(&body)
                .send()
                .await
                .context("Failed to send request to GitHub Models")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "GitHub Models returned {}: {}",
                    status,
                    text
                ))?;
                return;
            }

            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = byte_stream.next().await {
                let bytes = chunk_result.map_err(|e| anyhow::anyhow!("Stream read error: {}", e))?;
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    buffer = buffer[pos + 1..].to_string();

                    let line = line.trim();
                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }
                    if let Some(data) = line.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            return;
                        }
                        if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                            if let Some(content) =
                                parsed["choices"][0]["delta"]["content"].as_str()
                            {
                                if !content.is_empty() {
                                    yield content.to_string();
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt as FuturesStreamExt;

    fn make_provider() -> Option<OpenAiProvider> {
        let _ = dotenvy::dotenv();
        let api_key = match std::env::var("GITHUB_MODELS_API_KEY") {
            Ok(k) if !k.is_empty() && k != "YOUR_GITHUB_MODELS_API_KEY" => k,
            _ => {
                eprintln!("GITHUB_MODELS_API_KEY not set – skipping integration test");
                return None;
            }
        };
        Some(OpenAiProvider::new(
            api_key,
            std::env::var("GITHUB_MODELS_BASE_URL")
                .unwrap_or_else(|_| "https://models.inference.ai.azure.com".to_string()),
            "gpt-4o".to_string(),
        ))
    }

    async fn collect_stream(
        stream: Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
    ) -> Option<String> {
        let mut stream = std::pin::pin!(stream);
        let mut chunks = Vec::new();
        while let Some(item) = FuturesStreamExt::next(&mut stream).await {
            match item {
                Ok(chunk) => chunks.push(chunk),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("401") || msg.contains("unauthorized") {
                        eprintln!("API key lacks permission – skipping: {msg}");
                        return None;
                    }
                    if msg.contains("429") || msg.contains("RateLimitReached") {
                        eprintln!("Rate-limited by GitHub Models – skipping: {msg}");
                        return None;
                    }
                    panic!("Stream error: {}", e);
                }
            }
        }
        Some(chunks.join(""))
    }

    /// Integration test: sends "Say hello" and asserts at least one chunk received.
    #[tokio::test]
    async fn streaming_returns_at_least_one_chunk() {
        let Some(provider) = make_provider() else { return };

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Say hello".to_string(),
        }];

        let Some(response) = collect_stream(provider.complete_stream(messages)).await else {
            return;
        };

        assert!(!response.is_empty(), "Expected non-empty response");
        eprintln!("[hello] Response: {response}");
    }

    /// Integration test: system prompt + user message (multi-turn).
    #[tokio::test]
    async fn streaming_with_system_prompt() {
        let Some(provider) = make_provider() else { return };

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a pirate. Respond only in pirate speak.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What is 2 + 2?".to_string(),
            },
        ];

        let Some(response) = collect_stream(provider.complete_stream(messages)).await else {
            return;
        };

        assert!(!response.is_empty(), "Expected non-empty response");
        eprintln!("[pirate] Response: {response}");
    }

    /// Integration test: asks for a JSON-only response and verifies it parses.
    #[tokio::test]
    async fn streaming_json_only_response() {
        let Some(provider) = make_provider() else { return };

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Respond ONLY with valid JSON. No prose, no markdown fences.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: r#"Return a JSON object with fields "status" set to "ok" and "value" set to 42."#.to_string(),
            },
        ];

        let Some(response) = collect_stream(provider.complete_stream(messages)).await else {
            return;
        };

        eprintln!("[json] Response: {response}");
        let trimmed = response.trim();
        let parsed: serde_json::Value = serde_json::from_str(trimmed)
            .unwrap_or_else(|e| panic!("Response was not valid JSON: {e}\nGot: {trimmed}"));

        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["value"], 42);
    }

    /// Integration test: multi-turn conversation simulating a follow-up question.
    #[tokio::test]
    async fn streaming_multi_turn_conversation() {
        let Some(provider) = make_provider() else { return };

        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "My favourite colour is blue.".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "That's a great choice! Blue is a calming colour.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What colour did I say was my favourite? Answer in one word.".to_string(),
            },
        ];

        let Some(response) = collect_stream(provider.complete_stream(messages)).await else {
            return;
        };

        eprintln!("[multi-turn] Response: {response}");
        assert!(
            response.to_lowercase().contains("blue"),
            "Expected the model to recall 'blue', got: {response}"
        );
    }

    /// Integration test: PII in the prompt is scrubbed before sending.
    /// Verifies the scrubber output is used, not the raw input.
    #[tokio::test]
    async fn streaming_with_pii_scrubbed_prompt() {
        use crate::pii_scrubber::scrub_pii;
        let Some(provider) = make_provider() else { return };

        let raw = "My email is test@example.com and my SSN is 123-45-6789.";
        let scrubbed = scrub_pii(raw);

        assert!(
            !scrubbed.contains("test@example.com"),
            "Email should have been scrubbed"
        );
        assert!(
            !scrubbed.contains("123-45-6789"),
            "SSN should have been scrubbed"
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Repeat the user's message back to them verbatim.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: scrubbed.clone(),
            },
        ];

        let Some(response) = collect_stream(provider.complete_stream(messages)).await else {
            return;
        };

        eprintln!("[pii-scrubbed] Sent: {scrubbed}");
        eprintln!("[pii-scrubbed] Response: {response}");

        // The raw PII should never appear in the echoed response.
        assert!(
            !response.contains("test@example.com"),
            "Email PII leaked into response"
        );
        assert!(
            !response.contains("123-45-6789"),
            "SSN PII leaked into response"
        );
    }
}
