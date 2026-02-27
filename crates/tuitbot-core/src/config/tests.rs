//! General configuration tests.

use super::env_overrides::{parse_env_bool, parse_env_u32, split_csv};
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
    assert_eq!(config.scoring.threshold, 60);
    assert_eq!(config.limits.max_replies_per_day, 5);
    assert_eq!(config.intervals.mentions_check_seconds, 300);
    assert_eq!(config.storage.db_path, "~/.tuitbot/tuitbot.db");
    assert_eq!(config.storage.retention_days, 90);
    assert_eq!(config.logging.status_interval_seconds, 0);
}

#[test]
fn env_var_override_string() {
    // Use a unique env var prefix to avoid test interference
    env::set_var("TUITBOT_LLM__PROVIDER", "anthropic");
    let mut config = Config::default();
    config.apply_env_overrides().expect("env override");
    assert_eq!(config.llm.provider, "anthropic");
    env::remove_var("TUITBOT_LLM__PROVIDER");
}

#[test]
fn env_var_override_numeric() {
    env::set_var("TUITBOT_SCORING__THRESHOLD", "85");
    let mut config = Config::default();
    config.apply_env_overrides().expect("env override");
    assert_eq!(config.scoring.threshold, 85);
    env::remove_var("TUITBOT_SCORING__THRESHOLD");
}

#[test]
fn env_var_override_csv() {
    env::set_var("TUITBOT_BUSINESS__PRODUCT_KEYWORDS", "rust, cli, tools");
    let mut config = Config::default();
    config.apply_env_overrides().expect("env override");
    assert_eq!(
        config.business.product_keywords,
        vec!["rust", "cli", "tools"]
    );
    env::remove_var("TUITBOT_BUSINESS__PRODUCT_KEYWORDS");
}

#[test]
fn env_var_invalid_numeric_returns_error() {
    // Test the parse function directly to avoid env var race conditions
    // with other tests that call apply_env_overrides()
    let result = parse_env_u32("TUITBOT_SCORING__THRESHOLD", "not_a_number");
    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::InvalidValue { field, .. } => {
            assert_eq!(field, "TUITBOT_SCORING__THRESHOLD");
        }
        other => panic!("expected InvalidValue, got: {other}"),
    }
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
    assert!(errors
        .iter()
        .any(|e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "llm.provider")));
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
    let expanded = expand_tilde("~/.tuitbot/config.toml");
    assert!(!expanded.to_string_lossy().starts_with('~'));
}

#[test]
fn split_csv_trims_and_filters() {
    let result = split_csv("  rust , cli ,, tools  ");
    assert_eq!(result, vec!["rust", "cli", "tools"]);
}

#[test]
fn validate_preferred_times_valid() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.schedule.preferred_times = vec!["09:15".to_string(), "12:30".to_string()];
    assert!(config.validate().is_ok());
}

#[test]
fn validate_preferred_times_auto() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.schedule.preferred_times = vec!["auto".to_string()];
    assert!(config.validate().is_ok());
}

#[test]
fn validate_preferred_times_invalid_format() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.schedule.preferred_times = vec!["9:15".to_string(), "25:00".to_string()];
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.preferred_times")
    ));
}

#[test]
fn validate_preferred_times_exceeds_max_tweets() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.limits.max_tweets_per_day = 2;
    config.schedule.preferred_times = vec![
        "09:00".to_string(),
        "12:00".to_string(),
        "17:00".to_string(),
    ];
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, message } if field == "schedule.preferred_times" && message.contains("3 slots"))
    ));
}

#[test]
fn validate_thread_preferred_day_invalid() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.schedule.thread_preferred_day = Some("Monday".to_string());
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.thread_preferred_day")
    ));
}

#[test]
fn validate_thread_preferred_time_invalid() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.schedule.thread_preferred_time = "25:00".to_string();
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.thread_preferred_time")
    ));
}

#[test]
fn preferred_times_override_invalid_day() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config
        .schedule
        .preferred_times_override
        .insert("Monday".to_string(), vec!["09:00".to_string()]);
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.preferred_times_override")
    ));
}

#[test]
fn preferred_times_toml_roundtrip() {
    let toml_str = r#"
[x_api]
client_id = "test"

[business]
product_name = "Test"
product_keywords = ["test"]

[llm]
provider = "ollama"
model = "llama2"

[schedule]
timezone = "America/New_York"
preferred_times = ["09:15", "12:30", "17:00"]
thread_preferred_day = "Tue"
thread_preferred_time = "10:00"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(
        config.schedule.preferred_times,
        vec!["09:15", "12:30", "17:00"]
    );
    assert_eq!(
        config.schedule.thread_preferred_day,
        Some("Tue".to_string())
    );
    assert_eq!(config.schedule.thread_preferred_time, "10:00");
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

#[test]
fn parse_env_bool_values() {
    assert!(parse_env_bool("TEST", "true").unwrap());
    assert!(parse_env_bool("TEST", "True").unwrap());
    assert!(parse_env_bool("TEST", "1").unwrap());
    assert!(parse_env_bool("TEST", "yes").unwrap());
    assert!(parse_env_bool("TEST", "YES").unwrap());
    assert!(!parse_env_bool("TEST", "false").unwrap());
    assert!(!parse_env_bool("TEST", "False").unwrap());
    assert!(!parse_env_bool("TEST", "0").unwrap());
    assert!(!parse_env_bool("TEST", "no").unwrap());
    assert!(!parse_env_bool("TEST", "NO").unwrap());
    assert!(parse_env_bool("TEST", "maybe").is_err());
}

#[test]
fn env_var_override_approval_mode() {
    env::set_var("TUITBOT_APPROVAL_MODE", "true");
    let mut config = Config::default();
    config.approval_mode = false;
    config.apply_env_overrides().expect("env override");
    assert!(config.approval_mode);
    env::remove_var("TUITBOT_APPROVAL_MODE");
}

#[test]
fn env_var_override_approval_mode_false() {
    env::set_var("TUITBOT_APPROVAL_MODE", "false");
    let mut config = Config::default();
    config.apply_env_overrides().expect("env override");
    assert!(!config.approval_mode);
    env::remove_var("TUITBOT_APPROVAL_MODE");
}

#[test]
fn openclaw_env_enables_approval_mode() {
    env::set_var("OPENCLAW_AGENT_ID", "test");
    let mut config = Config::default();
    config.approval_mode = false;
    config.apply_env_overrides().expect("env override");
    assert!(config.approval_mode);
    env::remove_var("OPENCLAW_AGENT_ID");
}

#[test]
fn openclaw_env_respects_explicit_override() {
    env::set_var("OPENCLAW_AGENT_ID", "test");
    env::set_var("TUITBOT_APPROVAL_MODE", "false");
    let mut config = Config::default();
    config.apply_env_overrides().expect("env override");
    assert!(!config.approval_mode);
    env::remove_var("OPENCLAW_AGENT_ID");
    env::remove_var("TUITBOT_APPROVAL_MODE");
}

// --- BusinessProfile quickstart/enrichment tests ---

#[test]
fn quickstart_minimal_config_validates() {
    let mut config = Config::default();
    config.business = BusinessProfile::quickstart(
        "MyApp".to_string(),
        vec!["rust cli".to_string(), "developer tools".to_string()],
    );
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-client-id".to_string();
    assert!(config.validate().is_ok());
}

#[test]
fn quickstart_industry_topics_derived_from_keywords() {
    let profile = BusinessProfile {
        product_keywords: vec!["rust".to_string(), "cli".to_string()],
        industry_topics: vec![],
        ..Default::default()
    };
    assert_eq!(
        profile.effective_industry_topics(),
        &["rust".to_string(), "cli".to_string()]
    );
}

#[test]
fn explicit_industry_topics_override_derived() {
    let profile = BusinessProfile {
        product_keywords: vec!["rust".to_string()],
        industry_topics: vec!["Rust development".to_string()],
        ..Default::default()
    };
    assert_eq!(
        profile.effective_industry_topics(),
        &["Rust development".to_string()]
    );
}

#[test]
fn is_enriched_false_for_quickstart() {
    let profile = BusinessProfile::quickstart("App".to_string(), vec!["test".to_string()]);
    assert!(!profile.is_enriched());
}

#[test]
fn is_enriched_true_with_brand_voice() {
    let mut profile = BusinessProfile::quickstart("App".to_string(), vec!["test".to_string()]);
    profile.brand_voice = Some("Friendly and casual".to_string());
    assert!(profile.is_enriched());
}
