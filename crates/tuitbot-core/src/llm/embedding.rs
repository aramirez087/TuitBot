//! Embedding provider abstraction for semantic search.
//!
//! Provides a trait-based abstraction for embedding providers (OpenAI, Ollama)
//! with typed responses, usage tracking, and health checking.

use std::fmt;

/// Input texts to embed.
pub type EmbeddingInput = Vec<String>;

/// A single embedding vector.
pub type EmbeddingVector = Vec<f32>;

/// Token usage from an embedding request.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingUsage {
    /// Total tokens consumed across all inputs.
    pub total_tokens: u32,
}

/// Response from an embedding request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingResponse {
    /// One embedding vector per input text, in order.
    pub embeddings: Vec<EmbeddingVector>,
    /// The model that produced these embeddings.
    pub model: String,
    /// Dimensionality of each vector.
    pub dimension: usize,
    /// Token usage for this request.
    pub usage: EmbeddingUsage,
}

/// Errors from embedding operations.
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    /// No embedding provider is configured.
    #[error("embedding provider not configured: {0}")]
    NotConfigured(String),

    /// The embedding API returned an error.
    #[error("embedding API error (status {status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Error message from the provider.
        message: String,
    },

    /// Network-level failure communicating with the provider.
    #[error("embedding network error: {0}")]
    Network(String),

    /// Returned vectors have unexpected dimensions.
    #[error("embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// The expected dimension.
        expected: usize,
        /// The actual dimension received.
        actual: usize,
    },

    /// Batch exceeds the provider's maximum.
    #[error("embedding batch too large: {size} exceeds max {max}")]
    BatchTooLarge {
        /// The batch size attempted.
        size: usize,
        /// The provider's maximum batch size.
        max: usize,
    },

    /// Provider rate limit hit.
    #[error("embedding rate limited, retry after {retry_after_secs}s")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },

    /// Internal storage or processing error.
    #[error("embedding internal error: {0}")]
    Internal(String),
}

impl fmt::Display for EmbeddingUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmbeddingUsage(tokens={})", self.total_tokens)
    }
}

/// Trait abstracting embedding provider operations.
///
/// Implementations include `OpenAiEmbeddingProvider` and `OllamaEmbeddingProvider`.
/// Object-safe for use as `Box<dyn EmbeddingProvider>`.
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Display name of this provider (e.g., "openai", "ollama").
    fn name(&self) -> &str;

    /// Vector dimension produced by this provider's model.
    fn dimension(&self) -> usize;

    /// Model identifier string.
    fn model_id(&self) -> &str;

    /// Embed a batch of texts into vectors.
    async fn embed(&self, inputs: EmbeddingInput) -> Result<EmbeddingResponse, EmbeddingError>;

    /// Check if the provider is reachable and configured correctly.
    async fn health_check(&self) -> Result<(), EmbeddingError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_usage_default_is_zero() {
        let usage = EmbeddingUsage::default();
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn embedding_usage_display() {
        let usage = EmbeddingUsage { total_tokens: 42 };
        assert_eq!(usage.to_string(), "EmbeddingUsage(tokens=42)");
    }

    #[test]
    fn embedding_response_fields() {
        let response = EmbeddingResponse {
            embeddings: vec![vec![0.1, 0.2, 0.3]],
            model: "test-model".to_string(),
            dimension: 3,
            usage: EmbeddingUsage { total_tokens: 10 },
        };
        assert_eq!(response.embeddings.len(), 1);
        assert_eq!(response.dimension, 3);
        assert_eq!(response.model, "test-model");
        assert_eq!(response.usage.total_tokens, 10);
    }

    #[test]
    fn embedding_response_serde_roundtrip() {
        let response = EmbeddingResponse {
            embeddings: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            model: "test".to_string(),
            dimension: 2,
            usage: EmbeddingUsage { total_tokens: 5 },
        };
        let json = serde_json::to_string(&response).expect("serialize");
        let deserialized: EmbeddingResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.embeddings.len(), 2);
        assert_eq!(deserialized.dimension, 2);
        assert_eq!(deserialized.usage.total_tokens, 5);
    }

    #[test]
    fn embedding_error_display_not_configured() {
        let err = EmbeddingError::NotConfigured("missing api_key".to_string());
        assert!(err.to_string().contains("not configured"));
    }

    #[test]
    fn embedding_error_display_api() {
        let err = EmbeddingError::Api {
            status: 500,
            message: "server error".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("500"));
        assert!(msg.contains("server error"));
    }

    #[test]
    fn embedding_error_display_dimension_mismatch() {
        let err = EmbeddingError::DimensionMismatch {
            expected: 768,
            actual: 1536,
        };
        let msg = err.to_string();
        assert!(msg.contains("768"));
        assert!(msg.contains("1536"));
    }

    #[test]
    fn embedding_error_display_batch_too_large() {
        let err = EmbeddingError::BatchTooLarge {
            size: 3000,
            max: 2048,
        };
        let msg = err.to_string();
        assert!(msg.contains("3000"));
        assert!(msg.contains("2048"));
    }

    #[test]
    fn embedding_error_display_rate_limited() {
        let err = EmbeddingError::RateLimited {
            retry_after_secs: 30,
        };
        assert!(err.to_string().contains("30"));
    }
}
