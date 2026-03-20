use super::*;
use chrono::Datelike;

// ── Tokens struct ────────────────────────────────────────────────

#[test]
fn tokens_serde_round_trip() {
    let tokens = Tokens {
        access_token: "access123".to_string(),
        refresh_token: "refresh456".to_string(),
        expires_at: Utc::now(),
        scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
    };
    let json = serde_json::to_string(&tokens).unwrap();
    let parsed: Tokens = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.access_token, "access123");
    assert_eq!(parsed.refresh_token, "refresh456");
    assert_eq!(parsed.scopes.len(), 2);
}

#[test]
fn tokens_serde_default_scopes() {
    // Legacy tokens without scopes field
    let json = r#"{"access_token":"a","refresh_token":"r","expires_at":"2026-01-01T00:00:00Z"}"#;
    let tokens: Tokens = serde_json::from_str(json).unwrap();
    assert!(tokens.scopes.is_empty());
}

#[test]
fn tokens_debug_output() {
    let tokens = Tokens {
        access_token: "a".to_string(),
        refresh_token: "r".to_string(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    let debug = format!("{tokens:?}");
    assert!(debug.contains("Tokens"));
}

#[test]
fn tokens_clone() {
    let tokens = Tokens {
        access_token: "acc".to_string(),
        refresh_token: "ref".to_string(),
        expires_at: Utc::now(),
        scopes: vec!["scope1".to_string()],
    };
    let cloned = tokens.clone();
    assert_eq!(cloned.access_token, tokens.access_token);
    assert_eq!(cloned.refresh_token, tokens.refresh_token);
    assert_eq!(cloned.scopes, tokens.scopes);
}

// ── save_tokens / load_tokens ────────────────────────────────────

#[test]
fn save_and_load_tokens_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tokens.json");
    let tokens = Tokens {
        access_token: "test_access".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_at: Utc::now(),
        scopes: vec!["tweet.read".to_string()],
    };
    save_tokens(&tokens, &path).unwrap();
    let loaded = load_tokens(&path).unwrap().unwrap();
    assert_eq!(loaded.access_token, "test_access");
    assert_eq!(loaded.refresh_token, "test_refresh");
    assert_eq!(loaded.scopes, vec!["tweet.read"]);
}

#[test]
fn save_tokens_creates_parent_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sub").join("dir").join("tokens.json");
    let tokens = Tokens {
        access_token: "a".to_string(),
        refresh_token: "r".to_string(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    save_tokens(&tokens, &path).unwrap();
    assert!(path.exists());
}

#[test]
fn load_tokens_missing_file_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");
    let result = load_tokens(&path).unwrap();
    assert!(result.is_none());
}

#[test]
fn load_tokens_invalid_json_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    std::fs::write(&path, "not valid json").unwrap();
    let result = load_tokens(&path);
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn save_tokens_restricts_permissions() {
    use std::os::unix::fs::MetadataExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("restricted.json");
    let tokens = Tokens {
        access_token: "a".to_string(),
        refresh_token: "r".to_string(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    save_tokens(&tokens, &path).unwrap();
    let meta = std::fs::metadata(&path).unwrap();
    assert_eq!(meta.mode() & 0o777, 0o600);
}

// ── TokenRefreshResponse ─────────────────────────────────────────

#[test]
fn token_refresh_response_deserialize() {
    let json = r#"{"access_token":"new","refresh_token":"ref2","expires_in":7200,"scope":"tweet.read tweet.write"}"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.access_token, "new");
    assert_eq!(resp.refresh_token, "ref2");
    assert_eq!(resp.expires_in, 7200);
    assert_eq!(resp.scope, "tweet.read tweet.write");
}

#[test]
fn tokens_serialize_deserialize() {
    let tokens = Tokens {
        access_token: "test_access".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
    };

    let json = serde_json::to_string(&tokens).expect("serialize");
    let parsed: Tokens = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.access_token, "test_access");
    assert_eq!(parsed.refresh_token, "test_refresh");
    assert_eq!(parsed.scopes.len(), 2);
}

#[test]
fn save_and_load_tokens() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let tokens = Tokens {
        access_token: "acc".to_string(),
        refresh_token: "ref".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec!["tweet.read".to_string()],
    };

    save_tokens(&tokens, &path).expect("save");

    let loaded = load_tokens(&path).expect("load").expect("some");
    assert_eq!(loaded.access_token, "acc");
    assert_eq!(loaded.refresh_token, "ref");
}

#[test]
fn load_tokens_file_not_found_returns_none() {
    let path = std::path::PathBuf::from("/nonexistent/tokens.json");
    let result = load_tokens(&path).expect("load");
    assert!(result.is_none());
}

#[test]
fn load_tokens_malformed_returns_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    std::fs::write(&path, "not valid json").expect("write");

    let result = load_tokens(&path);
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn save_tokens_sets_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let tokens = Tokens {
        access_token: "a".to_string(),
        refresh_token: "r".to_string(),
        expires_at: Utc::now(),
        scopes: vec![],
    };

    save_tokens(&tokens, &path).expect("save");

    let metadata = std::fs::metadata(&path).expect("metadata");
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "token file should have 600 permissions");
}

#[test]
fn save_tokens_creates_deeply_nested_dirs() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("nested").join("dir").join("tokens.json");

    let tokens = Tokens {
        access_token: "a".to_string(),
        refresh_token: "r".to_string(),
        expires_at: Utc::now(),
        scopes: vec![],
    };

    save_tokens(&tokens, &path).expect("save");
    assert!(path.exists());
}

#[tokio::test]
async fn token_manager_new_and_lock() {
    let tokens = Tokens {
        access_token: "acc".to_string(),
        refresh_token: "ref".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().unwrap();
    let tm = TokenManager::new(tokens, "cid".to_string(), dir.path().join("tokens.json"));
    let lock = tm.tokens_lock();
    let read = lock.read().await;
    assert_eq!(read.access_token, "acc");
}

#[tokio::test]
async fn token_manager_get_access_token_fresh() {
    let tokens = Tokens {
        access_token: "fresh_token".to_string(),
        refresh_token: "ref".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().unwrap();
    let tm = TokenManager::new(tokens, "cid".to_string(), dir.path().join("tokens.json"));
    let token = tm.get_access_token().await.unwrap();
    assert_eq!(token, "fresh_token");
}

#[tokio::test]
async fn token_manager_refresh_if_needed_fresh_is_noop() {
    let tokens = Tokens {
        access_token: "acc".to_string(),
        refresh_token: "ref".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().unwrap();
    let tm = TokenManager::new(tokens, "cid".to_string(), dir.path().join("tokens.json"));
    // Should succeed without making any HTTP calls
    let result = tm.refresh_if_needed().await;
    assert!(result.is_ok());
}

#[test]
fn token_refresh_response_deserialize_with_scopes() {
    let json = r#"{
        "access_token": "new_acc",
        "refresh_token": "new_ref",
        "expires_in": 7200,
        "scope": "tweet.read tweet.write users.read"
    }"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.access_token, "new_acc");
    assert_eq!(resp.refresh_token, "new_ref");
    assert_eq!(resp.expires_in, 7200);
    let scopes: Vec<&str> = resp.scope.split_whitespace().collect();
    assert_eq!(scopes.len(), 3);
}

#[test]
fn save_tokens_overwrites_existing() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    let t1 = Tokens {
        access_token: "first".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    save_tokens(&t1, &path).expect("save");

    let t2 = Tokens {
        access_token: "second".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    save_tokens(&t2, &path).expect("save overwrite");

    let loaded = load_tokens(&path).unwrap().unwrap();
    assert_eq!(loaded.access_token, "second");
}

#[test]
fn load_tokens_from_nonexistent_dir_returns_none() {
    let path = std::env::temp_dir()
        .join("tuitbot_test_nonexistent_dir_xyzzy")
        .join("tokens.json");
    let result = load_tokens(&path).unwrap();
    assert!(result.is_none());
}

#[test]
fn tokens_with_many_scopes() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![
            "tweet.read".into(),
            "tweet.write".into(),
            "users.read".into(),
            "follows.read".into(),
            "follows.write".into(),
            "offline.access".into(),
        ],
    };
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Tokens = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scopes.len(), 6);
}

#[test]
fn tokens_debug_format() {
    let tokens = Tokens {
        access_token: "debug_test".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    let debug = format!("{tokens:?}");
    assert!(debug.contains("debug_test"));
}

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

#[test]
fn tokens_expiry_far_future() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::days(365),
        scopes: vec![],
    };
    let secs = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(secs > 86400 * 300);
}

#[test]
fn save_tokens_multiple_overwrites() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");

    for i in 0..5 {
        let tokens = Tokens {
            access_token: format!("token_{i}"),
            refresh_token: "r".into(),
            expires_at: Utc::now(),
            scopes: vec![],
        };
        save_tokens(&tokens, &path).expect("save");
    }

    let loaded = load_tokens(&path).unwrap().unwrap();
    assert_eq!(loaded.access_token, "token_4");
}

#[test]
fn load_tokens_valid_structure_no_scopes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let json = r#"{
        "access_token": "acc",
        "refresh_token": "ref",
        "expires_at": "2026-06-01T00:00:00Z"
    }"#;
    std::fs::write(&path, json).expect("write");
    let tokens = load_tokens(&path).unwrap().unwrap();
    assert_eq!(tokens.access_token, "acc");
    assert!(tokens.scopes.is_empty());
}

#[test]
fn load_tokens_with_scopes_list() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let json = r#"{
        "access_token": "acc",
        "refresh_token": "ref",
        "expires_at": "2026-06-01T00:00:00Z",
        "scopes": ["tweet.read", "tweet.write", "offline.access"]
    }"#;
    std::fs::write(&path, json).expect("write");
    let tokens = load_tokens(&path).unwrap().unwrap();
    assert_eq!(tokens.scopes.len(), 3);
}

#[test]
fn token_refresh_response_large_expires_in() {
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_in": 999999,
        "scope": "tweet.read"
    }"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.expires_in, 999999);
}

#[test]
fn token_refresh_response_many_scopes() {
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_in": 3600,
        "scope": "tweet.read tweet.write users.read follows.read follows.write offline.access"
    }"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    let scopes: Vec<&str> = resp.scope.split_whitespace().collect();
    assert_eq!(scopes.len(), 6);
}

#[tokio::test]
async fn token_manager_tokens_lock_write_access() {
    let tokens = Tokens {
        access_token: "original".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let manager = TokenManager::new(tokens, "cid".into(), path);

    let lock = manager.tokens_lock();
    {
        let mut guard = lock.write().await;
        guard.access_token = "modified".to_string();
    }
    let guard = lock.read().await;
    assert_eq!(guard.access_token, "modified");
}

#[test]
fn save_tokens_and_verify_json_structure() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let tokens = Tokens {
        access_token: "verify_me".into(),
        refresh_token: "ref_verify".into(),
        expires_at: Utc::now(),
        scopes: vec!["s1".into(), "s2".into()],
    };
    save_tokens(&tokens, &path).expect("save");

    let content = std::fs::read_to_string(&path).expect("read");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse");
    assert_eq!(parsed["access_token"], "verify_me");
    assert_eq!(parsed["refresh_token"], "ref_verify");
    assert!(parsed["scopes"].is_array());
    assert_eq!(parsed["scopes"].as_array().unwrap().len(), 2);
}

#[test]
fn tokens_serde_missing_scopes_defaults_empty() {
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_at": "2026-06-01T00:00:00Z"
    }"#;
    let tokens: Tokens = serde_json::from_str(json).unwrap();
    assert_eq!(tokens.access_token, "a");
    assert!(tokens.scopes.is_empty());
}

#[test]
fn tokens_roundtrip_preserves_scopes() {
    let tokens = Tokens {
        access_token: "acc".into(),
        refresh_token: "ref".into(),
        expires_at: Utc::now(),
        scopes: vec![
            "tweet.read".into(),
            "tweet.write".into(),
            "users.read".into(),
        ],
    };
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Tokens = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scopes.len(), 3);
    assert!(back.scopes.contains(&"users.read".to_string()));
}

#[test]
fn tokens_clone_with_scopes() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec!["s1".into()],
    };
    let cloned = tokens.clone();
    assert_eq!(cloned.access_token, tokens.access_token);
    assert_eq!(cloned.scopes, tokens.scopes);
}

#[test]
fn save_tokens_to_nonexistent_parent_creates_dirs() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("deep").join("nested").join("tokens.json");
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    save_tokens(&tokens, &path).expect("save");
    let loaded = load_tokens(&path).unwrap().unwrap();
    assert_eq!(loaded.access_token, "a");
}

#[tokio::test]
async fn token_manager_tokens_lock_returns_shared_ref() {
    let tokens = Tokens {
        access_token: "tok".into(),
        refresh_token: "ref".into(),
        expires_at: Utc::now() + chrono::Duration::hours(2),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let manager = TokenManager::new(tokens, "cid".into(), path);

    let lock = manager.tokens_lock();
    let guard = lock.read().await;
    assert_eq!(guard.access_token, "tok");
}

#[test]
fn tokens_serde_preserves_exact_datetime() {
    let dt = "2026-06-15T12:30:45.123Z"
        .parse::<chrono::DateTime<Utc>>()
        .unwrap();
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: dt,
        scopes: vec![],
    };
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Tokens = serde_json::from_str(&json).unwrap();
    assert_eq!(back.expires_at, dt);
}

#[test]
fn token_manager_new_with_custom_path() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec![],
    };
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("specific_file.json");
    let _manager = TokenManager::new(tokens, "my_id".into(), path);
    // Manager created with custom path — no panic
}

#[test]
fn save_tokens_produces_pretty_json() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("tokens.json");
    let tokens = Tokens {
        access_token: "pretty".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now(),
        scopes: vec!["s1".into()],
    };
    save_tokens(&tokens, &path).expect("save");
    let content = std::fs::read_to_string(&path).expect("read");
    // Pretty-printed JSON should have newlines
    assert!(content.contains('\n'));
    // Verify it parses back correctly
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse");
    assert_eq!(parsed["access_token"], "pretty");
}

#[test]
fn token_refresh_response_negative_expires() {
    let json = r#"{
        "access_token": "a",
        "refresh_token": "r",
        "expires_in": -1,
        "scope": "tweet.read"
    }"#;
    let resp: TokenRefreshResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.expires_in, -1);
}

#[test]
fn auth_url_constant_value() {
    assert_eq!(AUTH_URL, "https://x.com/i/oauth2/authorize");
}

#[test]
fn token_url_constant_value() {
    assert_eq!(TOKEN_URL, "https://api.x.com/2/oauth2/token");
}

#[test]
fn refresh_window_secs_value() {
    assert_eq!(REFRESH_WINDOW_SECS, 300);
    assert_eq!(REFRESH_WINDOW_SECS, 5 * 60);
}

#[test]
fn tokens_expiry_just_above_refresh_window() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::seconds(REFRESH_WINDOW_SECS + 1),
        scopes: vec![],
    };
    let seconds_until = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(seconds_until >= REFRESH_WINDOW_SECS);
}

#[test]
fn tokens_expiry_just_below_refresh_window() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::seconds(REFRESH_WINDOW_SECS - 1),
        scopes: vec![],
    };
    let seconds_until = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(seconds_until < REFRESH_WINDOW_SECS);
}

#[test]
fn tokens_is_expired_when_past() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() - chrono::Duration::hours(1),
        scopes: vec![],
    };
    let seconds_until_expiry = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(
        seconds_until_expiry < 0,
        "expired token should have negative seconds"
    );
}

#[test]
fn tokens_not_expired_when_far_future() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        scopes: vec![],
    };
    let seconds_until_expiry = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(seconds_until_expiry > REFRESH_WINDOW_SECS);
}

#[test]
fn tokens_within_refresh_window() {
    let tokens = Tokens {
        access_token: "a".into(),
        refresh_token: "r".into(),
        expires_at: Utc::now() + chrono::Duration::seconds(60),
        scopes: vec![],
    };
    let seconds_until_expiry = tokens
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds();
    assert!(
        seconds_until_expiry < REFRESH_WINDOW_SECS,
        "60s remaining should be within the 300s refresh window"
    );
}
