use super::*;

// ============================================================
// X Auth: Unlink
// ============================================================

#[tokio::test]
async fn x_auth_unlink_removes_tokens() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "Unlink Test").await;

    // Write a mock token file.
    let token_path = tuitbot_core::storage::accounts::account_token_path(dir.path(), &acct);
    let tokens = tuitbot_core::x_api::auth::Tokens {
        access_token: "test_access".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_at: chrono::Utc::now() + chrono::TimeDelta::hours(2),
        scopes: vec!["tweet.read".to_string()],
    };
    tuitbot_core::x_api::auth::save_tokens(&tokens, &token_path).expect("save tokens");
    assert!(token_path.exists(), "token file should exist before unlink");

    // Verify status shows linked.
    let (status, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/status"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["oauth_linked"], true);

    // Unlink.
    let (status, body) = delete_json_for(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/tokens"),
        &acct,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "unlink: {body}");
    assert_eq!(body["deleted"], true);
    assert!(!token_path.exists(), "token file should be deleted");

    // Verify status shows unlinked.
    let (status, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/status"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["oauth_linked"], false);
    assert_eq!(body["has_credentials"], false);
}

/// Test: unlinking when no tokens exist returns deleted: false (no error).
#[tokio::test]
async fn x_auth_unlink_no_tokens_returns_false() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "Unlink Empty").await;

    let (status, body) = delete_json_for(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/tokens"),
        &acct,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["deleted"], false);
}

/// Test: unlinking account A's OAuth does not affect account B's scraper session.
#[tokio::test]
async fn x_auth_unlink_cross_account_isolation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "Account A").await;
    let acct_b = create_test_account(&pool, "Account B").await;

    // Give A OAuth tokens.
    let token_path = tuitbot_core::storage::accounts::account_token_path(dir.path(), &acct_a);
    let tokens = tuitbot_core::x_api::auth::Tokens {
        access_token: "a_access".to_string(),
        refresh_token: "a_refresh".to_string(),
        expires_at: chrono::Utc::now() + chrono::TimeDelta::hours(2),
        scopes: vec![],
    };
    tuitbot_core::x_api::auth::save_tokens(&tokens, &token_path).expect("save A tokens");

    // Give B a scraper session.
    let (status, _) = post_json_for(
        router.clone(),
        "/api/settings/scraper-session",
        &acct_b,
        serde_json::json!({
            "auth_token": "b_auth",
            "ct0": "b_ct0",
            "username": "user_b"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Unlink A's OAuth.
    let (status, body) = delete_json_for(
        router.clone(),
        &format!("/api/accounts/{acct_a}/x-auth/tokens"),
        &acct_a,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["deleted"], true);

    // A should have no credentials.
    let (_, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct_a}/x-auth/status"),
    )
    .await;
    assert_eq!(body["oauth_linked"], false);
    assert_eq!(body["has_credentials"], false);

    // B should still have scraper session.
    let (_, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct_b}/x-auth/status"),
    )
    .await;
    assert_eq!(body["scraper_linked"], true);
    assert_eq!(body["has_credentials"], true);
}
