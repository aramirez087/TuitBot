//! LLM and embedding provider configuration types.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// LLM
// ---------------------------------------------------------------------------

/// LLM provider configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LlmConfig {
    /// LLM provider name: "openai", "anthropic", "ollama", or "groq".
    #[serde(default)]
    pub provider: String,

    /// API key for the LLM provider (not needed for ollama).
    #[serde(default)]
    pub api_key: Option<String>,

    /// Provider-specific model name.
    #[serde(default)]
    pub model: String,

    /// Override URL for custom endpoints.
    #[serde(default)]
    pub base_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Embedding
// ---------------------------------------------------------------------------

/// Embedding provider configuration for semantic search indexing.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingConfig {
    /// Embedding provider: "ollama" (default) or "openai".
    #[serde(default = "default_embedding_provider")]
    pub provider: String,

    /// API key for the embedding provider (not needed for ollama).
    #[serde(default)]
    pub api_key: Option<String>,

    /// Provider-specific model name override.
    #[serde(default)]
    pub model: Option<String>,

    /// Override URL for custom endpoints.
    #[serde(default)]
    pub base_url: Option<String>,

    /// Maximum texts to embed per batch.
    #[serde(default = "default_embedding_batch_size")]
    pub batch_size: usize,

    /// Whether embedding indexing is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: default_embedding_provider(),
            api_key: None,
            model: None,
            base_url: None,
            batch_size: default_embedding_batch_size(),
            enabled: true,
        }
    }
}

fn default_embedding_provider() -> String {
    "ollama".to_string()
}

fn default_embedding_batch_size() -> usize {
    100
}

fn default_true() -> bool {
    true
}
