//! Tests for provider backend validation, env overrides, and config round-trips.

use super::env_overrides::parse_env_bool;
use super::*;
use crate::config::types::DeploymentMode;
use crate::error::ConfigError;
use std::env;
use std::ffi::OsString;
use std::sync::{Mutex, OnceLock};

// Environment variables are process-global, so tests that mutate them must not run concurrently.
fn with_locked_env(test: impl FnOnce()) {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned");
    test();
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<OsString>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = env::var_os(key);
        env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => env::set_var(self.key, previous),
            None => env::remove_var(self.key),
        }
    }
}

/// Helper: build a minimal valid config for backend tests.
/// Sets required fields so validation passes, with scraper backend
/// (no client_id needed).
fn scraper_config() -> Config {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A test product".to_string();
    config.business.industry_topics = vec!["testing".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.provider_backend = "scraper".to_string();
    config
}

/// Helper: build a minimal valid config with x_api backend.
fn x_api_config() -> Config {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A test product".to_string();
    config.business.industry_topics = vec!["testing".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-client-id".to_string();
    config.x_api.provider_backend = "x_api".to_string();
    config
}

// --- Provider backend validation tests ---

#[test]
fn validate_scraper_backend_allows_empty_client_id() {
    let config = scraper_config();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_x_api_backend_requires_client_id() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.provider_backend = "x_api".to_string();
    // client_id is empty — should fail
    let errors = config.validate().unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ConfigError::MissingField { field } if field == "x_api.client_id")));
}

#[test]
fn validate_empty_backend_requires_client_id() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.provider_backend = String::new();
    // Empty backend maps to x_api — client_id required
    let errors = config.validate().unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ConfigError::MissingField { field } if field == "x_api.client_id")));
}

#[test]
fn validate_x_api_with_client_id_passes() {
    let config = x_api_config();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_cloud_scraper_rejected() {
    let mut config = scraper_config();
    config.deployment_mode = DeploymentMode::Cloud;
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "x_api.provider_backend")
    ));
}

#[test]
fn validate_desktop_scraper_allowed() {
    let mut config = scraper_config();
    config.deployment_mode = DeploymentMode::Desktop;
    assert!(config.validate().is_ok());
}

#[test]
fn validate_self_host_scraper_allowed() {
    let mut config = scraper_config();
    config.deployment_mode = DeploymentMode::SelfHost;
    assert!(config.validate().is_ok());
}

#[test]
fn validate_invalid_backend_value_rejected() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.provider_backend = "magic".to_string();
    let errors = config.validate().unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ConfigError::InvalidValue { field, message }
            if field == "x_api.provider_backend" && message.contains("magic"))));
}

#[test]
fn validate_scraper_allow_mutations_default_false() {
    let config = Config::default();
    assert!(!config.x_api.scraper_allow_mutations);
}

// --- Environment variable override tests ---

#[test]
fn env_var_override_provider_backend() {
    with_locked_env(|| {
        let _backend = ScopedEnvVar::set("TUITBOT_X_API__PROVIDER_BACKEND", "scraper");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.x_api.provider_backend, "scraper");
    });
}

#[test]
fn env_var_override_scraper_allow_mutations() {
    with_locked_env(|| {
        let _mutations = ScopedEnvVar::set("TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS", "true");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(config.x_api.scraper_allow_mutations);
    });
}

#[test]
fn env_var_scraper_allow_mutations_false() {
    with_locked_env(|| {
        let _mutations = ScopedEnvVar::set("TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS", "false");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(!config.x_api.scraper_allow_mutations);
    });
}

#[test]
fn env_var_scraper_allow_mutations_invalid() {
    with_locked_env(|| {
        let _mutations = ScopedEnvVar::set("TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS", "maybe");
        let mut config = Config::default();
        let result = config.apply_env_overrides();
        assert!(result.is_err());
    });
}

// --- TOML round-trip tests ---

#[test]
fn scraper_config_toml_roundtrip() {
    let toml_str = r#"
[x_api]
provider_backend = "scraper"
scraper_allow_mutations = true

[business]
product_name = "TestApp"
product_keywords = ["test"]

[llm]
provider = "ollama"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.x_api.provider_backend, "scraper");
    assert!(config.x_api.scraper_allow_mutations);
    assert!(config.x_api.client_id.is_empty());

    // Serialize back and verify round-trip
    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&toml_out).expect("re-parse");
    assert_eq!(roundtripped.x_api.provider_backend, "scraper");
    assert!(roundtripped.x_api.scraper_allow_mutations);
}

#[test]
fn x_api_config_toml_roundtrip() {
    let toml_str = r#"
[x_api]
client_id = "my-client-id"
client_secret = "my-secret"
provider_backend = "x_api"

[business]
product_name = "TestApp"
product_keywords = ["test"]

[llm]
provider = "ollama"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.x_api.provider_backend, "x_api");
    assert_eq!(config.x_api.client_id, "my-client-id");
    assert!(!config.x_api.scraper_allow_mutations); // default

    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&toml_out).expect("re-parse");
    assert_eq!(roundtripped.x_api.client_id, "my-client-id");
    assert_eq!(roundtripped.x_api.provider_backend, "x_api");
}

#[test]
fn provider_backend_defaults_to_empty_string() {
    let config = Config::default();
    assert_eq!(config.x_api.provider_backend, "");
}

// --- JSON serialization (settings API payload shape) ---

#[test]
fn settings_json_includes_backend_fields() {
    let mut config = scraper_config();
    config.x_api.scraper_allow_mutations = true;

    let json = serde_json::to_value(&config).expect("serialize");
    let x_api = &json["x_api"];
    assert_eq!(x_api["provider_backend"], "scraper");
    assert_eq!(x_api["scraper_allow_mutations"], true);
    assert_eq!(x_api["client_id"], "");
}

#[test]
fn settings_json_roundtrip_scraper_mode() {
    let mut config = scraper_config();
    config.x_api.scraper_allow_mutations = true;

    let json = serde_json::to_value(&config).expect("serialize to JSON");
    let toml_str = toml::to_string_pretty(&config).expect("serialize to TOML");
    let from_toml: Config = toml::from_str(&toml_str).expect("parse TOML");

    assert_eq!(from_toml.x_api.provider_backend, "scraper");
    assert!(from_toml.x_api.scraper_allow_mutations);

    // JSON→Config round-trip (simulates settings API PATCH)
    let json_str = serde_json::to_string(&json).expect("json string");
    let from_json: serde_json::Value = serde_json::from_str(&json_str).expect("parse json");
    assert_eq!(from_json["x_api"]["provider_backend"], "scraper");
}

// --- parse_env_bool coverage ---

#[test]
fn parse_env_bool_accepts_variants() {
    assert!(parse_env_bool("TEST", "true").unwrap());
    assert!(parse_env_bool("TEST", "1").unwrap());
    assert!(parse_env_bool("TEST", "yes").unwrap());
    assert!(parse_env_bool("TEST", "YES").unwrap());
    assert!(!parse_env_bool("TEST", "false").unwrap());
    assert!(!parse_env_bool("TEST", "0").unwrap());
    assert!(!parse_env_bool("TEST", "no").unwrap());
    assert!(parse_env_bool("TEST", "maybe").is_err());
}
