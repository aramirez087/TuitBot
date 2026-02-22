//! Configuration management for ReplyGuy.
//!
//! Supports three-layer configuration loading:
//! 1. Built-in defaults
//! 2. TOML config file (`~/.replyguy/config.toml`)
//! 3. Environment variable overrides (`REPLYGUY_` prefix)
//!
//! CLI flag overrides are applied by the binary crate after loading.

mod defaults;

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Top-level configuration for the ReplyGuy agent.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    /// X API credentials.
    #[serde(default)]
    pub x_api: XApiConfig,

    /// Authentication settings.
    #[serde(default)]
    pub auth: AuthConfig,

    /// Business profile for content targeting.
    #[serde(default)]
    pub business: BusinessProfile,

    /// Scoring engine weights and threshold.
    #[serde(default)]
    pub scoring: ScoringConfig,

    /// Safety limits for API actions.
    #[serde(default)]
    pub limits: LimitsConfig,

    /// Automation interval settings.
    #[serde(default)]
    pub intervals: IntervalsConfig,

    /// LLM provider configuration.
    #[serde(default)]
    pub llm: LlmConfig,

    /// Data storage configuration.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Logging and observability settings.
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// X API credentials.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct XApiConfig {
    /// OAuth 2.0 client ID.
    #[serde(default)]
    pub client_id: String,

    /// OAuth 2.0 client secret (optional for public clients).
    #[serde(default)]
    pub client_secret: Option<String>,
}

/// Authentication mode and callback settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Auth mode: "manual" or "local_callback".
    #[serde(default = "default_auth_mode")]
    pub mode: String,

    /// Host for local callback server.
    #[serde(default = "default_callback_host")]
    pub callback_host: String,

    /// Port for local callback server.
    #[serde(default = "default_callback_port")]
    pub callback_port: u16,
}

/// Business profile for content targeting and keyword matching.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BusinessProfile {
    /// Name of the user's product.
    #[serde(default)]
    pub product_name: String,

    /// One-line description of the product.
    #[serde(default)]
    pub product_description: String,

    /// URL to the product website.
    #[serde(default)]
    pub product_url: Option<String>,

    /// Description of the target audience.
    #[serde(default)]
    pub target_audience: String,

    /// Keywords for tweet discovery.
    #[serde(default)]
    pub product_keywords: Vec<String>,

    /// Competitor-related keywords for discovery.
    #[serde(default)]
    pub competitor_keywords: Vec<String>,

    /// Topics for content generation.
    #[serde(default)]
    pub industry_topics: Vec<String>,
}

/// Scoring engine weights and threshold.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoringConfig {
    /// Minimum score (0-100) to trigger a reply.
    #[serde(default = "default_threshold")]
    pub threshold: u32,

    /// Maximum points for keyword relevance.
    #[serde(default = "default_keyword_relevance_max")]
    pub keyword_relevance_max: f32,

    /// Maximum points for author follower count.
    #[serde(default = "default_follower_count_max")]
    pub follower_count_max: f32,

    /// Maximum points for tweet recency.
    #[serde(default = "default_recency_max")]
    pub recency_max: f32,

    /// Maximum points for engagement rate.
    #[serde(default = "default_engagement_rate_max")]
    pub engagement_rate_max: f32,
}

/// Safety limits for API actions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    /// Maximum replies per day.
    #[serde(default = "default_max_replies_per_day")]
    pub max_replies_per_day: u32,

    /// Maximum original tweets per day.
    #[serde(default = "default_max_tweets_per_day")]
    pub max_tweets_per_day: u32,

    /// Maximum threads per week.
    #[serde(default = "default_max_threads_per_week")]
    pub max_threads_per_week: u32,

    /// Minimum delay between actions in seconds.
    #[serde(default = "default_min_action_delay_seconds")]
    pub min_action_delay_seconds: u64,

    /// Maximum delay between actions in seconds.
    #[serde(default = "default_max_action_delay_seconds")]
    pub max_action_delay_seconds: u64,
}

/// Automation interval settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntervalsConfig {
    /// Seconds between mention checks.
    #[serde(default = "default_mentions_check_seconds")]
    pub mentions_check_seconds: u64,

    /// Seconds between discovery searches.
    #[serde(default = "default_discovery_search_seconds")]
    pub discovery_search_seconds: u64,

    /// Seconds for content post window.
    #[serde(default = "default_content_post_window_seconds")]
    pub content_post_window_seconds: u64,

    /// Seconds between thread posts.
    #[serde(default = "default_thread_interval_seconds")]
    pub thread_interval_seconds: u64,
}

/// LLM provider configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LlmConfig {
    /// LLM provider name: "openai", "anthropic", or "ollama".
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

/// Data storage configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Path to the SQLite database file.
    #[serde(default = "default_db_path")]
    pub db_path: String,

    /// Number of days to retain data.
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

/// Logging and observability settings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Seconds between periodic status summaries (0 = disabled).
    #[serde(default)]
    pub status_interval_seconds: u64,
}

// --- Default value functions for serde ---

fn default_auth_mode() -> String {
    "manual".to_string()
}
fn default_callback_host() -> String {
    "127.0.0.1".to_string()
}
fn default_callback_port() -> u16 {
    8080
}
fn default_threshold() -> u32 {
    70
}
fn default_keyword_relevance_max() -> f32 {
    40.0
}
fn default_follower_count_max() -> f32 {
    20.0
}
fn default_recency_max() -> f32 {
    15.0
}
fn default_engagement_rate_max() -> f32 {
    25.0
}
fn default_max_replies_per_day() -> u32 {
    20
}
fn default_max_tweets_per_day() -> u32 {
    4
}
fn default_max_threads_per_week() -> u32 {
    1
}
fn default_min_action_delay_seconds() -> u64 {
    30
}
fn default_max_action_delay_seconds() -> u64 {
    120
}
fn default_mentions_check_seconds() -> u64 {
    300
}
fn default_discovery_search_seconds() -> u64 {
    600
}
fn default_content_post_window_seconds() -> u64 {
    14400
}
fn default_thread_interval_seconds() -> u64 {
    604800
}
fn default_db_path() -> String {
    "~/.replyguy/replyguy.db".to_string()
}
fn default_retention_days() -> u32 {
    90
}

impl Config {
    /// Load configuration from a TOML file with environment variable overrides.
    ///
    /// The loading sequence:
    /// 1. Determine config file path (argument > `REPLYGUY_CONFIG` env var > default)
    /// 2. Parse TOML file (or use defaults if default path doesn't exist)
    /// 3. Apply environment variable overrides
    pub fn load(config_path: Option<&str>) -> Result<Config, ConfigError> {
        let (path, explicit) = Self::resolve_config_path(config_path);

        let mut config = match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str::<Config>(&contents)
                .map_err(|e| ConfigError::ParseError { source: e })?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if explicit {
                    return Err(ConfigError::FileNotFound {
                        path: path.display().to_string(),
                    });
                }
                Config::default()
            }
            Err(_) => {
                return Err(ConfigError::FileNotFound {
                    path: path.display().to_string(),
                });
            }
        };

        config.apply_env_overrides()?;

        Ok(config)
    }

    /// Load configuration and validate it, returning all validation errors at once.
    pub fn load_and_validate(config_path: Option<&str>) -> Result<Config, Vec<ConfigError>> {
        let config = Config::load(config_path).map_err(|e| vec![e])?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration, returning all errors found (not just the first).
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        // Validate business profile
        if self.business.product_name.is_empty() {
            errors.push(ConfigError::MissingField {
                field: "business.product_name".to_string(),
            });
        }

        if self.business.product_keywords.is_empty() && self.business.competitor_keywords.is_empty()
        {
            errors.push(ConfigError::MissingField {
                field: "business.product_keywords or business.competitor_keywords".to_string(),
            });
        }

        // Validate LLM provider
        if !self.llm.provider.is_empty() {
            match self.llm.provider.as_str() {
                "openai" | "anthropic" | "ollama" => {}
                _ => {
                    errors.push(ConfigError::InvalidValue {
                        field: "llm.provider".to_string(),
                        message: "must be openai, anthropic, or ollama".to_string(),
                    });
                }
            }

            if matches!(self.llm.provider.as_str(), "openai" | "anthropic") {
                match &self.llm.api_key {
                    Some(key) if !key.is_empty() => {}
                    _ => {
                        errors.push(ConfigError::MissingField {
                            field: format!(
                                "llm.api_key (required for {} provider)",
                                self.llm.provider
                            ),
                        });
                    }
                }
            }
        }

        // Validate auth mode
        if !self.auth.mode.is_empty() {
            match self.auth.mode.as_str() {
                "manual" | "local_callback" => {}
                _ => {
                    errors.push(ConfigError::InvalidValue {
                        field: "auth.mode".to_string(),
                        message: "must be manual or local_callback".to_string(),
                    });
                }
            }
        }

        // Validate scoring threshold
        if self.scoring.threshold > 100 {
            errors.push(ConfigError::InvalidValue {
                field: "scoring.threshold".to_string(),
                message: "must be between 0 and 100".to_string(),
            });
        }

        // Validate limits
        if self.limits.max_replies_per_day == 0 {
            errors.push(ConfigError::InvalidValue {
                field: "limits.max_replies_per_day".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }

        if self.limits.max_tweets_per_day == 0 {
            errors.push(ConfigError::InvalidValue {
                field: "limits.max_tweets_per_day".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }

        if self.limits.max_threads_per_week == 0 {
            errors.push(ConfigError::InvalidValue {
                field: "limits.max_threads_per_week".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }

        if self.limits.min_action_delay_seconds > self.limits.max_action_delay_seconds {
            errors.push(ConfigError::InvalidValue {
                field: "limits.min_action_delay_seconds".to_string(),
                message: "must be less than or equal to max_action_delay_seconds".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Resolve the config file path from arguments, env vars, or default.
    ///
    /// Returns `(path, explicit)` where `explicit` is true if the path was
    /// explicitly provided (via argument or env var) rather than using the default.
    fn resolve_config_path(config_path: Option<&str>) -> (PathBuf, bool) {
        if let Some(path) = config_path {
            return (expand_tilde(path), true);
        }

        if let Ok(env_path) = env::var("REPLYGUY_CONFIG") {
            return (expand_tilde(&env_path), true);
        }

        (expand_tilde("~/.replyguy/config.toml"), false)
    }

    /// Apply environment variable overrides to the configuration.
    ///
    /// Environment variables use the `REPLYGUY_` prefix with double underscores
    /// separating nested keys (e.g., `REPLYGUY_LLM__API_KEY`).
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // X API
        if let Ok(val) = env::var("REPLYGUY_X_API__CLIENT_ID") {
            self.x_api.client_id = val;
        }
        if let Ok(val) = env::var("REPLYGUY_X_API__CLIENT_SECRET") {
            self.x_api.client_secret = Some(val);
        }

        // Auth
        if let Ok(val) = env::var("REPLYGUY_AUTH__MODE") {
            self.auth.mode = val;
        }
        if let Ok(val) = env::var("REPLYGUY_AUTH__CALLBACK_HOST") {
            self.auth.callback_host = val;
        }
        if let Ok(val) = env::var("REPLYGUY_AUTH__CALLBACK_PORT") {
            self.auth.callback_port = parse_env_u16("REPLYGUY_AUTH__CALLBACK_PORT", &val)?;
        }

        // Business
        if let Ok(val) = env::var("REPLYGUY_BUSINESS__PRODUCT_NAME") {
            self.business.product_name = val;
        }
        if let Ok(val) = env::var("REPLYGUY_BUSINESS__PRODUCT_DESCRIPTION") {
            self.business.product_description = val;
        }
        if let Ok(val) = env::var("REPLYGUY_BUSINESS__PRODUCT_URL") {
            self.business.product_url = Some(val);
        }
        if let Ok(val) = env::var("REPLYGUY_BUSINESS__TARGET_AUDIENCE") {
            self.business.target_audience = val;
        }
        if let Ok(val) = env::var("REPLYGUY_BUSINESS__PRODUCT_KEYWORDS") {
            self.business.product_keywords = split_csv(&val);
        }
        if let Ok(val) = env::var("REPLYGUY_BUSINESS__COMPETITOR_KEYWORDS") {
            self.business.competitor_keywords = split_csv(&val);
        }
        if let Ok(val) = env::var("REPLYGUY_BUSINESS__INDUSTRY_TOPICS") {
            self.business.industry_topics = split_csv(&val);
        }

        // Scoring
        if let Ok(val) = env::var("REPLYGUY_SCORING__THRESHOLD") {
            self.scoring.threshold = parse_env_u32("REPLYGUY_SCORING__THRESHOLD", &val)?;
        }

        // Limits
        if let Ok(val) = env::var("REPLYGUY_LIMITS__MAX_REPLIES_PER_DAY") {
            self.limits.max_replies_per_day =
                parse_env_u32("REPLYGUY_LIMITS__MAX_REPLIES_PER_DAY", &val)?;
        }
        if let Ok(val) = env::var("REPLYGUY_LIMITS__MAX_TWEETS_PER_DAY") {
            self.limits.max_tweets_per_day =
                parse_env_u32("REPLYGUY_LIMITS__MAX_TWEETS_PER_DAY", &val)?;
        }
        if let Ok(val) = env::var("REPLYGUY_LIMITS__MAX_THREADS_PER_WEEK") {
            self.limits.max_threads_per_week =
                parse_env_u32("REPLYGUY_LIMITS__MAX_THREADS_PER_WEEK", &val)?;
        }
        if let Ok(val) = env::var("REPLYGUY_LIMITS__MIN_ACTION_DELAY_SECONDS") {
            self.limits.min_action_delay_seconds =
                parse_env_u64("REPLYGUY_LIMITS__MIN_ACTION_DELAY_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("REPLYGUY_LIMITS__MAX_ACTION_DELAY_SECONDS") {
            self.limits.max_action_delay_seconds =
                parse_env_u64("REPLYGUY_LIMITS__MAX_ACTION_DELAY_SECONDS", &val)?;
        }

        // Intervals
        if let Ok(val) = env::var("REPLYGUY_INTERVALS__MENTIONS_CHECK_SECONDS") {
            self.intervals.mentions_check_seconds =
                parse_env_u64("REPLYGUY_INTERVALS__MENTIONS_CHECK_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("REPLYGUY_INTERVALS__DISCOVERY_SEARCH_SECONDS") {
            self.intervals.discovery_search_seconds =
                parse_env_u64("REPLYGUY_INTERVALS__DISCOVERY_SEARCH_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("REPLYGUY_INTERVALS__CONTENT_POST_WINDOW_SECONDS") {
            self.intervals.content_post_window_seconds =
                parse_env_u64("REPLYGUY_INTERVALS__CONTENT_POST_WINDOW_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("REPLYGUY_INTERVALS__THREAD_INTERVAL_SECONDS") {
            self.intervals.thread_interval_seconds =
                parse_env_u64("REPLYGUY_INTERVALS__THREAD_INTERVAL_SECONDS", &val)?;
        }

        // LLM
        if let Ok(val) = env::var("REPLYGUY_LLM__PROVIDER") {
            self.llm.provider = val;
        }
        if let Ok(val) = env::var("REPLYGUY_LLM__API_KEY") {
            self.llm.api_key = Some(val);
        }
        if let Ok(val) = env::var("REPLYGUY_LLM__MODEL") {
            self.llm.model = val;
        }
        if let Ok(val) = env::var("REPLYGUY_LLM__BASE_URL") {
            self.llm.base_url = Some(val);
        }

        // Storage
        if let Ok(val) = env::var("REPLYGUY_STORAGE__DB_PATH") {
            self.storage.db_path = val;
        }
        if let Ok(val) = env::var("REPLYGUY_STORAGE__RETENTION_DAYS") {
            self.storage.retention_days = parse_env_u32("REPLYGUY_STORAGE__RETENTION_DAYS", &val)?;
        }

        // Logging
        if let Ok(val) = env::var("REPLYGUY_LOGGING__STATUS_INTERVAL_SECONDS") {
            self.logging.status_interval_seconds =
                parse_env_u64("REPLYGUY_LOGGING__STATUS_INTERVAL_SECONDS", &val)?;
        }

        Ok(())
    }
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Split a comma-separated string into trimmed, non-empty values.
fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Parse an environment variable value as `u16`.
fn parse_env_u16(var_name: &str, val: &str) -> Result<u16, ConfigError> {
    val.parse::<u16>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u16"),
    })
}

/// Parse an environment variable value as `u32`.
fn parse_env_u32(var_name: &str, val: &str) -> Result<u32, ConfigError> {
    val.parse::<u32>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u32"),
    })
}

/// Parse an environment variable value as `u64`.
fn parse_env_u64(var_name: &str, val: &str) -> Result<u64, ConfigError> {
    val.parse::<u64>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u64"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn load_valid_toml() {
        let toml_str = r#"
[x_api]
client_id = "test-client-id"

[business]
product_name = "TestProduct"
product_description = "A test product"
target_audience = "developers"
product_keywords = ["rust", "cli"]
industry_topics = ["rust", "development"]

[llm]
provider = "ollama"
model = "llama2"

[scoring]
threshold = 80
"#;
        let config: Config = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(config.x_api.client_id, "test-client-id");
        assert_eq!(config.business.product_name, "TestProduct");
        assert_eq!(config.scoring.threshold, 80);
        assert_eq!(config.llm.provider, "ollama");
    }

    #[test]
    fn missing_sections_use_defaults() {
        let toml_str = r#"
[x_api]
client_id = "test"
"#;
        let config: Config = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(config.auth.mode, "manual");
        assert_eq!(config.auth.callback_port, 8080);
        assert_eq!(config.scoring.threshold, 70);
        assert_eq!(config.limits.max_replies_per_day, 20);
        assert_eq!(config.intervals.mentions_check_seconds, 300);
        assert_eq!(config.storage.db_path, "~/.replyguy/replyguy.db");
        assert_eq!(config.storage.retention_days, 90);
        assert_eq!(config.logging.status_interval_seconds, 0);
    }

    #[test]
    fn env_var_override_string() {
        // Use a unique env var prefix to avoid test interference
        env::set_var("REPLYGUY_LLM__PROVIDER", "anthropic");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.llm.provider, "anthropic");
        env::remove_var("REPLYGUY_LLM__PROVIDER");
    }

    #[test]
    fn env_var_override_numeric() {
        env::set_var("REPLYGUY_SCORING__THRESHOLD", "85");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.scoring.threshold, 85);
        env::remove_var("REPLYGUY_SCORING__THRESHOLD");
    }

    #[test]
    fn env_var_override_csv() {
        env::set_var("REPLYGUY_BUSINESS__PRODUCT_KEYWORDS", "rust, cli, tools");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(
            config.business.product_keywords,
            vec!["rust", "cli", "tools"]
        );
        env::remove_var("REPLYGUY_BUSINESS__PRODUCT_KEYWORDS");
    }

    #[test]
    fn env_var_invalid_numeric_returns_error() {
        // Use a unique env var to avoid race conditions with other tests
        env::set_var("REPLYGUY_STORAGE__RETENTION_DAYS", "not_a_number");
        let mut config = Config::default();
        let result = config.apply_env_overrides();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidValue { field, .. } => {
                assert_eq!(field, "REPLYGUY_STORAGE__RETENTION_DAYS");
            }
            other => panic!("expected InvalidValue, got: {other}"),
        }
        env::remove_var("REPLYGUY_STORAGE__RETENTION_DAYS");
    }

    #[test]
    fn validate_missing_product_name() {
        let config = Config::default();
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::MissingField { field } if field == "business.product_name")
        ));
    }

    #[test]
    fn validate_invalid_llm_provider() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "invalid_provider".to_string();
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "llm.provider")
        ));
    }

    #[test]
    fn validate_threshold_over_100() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.scoring.threshold = 101;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "scoring.threshold")
        ));
    }

    #[test]
    fn validate_threshold_boundary_values() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();

        config.scoring.threshold = 0;
        assert!(config.validate().is_ok());

        config.scoring.threshold = 100;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_returns_multiple_errors() {
        let mut config = Config::default();
        // Missing product_name (default is empty)
        // Missing keywords (default is empty)
        config.llm.provider = "invalid".to_string();
        config.scoring.threshold = 101;
        config.limits.max_replies_per_day = 0;

        let errors = config.validate().unwrap_err();
        assert!(
            errors.len() >= 4,
            "expected at least 4 errors, got {}: {:?}",
            errors.len(),
            errors
        );
    }

    #[test]
    fn validate_valid_config_passes() {
        let mut config = Config::default();
        config.business.product_name = "TestProduct".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.llm.model = "llama2".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_openai_requires_api_key() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "openai".to_string();
        config.llm.api_key = None;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::MissingField { field } if field.contains("llm.api_key"))
        ));
    }

    #[test]
    fn validate_delay_ordering() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.limits.min_action_delay_seconds = 200;
        config.limits.max_action_delay_seconds = 100;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "limits.min_action_delay_seconds")));
    }

    #[test]
    fn expand_tilde_works() {
        let expanded = expand_tilde("~/.replyguy/config.toml");
        assert!(!expanded.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn split_csv_trims_and_filters() {
        let result = split_csv("  rust , cli ,, tools  ");
        assert_eq!(result, vec!["rust", "cli", "tools"]);
    }

    #[test]
    fn config_file_not_found_explicit_path() {
        let result = Config::load(Some("/nonexistent/path/config.toml"));
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::FileNotFound { path } => {
                assert_eq!(path, "/nonexistent/path/config.toml");
            }
            other => panic!("expected FileNotFound, got: {other}"),
        }
    }
}
