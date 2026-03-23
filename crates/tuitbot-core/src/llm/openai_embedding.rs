//! OpenAI embedding provider implementation.
//!
//! Supports the OpenAI embeddings API (`/v1/embeddings`) with
//! `text-embedding-3-small` as the default model.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::embedding::{
    EmbeddingError, EmbeddingInput, EmbeddingProvider, EmbeddingResponse, EmbeddingUsage,
    EmbeddingVector,
};

/// Default OpenAI embeddings API base URL.
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// Default model for OpenAI embeddings.
const DEFAULT_MODEL: &str = "text-embedding-3-small";

/// Dimension of text-embedding-3-small.
const DEFAULT_DIMENSION: usize = 1536;

/// OpenAI's maximum batch size for embeddings.
const MAX_BATCH_SIZE: usize = 2048;

/// OpenAI embedding provider.
pub struct OpenAiEmbeddingProvider {
    client: Client,
    base_url: String,
    model: String,
    dimension: usize,
    api_key: String,
    max_batch_size: usize,
}

impl OpenAiEmbeddingProvider {
    /// Create a new OpenAI embedding provider.
    pub fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            dimension: DEFAULT_DIMENSION,
            api_key,
            max_batch_size: MAX_BATCH_SIZE,
        }
    }
}

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbedResponseBody {
    data: Vec<EmbedDataItem>,
    model: String,
    usage: EmbedUsageBody,
}

#[derive(Deserialize)]
struct EmbedDataItem {
    embedding: EmbeddingVector,
}

#[derive(Deserialize)]
struct EmbedUsageBody {
    total_tokens: u32,
}

#[async_trait::async_trait]
impl EmbeddingProvider for OpenAiEmbeddingProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    async fn embed(&self, inputs: EmbeddingInput) -> Result<EmbeddingResponse, EmbeddingError> {
        if inputs.is_empty() {
            return Ok(EmbeddingResponse {
                embeddings: vec![],
                model: self.model.clone(),
                dimension: self.dimension,
                usage: EmbeddingUsage::default(),
            });
        }

        if inputs.len() > self.max_batch_size {
            return Err(EmbeddingError::BatchTooLarge {
                size: inputs.len(),
                max: self.max_batch_size,
            });
        }

        let url = format!("{}/embeddings", self.base_url);
        let body = EmbedRequest {
            model: self.model.clone(),
            input: inputs,
        };

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbeddingError::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        if status == 429 {
            return Err(EmbeddingError::RateLimited {
                retry_after_secs: 60,
            });
        }
        if !resp.status().is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(EmbeddingError::Api { status, message });
        }

        let parsed: EmbedResponseBody = resp.json().await.map_err(|e| EmbeddingError::Api {
            status: 0,
            message: format!("failed to parse response: {e}"),
        })?;

        // Validate dimensions
        for item in &parsed.data {
            if item.embedding.len() != self.dimension {
                return Err(EmbeddingError::DimensionMismatch {
                    expected: self.dimension,
                    actual: item.embedding.len(),
                });
            }
        }

        Ok(EmbeddingResponse {
            embeddings: parsed.data.into_iter().map(|d| d.embedding).collect(),
            model: parsed.model,
            dimension: self.dimension,
            usage: EmbeddingUsage {
                total_tokens: parsed.usage.total_tokens,
            },
        })
    }

    async fn health_check(&self) -> Result<(), EmbeddingError> {
        self.embed(vec!["hello".to_string()]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn mock_success_body() -> serde_json::Value {
        serde_json::json!({
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "index": 0,
                    "embedding": vec![0.1_f32; 1536]
                },
                {
                    "object": "embedding",
                    "index": 1,
                    "embedding": vec![0.2_f32; 1536]
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 10,
                "total_tokens": 10
            }
        })
    }

    #[tokio::test]
    async fn successful_batch_embedding() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_success_body()))
            .mount(&server)
            .await;

        let provider =
            OpenAiEmbeddingProvider::new("test-key".to_string(), None, Some(server.uri()));

        let result = provider
            .embed(vec!["hello".to_string(), "world".to_string()])
            .await
            .expect("should succeed");

        assert_eq!(result.embeddings.len(), 2);
        assert_eq!(result.dimension, 1536);
        assert_eq!(result.usage.total_tokens, 10);
    }

    #[tokio::test]
    async fn api_error_500() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&server)
            .await;

        let provider =
            OpenAiEmbeddingProvider::new("test-key".to_string(), None, Some(server.uri()));

        let err = provider.embed(vec!["test".to_string()]).await.unwrap_err();

        matches!(err, EmbeddingError::Api { status: 500, .. });
    }

    #[tokio::test]
    async fn rate_limit_429() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let provider =
            OpenAiEmbeddingProvider::new("test-key".to_string(), None, Some(server.uri()));

        let err = provider.embed(vec!["test".to_string()]).await.unwrap_err();

        matches!(err, EmbeddingError::RateLimited { .. });
    }

    #[tokio::test]
    async fn dimension_mismatch() {
        let server = MockServer::start().await;
        let bad_body = serde_json::json!({
            "object": "list",
            "data": [{
                "object": "embedding",
                "index": 0,
                "embedding": vec![0.1_f32; 768]
            }],
            "model": "text-embedding-3-small",
            "usage": { "prompt_tokens": 5, "total_tokens": 5 }
        });

        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(bad_body))
            .mount(&server)
            .await;

        let provider =
            OpenAiEmbeddingProvider::new("test-key".to_string(), None, Some(server.uri()));

        let err = provider.embed(vec!["test".to_string()]).await.unwrap_err();

        matches!(
            err,
            EmbeddingError::DimensionMismatch {
                expected: 1536,
                actual: 768
            }
        );
    }

    #[tokio::test]
    async fn empty_batch_returns_empty() {
        let provider = OpenAiEmbeddingProvider::new("test-key".to_string(), None, None);

        let result = provider.embed(vec![]).await.expect("should succeed");
        assert!(result.embeddings.is_empty());
    }

    #[tokio::test]
    async fn batch_exceeding_max_returns_error() {
        let provider = OpenAiEmbeddingProvider::new("test-key".to_string(), None, None);

        let inputs: Vec<String> = (0..2049).map(|i| format!("text {i}")).collect();
        let err = provider.embed(inputs).await.unwrap_err();
        matches!(err, EmbeddingError::BatchTooLarge { .. });
    }

    #[test]
    fn provider_accessors() {
        let provider =
            OpenAiEmbeddingProvider::new("key".to_string(), Some("custom-model".to_string()), None);
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.dimension(), 1536);
        assert_eq!(provider.model_id(), "custom-model");
    }
}
