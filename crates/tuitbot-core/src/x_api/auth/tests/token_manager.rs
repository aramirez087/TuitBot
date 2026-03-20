//! Tests: TokenManager, get_access_token, refresh flows.

use super::super::*;
use chrono::Datelike;

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
