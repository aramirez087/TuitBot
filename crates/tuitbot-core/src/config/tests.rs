//! General configuration tests.

use super::env_overrides::{parse_env_bool, parse_env_u32, split_csv};
use super::*;
use crate::config::types::{DeploymentCapabilities, DeploymentMode};
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
    env::set_var("TUITBOT_DEPLOYMENT_MODE", "cloud");
    let mut config = Config::default();
    config.apply_env_overrides().expect("env override");
    assert_eq!(config.deployment_mode, DeploymentMode::Cloud);
    env::remove_var("TUITBOT_DEPLOYMENT_MODE");
}

#[test]
fn deployment_mode_env_var_self_host_variants() {
    for variant in &["self_host", "selfhost", "self-host"] {
        env::set_var("TUITBOT_DEPLOYMENT_MODE", variant);
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.deployment_mode, DeploymentMode::SelfHost);
        env::remove_var("TUITBOT_DEPLOYMENT_MODE");
    }
}

#[test]
fn deployment_mode_env_var_invalid() {
    env::set_var("TUITBOT_DEPLOYMENT_MODE", "invalid_mode");
    let mut config = Config::default();
    let result = config.apply_env_overrides();
    assert!(result.is_err());
    env::remove_var("TUITBOT_DEPLOYMENT_MODE");
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
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: true,
        poll_interval_seconds: None,
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
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.deployment_mode = DeploymentMode::Cloud;
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "google_drive".to_string(),
        path: None,
        folder_id: Some("abc123".to_string()),
        service_account_key: Some("/keys/sa.json".to_string()),
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: false,
        poll_interval_seconds: Some(300),
    });
    assert!(config.validate().is_ok());
}

#[test]
fn validate_local_fs_source_allowed_in_desktop_mode() {
    let mut config = Config::default();
    config.business.product_name = "Test".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.llm.provider = "ollama".to_string();
    config.deployment_mode = DeploymentMode::Desktop;
    config.content_sources.sources.push(ContentSourceEntry {
        source_type: "local_fs".to_string(),
        path: Some("~/notes".to_string()),
        folder_id: None,
        service_account_key: None,
        watch: true,
        file_patterns: vec!["*.md".to_string()],
        loop_back_enabled: true,
        poll_interval_seconds: None,
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
    };
    let json = serde_json::to_value(&caps).expect("serialize");
    assert_eq!(json["local_folder"], true);
    assert_eq!(json["manual_local_path"], false);
    let roundtripped: DeploymentCapabilities = serde_json::from_value(json).expect("deserialize");
    assert_eq!(roundtripped, caps);
}

#[test]
fn deployment_mode_display() {
    assert_eq!(DeploymentMode::Desktop.to_string(), "desktop");
    assert_eq!(DeploymentMode::SelfHost.to_string(), "self_host");
    assert_eq!(DeploymentMode::Cloud.to_string(), "cloud");
}
