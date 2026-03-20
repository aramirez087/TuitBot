//! Backward-compatibility regression tests and content source enabled/change_detection semantics.

use super::*;

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
