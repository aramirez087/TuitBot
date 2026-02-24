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
