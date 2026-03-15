//! General configuration tests.

use super::env_overrides::{parse_env_bool, parse_env_u32, split_csv};
use super::*;
use crate::config::types::{DeploymentCapabilities, DeploymentMode};
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

/// Create a minimal config that passes all validation checks.
fn valid_test_config() -> Config {
    let mut config = Config::default();
    config.business.product_name = "TestProduct".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A test product".to_string();
    config.business.industry_topics = vec!["testing".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-id".to_string();
    config
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
    with_locked_env(|| {
        let _provider = ScopedEnvVar::set("TUITBOT_LLM__PROVIDER", "anthropic");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.llm.provider, "anthropic");
    });
}

#[test]
fn env_var_override_numeric() {
    with_locked_env(|| {
        let _threshold = ScopedEnvVar::set("TUITBOT_SCORING__THRESHOLD", "85");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.scoring.threshold, 85);
    });
}

#[test]
fn env_var_override_csv() {
    with_locked_env(|| {
        let _keywords = ScopedEnvVar::set("TUITBOT_BUSINESS__PRODUCT_KEYWORDS", "rust, cli, tools");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(
            config.business.product_keywords,
            vec!["rust", "cli", "tools"]
        );
    });
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
    let mut config = valid_test_config();

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
    let mut config = valid_test_config();
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
    let mut config = valid_test_config();
    config.schedule.preferred_times = vec!["09:15".to_string(), "12:30".to_string()];
    assert!(config.validate().is_ok());
}

#[test]
fn validate_preferred_times_auto() {
    let mut config = valid_test_config();
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
    with_locked_env(|| {
        let _approval = ScopedEnvVar::set("TUITBOT_APPROVAL_MODE", "true");
        let mut config = Config::default();
        config.approval_mode = false;
        config.apply_env_overrides().expect("env override");
        assert!(config.approval_mode);
    });
}

#[test]
fn env_var_override_approval_mode_false() {
    with_locked_env(|| {
        let _approval = ScopedEnvVar::set("TUITBOT_APPROVAL_MODE", "false");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(!config.approval_mode);
    });
}

#[test]
fn openclaw_env_enables_approval_mode() {
    with_locked_env(|| {
        let _agent_id = ScopedEnvVar::set("OPENCLAW_AGENT_ID", "test");
        let mut config = Config::default();
        config.approval_mode = false;
        config.apply_env_overrides().expect("env override");
        assert!(config.approval_mode);
    });
}

#[test]
fn openclaw_env_respects_explicit_override() {
    with_locked_env(|| {
        let _agent_id = ScopedEnvVar::set("OPENCLAW_AGENT_ID", "test");
        let _approval = ScopedEnvVar::set("TUITBOT_APPROVAL_MODE", "false");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(!config.approval_mode);
    });
}

// --- BusinessProfile quickstart/enrichment tests ---

#[test]
fn quickstart_minimal_config_validates() {
    let mut config = Config::default();
    config.business = BusinessProfile::quickstart(
        "MyApp".to_string(),
        vec!["rust cli".to_string(), "developer tools".to_string()],
    );
    config.business.product_description = "A developer tool".to_string();
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

// --- draft_context_keywords tests ---

#[test]
fn draft_context_keywords_empty_profile() {
    let profile = BusinessProfile::default();
    assert!(profile.draft_context_keywords().is_empty());
}

#[test]
fn draft_context_keywords_only_product() {
    let profile = BusinessProfile {
        product_keywords: vec!["rust".to_string(), "cli".to_string()],
        ..Default::default()
    };
    // When industry_topics is empty, effective_industry_topics() falls back to product_keywords.
    // So product_keywords appear twice: once from product, once from industry fallback.
    let keywords = profile.draft_context_keywords();
    assert_eq!(
        keywords,
        vec!["rust", "cli", "rust", "cli"],
        "product_keywords duplicated via effective_industry_topics fallback"
    );
}

#[test]
fn draft_context_keywords_all_keyword_types() {
    let profile = BusinessProfile {
        product_keywords: vec!["rust".to_string()],
        competitor_keywords: vec!["go".to_string(), "python".to_string()],
        industry_topics: vec!["systems programming".to_string()],
        ..Default::default()
    };
    let keywords = profile.draft_context_keywords();
    // Order: product → competitor → industry
    assert_eq!(
        keywords,
        vec!["rust", "go", "python", "systems programming"]
    );
}

#[test]
fn draft_context_keywords_deduplication_not_applied() {
    let profile = BusinessProfile {
        product_keywords: vec!["rust".to_string()],
        competitor_keywords: vec!["rust".to_string(), "go".to_string()],
        industry_topics: vec!["rust ecosystem".to_string()],
        ..Default::default()
    };
    let keywords = profile.draft_context_keywords();
    // "rust" appears in both product and competitor — no dedup applied.
    assert_eq!(keywords, vec!["rust", "rust", "go", "rust ecosystem"]);
}

// --- Content sources config tests ---

#[test]
fn content_sources_config_serde_roundtrip() {
    let toml_str = r#"
[[content_sources.sources]]
source_type = "local_fs"
path = "~/notes/vault"
watch = true
file_patterns = ["*.md", "*.txt"]
loop_back_enabled = true
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.content_sources.sources.len(), 1);
    let source = &config.content_sources.sources[0];
    assert_eq!(source.source_type, "local_fs");
    assert_eq!(source.path.as_deref(), Some("~/notes/vault"));
    assert!(source.watch);
    assert_eq!(source.file_patterns, vec!["*.md", "*.txt"]);
    assert!(source.loop_back_enabled);
}

#[test]
fn content_sources_defaults() {
    let toml_str = r#"
[[content_sources.sources]]
path = "~/notes"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    let source = &config.content_sources.sources[0];
    assert_eq!(source.source_type, "local_fs");
    assert!(source.watch);
    assert_eq!(source.file_patterns, vec!["*.md", "*.txt"]);
    assert!(source.loop_back_enabled);
}

#[test]
fn content_sources_optional_in_config() {
    let toml_str = r#"
[business]
product_name = "Test"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert!(config.content_sources.sources.is_empty());
}

#[test]
fn content_sources_json_patch_roundtrip() {
    // Simulates the JSON the frontend sends via PATCH /api/settings
    let patch = serde_json::json!({
        "content_sources": {
            "sources": [{
                "source_type": "local_fs",
                "path": "~/notes/vault",
                "watch": true,
                "file_patterns": ["*.md", "*.txt"],
                "loop_back_enabled": true
            }]
        }
    });
    let config: Config = serde_json::from_value(patch).expect("valid JSON");
    assert_eq!(config.content_sources.sources.len(), 1);
    let source = &config.content_sources.sources[0];
    assert_eq!(source.source_type, "local_fs");
    assert_eq!(source.path.as_deref(), Some("~/notes/vault"));
    assert!(source.watch);
    assert_eq!(source.file_patterns, vec!["*.md", "*.txt"]);
    assert!(source.loop_back_enabled);

    // Round-trip back to TOML and verify
    let toml_str = toml::to_string_pretty(&config).expect("serialize to TOML");
    let roundtripped: Config = toml::from_str(&toml_str).expect("re-parse TOML");
    assert_eq!(roundtripped.content_sources.sources.len(), 1);
    assert_eq!(
        roundtripped.content_sources.sources[0].path.as_deref(),
        Some("~/notes/vault")
    );
}

#[test]
fn content_sources_empty_json_patch() {
    // Frontend sends empty sources when user hasn't configured one
    let patch = serde_json::json!({
        "content_sources": {
            "sources": []
        }
    });
    let config: Config = serde_json::from_value(patch).expect("valid JSON");
    assert!(config.content_sources.sources.is_empty());
}

// --- Google Drive content sources ---

#[test]
fn content_sources_google_drive_roundtrip() {
    let toml_str = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "1aBcD_eFgHiJkLmNoPqRsTuVwXyZ"
service_account_key = "~/keys/my-project-sa.json"
watch = true
file_patterns = ["*.md", "*.txt"]
loop_back_enabled = false
poll_interval_seconds = 600
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.content_sources.sources.len(), 1);
    let source = &config.content_sources.sources[0];
    assert_eq!(source.source_type, "google_drive");
    assert_eq!(
        source.folder_id.as_deref(),
        Some("1aBcD_eFgHiJkLmNoPqRsTuVwXyZ")
    );
    assert_eq!(
        source.service_account_key.as_deref(),
        Some("~/keys/my-project-sa.json")
    );
    assert!(source.path.is_none());
    assert!(!source.loop_back_enabled);
    assert_eq!(source.poll_interval_seconds, Some(600));

    // Round-trip to TOML and back.
    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&toml_out).expect("re-parse");
    let rt_src = &roundtripped.content_sources.sources[0];
    assert_eq!(rt_src.source_type, "google_drive");
    assert_eq!(
        rt_src.folder_id.as_deref(),
        Some("1aBcD_eFgHiJkLmNoPqRsTuVwXyZ")
    );
}

#[test]
fn content_sources_mixed_sources_roundtrip() {
    let toml_str = r#"
[[content_sources.sources]]
source_type = "local_fs"
path = "~/notes/vault"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = true

[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc123"
service_account_key = "/etc/keys/sa.json"
watch = true
file_patterns = ["*.md", "*.txt"]
loop_back_enabled = false
poll_interval_seconds = 300
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.content_sources.sources.len(), 2);

    assert_eq!(config.content_sources.sources[0].source_type, "local_fs");
    assert_eq!(
        config.content_sources.sources[0].path.as_deref(),
        Some("~/notes/vault")
    );
    assert!(config.content_sources.sources[0].folder_id.is_none());

    assert_eq!(
        config.content_sources.sources[1].source_type,
        "google_drive"
    );
    assert_eq!(
        config.content_sources.sources[1].folder_id.as_deref(),
        Some("abc123")
    );
    assert!(config.content_sources.sources[1].path.is_none());
}

#[test]
fn content_sources_google_drive_json_patch() {
    let patch = serde_json::json!({
        "content_sources": {
            "sources": [{
                "source_type": "google_drive",
                "path": null,
                "folder_id": "drive_folder_abc",
                "service_account_key": "/path/to/key.json",
                "watch": true,
                "file_patterns": ["*.md", "*.txt"],
                "loop_back_enabled": false,
                "poll_interval_seconds": 300
            }]
        }
    });
    let config: Config = serde_json::from_value(patch).expect("valid JSON");
    assert_eq!(config.content_sources.sources.len(), 1);
    let source = &config.content_sources.sources[0];
    assert_eq!(source.source_type, "google_drive");
    assert_eq!(source.folder_id.as_deref(), Some("drive_folder_abc"));
    assert_eq!(
        source.service_account_key.as_deref(),
        Some("/path/to/key.json")
    );
    assert!(source.path.is_none());
    assert_eq!(source.poll_interval_seconds, Some(300));
}

// --- Deployment mode tests ---

#[test]
fn deployment_mode_desktop_allows_local_fs() {
    let mode = DeploymentMode::Desktop;
    assert!(mode.allows_source_type("local_fs"));
    assert!(mode.allows_source_type("google_drive"));
    assert!(mode.allows_source_type("manual"));
}

#[test]
fn deployment_mode_self_host_allows_local_fs() {
    let mode = DeploymentMode::SelfHost;
    assert!(mode.allows_source_type("local_fs"));
    assert!(mode.allows_source_type("google_drive"));
    assert!(mode.allows_source_type("manual"));
}

#[test]
fn deployment_mode_cloud_rejects_local_fs() {
    let mode = DeploymentMode::Cloud;
    assert!(!mode.allows_source_type("local_fs"));
    assert!(mode.allows_source_type("google_drive"));
    assert!(mode.allows_source_type("manual"));
}

#[test]
fn deployment_mode_unknown_source_type_rejected() {
    assert!(!DeploymentMode::Desktop.allows_source_type("unknown"));
    assert!(!DeploymentMode::Cloud.allows_source_type("ftp"));
}

#[test]
fn deployment_mode_capabilities_desktop() {
    let caps = DeploymentMode::Desktop.capabilities();
    assert!(caps.local_folder);
    assert!(caps.manual_local_path);
    assert!(caps.google_drive);
    assert!(caps.inline_ingest);
    assert!(caps.file_picker_native);
}

#[test]
fn deployment_mode_capabilities_self_host() {
    let caps = DeploymentMode::SelfHost.capabilities();
    assert!(caps.local_folder);
    assert!(caps.manual_local_path);
    assert!(caps.google_drive);
    assert!(caps.inline_ingest);
    assert!(!caps.file_picker_native);
}

#[test]
fn deployment_mode_capabilities_cloud() {
    let caps = DeploymentMode::Cloud.capabilities();
    assert!(!caps.local_folder);
    assert!(!caps.manual_local_path);
    assert!(caps.google_drive);
    assert!(caps.inline_ingest);
    assert!(!caps.file_picker_native);
}

#[test]
fn deployment_mode_serde_roundtrip_toml() {
    let toml_str = r#"
deployment_mode = "self_host"

[business]
product_name = "Test"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.deployment_mode, DeploymentMode::SelfHost);

    let serialized = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&serialized).expect("re-parse");
    assert_eq!(roundtripped.deployment_mode, DeploymentMode::SelfHost);
}

#[test]
fn deployment_mode_serde_roundtrip_json() {
    let json = serde_json::json!({
        "deployment_mode": "cloud"
    });
    let config: Config = serde_json::from_value(json).expect("valid JSON");
    assert_eq!(config.deployment_mode, DeploymentMode::Cloud);

    let serialized = serde_json::to_value(&config).expect("serialize");
    let roundtripped: Config = serde_json::from_value(serialized).expect("re-parse");
    assert_eq!(roundtripped.deployment_mode, DeploymentMode::Cloud);
}

#[test]
fn deployment_mode_default_is_desktop() {
    let config = Config::default();
    assert_eq!(config.deployment_mode, DeploymentMode::Desktop);
}

#[test]
fn deployment_mode_missing_from_config_defaults_to_desktop() {
    let toml_str = r#"
[business]
product_name = "Test"
product_keywords = ["test"]
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.deployment_mode, DeploymentMode::Desktop);
}

#[test]
fn deployment_mode_env_var_override() {
    with_locked_env(|| {
        let _mode = ScopedEnvVar::set("TUITBOT_DEPLOYMENT_MODE", "cloud");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.deployment_mode, DeploymentMode::Cloud);
    });
}

#[test]
fn deployment_mode_env_var_self_host_variants() {
    with_locked_env(|| {
        for variant in &["self_host", "selfhost", "self-host"] {
            let _mode = ScopedEnvVar::set("TUITBOT_DEPLOYMENT_MODE", variant);
            let mut config = Config::default();
            config.apply_env_overrides().expect("env override");
            assert_eq!(config.deployment_mode, DeploymentMode::SelfHost);
        }
    });
}

#[test]
fn deployment_mode_env_var_invalid() {
    with_locked_env(|| {
        let _mode = ScopedEnvVar::set("TUITBOT_DEPLOYMENT_MODE", "invalid_mode");
        let mut config = Config::default();
        let result = config.apply_env_overrides();
        assert!(result.is_err());
    });
}

#[test]
fn validate_local_fs_source_rejected_in_cloud_mode() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.deployment_mode = DeploymentMode::Cloud;
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/notes/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: true,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "auto".to_string(),
    });
    let errors = config.validate().unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ConfigError::InvalidValue { field, message }
            if field.contains("content_sources.sources[0]")
            && message.contains("cloud")
    )));
}

#[test]
fn validate_google_drive_source_allowed_in_cloud_mode() {
    let mut config = valid_test_config();
    config.deployment_mode = DeploymentMode::Cloud;
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "google_drive".to_string(),
        path: None,
        folder_id: Some("abc123".to_string()),
        service_account_key: Some("/keys/sa.json".to_string()),
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: Some(300),
        enabled: None,
        change_detection: "auto".to_string(),
    });
    assert!(config.validate().is_ok());
}

#[test]
fn validate_local_fs_source_allowed_in_desktop_mode() {
    let mut config = valid_test_config();
    config.deployment_mode = DeploymentMode::Desktop;
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/notes".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: true,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "auto".to_string(),
    });
    assert!(config.validate().is_ok());
}

#[test]
fn deployment_capabilities_serde_json_roundtrip() {
    let caps = DeploymentCapabilities {
        local_folder: true,
        manual_local_path: false,
        google_drive: true,
        inline_ingest: true,
        file_picker_native: false,
        preferred_source_default: "google_drive".to_string(),
    };
    let json = serde_json::to_value(&caps).expect("serialize");
    assert_eq!(json["local_folder"], true);
    assert_eq!(json["manual_local_path"], false);
    assert_eq!(json["preferred_source_default"], "google_drive");
    let roundtripped: DeploymentCapabilities = serde_json::from_value(json).expect("deserialize");
    assert_eq!(roundtripped, caps);
}

#[test]
fn deployment_mode_display() {
    assert_eq!(DeploymentMode::Desktop.to_string(), "desktop");
    assert_eq!(DeploymentMode::SelfHost.to_string(), "self_host");
    assert_eq!(DeploymentMode::Cloud.to_string(), "cloud");
}

// --- Preferred source default tests ---

#[test]
fn preferred_source_default_desktop() {
    let caps = DeploymentMode::Desktop.capabilities();
    assert_eq!(caps.preferred_source_default, "local_fs");
}

#[test]
fn preferred_source_default_self_host() {
    let caps = DeploymentMode::SelfHost.capabilities();
    assert_eq!(caps.preferred_source_default, "google_drive");
}

#[test]
fn preferred_source_default_cloud() {
    let caps = DeploymentMode::Cloud.capabilities();
    assert_eq!(caps.preferred_source_default, "google_drive");
}

#[test]
fn preferred_source_default_json_roundtrip() {
    let caps = DeploymentMode::SelfHost.capabilities();
    let json = serde_json::to_value(&caps).expect("serialize");
    assert_eq!(json["preferred_source_default"], "google_drive");
    let roundtripped: DeploymentCapabilities = serde_json::from_value(json).expect("deserialize");
    assert_eq!(roundtripped.preferred_source_default, "google_drive");
}

// --- connection_id tests ---

#[test]
fn content_source_connection_id_defaults_to_none() {
    let toml_str = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc123"
service_account_key = "/keys/sa.json"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert!(config.content_sources.sources[0].connection_id.is_none());
}

#[test]
fn content_source_connection_id_roundtrip() {
    let toml_str = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc123"
connection_id = 42
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(config.content_sources.sources[0].connection_id, Some(42));

    // Round-trip to TOML.
    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&toml_out).expect("re-parse");
    assert_eq!(
        roundtripped.content_sources.sources[0].connection_id,
        Some(42)
    );
}

#[test]
fn legacy_google_drive_config_still_parses() {
    // Existing TOML with service_account_key + folder_id (no connection_id).
    let toml_str = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "1aBcD_eFgHiJkLmNoPqRsTuVwXyZ"
service_account_key = "~/keys/my-project-sa.json"
watch = true
file_patterns = ["*.md", "*.txt"]
loop_back_enabled = false
poll_interval_seconds = 600
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    let source = &config.content_sources.sources[0];
    assert_eq!(source.source_type, "google_drive");
    assert_eq!(
        source.folder_id.as_deref(),
        Some("1aBcD_eFgHiJkLmNoPqRsTuVwXyZ")
    );
    assert_eq!(
        source.service_account_key.as_deref(),
        Some("~/keys/my-project-sa.json")
    );
    assert!(source.connection_id.is_none());
}

#[test]
fn connection_id_json_patch_roundtrip() {
    let patch = serde_json::json!({
        "content_sources": {
            "sources": [{
                "source_type": "google_drive",
                "folder_id": "drive_folder_abc",
                "connection_id": 7,
                "watch": true,
                "file_patterns": ["*.md"],
                "loop_back_enabled": false
            }]
        }
    });
    let config: Config = serde_json::from_value(patch).expect("valid JSON");
    assert_eq!(config.content_sources.sources[0].connection_id, Some(7));

    // Round-trip to TOML and back.
    let toml_str = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&toml_str).expect("re-parse");
    assert_eq!(
        roundtripped.content_sources.sources[0].connection_id,
        Some(7)
    );
}

// --- Connector config tests ---

#[test]
fn connector_config_optional() {
    let toml_str = r#"
[business]
product_name = "Test"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert!(config.connectors.google_drive.client_id.is_none());
    assert!(config.connectors.google_drive.client_secret.is_none());
    assert!(config.connectors.google_drive.redirect_uri.is_none());
}

#[test]
fn connector_config_toml_roundtrip() {
    let toml_str = r#"
[connectors.google_drive]
client_id = "123456.apps.googleusercontent.com"
client_secret = "GOCSPX-secret"
redirect_uri = "http://localhost:3001/api/connectors/google-drive/callback"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(
        config.connectors.google_drive.client_id.as_deref(),
        Some("123456.apps.googleusercontent.com")
    );
    assert_eq!(
        config.connectors.google_drive.client_secret.as_deref(),
        Some("GOCSPX-secret")
    );

    // Round-trip.
    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&toml_out).expect("re-parse");
    assert_eq!(
        roundtripped.connectors.google_drive.client_id.as_deref(),
        Some("123456.apps.googleusercontent.com")
    );
}

#[test]
fn connector_config_env_overrides() {
    with_locked_env(|| {
        let _client_id = ScopedEnvVar::set(
            "TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_ID",
            "env-client-id",
        );
        let _client_secret = ScopedEnvVar::set(
            "TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_SECRET",
            "env-client-secret",
        );
        let _redirect = ScopedEnvVar::set(
            "TUITBOT_CONNECTORS__GOOGLE_DRIVE__REDIRECT_URI",
            "http://myhost:3001/callback",
        );
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(
            config.connectors.google_drive.client_id.as_deref(),
            Some("env-client-id")
        );
        assert_eq!(
            config.connectors.google_drive.client_secret.as_deref(),
            Some("env-client-secret")
        );
        assert_eq!(
            config.connectors.google_drive.redirect_uri.as_deref(),
            Some("http://myhost:3001/callback")
        );
    });
}

#[test]
fn service_account_key_skipped_in_serialization_when_none() {
    // When service_account_key is None, it should not appear in TOML output.
    let toml_str = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc"
connection_id = 1
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    assert!(
        !toml_out.contains("service_account_key"),
        "service_account_key should not appear when None"
    );
}

#[test]
fn connection_id_skipped_in_serialization_when_none() {
    // When connection_id is None, it should not appear in TOML output.
    let toml_str = r#"
[[content_sources.sources]]
source_type = "local_fs"
path = "~/notes"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = true
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    assert!(
        !toml_out.contains("connection_id"),
        "connection_id should not appear when None"
    );
}

// --- Session 06: Backward-compatibility regression tests ---

#[test]
fn mixed_old_and_new_google_drive_source() {
    // Config with both service_account_key and connection_id parses and round-trips.
    let toml_str = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "1aBcD_eFgHiJkLmNoPqRsTuVwXyZ"
service_account_key = "~/keys/sa.json"
connection_id = 42
watch = true
file_patterns = ["*.md", "*.txt"]
loop_back_enabled = false
poll_interval_seconds = 300
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    let source = &config.content_sources.sources[0];
    assert_eq!(source.source_type, "google_drive");
    assert_eq!(
        source.service_account_key.as_deref(),
        Some("~/keys/sa.json")
    );
    assert_eq!(source.connection_id, Some(42));

    // Round-trip to TOML
    let toml_out = toml::to_string_pretty(&config).expect("serialize");
    let roundtripped: Config = toml::from_str(&toml_out).expect("re-parse");
    let rt_src = &roundtripped.content_sources.sources[0];
    assert_eq!(rt_src.connection_id, Some(42));
    assert_eq!(
        rt_src.service_account_key.as_deref(),
        Some("~/keys/sa.json")
    );

    // Validation passes (non-blocking warning only for dual-auth).
    let mut valid_config = config;
    valid_config.business.product_name = "Test".to_string();
    valid_config.business.product_keywords = vec!["test".to_string()];
    valid_config.business.product_description = "A test product".to_string();
    valid_config.business.industry_topics = vec!["testing".to_string()];
    valid_config.llm.provider = "ollama".to_string();
    valid_config.x_api.client_id = "test-id".to_string();
    assert!(valid_config.validate().is_ok());
}

#[test]
fn legacy_local_fs_config_unaffected_by_deployment_mode() {
    // Desktop mode with local_fs validates OK.
    let mut config = valid_test_config();
    config.deployment_mode = DeploymentMode::Desktop;
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/Obsidian/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: true,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "auto".to_string(),
    });
    assert!(config.validate().is_ok());

    // SelfHost mode with local_fs also validates OK (it's allowed).
    config.deployment_mode = DeploymentMode::SelfHost;
    assert!(config.validate().is_ok());
}

#[test]
fn legacy_sa_key_only_config_still_valid() {
    // A Google Drive source with only service_account_key (no connection_id)
    // passes validation in all deployment modes that allow google_drive.
    let mut config = valid_test_config();
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "google_drive".to_string(),
        path: None,
        folder_id: Some("abc123".to_string()),
        service_account_key: Some("/keys/sa.json".to_string()),
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: Some(300),
        enabled: None,
        change_detection: "auto".to_string(),
    });

    for mode in &[
        DeploymentMode::Desktop,
        DeploymentMode::SelfHost,
        DeploymentMode::Cloud,
    ] {
        config.deployment_mode = mode.clone();
        assert!(
            config.validate().is_ok(),
            "legacy SA-key config should be valid in {:?} mode",
            mode
        );
    }
}

#[test]
fn empty_content_sources_valid() {
    let mut config = valid_test_config();
    // Explicitly empty sources
    config.content_sources.sources = vec![];
    assert!(config.validate().is_ok());
}

#[test]
fn connection_id_without_sa_key_valid() {
    // A Google Drive source with only connection_id (no service_account_key).
    let mut config = valid_test_config();
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "google_drive".to_string(),
        path: None,
        folder_id: Some("folder_xyz".to_string()),
        service_account_key: None,
        connection_id: Some(7),
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "auto".to_string(),
    });
    assert!(config.validate().is_ok());
}

#[test]
fn google_drive_source_no_auth_warns() {
    // A Google Drive source with NEITHER connection_id NOR service_account_key
    // triggers a non-blocking validation warning (logged, not an error).
    let mut config = valid_test_config();
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "google_drive".to_string(),
        path: None,
        folder_id: Some("folder_abc".to_string()),
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "auto".to_string(),
    });
    // Should still pass validation (warning is non-blocking).
    assert!(config.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Content source enabled / change_detection semantics
// ---------------------------------------------------------------------------

#[test]
fn source_is_enabled_defaults_to_watch() {
    let entry = ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec![],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "auto".to_string(),
    };
    assert!(entry.is_enabled());

    let entry2 = ContentSourceEntry {
        watch: false,
        enabled: None,
        ..entry.clone()
    };
    assert!(!entry2.is_enabled());
}

#[test]
fn source_enabled_overrides_watch() {
    let entry = ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec![],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: Some(false),
        change_detection: "auto".to_string(),
    };
    assert!(
        !entry.is_enabled(),
        "enabled=false should override watch=true"
    );
}

#[test]
fn source_change_detection_defaults_auto() {
    let toml_str = r#"
[x_api]
client_id = "test-id"

[business]
product_name = "Test"
product_keywords = ["test"]

[llm]
provider = "ollama"
model = "llama2"

[[content_sources.sources]]
source_type = "local_fs"
path = "~/vault"
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    let source = &config.content_sources.sources[0];
    assert_eq!(source.change_detection, "auto");
    assert!(source.is_enabled());
    assert!(!source.is_poll_only());
    assert!(!source.is_scan_only());
}

#[test]
fn source_change_detection_poll() {
    let entry = ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec![],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "poll".to_string(),
    };
    assert!(entry.is_poll_only());
    assert!(!entry.is_scan_only());
}

#[test]
fn source_change_detection_none_means_scan_only() {
    let entry = ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec![],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "none".to_string(),
    };
    assert!(entry.is_scan_only());
}

#[test]
fn validate_invalid_change_detection() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-id".to_string();
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: None,
        change_detection: "invalid_value".to_string(),
    });
    let result = config.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        matches!(e, crate::error::ConfigError::InvalidValue { field, .. } if field.contains("change_detection"))
    }));
}

#[test]
fn validate_poll_interval_too_low() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-id".to_string();
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/vault".to_string()),
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: Some(10),
        enabled: None,
        change_detection: "auto".to_string(),
    });
    let result = config.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        matches!(e, crate::error::ConfigError::InvalidValue { field, .. } if field.contains("poll_interval_seconds"))
    }));
}

#[test]
fn validate_enabled_local_fs_without_path() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-id".to_string();
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: None,
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: Some(true),
        change_detection: "auto".to_string(),
    });
    let result = config.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        matches!(e, crate::error::ConfigError::MissingField { field } if field.contains("path"))
    }));
}

#[test]
fn validate_disabled_source_skips_path_check() {
    let mut config = valid_test_config();
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: None,
        folder_id: None,
        service_account_key: None,
        connection_id: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: None,
        enabled: Some(false),
        change_detection: "auto".to_string(),
    });
    // Disabled source without path should NOT produce an error.
    assert!(config.validate().is_ok());
}

#[test]
fn source_legacy_watch_false_parses_as_disabled() {
    let toml_str = r#"
[x_api]
client_id = "test-id"

[business]
product_name = "Test"
product_keywords = ["test"]

[llm]
provider = "ollama"
model = "llama2"

[[content_sources.sources]]
source_type = "local_fs"
path = "~/vault"
watch = false
"#;
    let config: Config = toml::from_str(toml_str).expect("valid TOML");
    let source = &config.content_sources.sources[0];
    assert!(!source.watch);
    assert!(source.enabled.is_none());
    assert!(
        !source.is_enabled(),
        "watch=false with no enabled override should be disabled"
    );
}

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
    config.storage.db_path = "/tmp".to_string(); // /tmp is a directory
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

// ─── A3: Env override tests for uncovered paths ─────────────────────────────

#[test]
fn env_override_mode_autopilot() {
    with_locked_env(|| {
        let _mode = ScopedEnvVar::set("TUITBOT_MODE", "autopilot");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(matches!(config.mode, OperatingMode::Autopilot));
    });
}

#[test]
fn env_override_mode_composer() {
    with_locked_env(|| {
        let _mode = ScopedEnvVar::set("TUITBOT_MODE", "composer");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(matches!(config.mode, OperatingMode::Composer));
    });
}

#[test]
fn env_override_mode_invalid() {
    with_locked_env(|| {
        let _mode = ScopedEnvVar::set("TUITBOT_MODE", "bad_mode");
        let mut config = Config::default();
        let result = config.apply_env_overrides();
        assert!(result.is_err());
    });
}

#[test]
fn env_override_deployment_mode_all_variants() {
    with_locked_env(|| {
        for (val, expected_display) in &[
            ("desktop", "desktop"),
            ("self_host", "self_host"),
            ("selfhost", "self_host"),
            ("self-host", "self_host"),
            ("cloud", "cloud"),
        ] {
            let _mode = ScopedEnvVar::set("TUITBOT_DEPLOYMENT_MODE", val);
            let mut config = Config::default();
            config.apply_env_overrides().expect("env override");
            assert_eq!(
                config.deployment_mode.to_string(),
                *expected_display,
                "deployment mode '{}' should parse",
                val
            );
        }
    });
}

#[test]
fn env_override_deployment_mode_invalid() {
    with_locked_env(|| {
        let _mode = ScopedEnvVar::set("TUITBOT_DEPLOYMENT_MODE", "serverless");
        let mut config = Config::default();
        let result = config.apply_env_overrides();
        assert!(result.is_err());
    });
}

#[test]
fn env_override_x_api_fields() {
    with_locked_env(|| {
        let _cid = ScopedEnvVar::set("TUITBOT_X_API__CLIENT_ID", "my-client-id");
        let _cs = ScopedEnvVar::set("TUITBOT_X_API__CLIENT_SECRET", "my-secret");
        let _pb = ScopedEnvVar::set("TUITBOT_X_API__PROVIDER_BACKEND", "scraper");
        let _sam = ScopedEnvVar::set("TUITBOT_X_API__SCRAPER_ALLOW_MUTATIONS", "true");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.x_api.client_id, "my-client-id");
        assert_eq!(config.x_api.client_secret.as_deref(), Some("my-secret"));
        assert_eq!(config.x_api.provider_backend, "scraper");
        assert!(config.x_api.scraper_allow_mutations);
    });
}

#[test]
fn env_override_business_fields() {
    with_locked_env(|| {
        let _pn = ScopedEnvVar::set("TUITBOT_BUSINESS__PRODUCT_NAME", "MyApp");
        let _pd = ScopedEnvVar::set("TUITBOT_BUSINESS__PRODUCT_DESCRIPTION", "An app");
        let _pu = ScopedEnvVar::set("TUITBOT_BUSINESS__PRODUCT_URL", "https://myapp.com");
        let _ta = ScopedEnvVar::set("TUITBOT_BUSINESS__TARGET_AUDIENCE", "devs");
        let _bv = ScopedEnvVar::set("TUITBOT_BUSINESS__BRAND_VOICE", "casual");
        let _rs = ScopedEnvVar::set("TUITBOT_BUSINESS__REPLY_STYLE", "friendly");
        let _cs = ScopedEnvVar::set("TUITBOT_BUSINESS__CONTENT_STYLE", "technical");
        let _ck = ScopedEnvVar::set("TUITBOT_BUSINESS__COMPETITOR_KEYWORDS", "alpha,beta");
        let _it = ScopedEnvVar::set("TUITBOT_BUSINESS__INDUSTRY_TOPICS", "ai,ml");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.business.product_name, "MyApp");
        assert_eq!(config.business.product_description, "An app");
        assert_eq!(
            config.business.product_url.as_deref(),
            Some("https://myapp.com")
        );
        assert_eq!(config.business.target_audience, "devs");
        assert_eq!(config.business.brand_voice.as_deref(), Some("casual"));
        assert_eq!(config.business.reply_style.as_deref(), Some("friendly"));
        assert_eq!(config.business.content_style.as_deref(), Some("technical"));
        assert_eq!(config.business.competitor_keywords, vec!["alpha", "beta"]);
        assert_eq!(config.business.industry_topics, vec!["ai", "ml"]);
    });
}

#[test]
fn env_override_llm_fields() {
    with_locked_env(|| {
        let _p = ScopedEnvVar::set("TUITBOT_LLM__PROVIDER", "openai");
        let _k = ScopedEnvVar::set("TUITBOT_LLM__API_KEY", "sk-test");
        let _m = ScopedEnvVar::set("TUITBOT_LLM__MODEL", "gpt-4");
        let _u = ScopedEnvVar::set("TUITBOT_LLM__BASE_URL", "https://api.example.com");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.api_key.as_deref(), Some("sk-test"));
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(
            config.llm.base_url.as_deref(),
            Some("https://api.example.com")
        );
    });
}

#[test]
fn env_override_limits_fields() {
    with_locked_env(|| {
        let _a = ScopedEnvVar::set("TUITBOT_LIMITS__MAX_REPLIES_PER_DAY", "10");
        let _b = ScopedEnvVar::set("TUITBOT_LIMITS__MAX_TWEETS_PER_DAY", "12");
        let _c = ScopedEnvVar::set("TUITBOT_LIMITS__MAX_THREADS_PER_WEEK", "3");
        let _d = ScopedEnvVar::set("TUITBOT_LIMITS__MIN_ACTION_DELAY_SECONDS", "30");
        let _e = ScopedEnvVar::set("TUITBOT_LIMITS__MAX_ACTION_DELAY_SECONDS", "300");
        let _f = ScopedEnvVar::set("TUITBOT_LIMITS__MAX_REPLIES_PER_AUTHOR_PER_DAY", "2");
        let _g = ScopedEnvVar::set("TUITBOT_LIMITS__BANNED_PHRASES", "spam,buy now");
        let _h = ScopedEnvVar::set("TUITBOT_LIMITS__PRODUCT_MENTION_RATIO", "0.5");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.limits.max_replies_per_day, 10);
        assert_eq!(config.limits.max_tweets_per_day, 12);
        assert_eq!(config.limits.max_threads_per_week, 3);
        assert_eq!(config.limits.min_action_delay_seconds, 30);
        assert_eq!(config.limits.max_action_delay_seconds, 300);
        assert_eq!(config.limits.max_replies_per_author_per_day, 2);
        assert_eq!(config.limits.banned_phrases, vec!["spam", "buy now"]);
        assert!((config.limits.product_mention_ratio - 0.5).abs() < 0.001);
    });
}

#[test]
fn env_override_intervals_fields() {
    with_locked_env(|| {
        let _a = ScopedEnvVar::set("TUITBOT_INTERVALS__MENTIONS_CHECK_SECONDS", "600");
        let _b = ScopedEnvVar::set("TUITBOT_INTERVALS__DISCOVERY_SEARCH_SECONDS", "1800");
        let _c = ScopedEnvVar::set("TUITBOT_INTERVALS__CONTENT_POST_WINDOW_SECONDS", "7200");
        let _d = ScopedEnvVar::set("TUITBOT_INTERVALS__THREAD_INTERVAL_SECONDS", "86400");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.intervals.mentions_check_seconds, 600);
        assert_eq!(config.intervals.discovery_search_seconds, 1800);
        assert_eq!(config.intervals.content_post_window_seconds, 7200);
        assert_eq!(config.intervals.thread_interval_seconds, 86400);
    });
}

#[test]
fn env_override_storage_fields() {
    with_locked_env(|| {
        let _p = ScopedEnvVar::set("TUITBOT_STORAGE__DB_PATH", "/custom/path.db");
        let _r = ScopedEnvVar::set("TUITBOT_STORAGE__RETENTION_DAYS", "30");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.storage.db_path, "/custom/path.db");
        assert_eq!(config.storage.retention_days, 30);
    });
}

#[test]
fn env_override_schedule_fields() {
    with_locked_env(|| {
        let _tz = ScopedEnvVar::set("TUITBOT_SCHEDULE__TIMEZONE", "US/Pacific");
        let _ahs = ScopedEnvVar::set("TUITBOT_SCHEDULE__ACTIVE_HOURS_START", "9");
        let _ahe = ScopedEnvVar::set("TUITBOT_SCHEDULE__ACTIVE_HOURS_END", "21");
        let _ad = ScopedEnvVar::set("TUITBOT_SCHEDULE__ACTIVE_DAYS", "Mon,Wed,Fri");
        let _pt = ScopedEnvVar::set("TUITBOT_SCHEDULE__PREFERRED_TIMES", "09:00,12:00");
        let _tpd = ScopedEnvVar::set("TUITBOT_SCHEDULE__THREAD_PREFERRED_DAY", "Tue");
        let _tpt = ScopedEnvVar::set("TUITBOT_SCHEDULE__THREAD_PREFERRED_TIME", "14:00");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.schedule.timezone, "US/Pacific");
        assert_eq!(config.schedule.active_hours_start, 9);
        assert_eq!(config.schedule.active_hours_end, 21);
        assert_eq!(config.schedule.active_days, vec!["Mon", "Wed", "Fri"]);
        assert_eq!(config.schedule.preferred_times, vec!["09:00", "12:00"]);
        assert_eq!(config.schedule.thread_preferred_day.as_deref(), Some("Tue"));
        assert_eq!(config.schedule.thread_preferred_time, "14:00");
    });
}

#[test]
fn env_override_thread_preferred_day_none() {
    with_locked_env(|| {
        let _tpd = ScopedEnvVar::set("TUITBOT_SCHEDULE__THREAD_PREFERRED_DAY", "none");
        let mut config = Config::default();
        config.schedule.thread_preferred_day = Some("Mon".to_string());
        config.apply_env_overrides().expect("env override");
        assert!(config.schedule.thread_preferred_day.is_none());
    });
}

#[test]
fn env_override_targets_fields() {
    with_locked_env(|| {
        let _a = ScopedEnvVar::set("TUITBOT_TARGETS__ACCOUNTS", "alice,bob,charlie");
        let _b = ScopedEnvVar::set("TUITBOT_TARGETS__MAX_TARGET_REPLIES_PER_DAY", "5");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.targets.accounts, vec!["alice", "bob", "charlie"]);
        assert_eq!(config.targets.max_target_replies_per_day, 5);
    });
}

#[test]
fn env_override_mcp_policy_fields() {
    with_locked_env(|| {
        let _efm = ScopedEnvVar::set("TUITBOT_MCP_POLICY__ENFORCE_FOR_MUTATIONS", "false");
        let _raf = ScopedEnvVar::set(
            "TUITBOT_MCP_POLICY__REQUIRE_APPROVAL_FOR",
            "post_tweet,follow_user",
        );
        let _bt = ScopedEnvVar::set("TUITBOT_MCP_POLICY__BLOCKED_TOOLS", "delete_tweet");
        let _drm = ScopedEnvVar::set("TUITBOT_MCP_POLICY__DRY_RUN_MUTATIONS", "yes");
        let _mmph = ScopedEnvVar::set("TUITBOT_MCP_POLICY__MAX_MUTATIONS_PER_HOUR", "50");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(!config.mcp_policy.enforce_for_mutations);
        assert_eq!(
            config.mcp_policy.require_approval_for,
            vec!["post_tweet", "follow_user"]
        );
        assert_eq!(config.mcp_policy.blocked_tools, vec!["delete_tweet"]);
        assert!(config.mcp_policy.dry_run_mutations);
        assert_eq!(config.mcp_policy.max_mutations_per_hour, 50);
    });
}

#[test]
fn env_override_auth_fields() {
    with_locked_env(|| {
        let _m = ScopedEnvVar::set("TUITBOT_AUTH__MODE", "local_callback");
        let _h = ScopedEnvVar::set("TUITBOT_AUTH__CALLBACK_HOST", "0.0.0.0");
        let _p = ScopedEnvVar::set("TUITBOT_AUTH__CALLBACK_PORT", "9090");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.auth.mode, "local_callback");
        assert_eq!(config.auth.callback_host, "0.0.0.0");
        assert_eq!(config.auth.callback_port, 9090);
    });
}

#[test]
fn env_override_scoring_fields() {
    with_locked_env(|| {
        let _rc = ScopedEnvVar::set("TUITBOT_SCORING__REPLY_COUNT_MAX", "20.0");
        let _ct = ScopedEnvVar::set("TUITBOT_SCORING__CONTENT_TYPE_MAX", "12.5");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!((config.scoring.reply_count_max - 20.0).abs() < 0.01);
        assert!((config.scoring.content_type_max - 12.5).abs() < 0.01);
    });
}

#[test]
fn env_override_logging_fields() {
    with_locked_env(|| {
        let _s = ScopedEnvVar::set("TUITBOT_LOGGING__STATUS_INTERVAL_SECONDS", "120");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.logging.status_interval_seconds, 120);
    });
}

#[test]
fn env_no_overrides_is_noop() {
    with_locked_env(|| {
        let mut config = Config::default();
        let before_threshold = config.scoring.threshold;
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.scoring.threshold, before_threshold);
    });
}

// ─── A4: Types tests — serde roundtrip, Default, method coverage ────────────

#[test]
fn config_default_serde_roundtrip() {
    let config = Config::default();
    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: Config = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.scoring.threshold, config.scoring.threshold);
    assert_eq!(
        deserialized.limits.max_replies_per_day,
        config.limits.max_replies_per_day
    );
    assert_eq!(deserialized.storage.db_path, config.storage.db_path);
    assert_eq!(deserialized.auth.mode, config.auth.mode);
}

#[test]
fn server_config_default() {
    let sc = super::types::ServerConfig::default();
    assert_eq!(sc.host, "127.0.0.1");
    assert_eq!(sc.port, 3001);
}

#[test]
fn operating_mode_serde_roundtrip() {
    for mode_str in &["autopilot", "composer"] {
        let json = format!("\"{}\"", mode_str);
        let mode: OperatingMode = serde_json::from_str(&json).expect("deserialize");
        let back = serde_json::to_string(&mode).expect("serialize");
        assert_eq!(back, json);
    }
}

#[test]
fn content_source_manual_type_allowed_by_all_modes() {
    for mode in &[
        DeploymentMode::Desktop,
        DeploymentMode::SelfHost,
        DeploymentMode::Cloud,
    ] {
        assert!(
            mode.allows_source_type("manual"),
            "manual should be allowed in {:?}",
            mode
        );
    }
}

#[test]
fn content_source_unknown_type_rejected() {
    for mode in &[
        DeploymentMode::Desktop,
        DeploymentMode::SelfHost,
        DeploymentMode::Cloud,
    ] {
        assert!(
            !mode.allows_source_type("s3"),
            "s3 should not be allowed in {:?}",
            mode
        );
    }
}

#[test]
fn business_profile_effective_industry_topics_fallback() {
    let mut bp = super::types::BusinessProfile::default();
    bp.product_keywords = vec!["rust".to_string(), "cli".to_string()];
    // industry_topics empty -> falls back to product_keywords
    assert_eq!(bp.effective_industry_topics(), &["rust", "cli"]);
    // Set industry_topics -> uses those instead
    bp.industry_topics = vec!["programming".to_string()];
    assert_eq!(bp.effective_industry_topics(), &["programming"]);
}

#[test]
fn content_source_entry_is_poll_only() {
    use super::types::{ContentSourceEntry, CHANGE_DETECTION_POLL};
    let toml_str = format!(
        r#"
source_type = "local_fs"
path = "/tmp/content"
change_detection = "{}"
"#,
        CHANGE_DETECTION_POLL
    );
    let entry: ContentSourceEntry = toml::from_str(&toml_str).expect("parse");
    assert!(entry.is_poll_only());
    assert!(!entry.is_scan_only());
}

#[test]
fn content_source_entry_is_scan_only() {
    use super::types::{ContentSourceEntry, CHANGE_DETECTION_NONE};
    let toml_str = format!(
        r#"
source_type = "local_fs"
path = "/tmp/content"
change_detection = "{}"
"#,
        CHANGE_DETECTION_NONE
    );
    let entry: ContentSourceEntry = toml::from_str(&toml_str).expect("parse");
    assert!(entry.is_scan_only());
    assert!(!entry.is_poll_only());
}

#[test]
fn content_source_disabled_effective_change_detection_is_none() {
    use super::types::ContentSourceEntry;
    let toml_str = r#"
source_type = "local_fs"
path = "/tmp/content"
enabled = false
change_detection = "poll"
"#;
    let entry: ContentSourceEntry = toml::from_str(toml_str).expect("parse");
    assert_eq!(entry.effective_change_detection(), "none");
}

#[test]
fn config_effective_approval_mode() {
    let mut config = Config::default();
    config.approval_mode = true;
    assert!(config.effective_approval_mode());
    config.approval_mode = false;
    config.mode = OperatingMode::Composer;
    assert!(config.effective_approval_mode()); // composer mode implies approval
}

#[test]
fn config_is_composer_mode() {
    let mut config = Config::default();
    assert!(!config.is_composer_mode());
    config.mode = OperatingMode::Composer;
    assert!(config.is_composer_mode());
}
