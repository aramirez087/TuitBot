use super::*;

const OLD_CONFIG: &str = r#"
# =============================================================================
# Tuitbot Configuration -- Docklet (@getdocklet)
# =============================================================================

# --- X API Credentials ---
[x_api]
client_id = "YOUR_CLIENT_ID"

# --- Authentication Settings ---
[auth]
mode = "local_callback"
callback_host = "127.0.0.1"
callback_port = 8080

# --- Business Profile ---
[business]
product_name = "Docklet"
product_description = "A floating command strip for macOS"
product_url = "https://getdocklet.app"
target_audience = "Mac power users"
product_keywords = ["macos productivity", "mac menu bar"]
competitor_keywords = ["notchnook", "bartender mac"]
industry_topics = ["Mac productivity tips"]
brand_voice = "Confident but not cocky."
reply_style = "Lead with genuine value."
content_style = "Share genuinely useful Mac tips."

# --- Scoring Engine ---
[scoring]
threshold = 65
keyword_relevance_max = 40.0
follower_count_max = 15.0
recency_max = 20.0
engagement_rate_max = 25.0

# --- Safety Limits ---
[limits]
max_replies_per_day = 15
max_tweets_per_day = 3
max_threads_per_week = 1
min_action_delay_seconds = 45
max_action_delay_seconds = 180

# --- Automation Intervals ---
[intervals]
mentions_check_seconds = 300
discovery_search_seconds = 900
content_post_window_seconds = 18000
thread_interval_seconds = 604800

# --- LLM Provider ---
[llm]
provider = "anthropic"
api_key = "YOUR_KEY"
model = "claude-sonnet-4-6"

# --- Data Storage ---
[storage]
db_path = "~/.tuitbot/tuitbot.db"
retention_days = 90

# --- Logging ---
[logging]
status_interval_seconds = 3600
"#;

#[test]
fn detect_missing_from_old_config() {
    let missing = detect::detect_missing_features_from_str(OLD_CONFIG).unwrap();
    assert!(
        missing.contains(&UpgradeGroup::Persona),
        "should detect missing persona"
    );
    assert!(
        missing.contains(&UpgradeGroup::Targets),
        "should detect missing targets"
    );
    assert!(
        missing.contains(&UpgradeGroup::ApprovalMode),
        "should detect missing approval_mode"
    );
    assert!(
        missing.contains(&UpgradeGroup::EnhancedLimits),
        "should detect missing enhanced limits"
    );
    assert!(
        missing.contains(&UpgradeGroup::DeploymentMode),
        "should detect missing deployment_mode"
    );
    assert!(
        missing.contains(&UpgradeGroup::Connectors),
        "should detect missing connectors"
    );
    assert!(
        missing.contains(&UpgradeGroup::ContentSources),
        "should detect missing content_sources"
    );
    assert_eq!(missing.len(), 7);
}

#[test]
fn detect_nothing_missing_from_full_config() {
    let full = r#"
approval_mode = false
deployment_mode = "desktop"

[x_api]
client_id = "cid"

[business]
product_name = "Test"
product_keywords = ["test"]
industry_topics = ["topic"]
persona_opinions = []
persona_experiences = []
content_pillars = []

[scoring]
threshold = 60

[limits]
max_replies_per_day = 5
max_tweets_per_day = 6
max_threads_per_week = 1
min_action_delay_seconds = 45
max_action_delay_seconds = 180
max_replies_per_author_per_day = 1
banned_phrases = ["check out"]
product_mention_ratio = 0.2

[intervals]
mentions_check_seconds = 300

[targets]
accounts = []

[llm]
provider = "ollama"
model = "llama3.2"

[storage]
db_path = "~/.tuitbot/tuitbot.db"

[logging]
status_interval_seconds = 0

[connectors.google_drive]
client_id = "test.apps.googleusercontent.com"

[content_sources]
"#;
    let missing = detect::detect_missing_features_from_str(full).unwrap();
    assert!(
        missing.is_empty(),
        "full config should have no missing groups, got: {:?}",
        missing
    );
}

#[test]
fn key_exists_helper() {
    let toml_str = r#"
[business]
product_name = "Test"

[limits]
max_replies_per_day = 5
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let root = table.as_table().unwrap();

    assert!(detect::key_exists_public(root, "business"));
    assert!(detect::key_exists_public(root, "business.product_name"));
    assert!(!detect::key_exists_public(
        root,
        "business.persona_opinions"
    ));
    assert!(detect::key_exists_public(root, "limits"));
    assert!(detect::key_exists_public(
        root,
        "limits.max_replies_per_day"
    ));
    assert!(!detect::key_exists_public(root, "limits.banned_phrases"));
    assert!(!detect::key_exists_public(root, "targets"));
    assert!(!detect::key_exists_public(root, "approval_mode"));
}

#[test]
fn patch_config_preserves_comments() {
    let config_str = r#"# My custom header comment

# --- X API Credentials ---
[x_api]
client_id = "test-id"

# --- Business Profile ---
# This is my business section comment
[business]
product_name = "TestApp"

# --- Limits ---
[limits]
max_replies_per_day = 10
"#;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: Some((
            vec!["Rust is great".to_string()],
            vec!["Built 3 apps".to_string()],
            vec!["Dev tools".to_string()],
        )),
        targets: Some(vec!["elonmusk".to_string()]),
        approval_mode: Some(true),
        enhanced_limits: Some((
            2,
            vec!["check out".to_string(), "link in bio".to_string()],
            0.3,
        )),
        deployment_mode: None,
        connectors: None,
        content_sources_noticed: false,
    };

    let groups = vec![
        UpgradeGroup::Persona,
        UpgradeGroup::Targets,
        UpgradeGroup::ApprovalMode,
        UpgradeGroup::EnhancedLimits,
    ];

    patch::patch_config(tmp.path(), &groups, &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();

    // Original comments preserved
    assert!(
        result.contains("My custom header comment"),
        "header comment should be preserved"
    );
    assert!(
        result.contains("This is my business section comment"),
        "business section comment should be preserved"
    );

    // New keys are present and parseable
    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");

    assert_eq!(config.business.persona_opinions, vec!["Rust is great"]);
    assert_eq!(config.business.persona_experiences, vec!["Built 3 apps"]);
    assert_eq!(config.business.content_pillars, vec!["Dev tools"]);
    assert_eq!(config.targets.accounts, vec!["elonmusk"]);
    assert!(config.approval_mode);
    assert_eq!(config.limits.max_replies_per_author_per_day, 2);
    assert_eq!(
        config.limits.banned_phrases,
        vec!["check out", "link in bio"]
    );
    assert!((config.limits.product_mention_ratio - 0.3).abs() < f32::EPSILON);

    // Original values preserved
    assert_eq!(config.x_api.client_id, "test-id");
    assert_eq!(config.business.product_name, "TestApp");
    assert_eq!(config.limits.max_replies_per_day, 10);

    // Backup was created
    let backup = tmp.path().with_extension("toml.bak");
    assert!(backup.exists(), "backup file should exist");
}

#[test]
fn patch_config_persona_into_business() {
    let config_str = r#"
[business]
product_name = "App"
product_keywords = ["test"]
industry_topics = ["topic"]
"#;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: Some((
            vec!["opinion1".to_string()],
            vec!["experience1".to_string()],
            vec!["pillar1".to_string()],
        )),
        targets: None,
        approval_mode: None,
        enhanced_limits: None,
        deployment_mode: None,
        connectors: None,
        content_sources_noticed: false,
    };

    patch::patch_config(tmp.path(), &[UpgradeGroup::Persona], &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");

    assert_eq!(config.business.persona_opinions, vec!["opinion1"]);
    assert_eq!(config.business.persona_experiences, vec!["experience1"]);
    assert_eq!(config.business.content_pillars, vec!["pillar1"]);
    // Original values preserved
    assert_eq!(config.business.product_name, "App");
}

#[test]
fn patch_config_targets_new_section() {
    let config_str = r#"
[x_api]
client_id = "test"

[business]
product_name = "App"
"#;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: None,
        targets: Some(vec!["levelsio".to_string(), "naval".to_string()]),
        approval_mode: None,
        enhanced_limits: None,
        deployment_mode: None,
        connectors: None,
        content_sources_noticed: false,
    };

    patch::patch_config(tmp.path(), &[UpgradeGroup::Targets], &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");

    assert_eq!(config.targets.accounts, vec!["levelsio", "naval"]);
    assert_eq!(config.targets.max_target_replies_per_day, 3);
}

#[test]
fn patch_config_approval_mode_top_level() {
    let config_str = r#"
[x_api]
client_id = "test"
"#;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: None,
        targets: None,
        approval_mode: Some(true),
        enhanced_limits: None,
        deployment_mode: None,
        connectors: None,
        content_sources_noticed: false,
    };

    patch::patch_config(tmp.path(), &[UpgradeGroup::ApprovalMode], &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");

    assert!(config.approval_mode);
}

#[test]
fn patch_config_partial_limits_already_present() {
    // Config that already has max_replies_per_author_per_day but missing the others
    let config_str = r#"
[limits]
max_replies_per_day = 10
max_replies_per_author_per_day = 2
"#;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: None,
        targets: None,
        approval_mode: None,
        enhanced_limits: Some((1, vec!["check out".to_string()], 0.15)),
        deployment_mode: None,
        connectors: None,
        content_sources_noticed: false,
    };

    patch::patch_config(tmp.path(), &[UpgradeGroup::EnhancedLimits], &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");

    // Existing value preserved (not overwritten)
    assert_eq!(config.limits.max_replies_per_author_per_day, 2);
    // New values inserted
    assert_eq!(config.limits.banned_phrases, vec!["check out"]);
    assert!((config.limits.product_mention_ratio - 0.15).abs() < f32::EPSILON);
    // Original value preserved
    assert_eq!(config.limits.max_replies_per_day, 10);
}

// --- Session 06: New group tests ---

#[test]
fn detect_missing_deployment_mode() {
    let config = r#"
approval_mode = false
[business]
product_name = "Test"
persona_opinions = []
persona_experiences = []
content_pillars = []
product_keywords = ["test"]
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
[connectors.google_drive]
client_id = "test"
[content_sources]
"#;
    let missing = detect::detect_missing_features_from_str(config).unwrap();
    assert!(
        missing.contains(&UpgradeGroup::DeploymentMode),
        "should detect missing deployment_mode"
    );
    assert!(
        !missing.contains(&UpgradeGroup::Connectors),
        "connectors should not be missing"
    );
    assert!(
        !missing.contains(&UpgradeGroup::ContentSources),
        "content_sources should not be missing"
    );
}

#[test]
fn detect_missing_connectors() {
    let config = r#"
approval_mode = false
deployment_mode = "desktop"
[business]
product_name = "Test"
persona_opinions = []
persona_experiences = []
content_pillars = []
product_keywords = ["test"]
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
[content_sources]
"#;
    let missing = detect::detect_missing_features_from_str(config).unwrap();
    assert!(
        missing.contains(&UpgradeGroup::Connectors),
        "should detect missing connectors"
    );
    assert!(
        !missing.contains(&UpgradeGroup::DeploymentMode),
        "deployment_mode should not be missing"
    );
}

#[test]
fn detect_missing_content_sources() {
    let config = r#"
approval_mode = false
deployment_mode = "self_host"
[business]
product_name = "Test"
persona_opinions = []
persona_experiences = []
content_pillars = []
product_keywords = ["test"]
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
[connectors.google_drive]
client_id = "test"
"#;
    let missing = detect::detect_missing_features_from_str(config).unwrap();
    assert!(
        missing.contains(&UpgradeGroup::ContentSources),
        "should detect missing content_sources"
    );
}

#[test]
fn patch_deployment_mode_top_level() {
    let config_str = r#"
[x_api]
client_id = "test"
"#;
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: None,
        targets: None,
        approval_mode: None,
        enhanced_limits: None,
        deployment_mode: Some("self_host".to_string()),
        connectors: None,
        content_sources_noticed: false,
    };

    patch::patch_config(tmp.path(), &[UpgradeGroup::DeploymentMode], &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    assert!(result.contains("deployment_mode = \"self_host\""));

    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");
    assert_eq!(
        config.deployment_mode,
        tuitbot_core::config::DeploymentMode::SelfHost
    );
}

#[test]
fn patch_connectors_section() {
    let config_str = r#"
[x_api]
client_id = "test"
"#;
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: None,
        targets: None,
        approval_mode: None,
        enhanced_limits: None,
        deployment_mode: None,
        connectors: Some(Some((
            "my-client.apps.googleusercontent.com".to_string(),
            "GOCSPX-secret".to_string(),
        ))),
        content_sources_noticed: false,
    };

    patch::patch_config(tmp.path(), &[UpgradeGroup::Connectors], &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    assert!(result.contains("[connectors.google_drive]"));
    assert!(result.contains("my-client.apps.googleusercontent.com"));

    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");
    assert_eq!(
        config.connectors.google_drive.client_id.as_deref(),
        Some("my-client.apps.googleusercontent.com")
    );
    assert_eq!(
        config.connectors.google_drive.client_secret.as_deref(),
        Some("GOCSPX-secret")
    );
}

#[test]
fn patch_content_sources_scaffold() {
    let config_str = r#"
[x_api]
client_id = "test"
"#;
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: None,
        targets: None,
        approval_mode: None,
        enhanced_limits: None,
        deployment_mode: None,
        connectors: None,
        content_sources_noticed: true,
    };

    patch::patch_config(tmp.path(), &[UpgradeGroup::ContentSources], &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    assert!(
        result.contains("[content_sources]"),
        "should contain content_sources section"
    );

    // Must be parseable as valid config.
    let _config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");
}

#[test]
fn detect_legacy_sa_key_notice() {
    let config_with_legacy = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc"
service_account_key = "/keys/sa.json"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#;
    assert!(
        content_sources::has_legacy_sa_key(config_with_legacy),
        "should detect legacy SA key"
    );

    let config_with_connection = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc"
service_account_key = "/keys/sa.json"
connection_id = 1
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#;
    assert!(
        !content_sources::has_legacy_sa_key(config_with_connection),
        "should not flag when connection_id is present"
    );

    let config_no_sources = r#"
[business]
product_name = "Test"
"#;
    assert!(
        !content_sources::has_legacy_sa_key(config_no_sources),
        "should not flag when no sources"
    );
}

#[test]
fn patch_all_new_groups_together() {
    let config_str = r#"
approval_mode = true
[x_api]
client_id = "test"
[business]
product_name = "App"
product_keywords = ["test"]
persona_opinions = []
persona_experiences = []
content_pillars = []
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
"#;
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), config_str).unwrap();

    let answers = UpgradeAnswers {
        persona: None,
        targets: None,
        approval_mode: None,
        enhanced_limits: None,
        deployment_mode: Some("self_host".to_string()),
        connectors: Some(Some(("cid".to_string(), "csecret".to_string()))),
        content_sources_noticed: true,
    };

    let groups = vec![
        UpgradeGroup::DeploymentMode,
        UpgradeGroup::Connectors,
        UpgradeGroup::ContentSources,
    ];

    patch::patch_config(tmp.path(), &groups, &answers).unwrap();

    let result = std::fs::read_to_string(tmp.path()).unwrap();
    let config: tuitbot_core::config::Config =
        toml::from_str(&result).expect("patched config should parse");

    assert_eq!(
        config.deployment_mode,
        tuitbot_core::config::DeploymentMode::SelfHost
    );
    assert_eq!(
        config.connectors.google_drive.client_id.as_deref(),
        Some("cid")
    );
    assert!(result.contains("[content_sources]"));
    // Original values preserved
    assert!(config.approval_mode);
    assert_eq!(config.business.product_name, "App");
}
