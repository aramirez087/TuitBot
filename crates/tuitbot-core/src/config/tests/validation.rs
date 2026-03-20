//! Tests for validate_minimum() and validate() — required fields and value constraints.

use super::*;

// ─── A1: validate_minimum() tests ───────────────────────────────────────────

#[test]
fn validate_minimum_valid_quickstart() {
    let mut config = Config::default();
    config.business.product_name = "TestProduct".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A test product".to_string();
    assert!(config.validate_minimum().is_ok());
}

#[test]
fn validate_minimum_empty_product_name() {
    let mut config = Config::default();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A test product".to_string();
    let errors = config.validate_minimum().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::MissingField { field } if field == "business.product_name")
    ));
}

#[test]
fn validate_minimum_empty_description() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "   ".to_string(); // whitespace only
    let errors = config.validate_minimum().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::MissingField { field } if field == "business.product_description")
    ));
}

#[test]
fn validate_minimum_no_keywords() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_description = "A product".to_string();
    // both product_keywords and competitor_keywords empty
    let errors = config.validate_minimum().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::MissingField { field } if field.contains("product_keywords"))
    ));
}

#[test]
fn validate_minimum_competitor_keywords_sufficient() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_description = "A product".to_string();
    config.business.competitor_keywords = vec!["competitor".to_string()];
    // product_keywords empty but competitor_keywords present -> should pass
    assert!(config.validate_minimum().is_ok());
}

#[test]
fn validate_minimum_invalid_llm_provider() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A product".to_string();
    config.llm.provider = "bad_provider".to_string();
    let errors = config.validate_minimum().unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "llm.provider")));
}

#[test]
fn validate_minimum_valid_llm_providers() {
    for provider in &["openai", "anthropic", "ollama"] {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.business.product_description = "A product".to_string();
        config.llm.provider = provider.to_string();
        assert!(
            config.validate_minimum().is_ok(),
            "provider '{}' should be valid for validate_minimum",
            provider
        );
    }
}

#[test]
fn validate_minimum_empty_provider_passes() {
    // Empty provider is OK for validate_minimum (optional)
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A product".to_string();
    assert!(config.validate_minimum().is_ok());
}

#[test]
fn validate_minimum_invalid_provider_backend() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A product".to_string();
    config.x_api.provider_backend = "invalid_backend".to_string();
    let errors = config.validate_minimum().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "x_api.provider_backend")
    ));
}

#[test]
fn validate_minimum_valid_provider_backends() {
    for backend in &["x_api", "scraper", ""] {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.business.product_description = "A product".to_string();
        config.x_api.provider_backend = backend.to_string();
        assert!(
            config.validate_minimum().is_ok(),
            "backend '{}' should be valid",
            backend
        );
    }
}

#[test]
fn validate_minimum_empty_db_path() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A product".to_string();
    config.storage.db_path = "   ".to_string();
    let errors = config.validate_minimum().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "storage.db_path")
    ));
}

#[test]
fn validate_minimum_db_path_is_directory() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A product".to_string();
    config.storage.db_path = std::env::temp_dir().to_string_lossy().to_string(); // temp dir is always a directory
    let errors = config.validate_minimum().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "storage.db_path")
    ));
}

#[test]
fn validate_minimum_accumulates_errors() {
    let config = Config::default(); // empty product_name, empty description, no keywords
    let errors = config.validate_minimum().unwrap_err();
    assert!(
        errors.len() >= 3,
        "expected at least 3 errors, got {}: {:?}",
        errors.len(),
        errors
    );
}

// ─── A2: validate() additional paths ────────────────────────────────────────

#[test]
fn validate_missing_industry_topics() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A product".to_string();
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-id".to_string();
    // industry_topics is empty by default
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::MissingField { field } if field == "business.industry_topics")
    ));
}

#[test]
fn validate_max_tweets_per_day_zero() {
    let mut config = valid_test_config();
    config.limits.max_tweets_per_day = 0;
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "limits.max_tweets_per_day")
    ));
}

#[test]
fn validate_max_threads_per_week_zero() {
    let mut config = valid_test_config();
    config.limits.max_threads_per_week = 0;
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "limits.max_threads_per_week")
    ));
}

#[test]
fn validate_active_hours_start_over_23() {
    let mut config = valid_test_config();
    config.schedule.active_hours_start = 24;
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.active_hours_start")
    ));
}

#[test]
fn validate_active_hours_end_over_23() {
    let mut config = valid_test_config();
    config.schedule.active_hours_end = 24;
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.active_hours_end")
    ));
}

#[test]
fn validate_invalid_timezone() {
    let mut config = valid_test_config();
    config.schedule.timezone = "Not/A/Timezone".to_string();
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.timezone")
    ));
}

#[test]
fn validate_valid_timezone() {
    let mut config = valid_test_config();
    config.schedule.timezone = "America/New_York".to_string();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_invalid_active_day() {
    let mut config = valid_test_config();
    config.schedule.active_days = vec!["Monday".to_string()]; // Should be "Mon"
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.active_days")
    ));
}

#[test]
fn validate_valid_active_days() {
    let mut config = valid_test_config();
    config.schedule.active_days = vec!["Mon".to_string(), "Wed".to_string(), "Fri".to_string()];
    assert!(config.validate().is_ok());
}

#[test]
fn validate_invalid_auth_mode() {
    let mut config = valid_test_config();
    config.auth.mode = "oauth".to_string(); // Not "manual" or "local_callback"
    let errors = config.validate().unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "auth.mode")));
}

#[test]
fn validate_valid_auth_modes() {
    for mode in &["manual", "local_callback"] {
        let mut config = valid_test_config();
        config.auth.mode = mode.to_string();
        assert!(
            config.validate().is_ok(),
            "auth mode '{}' should be valid",
            mode
        );
    }
}

#[test]
fn validate_mcp_policy_blocked_and_approval_overlap() {
    let mut config = valid_test_config();
    config.mcp_policy.blocked_tools = vec!["post_tweet".to_string()];
    config.mcp_policy.require_approval_for =
        vec!["post_tweet".to_string(), "reply_to_tweet".to_string()];
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "mcp_policy.blocked_tools")
    ));
}

#[test]
fn validate_scraper_in_cloud_deployment() {
    let mut config = valid_test_config();
    config.deployment_mode = DeploymentMode::Cloud;
    config.x_api.provider_backend = "scraper".to_string();
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, message } if field == "x_api.provider_backend" && message.contains("cloud"))
    ));
}

#[test]
fn validate_missing_x_api_client_id() {
    let mut config = valid_test_config();
    config.x_api.client_id = "".to_string();
    let errors = config.validate().unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ConfigError::MissingField { field } if field == "x_api.client_id")));
}

#[test]
fn validate_scraper_backend_no_client_id_required() {
    let mut config = valid_test_config();
    config.x_api.provider_backend = "scraper".to_string();
    config.x_api.client_id = "".to_string();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_anthropic_requires_api_key() {
    let mut config = valid_test_config();
    config.llm.provider = "anthropic".to_string();
    config.llm.api_key = None;
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::MissingField { field } if field.contains("llm.api_key"))
    ));
}

#[test]
fn validate_ollama_no_api_key_needed() {
    let mut config = valid_test_config();
    config.llm.provider = "ollama".to_string();
    config.llm.api_key = None;
    assert!(config.validate().is_ok());
}

#[test]
fn validate_empty_description() {
    let mut config = valid_test_config();
    config.business.product_description = "".to_string();
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::MissingField { field } if field == "business.product_description")
    ));
}

#[test]
fn validate_preferred_times_override_invalid_time() {
    let mut config = valid_test_config();
    config
        .schedule
        .preferred_times_override
        .insert("Mon".to_string(), vec!["25:00".to_string()]);
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.preferred_times_override")
    ));
}

#[test]
fn validate_auto_expands_to_3_slots() {
    let mut config = valid_test_config();
    config.limits.max_tweets_per_day = 2;
    config.schedule.preferred_times = vec!["auto".to_string()]; // "auto" expands to 3 slots
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, message } if field == "schedule.preferred_times" && message.contains("3 slots"))
    ));
}

#[test]
fn validate_effective_slots_override_exceeds_max() {
    let mut config = valid_test_config();
    config.limits.max_tweets_per_day = 2;
    config.schedule.preferred_times = vec!["09:00".to_string()]; // 1 base slot
                                                                 // Override day has 3 slots, which exceeds max_tweets_per_day of 2
    config.schedule.preferred_times_override.insert(
        "Mon".to_string(),
        vec![
            "09:00".to_string(),
            "12:00".to_string(),
            "17:00".to_string(),
        ],
    );
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(
        |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.preferred_times")
    ));
}
