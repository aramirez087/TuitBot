//! Tests: deployment mode, capabilities, preferred source, connection_id, connector config.

use super::*;

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
        privacy_envelope: "user_controlled".to_string(),
        ghostwriter_local_only: false,
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

// --- Privacy envelope tests ---

#[test]
fn is_local_first_desktop() {
    assert!(DeploymentMode::Desktop.is_local_first());
}

#[test]
fn is_local_first_self_host() {
    assert!(!DeploymentMode::SelfHost.is_local_first());
}

#[test]
fn is_local_first_cloud() {
    assert!(!DeploymentMode::Cloud.is_local_first());
}

#[test]
fn privacy_envelope_desktop() {
    assert_eq!(DeploymentMode::Desktop.privacy_envelope(), "local_first");
}

#[test]
fn privacy_envelope_self_host() {
    assert_eq!(
        DeploymentMode::SelfHost.privacy_envelope(),
        "user_controlled"
    );
}

#[test]
fn privacy_envelope_cloud() {
    assert_eq!(
        DeploymentMode::Cloud.privacy_envelope(),
        "provider_controlled"
    );
}

#[test]
fn capabilities_privacy_fields_desktop() {
    let caps = DeploymentMode::Desktop.capabilities();
    assert_eq!(caps.privacy_envelope, "local_first");
    assert!(caps.ghostwriter_local_only);
}

#[test]
fn capabilities_privacy_fields_self_host() {
    let caps = DeploymentMode::SelfHost.capabilities();
    assert_eq!(caps.privacy_envelope, "user_controlled");
    assert!(!caps.ghostwriter_local_only);
}

#[test]
fn capabilities_privacy_fields_cloud() {
    let caps = DeploymentMode::Cloud.capabilities();
    assert_eq!(caps.privacy_envelope, "provider_controlled");
    assert!(!caps.ghostwriter_local_only);
}

#[test]
fn capabilities_serde_backward_compat() {
    // Old JSON without privacy_envelope and ghostwriter_local_only should deserialize
    // with serde defaults (empty string and false).
    let json = r#"{
        "local_folder": true,
        "manual_local_path": true,
        "google_drive": true,
        "inline_ingest": true,
        "file_picker_native": false,
        "preferred_source_default": "local_fs"
    }"#;
    let caps: DeploymentCapabilities = serde_json::from_str(json).expect("deserialize");
    assert!(caps.local_folder);
    assert_eq!(caps.privacy_envelope, "");
    assert!(!caps.ghostwriter_local_only);
}

#[test]
fn capabilities_privacy_fields_json_roundtrip() {
    let caps = DeploymentMode::Desktop.capabilities();
    let json = serde_json::to_value(&caps).expect("serialize");
    assert_eq!(json["privacy_envelope"], "local_first");
    assert_eq!(json["ghostwriter_local_only"], true);
    let roundtripped: DeploymentCapabilities = serde_json::from_value(json).expect("deserialize");
    assert_eq!(roundtripped.privacy_envelope, "local_first");
    assert!(roundtripped.ghostwriter_local_only);
}
