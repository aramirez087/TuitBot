//! Tests: Tokens struct, serde, save_tokens, load_tokens, permissions.
use super::super::*;

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
