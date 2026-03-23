//! Factory for creating embedding providers from configuration.

use crate::config::EmbeddingConfig;

use super::embedding::{EmbeddingError, EmbeddingProvider};
use super::ollama_embedding::OllamaEmbeddingProvider;
use super::openai_embedding::OpenAiEmbeddingProvider;

/// Create an embedding provider from the given configuration.
pub fn create_embedding_provider(
    config: &EmbeddingConfig,
) -> Result<Box<dyn EmbeddingProvider>, EmbeddingError> {
    match config.provider.as_str() {
        "openai" => {
            let api_key = config.api_key.clone().ok_or_else(|| {
                EmbeddingError::NotConfigured(
                    "OpenAI embedding provider requires an api_key".to_string(),
                )
            })?;
            Ok(Box::new(OpenAiEmbeddingProvider::new(
                api_key,
                config.model.clone(),
                config.base_url.clone(),
            )))
        }
        "ollama" => Ok(Box::new(OllamaEmbeddingProvider::new(
            config.model.clone(),
            config.base_url.clone(),
        ))),
        other => Err(EmbeddingError::NotConfigured(format!(
            "unknown embedding provider: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_creates_openai_with_api_key() {
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            api_key: Some("sk-test".to_string()),
            ..Default::default()
        };
        let provider = create_embedding_provider(&config).expect("should create");
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.dimension(), 1536);
    }

    #[test]
    fn factory_creates_ollama_without_api_key() {
        let config = EmbeddingConfig {
            provider: "ollama".to_string(),
            ..Default::default()
        };
        let provider = create_embedding_provider(&config).expect("should create");
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.dimension(), 768);
    }

    #[test]
    fn factory_returns_not_configured_for_unknown_provider() {
        let config = EmbeddingConfig {
            provider: "unknown".to_string(),
            ..Default::default()
        };
        let result = create_embedding_provider(&config);
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("unknown"));
    }

    #[test]
    fn factory_returns_not_configured_for_openai_without_api_key() {
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            api_key: None,
            ..Default::default()
        };
        let result = create_embedding_provider(&config);
        assert!(result.is_err());
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("api_key"));
    }
}
