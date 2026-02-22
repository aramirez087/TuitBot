//! Provider factory for creating LLM providers from configuration.
//!
//! Reads the `LlmConfig` and returns the correct provider instance,
//! abstracting away provider-specific construction details.

use super::anthropic::AnthropicProvider;
use super::openai_compat::OpenAiCompatProvider;
use super::LlmProvider;
use crate::config::LlmConfig;
use crate::error::LlmError;

/// Create an LLM provider from configuration.
///
/// Returns `Box<dyn LlmProvider>` so callers are decoupled from the concrete type.
/// Logs the constructed provider at info level (without the API key).
pub fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
    match config.provider.as_str() {
        "openai" => {
            let api_key = config
                .api_key
                .as_deref()
                .filter(|k| !k.is_empty())
                .ok_or(LlmError::NotConfigured)?
                .to_string();

            let base_url = config
                .base_url
                .as_deref()
                .filter(|u| !u.is_empty())
                .unwrap_or("https://api.openai.com/v1")
                .to_string();

            let model = if config.model.is_empty() {
                "gpt-4o-mini".to_string()
            } else {
                config.model.clone()
            };

            tracing::info!(provider = "openai", model = %model, base_url = %base_url, "Creating LLM provider");

            Ok(Box::new(OpenAiCompatProvider::new(
                base_url,
                api_key,
                model,
                "openai".to_string(),
            )))
        }
        "ollama" => {
            let base_url = config
                .base_url
                .as_deref()
                .filter(|u| !u.is_empty())
                .unwrap_or("http://localhost:11434/v1")
                .to_string();

            let model = if config.model.is_empty() {
                "llama3.1".to_string()
            } else {
                config.model.clone()
            };

            tracing::info!(provider = "ollama", model = %model, base_url = %base_url, "Creating LLM provider");

            Ok(Box::new(OpenAiCompatProvider::new(
                base_url,
                "ollama".to_string(),
                model,
                "ollama".to_string(),
            )))
        }
        "anthropic" => {
            let api_key = config
                .api_key
                .as_deref()
                .filter(|k| !k.is_empty())
                .ok_or(LlmError::NotConfigured)?
                .to_string();

            let model = if config.model.is_empty() {
                "claude-sonnet-4-5-20250514".to_string()
            } else {
                config.model.clone()
            };

            tracing::info!(provider = "anthropic", model = %model, "Creating LLM provider");

            if let Some(base_url) = config.base_url.as_deref().filter(|u| !u.is_empty()) {
                Ok(Box::new(AnthropicProvider::with_base_url(
                    api_key,
                    model,
                    base_url.to_string(),
                )))
            } else {
                Ok(Box::new(AnthropicProvider::new(api_key, model)))
            }
        }
        "" => Err(LlmError::NotConfigured),
        _other => Err(LlmError::NotConfigured),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LlmConfig;

    #[test]
    fn create_openai_provider() {
        let config = LlmConfig {
            provider: "openai".to_string(),
            api_key: Some("sk-test".to_string()),
            model: "gpt-4o".to_string(),
            base_url: None,
        };
        let provider = create_provider(&config).expect("create");
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn create_openai_requires_api_key() {
        let config = LlmConfig {
            provider: "openai".to_string(),
            api_key: None,
            model: String::new(),
            base_url: None,
        };
        assert!(matches!(
            create_provider(&config),
            Err(LlmError::NotConfigured)
        ));
    }

    #[test]
    fn create_ollama_provider() {
        let config = LlmConfig {
            provider: "ollama".to_string(),
            api_key: None,
            model: String::new(),
            base_url: None,
        };
        let provider = create_provider(&config).expect("create");
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn create_anthropic_provider() {
        let config = LlmConfig {
            provider: "anthropic".to_string(),
            api_key: Some("sk-ant-test".to_string()),
            model: String::new(),
            base_url: None,
        };
        let provider = create_provider(&config).expect("create");
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn create_anthropic_requires_api_key() {
        let config = LlmConfig {
            provider: "anthropic".to_string(),
            api_key: None,
            model: String::new(),
            base_url: None,
        };
        assert!(matches!(
            create_provider(&config),
            Err(LlmError::NotConfigured)
        ));
    }

    #[test]
    fn create_unknown_provider_returns_not_configured() {
        let config = LlmConfig {
            provider: "unknown".to_string(),
            api_key: None,
            model: String::new(),
            base_url: None,
        };
        assert!(matches!(
            create_provider(&config),
            Err(LlmError::NotConfigured)
        ));
    }

    #[test]
    fn create_empty_provider_returns_not_configured() {
        let config = LlmConfig::default();
        assert!(matches!(
            create_provider(&config),
            Err(LlmError::NotConfigured)
        ));
    }

    #[test]
    fn create_openai_with_custom_base_url() {
        let config = LlmConfig {
            provider: "openai".to_string(),
            api_key: Some("key".to_string()),
            model: String::new(),
            base_url: Some("https://custom.api.com/v1".to_string()),
        };
        let provider = create_provider(&config).expect("create");
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn create_ollama_with_custom_base_url() {
        let config = LlmConfig {
            provider: "ollama".to_string(),
            api_key: None,
            model: "custom-model".to_string(),
            base_url: Some("http://remote:11434/v1".to_string()),
        };
        let provider = create_provider(&config).expect("create");
        assert_eq!(provider.name(), "ollama");
    }
}
