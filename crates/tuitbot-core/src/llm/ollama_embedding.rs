//! Ollama embedding provider implementation.
//!
//! Supports the Ollama embeddings API (`/api/embed`) with
//! `nomic-embed-text` as the default model.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::embedding::{
    EmbeddingError, EmbeddingInput, EmbeddingProvider, EmbeddingResponse, EmbeddingUsage,
    EmbeddingVector,
};

/// Default Ollama API base URL.
const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Default model for Ollama embeddings.
const DEFAULT_MODEL: &str = "nomic-embed-text";

/// Dimension of nomic-embed-text.
const DEFAULT_DIMENSION: usize = 768;

/// Ollama's practical batch size limit.
const MAX_BATCH_SIZE: usize = 100;

/// Ollama embedding provider.
pub struct OllamaEmbeddingProvider {
    client: Client,
    base_url: String,
    model: String,
    dimension: usize,
    max_batch_size: usize,
}

impl OllamaEmbeddingProvider {
    /// Create a new Ollama embedding provider.
    pub fn new(model: Option<String>, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            dimension: DEFAULT_DIMENSION,
            max_batch_size: MAX_BATCH_SIZE,
        }
    }
}

#[derive(Serialize)]
struct OllamaEmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Vec<EmbeddingVector>,
}

#[async_trait::async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    fn name(&self) -> &str {
        "ollama"
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

        let url = format!("{}/api/embed", self.base_url);
        let body = OllamaEmbedRequest {
            model: self.model.clone(),
            input: inputs,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbeddingError::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(EmbeddingError::Api { status, message });
        }

        let parsed: OllamaEmbedResponse = resp.json().await.map_err(|e| EmbeddingError::Api {
            status: 0,
            message: format!("failed to parse response: {e}"),
        })?;

        let dimension = parsed
            .embeddings
            .first()
            .map(|v| v.len())
            .unwrap_or(self.dimension);

        Ok(EmbeddingResponse {
            embeddings: parsed.embeddings,
            model: self.model.clone(),
            dimension,
            usage: EmbeddingUsage::default(),
        })
    }

    async fn health_check(&self) -> Result<(), EmbeddingError> {
        let url = format!("{}/api/tags", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| EmbeddingError::Network(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn successful_batch_embedding() {
        let server = MockServer::start().await;
        let response_body = serde_json::json!({
            "embeddings": [
                vec![0.1_f32; 768],
                vec![0.2_f32; 768]
            ]
        });

        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&server)
            .await;

        let provider = OllamaEmbeddingProvider::new(None, Some(server.uri()));

        let result = provider
            .embed(vec!["hello".to_string(), "world".to_string()])
            .await
            .expect("should succeed");

        assert_eq!(result.embeddings.len(), 2);
        assert_eq!(result.dimension, 768);
    }

    #[tokio::test]
    async fn network_error() {
        let provider = OllamaEmbeddingProvider::new(None, Some("http://127.0.0.1:1".to_string()));

        let err = provider.embed(vec!["test".to_string()]).await.unwrap_err();

        matches!(err, EmbeddingError::Network(_));
    }

    #[tokio::test]
    async fn empty_batch_returns_empty() {
        let provider = OllamaEmbeddingProvider::new(None, None);

        let result = provider.embed(vec![]).await.expect("should succeed");
        assert!(result.embeddings.is_empty());
    }

    #[test]
    fn provider_accessors() {
        let provider = OllamaEmbeddingProvider::new(Some("mxbai-embed-large".to_string()), None);
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.dimension(), 768);
        assert_eq!(provider.model_id(), "mxbai-embed-large");
    }

    #[tokio::test]
    async fn batch_exceeding_max_returns_error() {
        let provider = OllamaEmbeddingProvider::new(None, None);
        let inputs: Vec<String> = (0..101).map(|i| format!("text {i}")).collect();
        let err = provider.embed(inputs).await.unwrap_err();
        assert!(
            matches!(
                err,
                EmbeddingError::BatchTooLarge {
                    size: 101,
                    max: 100
                }
            ),
            "expected BatchTooLarge, got: {err}"
        );
    }

    #[tokio::test]
    async fn api_error_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(500).set_body_string("model not found"))
            .mount(&server)
            .await;

        let provider = OllamaEmbeddingProvider::new(None, Some(server.uri()));
        let err = provider.embed(vec!["test".to_string()]).await.unwrap_err();
        assert!(
            matches!(err, EmbeddingError::Api { status: 500, ref message } if message.contains("model not found")),
            "expected Api error with status 500, got: {err}"
        );
    }

    #[tokio::test]
    async fn health_check_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"models": []})),
            )
            .mount(&server)
            .await;

        let provider = OllamaEmbeddingProvider::new(None, Some(server.uri()));
        provider
            .health_check()
            .await
            .expect("health check should succeed");
    }

    #[tokio::test]
    async fn health_check_network_failure() {
        let provider = OllamaEmbeddingProvider::new(None, Some("http://127.0.0.1:1".to_string()));
        let err = provider.health_check().await.unwrap_err();
        assert!(
            matches!(err, EmbeddingError::Network(_)),
            "expected Network error, got: {err}"
        );
    }

    #[tokio::test]
    async fn malformed_response_returns_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;

        let provider = OllamaEmbeddingProvider::new(None, Some(server.uri()));
        let err = provider.embed(vec!["test".to_string()]).await.unwrap_err();
        assert!(
            matches!(err, EmbeddingError::Api { status: 0, .. }),
            "expected Api parse error, got: {err}"
        );
    }

    #[tokio::test]
    async fn dimension_inferred_from_response() {
        let server = MockServer::start().await;
        let response_body = serde_json::json!({
            "embeddings": [vec![0.1_f32; 768]]
        });

        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&server)
            .await;

        let provider = OllamaEmbeddingProvider::new(None, Some(server.uri()));
        let result = provider
            .embed(vec!["hello".to_string()])
            .await
            .expect("should succeed");

        assert_eq!(result.dimension, 768);
        assert_eq!(result.embeddings.len(), 1);
    }

    #[test]
    fn default_values() {
        let provider = OllamaEmbeddingProvider::new(None, None);
        assert_eq!(provider.model_id(), "nomic-embed-text");
        assert_eq!(provider.dimension(), 768);
        assert_eq!(provider.base_url, "http://localhost:11434");
    }
}
