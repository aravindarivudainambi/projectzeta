use std::convert::Infallible;

use axum::{
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use core_types::agent::AgentConfig;
use futures_util::StreamExt;
use llm_client::{
    openai::OpenAiProvider,
    pii_scrubber::scrub_pii,
    provider::{ChatMessage, LlmProvider},
};
use schemars::schema_for;
use serde::Deserialize;

/// Request body for the agent builder endpoint.
#[derive(Debug, Deserialize)]
pub struct BuildRequest {
    pub description: String,
}

/// POST /agents/build
///
/// Accepts a plain-English workflow description, scrubs PII, streams the LLM
/// response as SSE events, and validates the final JSON against the
/// `AgentConfig` schema.  Returns 422 if the accumulated output is not valid.
pub async fn build_agent(
    Json(payload): Json<BuildRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let provider = OpenAiProvider::from_env().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("LLM provider init failed: {e}"),
        )
    })?;

    // Scrub PII from the user description before sending to the LLM.
    let scrubbed_description = scrub_pii(&payload.description);

    // Build the AgentConfig JSON Schema to inject into the system prompt.
    let schema = schema_for!(AgentConfig);
    let schema_json =
        serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".to_string());

    let system_prompt = format!(
        "You are an agent config generator. Given a plain English workflow, respond ONLY with valid JSON matching this schema:\n\n{schema_json}"
    );

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: scrubbed_description,
        },
    ];

    let llm_stream = provider.complete_stream(messages);

    // We stream SSE events to the client and simultaneously accumulate the
    // full response so we can validate it once the stream ends.
    let sse_stream = async_stream::stream! {
        let mut llm_stream = std::pin::pin!(llm_stream);
        let mut accumulated = String::new();

        while let Some(item) = llm_stream.next().await {
            match item {
                Ok(chunk) => {
                    accumulated.push_str(&chunk);
                    yield Ok::<_, Infallible>(Event::default().data(&chunk));
                }
                Err(e) => {
                    yield Ok(Event::default().event("error").data(e.to_string()));
                    return;
                }
            }
        }

        // Validate the accumulated JSON against AgentConfig.
        match serde_json::from_str::<AgentConfig>(&accumulated) {
            Ok(_) => {
                yield Ok(Event::default().event("done").data("valid"));
            }
            Err(e) => {
                yield Ok(
                    Event::default()
                        .event("validation_error")
                        .data(format!("422: invalid AgentConfig JSON: {e}")),
                );
            }
        }
    };

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}
