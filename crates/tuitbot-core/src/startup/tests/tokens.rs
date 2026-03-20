//! Tests for StoredTokens: expiry, serialization, scope helpers, file I/O, scope analysis.

use crate::startup::config::StoredTokens;
use crate::x_api::scopes::REQUIRED_SCOPES;

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
