//! Tests for config type structs.

use super::*;

// --- XApiConfig ---

#[test]
fn x_api_config_default() {
    let cfg = XApiConfig::default();
    assert!(cfg.client_id.is_empty());
    assert!(cfg.client_secret.is_none());
    assert!(cfg.provider_backend.is_empty());
    assert!(!cfg.scraper_allow_mutations);
}

#[test]
fn x_api_config_serde_roundtrip() {
    let cfg = XApiConfig {
        client_id: "my-client-id".into(),
        client_secret: Some("secret".into()),
        provider_backend: "x_api".into(),
        scraper_allow_mutations: true,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: XApiConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.client_id, "my-client-id");
    assert_eq!(back.client_secret.as_deref(), Some("secret"));
    assert_eq!(back.provider_backend, "x_api");
    assert!(back.scraper_allow_mutations);
}

#[test]
fn x_api_config_deserialize_empty() {
    let cfg: XApiConfig = serde_json::from_str("{}").unwrap();
    assert!(cfg.client_id.is_empty());
    assert!(cfg.client_secret.is_none());
    assert!(!cfg.scraper_allow_mutations);
}

// --- AuthConfig ---

#[test]
fn auth_config_serde_roundtrip() {
    let cfg = AuthConfig {
        mode: "local_callback".into(),
        callback_host: "0.0.0.0".into(),
        callback_port: 9090,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, "local_callback");
    assert_eq!(back.callback_host, "0.0.0.0");
    assert_eq!(back.callback_port, 9090);
}

#[test]
fn auth_config_deserialize_defaults() {
    let cfg: AuthConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(cfg.mode, "manual");
    assert_eq!(cfg.callback_host, "127.0.0.1");
    assert_eq!(cfg.callback_port, 8080);
}

// --- BusinessProfile ---

#[test]
fn business_profile_default() {
    let bp = BusinessProfile::default();
    assert!(bp.product_name.is_empty());
    assert!(bp.product_keywords.is_empty());
    assert!(bp.product_description.is_empty());
    assert!(bp.product_url.is_none());
    assert!(bp.target_audience.is_empty());
    assert!(bp.competitor_keywords.is_empty());
    assert!(bp.industry_topics.is_empty());
    assert!(bp.brand_voice.is_none());
    assert!(bp.reply_style.is_none());
    assert!(bp.content_style.is_none());
    assert!(bp.persona_opinions.is_empty());
    assert!(bp.persona_experiences.is_empty());
    assert!(bp.content_pillars.is_empty());
}

#[test]
fn business_profile_quickstart() {
    let bp = BusinessProfile::quickstart(
        "MyApp".to_string(),
        vec!["rust".to_string(), "cli".to_string()],
    );
    assert_eq!(bp.product_name, "MyApp");
    assert_eq!(bp.product_keywords, vec!["rust", "cli"]);
    assert_eq!(bp.industry_topics, vec!["rust", "cli"]);
    assert!(bp.product_description.is_empty());
}

#[test]
fn business_profile_effective_industry_topics_nonempty() {
    let bp = BusinessProfile {
        product_keywords: vec!["a".into()],
        industry_topics: vec!["b".into(), "c".into()],
        ..Default::default()
    };
    assert_eq!(bp.effective_industry_topics(), &["b", "c"]);
}

#[test]
fn business_profile_effective_industry_topics_empty_falls_back() {
    let bp = BusinessProfile {
        product_keywords: vec!["fallback".into()],
        industry_topics: vec![],
        ..Default::default()
    };
    assert_eq!(bp.effective_industry_topics(), &["fallback"]);
}

#[test]
fn business_profile_draft_context_keywords() {
    let bp = BusinessProfile {
        product_keywords: vec!["prod".into()],
        competitor_keywords: vec!["comp".into()],
        industry_topics: vec!["topic".into()],
        ..Default::default()
    };
    let kw = bp.draft_context_keywords();
    assert!(kw.contains(&"prod".to_string()));
    assert!(kw.contains(&"comp".to_string()));
    assert!(kw.contains(&"topic".to_string()));
}

#[test]
fn business_profile_draft_context_keywords_dedup_with_fallback() {
    let bp = BusinessProfile {
        product_keywords: vec!["rust".into()],
        competitor_keywords: vec![],
        industry_topics: vec![], // falls back to product_keywords
        ..Default::default()
    };
    let kw = bp.draft_context_keywords();
    assert_eq!(kw.iter().filter(|k| *k == "rust").count(), 2);
}

#[test]
fn business_profile_is_enriched_false_when_empty() {
    let bp = BusinessProfile::default();
    assert!(!bp.is_enriched());
}

#[test]
fn business_profile_is_enriched_true_with_brand_voice() {
    let bp = BusinessProfile {
        brand_voice: Some("Friendly".into()),
        ..Default::default()
    };
    assert!(bp.is_enriched());
}

#[test]
fn business_profile_is_enriched_true_with_reply_style() {
    let bp = BusinessProfile {
        reply_style: Some("Casual".into()),
        ..Default::default()
    };
    assert!(bp.is_enriched());
}

#[test]
fn business_profile_is_enriched_true_with_content_style() {
    let bp = BusinessProfile {
        content_style: Some("Technical".into()),
        ..Default::default()
    };
    assert!(bp.is_enriched());
}

#[test]
fn business_profile_is_enriched_true_with_opinions() {
    let bp = BusinessProfile {
        persona_opinions: vec!["Rust is great".into()],
        ..Default::default()
    };
    assert!(bp.is_enriched());
}

#[test]
fn business_profile_is_enriched_true_with_experiences() {
    let bp = BusinessProfile {
        persona_experiences: vec!["Built CLI tools".into()],
        ..Default::default()
    };
    assert!(bp.is_enriched());
}

#[test]
fn business_profile_is_enriched_true_with_pillars() {
    let bp = BusinessProfile {
        content_pillars: vec!["Developer productivity".into()],
        ..Default::default()
    };
    assert!(bp.is_enriched());
}

#[test]
fn business_profile_is_enriched_false_with_empty_brand_voice() {
    let bp = BusinessProfile {
        brand_voice: Some(String::new()),
        ..Default::default()
    };
    assert!(!bp.is_enriched());
}

#[test]
fn business_profile_serde_roundtrip() {
    let bp = BusinessProfile {
        product_name: "TestApp".into(),
        product_keywords: vec!["test".into(), "qa".into()],
        product_description: "A testing tool".into(),
        product_url: Some("https://test.com".into()),
        target_audience: "developers".into(),
        competitor_keywords: vec!["alt".into()],
        industry_topics: vec!["testing".into()],
        brand_voice: Some("Friendly".into()),
        reply_style: Some("Casual".into()),
        content_style: Some("Sharp".into()),
        persona_opinions: vec!["Testing first".into()],
        persona_experiences: vec!["5 years QA".into()],
        content_pillars: vec!["Quality".into()],
    };
    let json = serde_json::to_string(&bp).unwrap();
    let back: BusinessProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.product_name, "TestApp");
    assert_eq!(back.product_keywords.len(), 2);
    assert_eq!(back.product_url.as_deref(), Some("https://test.com"));
    assert_eq!(back.brand_voice.as_deref(), Some("Friendly"));
    assert_eq!(back.persona_opinions.len(), 1);
}

// --- ScoringConfig ---

#[test]
fn scoring_config_deserialize_defaults() {
    let cfg: ScoringConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(cfg.threshold, 60);
    assert!((cfg.keyword_relevance_max - 25.0).abs() < 0.001);
    assert!((cfg.follower_count_max - 15.0).abs() < 0.001);
    assert!((cfg.recency_max - 10.0).abs() < 0.001);
    assert!((cfg.engagement_rate_max - 15.0).abs() < 0.001);
    assert!((cfg.reply_count_max - 15.0).abs() < 0.001);
    assert!((cfg.content_type_max - 10.0).abs() < 0.001);
}

#[test]
fn scoring_config_serde_roundtrip() {
    let cfg = ScoringConfig {
        threshold: 80,
        keyword_relevance_max: 30.0,
        follower_count_max: 20.0,
        recency_max: 15.0,
        engagement_rate_max: 20.0,
        reply_count_max: 10.0,
        content_type_max: 5.0,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: ScoringConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.threshold, 80);
    assert!((back.keyword_relevance_max - 30.0).abs() < 0.001);
}

// --- LimitsConfig ---

#[test]
fn limits_config_deserialize_defaults() {
    let cfg: LimitsConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(cfg.max_replies_per_day, 5);
    assert_eq!(cfg.max_tweets_per_day, 6);
    assert_eq!(cfg.max_threads_per_week, 1);
    assert_eq!(cfg.min_action_delay_seconds, 45);
    assert_eq!(cfg.max_action_delay_seconds, 180);
    assert_eq!(cfg.max_replies_per_author_per_day, 1);
    assert!(!cfg.banned_phrases.is_empty());
    assert!((cfg.product_mention_ratio - 0.2).abs() < 0.001);
}

#[test]
fn limits_config_serde_roundtrip() {
    let cfg = LimitsConfig {
        max_replies_per_day: 10,
        max_tweets_per_day: 8,
        max_threads_per_week: 3,
        min_action_delay_seconds: 60,
        max_action_delay_seconds: 300,
        max_replies_per_author_per_day: 2,
        banned_phrases: vec!["spam".into()],
        product_mention_ratio: 0.3,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: LimitsConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_replies_per_day, 10);
    assert_eq!(back.max_tweets_per_day, 8);
    assert_eq!(back.banned_phrases, vec!["spam"]);
}

// --- IntervalsConfig ---

#[test]
fn intervals_config_deserialize_defaults() {
    let cfg: IntervalsConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(cfg.mentions_check_seconds, 300);
    assert_eq!(cfg.discovery_search_seconds, 900);
    assert_eq!(cfg.content_post_window_seconds, 10800);
    assert_eq!(cfg.thread_interval_seconds, 604800);
}

#[test]
fn intervals_config_serde_roundtrip() {
    let cfg = IntervalsConfig {
        mentions_check_seconds: 120,
        discovery_search_seconds: 600,
        content_post_window_seconds: 7200,
        thread_interval_seconds: 86400,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: IntervalsConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mentions_check_seconds, 120);
    assert_eq!(back.discovery_search_seconds, 600);
}

// --- TargetsConfig ---

#[test]
fn targets_config_default() {
    let cfg = TargetsConfig::default();
    assert!(cfg.accounts.is_empty());
    assert_eq!(cfg.max_target_replies_per_day, 0);
}

#[test]
fn targets_config_serde_roundtrip() {
    let cfg = TargetsConfig {
        accounts: vec!["user1".into(), "user2".into()],
        max_target_replies_per_day: 5,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: TargetsConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.accounts.len(), 2);
    assert_eq!(back.max_target_replies_per_day, 5);
}

#[test]
fn targets_config_deserialize_defaults() {
    let cfg: TargetsConfig = serde_json::from_str("{}").unwrap();
    assert!(cfg.accounts.is_empty());
    assert_eq!(cfg.max_target_replies_per_day, 3);
}

// --- LlmConfig ---

#[test]
fn llm_config_default() {
    let cfg = LlmConfig::default();
    assert!(cfg.provider.is_empty());
    assert!(cfg.api_key.is_none());
    assert!(cfg.model.is_empty());
    assert!(cfg.base_url.is_none());
}

#[test]
fn llm_config_serde_roundtrip() {
    let cfg = LlmConfig {
        provider: "anthropic".into(),
        api_key: Some("sk-test".into()),
        model: "claude-3-5-sonnet".into(),
        base_url: Some("https://api.anthropic.com".into()),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: LlmConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.provider, "anthropic");
    assert_eq!(back.api_key.as_deref(), Some("sk-test"));
    assert_eq!(back.model, "claude-3-5-sonnet");
    assert_eq!(back.base_url.as_deref(), Some("https://api.anthropic.com"));
}

// --- StorageConfig ---

#[test]
fn storage_config_deserialize_defaults() {
    let cfg: StorageConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(cfg.db_path, "~/.tuitbot/tuitbot.db");
    assert_eq!(cfg.retention_days, 90);
}

#[test]
fn storage_config_serde_roundtrip() {
    let cfg = StorageConfig {
        db_path: "/custom/path.db".into(),
        retention_days: 30,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: StorageConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.db_path, "/custom/path.db");
    assert_eq!(back.retention_days, 30);
}

// --- ServerConfig ---

#[test]
fn server_config_default() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.host, "127.0.0.1");
    assert_eq!(cfg.port, 3001);
}

#[test]
fn server_config_serde_roundtrip() {
    let cfg = ServerConfig {
        host: "0.0.0.0".into(),
        port: 8080,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: ServerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.host, "0.0.0.0");
    assert_eq!(back.port, 8080);
}

// --- LoggingConfig ---

#[test]
fn logging_config_default() {
    let cfg = LoggingConfig::default();
    assert_eq!(cfg.status_interval_seconds, 0);
}

#[test]
fn logging_config_serde_roundtrip() {
    let cfg = LoggingConfig {
        status_interval_seconds: 60,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: LoggingConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status_interval_seconds, 60);
}

// --- ContentSourcesConfig ---

#[test]
fn content_sources_config_default() {
    let cfg = ContentSourcesConfig::default();
    assert!(cfg.sources.is_empty());
}

#[test]
fn content_sources_config_serde_roundtrip() {
    let cfg = ContentSourcesConfig {
        sources: vec![ContentSourceEntry {
            source_type: "local_fs".into(),
            path: Some("/notes".into()),
            folder_id: None,
            service_account_key: None,
            connection_id: None,
            watch: true,
            enabled: Some(true),
            change_detection: "auto".into(),
            file_patterns: vec!["*.md".into()],
            loop_back_enabled: false,
            analytics_sync_enabled: false,
            poll_interval_seconds: None,
        }],
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: ContentSourcesConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.sources.len(), 1);
    assert_eq!(back.sources[0].source_type, "local_fs");
    assert_eq!(back.sources[0].path.as_deref(), Some("/notes"));
}

// --- ContentSourceEntry ---

#[test]
fn content_source_entry_is_enabled_prefers_enabled() {
    let entry = ContentSourceEntry {
        enabled: Some(false),
        watch: true,
        ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
    };
    assert!(!entry.is_enabled());
}

#[test]
fn content_source_entry_is_enabled_fallback_to_watch() {
    let entry = ContentSourceEntry {
        enabled: None,
        watch: true,
        ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
    };
    assert!(entry.is_enabled());
}

#[test]
fn content_source_entry_effective_change_detection_disabled() {
    let entry = ContentSourceEntry {
        enabled: Some(false),
        change_detection: "auto".into(),
        ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
    };
    assert_eq!(entry.effective_change_detection(), "none");
}

#[test]
fn content_source_entry_effective_change_detection_poll() {
    let entry = ContentSourceEntry {
        enabled: Some(true),
        change_detection: "poll".into(),
        ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
    };
    assert_eq!(entry.effective_change_detection(), "poll");
    assert!(entry.is_poll_only());
    assert!(!entry.is_scan_only());
}

#[test]
fn content_source_entry_is_scan_only() {
    let entry = ContentSourceEntry {
        enabled: Some(true),
        change_detection: "none".into(),
        ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
    };
    assert!(entry.is_scan_only());
    assert!(!entry.is_poll_only());
}

#[test]
fn content_source_entry_deserialize_defaults() {
    let entry: ContentSourceEntry = serde_json::from_str("{}").unwrap();
    assert_eq!(entry.source_type, "local_fs");
    assert!(entry.watch);
    assert!(entry.enabled.is_none());
    assert_eq!(entry.change_detection, "auto");
    assert_eq!(entry.file_patterns, vec!["*.md", "*.txt"]);
    assert!(entry.loop_back_enabled);
    assert!(!entry.analytics_sync_enabled);
}

#[test]
fn content_source_entry_analytics_sync_roundtrip() {
    let mut entry: ContentSourceEntry = serde_json::from_str("{}").unwrap();
    entry.analytics_sync_enabled = true;
    let json = serde_json::to_string(&entry).unwrap();
    let back: ContentSourceEntry = serde_json::from_str(&json).unwrap();
    assert!(back.analytics_sync_enabled);
}

// --- DeploymentMode ---

#[test]
fn deployment_mode_default() {
    let mode = DeploymentMode::default();
    assert_eq!(mode, DeploymentMode::Desktop);
}

#[test]
fn deployment_mode_display() {
    assert_eq!(DeploymentMode::Desktop.to_string(), "desktop");
    assert_eq!(DeploymentMode::SelfHost.to_string(), "self_host");
    assert_eq!(DeploymentMode::Cloud.to_string(), "cloud");
}

#[test]
fn deployment_mode_serde_roundtrip() {
    for mode in &[
        DeploymentMode::Desktop,
        DeploymentMode::SelfHost,
        DeploymentMode::Cloud,
    ] {
        let json = serde_json::to_string(mode).unwrap();
        let back: DeploymentMode = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, mode);
    }
}

#[test]
fn deployment_mode_capabilities_desktop() {
    let caps = DeploymentMode::Desktop.capabilities();
    assert!(caps.local_folder);
    assert!(caps.manual_local_path);
    assert!(caps.google_drive);
    assert!(caps.inline_ingest);
    assert!(caps.file_picker_native);
    assert_eq!(caps.preferred_source_default, "local_fs");
}

#[test]
fn deployment_mode_capabilities_self_host() {
    let caps = DeploymentMode::SelfHost.capabilities();
    assert!(caps.local_folder);
    assert!(caps.manual_local_path);
    assert!(caps.google_drive);
    assert!(caps.inline_ingest);
    assert!(!caps.file_picker_native);
    assert_eq!(caps.preferred_source_default, "google_drive");
}

#[test]
fn deployment_mode_capabilities_cloud() {
    let caps = DeploymentMode::Cloud.capabilities();
    assert!(!caps.local_folder);
    assert!(!caps.manual_local_path);
    assert!(caps.google_drive);
    assert!(caps.inline_ingest);
    assert!(!caps.file_picker_native);
    assert_eq!(caps.preferred_source_default, "google_drive");
}

#[test]
fn deployment_mode_allows_source_type() {
    assert!(DeploymentMode::Desktop.allows_source_type("local_fs"));
    assert!(DeploymentMode::Desktop.allows_source_type("google_drive"));
    assert!(DeploymentMode::Desktop.allows_source_type("manual"));
    assert!(!DeploymentMode::Desktop.allows_source_type("unknown"));

    assert!(!DeploymentMode::Cloud.allows_source_type("local_fs"));
    assert!(DeploymentMode::Cloud.allows_source_type("google_drive"));
}

// --- DeploymentCapabilities ---

#[test]
fn deployment_capabilities_serde_roundtrip() {
    let caps = DeploymentCapabilities {
        local_folder: true,
        manual_local_path: true,
        google_drive: true,
        inline_ingest: false,
        file_picker_native: true,
        preferred_source_default: "local_fs".into(),
        privacy_envelope: "local_first".into(),
        ghostwriter_local_only: true,
    };
    let json = serde_json::to_string(&caps).unwrap();
    let back: DeploymentCapabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(back, caps);
}

// --- ConnectorConfig ---

#[test]
fn connector_config_default() {
    let cfg = ConnectorConfig::default();
    assert!(cfg.google_drive.client_id.is_none());
    assert!(cfg.google_drive.client_secret.is_none());
    assert!(cfg.google_drive.redirect_uri.is_none());
}

#[test]
fn connector_config_serde_roundtrip() {
    let cfg = ConnectorConfig {
        google_drive: GoogleDriveConnectorConfig {
            client_id: Some("gcp-client-id".into()),
            client_secret: Some("gcp-secret".into()),
            redirect_uri: Some("http://localhost:3001/callback".into()),
        },
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: ConnectorConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(
        back.google_drive.client_id.as_deref(),
        Some("gcp-client-id")
    );
    assert_eq!(
        back.google_drive.client_secret.as_deref(),
        Some("gcp-secret")
    );
}

// --- Constants ---

#[test]
fn change_detection_constants() {
    assert_eq!(CHANGE_DETECTION_AUTO, "auto");
    assert_eq!(CHANGE_DETECTION_POLL, "poll");
    assert_eq!(CHANGE_DETECTION_NONE, "none");
    assert_eq!(MIN_POLL_INTERVAL_SECONDS, 30);
}
