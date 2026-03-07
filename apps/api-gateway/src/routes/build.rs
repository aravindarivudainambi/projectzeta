use std::convert::Infallible;

use anyhow::Result as AnyhowResult;
use axum::{
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use core_types::agent::{AgentConfig, AgentStep, Trigger};
use futures_util::{Stream, StreamExt};
use llm_client::{
    openai::OpenAiProvider,
    pii_scrubber::scrub_pii,
    provider::{ChatMessage, LlmProvider},
};
use schemars::schema_for;
use serde::Deserialize;
use std::pin::Pin;
use uuid::Uuid;

/// Request body for the agent builder endpoint.
#[derive(Debug, Deserialize)]
pub struct BuildRequest {
    pub description: String,
}

/// POST /agents/build
///
/// Accepts a plain-English workflow description, scrubs PII, streams the LLM
/// response as SSE events, and validates the final JSON against the
/// `AgentConfig` schema.
pub async fn build_agent(
    Json(payload): Json<BuildRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Scrub PII from the user description before sending to the LLM.
    let scrubbed_description = scrub_pii(&payload.description);

    // Build the AgentConfig JSON Schema to inject into the system prompt.
    let schema = schema_for!(AgentConfig);
    let schema_json = serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".to_string());

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
            content: scrubbed_description.clone(),
        },
    ];

    // Use the remote LLM stream when credentials are available; otherwise
    // fallback to a deterministic local stream so local development still
    // provides a valid, token-streamed AgentConfig response.
    let llm_stream = match OpenAiProvider::from_env() {
        Ok(provider) => provider.complete_stream(messages),
        Err(_) => fallback_agent_config_stream(&scrubbed_description),
    };

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

/// Builds a deterministic `AgentConfig` and streams its JSON output in
/// token-sized chunks for local/offline development.
fn fallback_agent_config_stream(
    description: &str,
) -> Pin<Box<dyn Stream<Item = AnyhowResult<String>> + Send>> {
    let config = fallback_agent_config(description);
    let json = match serde_json::to_string(&config) {
        Ok(serialized) => serialized,
        Err(_) => "{}".to_string(),
    };

    Box::pin(async_stream::try_stream! {
        for token in json_tokens(&json) {
            yield token;
        }
    })
}

/// Constructs a best-effort `AgentConfig` from plain-English workflow text.
fn fallback_agent_config(description: &str) -> AgentConfig {
    let trigger = infer_trigger(description);
    let step_phrases = infer_step_phrases(description);

    let steps = if step_phrases.is_empty() {
        vec![AgentStep {
            id: Uuid::new_v4(),
            name: "Process workflow request".to_string(),
        }]
    } else {
        step_phrases
            .into_iter()
            .map(|name| AgentStep {
                id: Uuid::new_v4(),
                name,
            })
            .collect()
    };

    AgentConfig {
        id: Uuid::new_v4(),
        name: infer_agent_name(description),
        trigger,
        steps,
    }
}

/// Splits JSON into small, safe chunks that mimic token-by-token streaming.
fn json_tokens(json: &str) -> Vec<String> {
    json.chars().map(|ch| ch.to_string()).collect()
}

/// Derives a stable display name from the first meaningful words.
fn infer_agent_name(description: &str) -> String {
    let words: Vec<&str> = description
        .split_whitespace()
        .filter(|w| !w.trim().is_empty())
        .take(6)
        .collect();

    if words.is_empty() {
        "Generated Agent".to_string()
    } else {
        format!("{} Agent", words.join(" "))
    }
}

/// Infers an `AgentConfig` trigger from lightweight workflow keywords.
fn infer_trigger(description: &str) -> Trigger {
    let lower = description.to_lowercase();

    if lower.contains("every day") || lower.contains("daily") {
        return Trigger::Schedule {
            cron: "0 9 * * *".to_string(),
        };
    }

    if lower.contains("every hour") || lower.contains("hourly") {
        return Trigger::Schedule {
            cron: "0 * * * *".to_string(),
        };
    }

    if lower.contains("when") || lower.contains("on ") {
        return Trigger::Event {
            source: "workflow-input".to_string(),
            event: "description-submitted".to_string(),
        };
    }

    Trigger::Manual
}

/// Infers step names from comma/connector-delimited clauses.
fn infer_step_phrases(description: &str) -> Vec<String> {
    let normalized = description
        .replace(" and then ", ",")
        .replace(" then ", ",")
        .replace(" and ", ",");

    normalized
        .split([',', ';'])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(sentence_case)
        .collect()
}

/// Converts a phrase into a clean, sentence-cased step label.
fn sentence_case(input: &str) -> String {
    let cleaned = input.trim().trim_matches('.');
    let mut chars = cleaned.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => "Untitled Step".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{fallback_agent_config, json_tokens};

    #[test]
    fn fallback_config_serializes_and_parses_as_agent_config() {
        let config = fallback_agent_config(
            "When a lead submits a form, enrich with CRM, notify sales, and create a task",
        );

        let json = serde_json::to_string(&config).expect("fallback config should serialize");
        let parsed: core_types::agent::AgentConfig =
            serde_json::from_str(&json).expect("serialized config should parse");

        assert!(!parsed.steps.is_empty());
    }

    #[test]
    fn json_tokens_round_trip_to_original_json() {
        let sample = r#"{"status":"ok"}"#;
        let reconstructed = json_tokens(sample).join("");
        assert_eq!(reconstructed, sample);
    }
}
