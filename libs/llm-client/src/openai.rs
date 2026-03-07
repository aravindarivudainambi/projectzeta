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

    /// Integration test: sends "Say hello" through the OpenAI-compatible provider
    /// and asserts that at least one streamed chunk is received.
    ///
    /// Requires `GITHUB_MODELS_API_KEY` (set via `.env` or environment).
    #[tokio::test]
    async fn streaming_returns_at_least_one_chunk() {
        let _ = dotenvy::dotenv();

        let api_key = match std::env::var("GITHUB_MODELS_API_KEY") {
            Ok(k) if !k.is_empty() && k != "YOUR_GITHUB_MODELS_API_KEY" => k,
            _ => {
                eprintln!("GITHUB_MODELS_API_KEY not set – skipping integration test");
                return;
            }
        };

        let provider = OpenAiProvider::new(
            api_key,
            std::env::var("GITHUB_MODELS_BASE_URL")
                .unwrap_or_else(|_| "https://models.inference.ai.azure.com".to_string()),
            "gpt-4o".to_string(),
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Say hello".to_string(),
        }];

        let stream = provider.complete_stream(messages);

        use futures_util::StreamExt as FuturesStreamExt;
        let mut stream = std::pin::pin!(stream);
        let mut chunks = Vec::new();
        while let Some(item) = FuturesStreamExt::next(&mut stream).await {
            match item {
                Ok(chunk) => chunks.push(chunk),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("401") || msg.contains("unauthorized") {
                        eprintln!(
                            "GITHUB_MODELS_API_KEY lacks permission – skipping integration test: {msg}"
                        );
                        return;
                    }
                    panic!("Stream error: {}", e);
                }
            }
        }

        assert!(
            !chunks.is_empty(),
            "Expected at least one chunk from the stream"
        );
        let full_response: String = chunks.join("");
        assert!(
            !full_response.is_empty(),
            "Full response should be non-empty"
        );
    }
}
