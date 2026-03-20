//! Tests: TokenManager, get_access_token, token exchange, refresh.
use super::super::*;
use chrono::Datelike;

#[tokio::test]
async fn token_manager_get_access_token_returns_current() {
    let tokens = Tokens {
        access_token: "current_token".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let manager = TokenManager::new(tokens, "cid".into(), path);

    let tok = manager.get_access_token().await.unwrap();
    assert_eq!(tok, "current_token");
}

#[test]
fn save_tokens_with_special_characters() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let tokens = Tokens {
        access_token: "a+b/c=d&e?f".into(),
        refresh_token: "r!@#$%^&*()".into(),
        expires_at: Utc::now(),
        scopes: vec!["scope with spaces".into()],
    };

    save_tokens(&tokens, &path).expect("save");
    let loaded = load_tokens(&path).unwrap().unwrap();
    assert_eq!(loaded.access_token, "a+b/c=d&e?f");
    assert_eq!(loaded.refresh_token, "r!@#$%^&*()");
}

#[test]
fn save_tokens_large_scopes_list() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let scopes: Vec<String> = (0..100).map(|i| format!("scope_{i}")).collect();
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: scopes.clone(),
    };

    save_tokens(&tokens, &path).expect("save");
    let loaded = load_tokens(&path).unwrap().unwrap();
    assert_eq!(loaded.scopes.len(), 100);
    assert_eq!(loaded.scopes[50], "scope_50");
}

#[test]
fn load_tokens_empty_file_returns_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    std::fs::write(&path, "").expect("write");

    let result = load_tokens(&path);
    assert!(result.is_err());
}

#[test]
fn load_tokens_partial_json_returns_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    std::fs::write(&path, r#"{"access_token": "a"}"#).expect("write");

    // Missing required fields
    let result = load_tokens(&path);
    assert!(result.is_err());
}

#[test]
fn token_refresh_response_single_scope() {
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_in": 3600,
        "scope": "tweet.read"
    }"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    let scopes: Vec<&str> = resp.scope.split_whitespace().collect();
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0], "tweet.read");
}

#[test]
fn token_refresh_response_empty_scope() {
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_in": 3600,
        "scope": ""
    }"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    let scopes: Vec<&str> = resp.scope.split_whitespace().collect();
    assert!(scopes.is_empty());
}

#[test]
fn token_refresh_response_zero_expires_in() {
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_in": 0,
        "scope": "tweet.read"
    }"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.expires_in, 0);
}

#[test]
fn tokens_exactly_at_refresh_boundary() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::seconds(REFRESH_WINDOW_SECS),
        scopes: vec![],
    };
    let seconds_until_expiry = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(
        (seconds_until_expiry - REFRESH_WINDOW_SECS).abs() <= 1,
        "should be near boundary"
    );
}

#[tokio::test]
async fn token_manager_get_token_when_expired_fails() {
    let tokens = Tokens {
        access_token: "expired_token".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() - chrono::Duration::hours(1), // already expired
        scopes: vec![],
    };
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let manager = TokenManager::new(tokens, "cid".into(), path);
    // Should fail because refresh will fail (no real server)
    let result = manager.get_access_token().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn token_manager_multiple_access_calls_same_token() {
    let tokens = Tokens {
        access_token: "stable_token".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let manager = TokenManager::new(tokens, "cid".into(), path);

    let t1 = manager.get_access_token().await.unwrap();
    let t2 = manager.get_access_token().await.unwrap();
    let t3 = manager.get_access_token().await.unwrap();
    assert_eq!(t1, "stable_token");
    assert_eq!(t2, "stable_token");
    assert_eq!(t3, "stable_token");
}

#[test]
fn tokens_serde_with_iso8601_date_formats() {
    // RFC 3339 / ISO 8601 format
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_at": "2026-12-31T23:59:59.999Z"
    }"#;
    let tokens: Tokens = serde_json::from_str(json).unwrap();
    assert_eq!(tokens.expires_at.year(), 2026);

    // With timezone offset
    let json2 = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_at": "2026-06-15T12:00:00+00:00"
    }"#;
    let tokens2: Tokens = serde_json::from_str(json2).unwrap();
    assert_eq!(tokens2.expires_at.month(), 6);
}

#[cfg(unix)]
#[test]
fn save_tokens_overwrites_preserves_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let tokens = Tokens {
        access_token: "first".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    save_tokens(&tokens, &path).expect("save first");

    let tokens2 = Tokens {
        access_token: "second".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    save_tokens(&tokens2, &path).expect("save second");

    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "permissions should still be 600 after overwrite"
    );
}

#[test]
fn refresh_window_is_5_minutes() {
    assert_eq!(REFRESH_WINDOW_SECS, 300);
}

#[test]
fn auth_url_is_valid() {
    assert!(AUTH_URL.starts_with("https://"));
    assert!(AUTH_URL.contains("oauth2/authorize"));
}

#[test]
fn token_url_is_valid() {
    assert!(TOKEN_URL.starts_with("https://"));
    assert!(TOKEN_URL.contains("oauth2/token"));
}

#[test]
fn tokens_empty_access_token() {
    let tokens = Tokens {
        access_token: String::new(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    assert!(tokens.access_token.is_empty());
}

#[test]
fn tokens_with_unicode_scope() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec!["tweet.read".into(), "users.read".into()],
    };
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Tokens = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scopes, tokens.scopes);
}

#[cfg(unix)]
#[test]
fn save_tokens_to_readonly_dir_fails() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("temp dir");
    let readonly_dir = dir.path().join("readonly");
    std::fs::create_dir(&readonly_dir).expect("create dir");
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o444))
        .expect("set perms");

    let path = readonly_dir.join("tokens.json");
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };

    let result = save_tokens(&tokens, &path);
    assert!(result.is_err());

    // Cleanup: restore permissions so tempdir can be deleted
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o755))
        .expect("restore perms");
}

#[test]
fn load_tokens_invalid_json_key() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    // Valid JSON but wrong structure

    std::fs::write(&path, r#"{"wrong_key": "value"}"#).expect("write");
    let result = load_tokens(&path);
    assert!(result.is_err());
}

#[test]
fn token_manager_new_creates_manager() {
    let tokens = Tokens {
        access_token: "test_tok".into(),
        refresh_token: "test_ref".into(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
        scopes: vec!["s1".into()],
    };
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let _manager = TokenManager::new(tokens, "my_client".into(), path);
    // Manager created successfully — no panic
}

#[test]
fn tokens_access_token_long_value() {
    let long_token = "a".repeat(2048);
    let tokens = Tokens {
        access_token: long_token.clone(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Tokens = serde_json::from_str(&json).unwrap();
    assert_eq!(back.access_token.len(), 2048);
    assert_eq!(back.access_token, long_token);
}

#[test]
fn tokens_empty_refresh_token() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: String::new(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    assert!(tokens.refresh_token.is_empty());
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Tokens = serde_json::from_str(&json).unwrap();
    assert!(back.refresh_token.is_empty());
}

#[test]
fn tokens_expiry_far_past() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() - chrono::Duration::days(365),
        scopes: vec![],
    };
    let secs = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(secs < -86400);
}
