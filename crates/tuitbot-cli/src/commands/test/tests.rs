use super::*;
use tuitbot_core::config::LlmConfig;

fn valid_tokens() -> StoredTokens {
    StoredTokens {
        access_token: "access".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(1)),
        scopes: vec![
            "tweet.read".to_string(),
            "tweet.write".to_string(),
            "users.read".to_string(),
            "follows.read".to_string(),
            "follows.write".to_string(),
            "like.read".to_string(),
            "like.write".to_string(),
            "bookmark.read".to_string(),
            "bookmark.write".to_string(),
            "dm.read".to_string(),
            "dm.write".to_string(),
            "offline.access".to_string(),
        ],
    }
}

#[test]
fn check_auth_expired_token_fails_token_status() {
    let mut tokens = valid_tokens();
    tokens.expires_at = Some(chrono::Utc::now() - chrono::TimeDelta::minutes(5));

    let auth = evaluate_auth(Ok(tokens));

    let token_check = auth
        .checks
        .iter()
        .find(|check| check.label == "X API token")
        .expect("token check should exist");
    assert!(!token_check.passed);
}

#[test]
fn check_auth_missing_refresh_token_fails() {
    let mut tokens = valid_tokens();
    tokens.refresh_token = None;

    let auth = evaluate_auth(Ok(tokens));

    let refresh_check = auth
        .checks
        .iter()
        .find(|check| check.label == "X API refresh")
        .expect("refresh check should exist");
    assert!(!refresh_check.passed);
    assert!(refresh_check.message.contains("offline.access"));
}

#[test]
fn check_auth_full_scopes_all_ok() {
    let auth = evaluate_auth(Ok(valid_tokens()));

    assert_eq!(auth.checks.len(), 3);
    assert!(auth.checks.iter().all(|check| check.passed));
    assert!(auth.details.missing_scopes.is_empty());
    assert!(auth.details.degraded_features.is_empty());
}

#[test]
fn check_auth_partial_scopes_reports_degraded_features() {
    let mut tokens = valid_tokens();
    tokens.scopes.retain(|scope| scope != "like.write");

    let auth = evaluate_auth(Ok(tokens));

    let scope_check = auth
        .checks
        .iter()
        .find(|check| check.label == "X API scopes")
        .expect("scope check should exist");
    assert!(!scope_check.passed);
    assert!(scope_check.message.contains("like.write"));
    assert!(scope_check.message.contains("Like/unlike"));
    assert!(auth
        .details
        .degraded_features
        .contains(&"Like/unlike".to_string()));
}

#[test]
fn check_auth_no_tokens_returns_single_fail() {
    let auth = evaluate_auth(Err(StartupError::AuthRequired));

    assert_eq!(auth.checks.len(), 1);
    assert!(!auth.checks[0].passed);
    assert_eq!(auth.checks[0].label, "X API auth");
}

#[test]
fn check_auth_legacy_tokens_report_scope_not_tracked() {
    let mut tokens = valid_tokens();
    tokens.scopes.clear();

    let auth = evaluate_auth(Ok(tokens));

    let scope_check = auth
        .checks
        .iter()
        .find(|check| check.label == "X API scopes")
        .expect("scope check should exist");
    assert!(scope_check.passed);
    assert!(scope_check.message.contains("not tracked"));
    assert!(!auth.details.scopes_tracked);
}

#[test]
fn json_output_contains_auth_details() {
    let auth = evaluate_auth(Ok(valid_tokens()));
    let output = build_test_output(auth.checks, Some(auth.details));
    let value = serde_json::to_value(output).expect("serialize output");

    assert!(value.get("auth_details").is_some());
    assert!(value["auth_details"].is_object());
}

#[test]
fn check_llm_connectivity_not_configured() {
    let config = LlmConfig {
        provider: String::new(),
        api_key: None,
        model: String::new(),
        base_url: None,
    };
    let result = check_llm_connectivity_sync(&config);
    assert!(!result.passed);
    assert_eq!(result.label, "LLM connectivity");
    assert!(result.message.contains("not configured"));
}

#[test]
fn check_llm_connectivity_bad_provider_returns_fail() {
    let config = LlmConfig {
        provider: "nonexistent".to_string(),
        api_key: None,
        model: String::new(),
        base_url: None,
    };
    let result = check_llm_connectivity_sync(&config);
    assert!(!result.passed);
    assert_eq!(result.label, "LLM connectivity");
    assert!(result.message.contains("not configured"));
}

#[test]
fn next_step_guidance_all_pass() {
    let checks = vec![
        CheckResult::ok("Config", "ok"),
        CheckResult::ok("Auth", "ok"),
    ];
    let guidance = next_step_guidance(&checks);
    assert!(guidance.is_some());
    assert!(guidance.unwrap().contains("tuitbot tick --dry-run"));
}

#[test]
fn next_step_guidance_any_fail() {
    let checks = vec![
        CheckResult::ok("Config", "ok"),
        CheckResult::fail("Auth", "missing"),
    ];
    let guidance = next_step_guidance(&checks);
    assert!(guidance.is_none());
}

// ============================================================================
// check_business_profile
// ============================================================================

#[test]
fn check_business_profile_valid() {
    let mut config = tuitbot_core::config::Config::default();
    config.business.product_name = "TestApp".to_string();
    config.business.product_keywords = vec!["rust".to_string(), "cli".to_string()];

    let result = check_business_profile(&config);
    assert!(result.passed);
    assert!(
        result.message.contains("2 keywords"),
        "expected keyword count in message, got: {}",
        result.message
    );
}

#[test]
fn check_business_profile_empty_name_fails() {
    let mut config = tuitbot_core::config::Config::default();
    config.business.product_name = String::new();
    config.business.product_keywords = vec!["rust".to_string()];

    let result = check_business_profile(&config);
    assert!(!result.passed);
    assert!(result.message.contains("product_name"));
}

#[test]
fn check_business_profile_no_keywords_fails() {
    let mut config = tuitbot_core::config::Config::default();
    config.business.product_name = "App".to_string();
    config.business.product_keywords.clear();
    config.business.competitor_keywords.clear();

    let result = check_business_profile(&config);
    assert!(!result.passed);
    assert!(result.message.contains("keyword"));
}

#[test]
fn check_business_profile_competitor_keywords_count() {
    let mut config = tuitbot_core::config::Config::default();
    config.business.product_name = "App".to_string();
    config.business.product_keywords = vec!["rust".to_string()];
    config.business.competitor_keywords = vec!["golang".to_string(), "deno".to_string()];

    let result = check_business_profile(&config);
    assert!(result.passed);
    assert!(
        result.message.contains("3 keywords"),
        "expected combined count of 3, got: {}",
        result.message
    );
}

// ============================================================================
// check_llm_config
// ============================================================================

#[test]
fn check_llm_config_valid_openai() {
    let mut config = tuitbot_core::config::Config::default();
    config.llm.provider = "openai".to_string();
    config.llm.api_key = Some("sk-test".to_string());
    config.llm.model = "gpt-4o-mini".to_string();

    let result = check_llm_config(&config);
    assert!(result.passed);
}

#[test]
fn check_llm_config_empty_provider_fails() {
    let mut config = tuitbot_core::config::Config::default();
    config.llm.provider = String::new();

    let result = check_llm_config(&config);
    assert!(!result.passed);
    assert!(result.message.contains("no provider"));
}

#[test]
fn check_llm_config_unknown_provider_fails() {
    let mut config = tuitbot_core::config::Config::default();
    config.llm.provider = "gemini".to_string();

    let result = check_llm_config(&config);
    assert!(!result.passed);
    assert!(result.message.contains("unknown provider"));
}

#[test]
fn check_llm_config_openai_no_key_fails() {
    let mut config = tuitbot_core::config::Config::default();
    config.llm.provider = "openai".to_string();
    config.llm.api_key = None;

    let result = check_llm_config(&config);
    assert!(!result.passed);
    assert!(result.message.contains("api_key required"));
}

#[test]
fn check_llm_config_ollama_no_key_passes() {
    let mut config = tuitbot_core::config::Config::default();
    config.llm.provider = "ollama".to_string();
    config.llm.api_key = None;
    config.llm.model = "llama3.2".to_string();

    let result = check_llm_config(&config);
    assert!(result.passed);
}

#[test]
fn check_llm_config_reports_model() {
    let mut config = tuitbot_core::config::Config::default();
    config.llm.provider = "openai".to_string();
    config.llm.api_key = Some("sk-test".to_string());
    config.llm.model = "gpt-4o-mini".to_string();

    let result = check_llm_config(&config);
    assert!(
        result.message.contains("gpt-4o-mini"),
        "expected model name in message, got: {}",
        result.message
    );
}

// ============================================================================
// check_database
// ============================================================================

#[test]
fn check_database_nonexistent_reports_will_create() {
    let mut config = tuitbot_core::config::Config::default();
    config.storage.db_path = "/tmp/tuitbot-test-nonexistent-db-12345.sqlite".to_string();

    let result = check_database(&config);
    assert!(result.passed);
    assert!(
        result.message.contains("will be created"),
        "expected 'will be created' in message, got: {}",
        result.message
    );
}

#[test]
fn check_database_existing_reports_size() {
    let tmp = std::env::temp_dir().join("tuitbot-test-db-size.sqlite");
    std::fs::write(&tmp, "test content for size check").unwrap();

    let mut config = tuitbot_core::config::Config::default();
    config.storage.db_path = tmp.display().to_string();

    let result = check_database(&config);
    assert!(result.passed);
    assert!(
        result.message.contains("MB"),
        "expected MB size in message, got: {}",
        result.message
    );

    let _ = std::fs::remove_file(&tmp);
}
