//! Tests: BusinessProfile, draft_context_keywords, content sources, Google Drive sources.

use super::*;

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
