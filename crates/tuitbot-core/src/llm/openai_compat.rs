//! OpenAI-compatible LLM provider.
//!
//! Works with both OpenAI (cloud) and Ollama (local) since they share
//! the same chat completions request/response format.

use super::{GenerationParams, LlmProvider, LlmResponse, TokenUsage};
use crate::error::LlmError;
use serde::{Deserialize, Serialize};

/// An LLM provider using the OpenAI chat completions API format.
///
/// Compatible with OpenAI, Ollama, and any OpenAI-compatible endpoint.
pub struct OpenAiCompatProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    provider_name: String,
}

impl OpenAiCompatProvider {
    /// Create a new OpenAI-compatible provider.
    pub fn new(base_url: String, api_key: String, model: String, provider_name: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
            model,
            provider_name,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }

    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        params: &GenerationParams,
    ) -> Result<LlmResponse, LlmError> {
        let system_prompt = params.system_prompt.as_deref().unwrap_or(system);

        tracing::debug!(
            provider = %self.provider_name,
            model = %self.model,
            max_tokens = params.max_tokens,
            "LLM request",
        );

        let request = ChatCompletionRequest {
            model: &self.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: user_message,
                },
            ],
            max_tokens: params.max_tokens,
            temperature: params.temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();

            if status == 429 {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);
                return Err(LlmError::RateLimited {
                    retry_after_secs: retry_after,
                });
            }

            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api {
                status,
                message: body,
            });
        }

        let body: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Parse(format!("failed to parse response: {e}")))?;

        let text = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let usage = body.usage.map_or_else(TokenUsage::default, |u| TokenUsage {
            input_tokens: u.prompt_tokens.unwrap_or(0),
            output_tokens: u.completion_tokens.unwrap_or(0),
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
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    model: String,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    #[serde(default)]
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    prompt_tokens: Option<u32>,
    #[serde(default)]
    completion_tokens: Option<u32>,
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
            "choices": [{"message": {"content": "Hello world"}}],
            "model": "gpt-4o-mini",
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let provider = OpenAiCompatProvider::new(
            server.uri(),
            "test-key".into(),
            "gpt-4o-mini".into(),
            "openai".into(),
        );

        let resp = provider
            .complete("system", "hello", &GenerationParams::default())
            .await
            .expect("complete");

        assert_eq!(resp.text, "Hello world");
        assert_eq!(resp.model, "gpt-4o-mini");
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 5);
    }

    #[tokio::test]
    async fn complete_missing_usage_defaults_to_zero() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "choices": [{"message": {"content": "OK"}}],
            "model": "llama3.1"
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let provider = OpenAiCompatProvider::new(
            server.uri(),
            "ollama".into(),
            "llama3.1".into(),
            "ollama".into(),
        );

        let resp = provider
            .complete("system", "hello", &GenerationParams::default())
            .await
            .expect("complete");

        assert_eq!(resp.usage.input_tokens, 0);
        assert_eq!(resp.usage.output_tokens, 0);
    }

    #[tokio::test]
    async fn error_429_maps_to_rate_limited() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(429)
                    .append_header("retry-after", "30")
                    .set_body_string("rate limited"),
            )
            .mount(&server)
            .await;

        let provider =
            OpenAiCompatProvider::new(server.uri(), "key".into(), "model".into(), "openai".into());

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
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("invalid api key"))
            .mount(&server)
            .await;

        let provider = OpenAiCompatProvider::new(
            server.uri(),
            "bad-key".into(),
            "model".into(),
            "openai".into(),
        );

        let err = provider
            .complete("system", "hello", &GenerationParams::default())
            .await
            .unwrap_err();

        match err {
            LlmError::Api { status, message } => {
                assert_eq!(status, 401);
                assert!(message.contains("invalid api key"));
            }
            other => panic!("expected Api, got: {other}"),
        }
    }

    #[tokio::test]
    async fn error_500_maps_to_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&server)
            .await;

        let provider =
            OpenAiCompatProvider::new(server.uri(), "key".into(), "model".into(), "openai".into());

        let err = provider
            .complete("system", "hello", &GenerationParams::default())
            .await
            .unwrap_err();

        match err {
            LlmError::Api { status, .. } => assert_eq!(status, 500),
            other => panic!("expected Api, got: {other}"),
        }
    }

    #[tokio::test]
    async fn system_prompt_override() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "choices": [{"message": {"content": "overridden"}}],
            "model": "test"
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let provider =
            OpenAiCompatProvider::new(server.uri(), "key".into(), "model".into(), "test".into());

        let params = GenerationParams {
            system_prompt: Some("Override prompt".to_string()),
            ..Default::default()
        };

        let resp = provider
            .complete("original system", "hello", &params)
            .await
            .expect("complete");

        assert_eq!(resp.text, "overridden");
    }

    #[test]
    fn provider_name() {
        let provider = OpenAiCompatProvider::new(
            "http://localhost".into(),
            "key".into(),
            "model".into(),
            "ollama".into(),
        );
        assert_eq!(provider.name(), "ollama");
    }
}
