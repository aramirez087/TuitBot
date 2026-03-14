//! Standard test configuration fixtures for `tuitbot-core` tests.
//!
//! Provides ready-made [`Config`] instances covering common test scenarios
//! without needing to write TOML or set environment variables.
//!
//! # Example
//! ```rust
//! use tuitbot_core::testing::ConfigFixture;
//!
//! let cfg = ConfigFixture::default_config();         // safe defaults
//! let minimal = ConfigFixture::minimal_config();     // bare minimum fields
//! let maxed = ConfigFixture::maxed_out_config();     // all limits at maximum
//! let local = ConfigFixture::local_mode_config();    // scraper backend
//! ```

use crate::config::Config;
use crate::config::{
    BusinessProfile, LimitsConfig, LlmConfig, ScoringConfig, StorageConfig, XApiConfig,
};

/// Provides standard test [`Config`] instances.
pub struct ConfigFixture;

impl ConfigFixture {
    /// A safe, opinionated default config for most tests.
    /// Uses local/scraper backend (no real X API calls), approval mode on.
    pub fn default_config() -> Config {
        let mut cfg = Config::default();
        cfg.x_api = XApiConfig {
            client_id: "test-client-id".to_string(),
            client_secret: None,
            provider_backend: "local".to_string(),
            scraper_allow_mutations: false,
        };
        cfg.business = BusinessProfile {
            product_name: "TuitBot Test".to_string(),
            product_description: "Autonomous X growth assistant — test instance".to_string(),
            target_audience: "indie hackers and developers".to_string(),
            product_keywords: vec!["automation".to_string(), "growth".to_string()],
            industry_topics: vec!["saas".to_string(), "devtools".to_string()],
            ..Default::default()
        };
        cfg.llm = LlmConfig {
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };
        cfg.scoring = ScoringConfig {
            threshold: 70,
            ..Default::default()
        };
        cfg.approval_mode = true;
        cfg.storage = StorageConfig {
            db_path: ":memory:".to_string(),
            ..Default::default()
        };
        cfg
    }

    /// Minimal config — just enough to not panic. Use when testing config
    /// loading/validation code rather than business logic.
    pub fn minimal_config() -> Config {
        Config::default()
    }

    /// Config with all rate limits cranked to maximum. Use when testing
    /// that safety guardrails actually enforce limits rather than relying
    /// on low defaults.
    pub fn maxed_out_config() -> Config {
        let mut cfg = Self::default_config();
        cfg.limits = LimitsConfig {
            max_replies_per_day: 100,
            max_tweets_per_day: 50,
            max_threads_per_week: 20,
            ..Default::default()
        };
        cfg
    }

    /// Config in local/scraper mode with mutations explicitly enabled.
    /// Use when testing mutation-gated code paths.
    pub fn local_mode_config() -> Config {
        let mut cfg = Self::default_config();
        cfg.x_api.provider_backend = "local".to_string();
        cfg.x_api.scraper_allow_mutations = true;
        cfg
    }

    /// Config that represents a completely fresh install with no credentials.
    /// Use when testing onboarding/setup flows.
    pub fn unconfigured_config() -> Config {
        let mut cfg = Config::default();
        cfg.x_api = XApiConfig {
            client_id: String::new(),
            client_secret: None,
            provider_backend: String::new(),
            scraper_allow_mutations: false,
        };
        cfg
    }

    /// Build a config from a raw TOML string. Panics on parse failure.
    pub fn from_toml(toml_str: &str) -> Config {
        toml::from_str(toml_str).expect("ConfigFixture::from_toml: invalid TOML")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_approval_mode_on() {
        let cfg = ConfigFixture::default_config();
        assert!(cfg.approval_mode);
    }

    #[test]
    fn default_config_uses_local_backend() {
        let cfg = ConfigFixture::default_config();
        assert_eq!(cfg.x_api.provider_backend, "local");
        assert!(!cfg.x_api.scraper_allow_mutations);
    }

    #[test]
    fn maxed_config_has_high_limits() {
        let cfg = ConfigFixture::maxed_out_config();
        assert!(cfg.limits.max_replies_per_day > 10);
        assert!(cfg.limits.max_tweets_per_day > 10);
    }

    #[test]
    fn local_mode_enables_mutations() {
        let cfg = ConfigFixture::local_mode_config();
        assert!(cfg.x_api.scraper_allow_mutations);
    }

    #[test]
    fn from_toml_parses_valid_toml() {
        let cfg = ConfigFixture::from_toml(
            r#"
[x_api]
client_id = "ci-test-id"

[business]
product_name = "CI Bot"
product_description = "Test"
target_audience = "developers"
product_keywords = ["ci"]
industry_topics = ["devtools"]
"#,
        );
        assert_eq!(cfg.x_api.client_id, "ci-test-id");
        assert_eq!(cfg.business.product_name, "CI Bot");
    }

    #[test]
    fn unconfigured_has_empty_client_id() {
        let cfg = ConfigFixture::unconfigured_config();
        assert!(cfg.x_api.client_id.is_empty());
    }
}
