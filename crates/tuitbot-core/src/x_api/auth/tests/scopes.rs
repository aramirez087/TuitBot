//! Tests: scopes, TokenRefreshResponse, expiry logic.

use super::super::*;
use chrono::Datelike;

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
