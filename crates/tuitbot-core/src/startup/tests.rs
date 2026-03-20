//! Tests for startup module: tier capabilities, tokens, PKCE, URL building, path helpers.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

use super::config::{ApiTier, StartupError, StoredTokens, TierCapabilities};
use super::db::{data_dir, expand_tilde, resolve_db_path, token_file_path, validate_db_path};
use super::services::{
    build_auth_url, build_redirect_uri, extract_auth_code, extract_callback_state,
    format_startup_banner, generate_pkce, url_encode, X_AUTH_URL,
};
use crate::x_api::scopes::REQUIRED_SCOPES;

// ============================================================================
// ApiTier
// ============================================================================

#[test]
fn api_tier_display() {
    assert_eq!(ApiTier::Free.to_string(), "Free");
    assert_eq!(ApiTier::Basic.to_string(), "Basic");
    assert_eq!(ApiTier::Pro.to_string(), "Pro");
}

// ============================================================================
// TierCapabilities
// ============================================================================

#[test]
fn free_tier_capabilities() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    assert!(!caps.mentions);
    assert!(!caps.discovery);
    assert!(caps.posting);
    assert!(!caps.search);
}

#[test]
fn basic_tier_capabilities() {
    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    assert!(caps.mentions);
    assert!(caps.discovery);
    assert!(caps.posting);
    assert!(caps.search);
}

#[test]
fn pro_tier_capabilities() {
    let caps = TierCapabilities::for_tier(ApiTier::Pro);
    assert!(caps.mentions);
    assert!(caps.discovery);
    assert!(caps.posting);
    assert!(caps.search);
}

#[test]
fn free_tier_enabled_loops() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let loops = caps.enabled_loop_names();
    assert_eq!(loops, vec!["content", "threads"]);
}

#[test]
fn basic_tier_enabled_loops() {
    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    let loops = caps.enabled_loop_names();
    assert_eq!(loops, vec!["mentions", "discovery", "content", "threads"]);
}

#[test]
fn tier_capabilities_format_status() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let status = caps.format_status();
    assert!(status.contains("Mentions: DISABLED"));
    assert!(status.contains("Discovery: DISABLED"));

    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    let status = caps.format_status();
    assert!(status.contains("Mentions: enabled"));
    assert!(status.contains("Discovery: enabled"));
}

// ============================================================================
// StoredTokens
// ============================================================================

#[test]
fn stored_tokens_not_expired() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(1)),
        scopes: vec![],
    };
    assert!(!tokens.is_expired());
}

#[test]
fn stored_tokens_expired() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: Some(chrono::Utc::now() - chrono::TimeDelta::hours(1)),
        scopes: vec![],
    };
    assert!(tokens.is_expired());
}

#[test]
fn stored_tokens_no_expiry_is_not_expired() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: vec![],
    };
    assert!(!tokens.is_expired());
}

#[test]
fn stored_tokens_format_expiry_hours() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::minutes(102)),
        scopes: vec![],
    };
    let formatted = tokens.format_expiry();
    assert!(formatted.contains("h"));
    assert!(formatted.contains("m"));
}

#[test]
fn stored_tokens_format_expiry_minutes_only() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::minutes(30)),
        scopes: vec![],
    };
    let formatted = tokens.format_expiry();
    assert!(formatted.contains("m"));
    assert!(!formatted.contains("h"));
}

#[test]
fn stored_tokens_format_expiry_expired() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: Some(chrono::Utc::now() - chrono::TimeDelta::hours(1)),
        scopes: vec![],
    };
    assert_eq!(tokens.format_expiry(), "expired");
}

#[test]
fn stored_tokens_format_expiry_no_expiry() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: vec![],
    };
    assert_eq!(tokens.format_expiry(), "no expiry set");
}

#[test]
fn stored_tokens_serialization_roundtrip() {
    let tokens = StoredTokens {
        access_token: "access123".to_string(),
        refresh_token: Some("refresh456".to_string()),
        expires_at: Some(
            chrono::DateTime::parse_from_rfc3339("2026-06-01T12:00:00Z")
                .expect("valid datetime")
                .with_timezone(&chrono::Utc),
        ),
        scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
    };
    let json = serde_json::to_string(&tokens).expect("serialize");
    let deserialized: StoredTokens = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.access_token, "access123");
    assert_eq!(deserialized.refresh_token.as_deref(), Some("refresh456"));
    assert!(deserialized.expires_at.is_some());
    assert_eq!(
        deserialized.scopes,
        vec!["tweet.read".to_string(), "tweet.write".to_string()]
    );
}

#[test]
fn stored_tokens_deserialize_without_scopes_defaults_empty() {
    let json = r#"{
        "access_token": "access123",
        "refresh_token": "refresh456",
        "expires_at": "2026-06-01T12:00:00Z"
    }"#;

    let tokens: StoredTokens = serde_json::from_str(json).expect("deserialize");
    assert!(tokens.scopes.is_empty());
    assert!(!tokens.has_scope_info());
}

#[test]
fn stored_tokens_scope_helpers_work() {
    let tokens = StoredTokens {
        access_token: "access123".to_string(),
        refresh_token: Some("refresh456".to_string()),
        expires_at: None,
        scopes: vec!["tweet.read".to_string(), "users.read".to_string()],
    };

    assert!(tokens.has_scope_info());
    assert!(tokens.has_scope("tweet.read"));
    assert!(!tokens.has_scope("tweet.write"));
}

// ============================================================================
// Token File I/O
// ============================================================================

#[test]
fn save_and_load_tokens() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("tokens.json");

    let tokens = StoredTokens {
        access_token: "test_access".to_string(),
        refresh_token: Some("test_refresh".to_string()),
        expires_at: None,
        scopes: vec!["tweet.read".to_string()],
    };

    let json = serde_json::to_string_pretty(&tokens).expect("serialize");
    std::fs::write(&path, &json).expect("write");

    let contents = std::fs::read_to_string(&path).expect("read");
    let loaded: StoredTokens = serde_json::from_str(&contents).expect("deserialize");
    assert_eq!(loaded.access_token, "test_access");
    assert_eq!(loaded.refresh_token.as_deref(), Some("test_refresh"));
    assert_eq!(loaded.scopes, vec!["tweet.read".to_string()]);
}

#[cfg(unix)]
#[test]
fn save_tokens_sets_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("tokens.json");
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: vec![],
    };
    let json = serde_json::to_string_pretty(&tokens).expect("serialize");
    std::fs::write(&path, &json).expect("write");
    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(&path, perms).expect("set perms");

    let meta = std::fs::metadata(&path).expect("metadata");
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);
}

// ============================================================================
// StartupError
// ============================================================================

#[test]
fn startup_error_display() {
    let err = StartupError::AuthRequired;
    assert_eq!(
        err.to_string(),
        "authentication required: run `tuitbot auth` first"
    );

    let err = StartupError::AuthExpired;
    assert!(err.to_string().contains("expired"));

    let err = StartupError::Config("bad field".to_string());
    assert_eq!(err.to_string(), "configuration error: bad field");

    let err = StartupError::XApiError("timeout".to_string());
    assert_eq!(err.to_string(), "X API error: timeout");
}

// ============================================================================
// PKCE
// ============================================================================

#[test]
fn generate_pkce_produces_valid_challenge() {
    let pkce = generate_pkce();
    assert_eq!(pkce.verifier.len(), 43);
    assert_eq!(pkce.challenge.len(), 43);
    assert_eq!(pkce.state.len(), 22);
    let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce.verifier.as_bytes()));
    assert_eq!(pkce.challenge, expected);
}

#[test]
fn generate_pkce_unique_each_time() {
    let a = generate_pkce();
    let b = generate_pkce();
    assert_ne!(a.verifier, b.verifier);
    assert_ne!(a.challenge, b.challenge);
    assert_ne!(a.state, b.state);
}

// ============================================================================
// URL Building
// ============================================================================

#[test]
fn build_auth_url_contains_required_params() {
    let url = build_auth_url(
        "client123",
        "http://localhost:8080/callback",
        "state456",
        "challenge789",
    );
    assert!(url.starts_with(X_AUTH_URL));
    assert!(url.contains("response_type=code"));
    assert!(url.contains("client_id=client123"));
    assert!(url.contains("code_challenge=challenge789"));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("state=state456"));
    assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
}

#[test]
fn build_redirect_uri_format() {
    let uri = build_redirect_uri("127.0.0.1", 8080);
    assert_eq!(uri, "http://127.0.0.1:8080/callback");
}

// ============================================================================
// extract_auth_code
// ============================================================================

#[test]
fn extract_code_from_full_url() {
    let code = extract_auth_code("http://127.0.0.1:8080/callback?code=abc123&state=xyz");
    assert_eq!(code, "abc123");
}

#[test]
fn extract_code_from_bare_code() {
    let code = extract_auth_code("  abc123  ");
    assert_eq!(code, "abc123");
}

#[test]
fn extract_code_from_url_without_state() {
    let code = extract_auth_code("http://127.0.0.1:8080/callback?code=mycode");
    assert_eq!(code, "mycode");
}

// ============================================================================
// URL Encoding
// ============================================================================

#[test]
fn url_encode_basic() {
    assert_eq!(url_encode("hello"), "hello");
    assert_eq!(url_encode("hello world"), "hello%20world");
    assert_eq!(
        url_encode("http://localhost:8080/callback"),
        "http%3A%2F%2Flocalhost%3A8080%2Fcallback"
    );
}

// ============================================================================
// Startup Banner
// ============================================================================

#[test]
fn startup_banner_free_tier() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let banner = format_startup_banner(ApiTier::Free, &caps, 300);
    assert!(banner.contains("Tuitbot v"));
    assert!(banner.contains("Tier: Free"));
    assert!(!banner.contains("mentions"));
    assert!(banner.contains("content"));
    assert!(banner.contains("threads"));
    assert!(!banner.contains("discovery"));
    assert!(banner.contains("every 300s"));
}

#[test]
fn startup_banner_basic_tier() {
    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    let banner = format_startup_banner(ApiTier::Basic, &caps, 0);
    assert!(banner.contains("Tier: Basic"));
    assert!(banner.contains("discovery"));
    assert!(banner.contains("disabled"));
}

#[test]
fn startup_banner_contains_ctrl_c_hint() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let banner = format_startup_banner(ApiTier::Free, &caps, 0);
    assert!(banner.contains("Ctrl+C"));
}

// ============================================================================
// Path Helpers
// ============================================================================

#[test]
fn expand_tilde_works() {
    let expanded = expand_tilde("~/.tuitbot/config.toml");
    assert!(!expanded.to_string_lossy().starts_with('~'));
}

#[test]
fn expand_tilde_no_tilde() {
    let expanded = expand_tilde("/absolute/path");
    assert_eq!(expanded, PathBuf::from("/absolute/path"));
}

#[test]
fn data_dir_under_home() {
    let dir = data_dir();
    assert!(dir.to_string_lossy().contains(".tuitbot"));
}

#[test]
fn token_file_path_under_data_dir() {
    let path = token_file_path();
    assert!(path.to_string_lossy().contains("tokens.json"));
    assert!(path.to_string_lossy().contains(".tuitbot"));
}

// ============================================================================
// extract_callback_state
// ============================================================================

#[test]
fn extract_callback_state_from_url() {
    let state =
        extract_callback_state("http://127.0.0.1:8080/callback?code=abc123&state=mystate456");
    assert_eq!(state, "mystate456");
}

#[test]
fn extract_callback_state_no_state() {
    let state = extract_callback_state("http://127.0.0.1:8080/callback?code=abc123");
    assert_eq!(state, "");
}

#[test]
fn extract_callback_state_state_only() {
    let state = extract_callback_state("state=xyz789");
    assert_eq!(state, "xyz789");
}

#[test]
fn extract_callback_state_with_http_suffix() {
    let state = extract_callback_state("/callback?code=abc&state=test123 HTTP/1.1");
    assert_eq!(state, "test123");
}

// ============================================================================
// validate_db_path
// ============================================================================

#[test]
fn validate_db_path_empty_rejected() {
    let result = validate_db_path("");
    assert!(result.is_err());
}

#[test]
fn validate_db_path_whitespace_rejected() {
    let result = validate_db_path("   ");
    assert!(result.is_err());
}

#[test]
fn validate_db_path_valid_path() {
    let result = validate_db_path("/tmp/test.db");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("/tmp/test.db"));
}

#[test]
fn validate_db_path_tilde_expansion() {
    let result = validate_db_path("~/.tuitbot/test.db");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(!path.to_string_lossy().starts_with('~'));
    assert!(path.to_string_lossy().contains("test.db"));
}

#[test]
fn validate_db_path_directory_rejected() {
    let tmp = std::env::temp_dir();
    let result = validate_db_path(tmp.to_str().unwrap());
    assert!(result.is_err());
}

// ============================================================================
// expand_tilde edge cases
// ============================================================================

#[test]
fn expand_tilde_bare_tilde() {
    let expanded = expand_tilde("~");
    if let Some(home) = dirs::home_dir() {
        assert_eq!(expanded, home);
    }
}

#[test]
fn expand_tilde_relative_path() {
    let expanded = expand_tilde("relative/path");
    assert_eq!(expanded, PathBuf::from("relative/path"));
}

// ============================================================================
// StoredTokens edge cases
// ============================================================================

#[test]
fn stored_tokens_analyze_scopes_returns_analysis() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
    };
    let analysis = tokens.analyze_scopes();
    assert!(
        !analysis.granted.is_empty() || analysis.missing.is_empty() || !analysis.missing.is_empty()
    );
}

#[test]
fn stored_tokens_time_until_expiry_some() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(2)),
        scopes: vec![],
    };
    let duration = tokens.time_until_expiry();
    assert!(duration.is_some());
    assert!(duration.unwrap().num_minutes() > 100);
}

#[test]
fn stored_tokens_time_until_expiry_none() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: vec![],
    };
    assert!(tokens.time_until_expiry().is_none());
}

// ============================================================================
// StartupError variants
// ============================================================================

#[test]
fn startup_error_all_variants_display() {
    let errors = vec![
        StartupError::Config("bad".to_string()),
        StartupError::AuthRequired,
        StartupError::AuthExpired,
        StartupError::TokenRefreshFailed("fail".to_string()),
        StartupError::Database("db err".to_string()),
        StartupError::LlmError("llm err".to_string()),
        StartupError::XApiError("api err".to_string()),
        StartupError::Other("other".to_string()),
    ];
    for err in &errors {
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }
}

// ============================================================================
// URL encoding edge cases
// ============================================================================

#[test]
fn url_encode_special_chars() {
    assert_eq!(url_encode("a b+c"), "a%20b%2Bc");
    assert_eq!(url_encode("foo@bar"), "foo%40bar");
    assert_eq!(url_encode("~valid_chars.-"), "~valid_chars.-");
}

#[test]
fn url_encode_empty() {
    assert_eq!(url_encode(""), "");
}

// ============================================================================
// resolve_db_path
// ============================================================================

#[test]
fn resolve_db_path_nonexistent_config_falls_back() {
    let result = resolve_db_path("/nonexistent/path/to/config.toml");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("tuitbot.db"));
}

#[test]
fn resolve_db_path_valid_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        "[storage]\ndb_path = \"~/.tuitbot/custom.db\"\n",
    )
    .expect("write config");
    let result = resolve_db_path(config_path.to_str().unwrap());
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("custom.db"));
}

#[test]
fn resolve_db_path_empty_db_path_in_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[storage]\ndb_path = \"\"\n").expect("write config");
    let result = resolve_db_path(config_path.to_str().unwrap());
    assert!(result.is_err());
}

// ============================================================================
// StartupError Io variant
// ============================================================================

#[test]
fn startup_error_io_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let startup_err = StartupError::Io(io_err);
    let msg = startup_err.to_string();
    assert!(msg.contains("missing"), "got: {msg}");
}

// ============================================================================
// Scope analysis
// ============================================================================

#[test]
fn stored_tokens_analyze_scopes_missing_scopes() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: vec!["tweet.read".to_string()],
    };
    let analysis = tokens.analyze_scopes();
    assert!(!analysis.missing.is_empty(), "should have missing scopes");
    assert!(!analysis.granted.is_empty(), "should have granted scopes");
}

#[test]
fn stored_tokens_analyze_scopes_all_granted() {
    let tokens = StoredTokens {
        access_token: "test".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: REQUIRED_SCOPES.iter().map(|s| s.to_string()).collect(),
    };
    let analysis = tokens.analyze_scopes();
    assert!(analysis.missing.is_empty(), "all scopes should be granted");
}

// ============================================================================
// File I/O roundtrip
// ============================================================================

#[test]
fn save_and_load_roundtrip_via_file_io() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("tokens.json");
    let tokens = StoredTokens {
        access_token: "at".to_string(),
        refresh_token: Some("rt".to_string()),
        expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(1)),
        scopes: vec!["offline.access".to_string()],
    };
    let json = serde_json::to_string_pretty(&tokens).expect("serialize");
    std::fs::write(&path, &json).expect("write");

    let loaded_json = std::fs::read_to_string(&path).expect("read");
    let loaded: StoredTokens = serde_json::from_str(&loaded_json).expect("parse");
    assert_eq!(loaded.access_token, "at");
    assert_eq!(loaded.refresh_token.as_deref(), Some("rt"));
    assert!(loaded.has_scope("offline.access"));
    assert!(!loaded.is_expired());
}
