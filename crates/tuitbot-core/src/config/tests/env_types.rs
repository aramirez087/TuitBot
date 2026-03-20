//! Tests: env override edge cases and type serde/Default/method coverage.

use super::*;

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
