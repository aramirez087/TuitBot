//! LLM provider abstraction and implementations.
//!
//! Provides a trait-based abstraction for LLM providers (OpenAI, Anthropic, Ollama)
//! with typed responses, token usage tracking, and health checking.

pub mod anthropic;
pub mod factory;
pub mod openai_compat;
pub mod pricing;

use crate::error::LlmError;

/// Token usage information from an LLM completion.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the input/prompt.
    pub input_tokens: u32,
    /// Number of tokens in the output/completion.
    pub output_tokens: u32,
}

impl TokenUsage {
    /// Accumulate token counts from another usage record (e.g. across retries).
    pub fn accumulate(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
    }
}

/// Response from an LLM completion request.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// The generated text content.
    pub text: String,
    /// Token usage for this completion.
    pub usage: TokenUsage,
    /// The model that produced this response.
    pub model: String,
}

/// Parameters controlling LLM generation behavior.
#[derive(Debug, Clone)]
pub struct GenerationParams {
    /// Maximum number of tokens to generate.
    pub max_tokens: u32,
    /// Sampling temperature (0.0 = deterministic, 1.0+ = creative).
    pub temperature: f32,
    /// Optional system prompt override. If `Some`, replaces the caller's system prompt.
    pub system_prompt: Option<String>,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            system_prompt: None,
        }
    }
}

/// Trait abstracting all LLM provider operations.
///
/// Implementations include `OpenAiCompatProvider` (for OpenAI and Ollama)
/// and `AnthropicProvider`. The trait is object-safe for use as `Box<dyn LlmProvider>`.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Returns the display name of this provider (e.g., "openai", "anthropic", "ollama").
    fn name(&self) -> &str;

    /// Send a completion request to the LLM.
    ///
    /// If `params.system_prompt` is `Some`, it overrides the `system` parameter.
    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        params: &GenerationParams,
    ) -> Result<LlmResponse, LlmError>;

    /// Check if the provider is reachable and configured correctly.
    async fn health_check(&self) -> Result<(), LlmError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_usage_default_is_zero() {
        let usage = TokenUsage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
    }

    #[test]
    fn token_usage_accumulate() {
        let mut total = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        };
        let other = TokenUsage {
            input_tokens: 200,
            output_tokens: 80,
        };
        total.accumulate(&other);
        assert_eq!(total.input_tokens, 300);
        assert_eq!(total.output_tokens, 130);
    }

    #[test]
    fn token_usage_accumulate_multiple() {
        let mut total = TokenUsage::default();
        for i in 1..=5 {
            total.accumulate(&TokenUsage {
                input_tokens: i * 10,
                output_tokens: i * 5,
            });
        }
        // Sum of 10+20+30+40+50 = 150, sum of 5+10+15+20+25 = 75
        assert_eq!(total.input_tokens, 150);
        assert_eq!(total.output_tokens, 75);
    }

    #[test]
    fn token_usage_accumulate_zero() {
        let mut total = TokenUsage {
            input_tokens: 42,
            output_tokens: 17,
        };
        total.accumulate(&TokenUsage::default());
        assert_eq!(total.input_tokens, 42);
        assert_eq!(total.output_tokens, 17);
    }

    #[test]
    fn generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.max_tokens, 512);
        assert!((params.temperature - 0.7).abs() < f32::EPSILON);
        assert!(params.system_prompt.is_none());
    }

    #[test]
    fn generation_params_with_system_prompt() {
        let params = GenerationParams {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            ..Default::default()
        };
        assert_eq!(
            params.system_prompt.as_deref(),
            Some("You are a helpful assistant.")
        );
        assert_eq!(params.max_tokens, 512);
    }

    #[test]
    fn llm_response_fields() {
        let response = LlmResponse {
            text: "Hello, world!".to_string(),
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 3,
            },
            model: "gpt-4o-mini".to_string(),
        };
        assert_eq!(response.text, "Hello, world!");
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.usage.output_tokens, 3);
        assert_eq!(response.model, "gpt-4o-mini");
    }

    #[test]
    fn token_usage_serde_roundtrip() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        };
        let json = serde_json::to_string(&usage).expect("serialize");
        let deserialized: TokenUsage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.input_tokens, 100);
        assert_eq!(deserialized.output_tokens, 50);
    }

    #[test]
    fn token_usage_clone() {
        let usage = TokenUsage {
            input_tokens: 42,
            output_tokens: 17,
        };
        let cloned = usage.clone();
        assert_eq!(cloned.input_tokens, 42);
        assert_eq!(cloned.output_tokens, 17);
    }

    #[test]
    fn generation_params_clone() {
        let params = GenerationParams {
            max_tokens: 1024,
            temperature: 0.5,
            system_prompt: Some("test".to_string()),
        };
        let cloned = params.clone();
        assert_eq!(cloned.max_tokens, 1024);
        assert!((cloned.temperature - 0.5).abs() < f32::EPSILON);
        assert_eq!(cloned.system_prompt.as_deref(), Some("test"));
    }
}
