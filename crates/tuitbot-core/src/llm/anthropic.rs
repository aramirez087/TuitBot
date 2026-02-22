//! Anthropic native LLM provider.
//!
//! Uses the Anthropic Messages API which has a distinct request format,
//! authentication mechanism, and response structure from OpenAI-compatible endpoints.

use super::{GenerationParams, LlmProvider, LlmResponse, TokenUsage};
use crate::error::LlmError;
use serde::{Deserialize, Serialize};

/// The Anthropic Messages API base URL.
const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";

/// The Anthropic API version header value.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// LLM provider using the Anthropic Messages API.
pub struct AnthropicProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the default base URL.
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: ANTHROPIC_BASE_URL.to_string(),
            api_key,
            model,
        }
    }

    /// Create a new Anthropic provider with a custom base URL (for testing).
    pub fn with_base_url(api_key: String, model: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
            model,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        params: &GenerationParams,
    ) -> Result<LlmResponse, LlmError> {
        let system_prompt = params.system_prompt.as_deref().unwrap_or(system);

        tracing::debug!(
            provider = "anthropic",
            model = %self.model,
            max_tokens = params.max_tokens,
            "LLM request",
        );

        let request = AnthropicRequest {
            model: &self.model,
            max_tokens: params.max_tokens,
            system: if system_prompt.is_empty() {
                None
            } else {
                Some(system_prompt)
            },
            messages: vec![AnthropicMessage {
                role: "user",
                content: user_message,
            }],
            temperature: params.temperature,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();

            if status == 429 || status == 529 {
                let retry_after = if status == 529 {
                    30 // Anthropic "overloaded" default
                } else {
                    response
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(60)
                };
                return Err(LlmError::RateLimited {
                    retry_after_secs: retry_after,
                });
            }

            let body = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<AnthropicErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or(body);

            return Err(LlmError::Api { status, message });
        }

        let body: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Parse(format!("failed to parse Anthropic response: {e}")))?;

        let text = body
            .content
            .into_iter()
            .filter(|b| b.block_type == "text")
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");

        let usage = body.usage.map_or_else(TokenUsage::default, |u| TokenUsage {
            input_tokens: u.input_tokens.unwrap_or(0),
            output_tokens: u.output_tokens.unwrap_or(0),
        });

        tracing::debug!(
            input_tokens = usage.input_tokens,
            output_tokens = usage.output_tokens,
            chars = text.len(),
            "LLM response",
        );

        Ok(LlmResponse {
            text,
            usage,
            model: body.model,
        })
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        self.complete(
            "You are a test assistant.",
            "Say OK",
            &GenerationParams {
                max_tokens: 10,
                ..Default::default()
            },
        )
        .await?;
        Ok(())
    }
}

// --- Internal Serde types ---

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<AnthropicMessage<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<ContentBlock>,
    #[serde(default)]
    model: String,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type", default)]
    block_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: Option<u32>,
    #[serde(default)]
    output_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicErrorDetail,
}

#[derive(Deserialize)]
struct AnthropicErrorDetail {
    #[serde(default)]
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn complete_success() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "content": [{"type": "text", "text": "Hello from Claude"}],
            "model": "claude-sonnet-4-5-20250514",
            "usage": {"input_tokens": 15, "output_tokens": 8}
        });

        Mock::given(method("POST"))
            .and(path("/messages"))
            .and(header("x-api-key", "test-key"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let provider = AnthropicProvider::with_base_url(
            "test-key".into(),
            "claude-sonnet-4-5-20250514".into(),
            server.uri(),
        );

        let resp = provider
            .complete("system prompt", "hello", &GenerationParams::default())
            .await
            .expect("complete");

        assert_eq!(resp.text, "Hello from Claude");
        assert_eq!(resp.model, "claude-sonnet-4-5-20250514");
        assert_eq!(resp.usage.input_tokens, 15);
        assert_eq!(resp.usage.output_tokens, 8);
    }

    #[tokio::test]
    async fn error_429_maps_to_rate_limited() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/messages"))
            .respond_with(
                ResponseTemplate::new(429)
                    .append_header("retry-after", "45")
                    .set_body_json(serde_json::json!({
                        "error": {"type": "rate_limit_error", "message": "Too many requests"}
                    })),
            )
            .mount(&server)
            .await;

        let provider = AnthropicProvider::with_base_url("key".into(), "model".into(), server.uri());

        let err = provider
            .complete("system", "hello", &GenerationParams::default())
            .await
            .unwrap_err();

        match err {
            LlmError::RateLimited { retry_after_secs } => assert_eq!(retry_after_secs, 45),
            other => panic!("expected RateLimited, got: {other}"),
        }
    }

    #[tokio::test]
    async fn error_529_maps_to_rate_limited_with_default_retry() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/messages"))
            .respond_with(ResponseTemplate::new(529).set_body_json(serde_json::json!({
                "error": {"type": "overloaded_error", "message": "Overloaded"}
            })))
            .mount(&server)
            .await;

        let provider = AnthropicProvider::with_base_url("key".into(), "model".into(), server.uri());

        let err = provider
            .complete("system", "hello", &GenerationParams::default())
            .await
            .unwrap_err();

        match err {
            LlmError::RateLimited { retry_after_secs } => assert_eq!(retry_after_secs, 30),
            other => panic!("expected RateLimited, got: {other}"),
        }
    }

    #[tokio::test]
    async fn error_401_maps_to_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/messages"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": {"type": "authentication_error", "message": "Invalid API key"}
            })))
            .mount(&server)
            .await;

        let provider =
            AnthropicProvider::with_base_url("bad-key".into(), "model".into(), server.uri());

        let err = provider
            .complete("system", "hello", &GenerationParams::default())
            .await
            .unwrap_err();

        match err {
            LlmError::Api { status, message } => {
                assert_eq!(status, 401);
                assert!(message.contains("Invalid API key"));
            }
            other => panic!("expected Api, got: {other}"),
        }
    }

    #[tokio::test]
    async fn omits_system_when_empty() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "content": [{"type": "text", "text": "OK"}],
            "model": "claude-sonnet-4-5-20250514"
        });

        Mock::given(method("POST"))
            .and(path("/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let provider = AnthropicProvider::with_base_url("key".into(), "model".into(), server.uri());

        let resp = provider
            .complete("", "hello", &GenerationParams::default())
            .await
            .expect("complete");

        assert_eq!(resp.text, "OK");
    }

    #[test]
    fn provider_name() {
        let provider = AnthropicProvider::new("key".into(), "model".into());
        assert_eq!(provider.name(), "anthropic");
    }
}
