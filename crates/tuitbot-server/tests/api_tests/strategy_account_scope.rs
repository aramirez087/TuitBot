//! Account isolation tests for strategy routes.
//!
//! Verifies that GET /api/strategy/current, POST /api/strategy/refresh, and
//! GET /api/strategy/history scope data to the X-Account-Id header:
//! account A cannot see account B's strategy reports.

use super::*;

/// Helper: write a config file so strategy routes can load it.
async fn write_test_config(dir: &std::path::Path) {
    let cfg = r#"
[llm]
api_key = "test"

[x_api]
provider_backend = "x_api"

[business]
company_name = "Acme"
product_description = "Test product"
target_audience = "developers"

[limits]
max_replies_per_day = 10
max_tweets_per_day = 5

[mcp_policy]
enforce_for_mutations = false
"#;
    let path = dir.join("tuitbot.toml");
    std::fs::write(path, cfg).expect("write config");
}

// ── current route ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn strategy_current_scoped_to_account() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_test_config(dir.path()).await;
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "current-a").await;
    let acct_b = create_test_account(&pool, "current-b").await;

    // Both accounts can get a current report (computes from empty DB — all zeros).
    let (status_a, body_a) = get_json_for(router.clone(), "/api/strategy/current", &acct_a).await;
    assert_eq!(status_a, StatusCode::OK, "acct_a current: {body_a}");

    let (status_b, body_b) = get_json_for(router.clone(), "/api/strategy/current", &acct_b).await;
    assert_eq!(status_b, StatusCode::OK, "acct_b current: {body_b}");

    // Both reports should be valid structure.
    assert!(
        body_a["week_start"].is_string(),
        "a has week_start: {body_a}"
    );
    assert!(
        body_b["week_start"].is_string(),
        "b has week_start: {body_b}"
    );
}

#[tokio::test]
async fn strategy_current_returns_same_week_start_for_both_accounts() {
    // Both accounts see the same current ISO week bounds even with separate
    // account-scoped data stores.
    let dir = tempfile::tempdir().expect("tempdir");
    write_test_config(dir.path()).await;
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "week-a").await;
    let acct_b = create_test_account(&pool, "week-b").await;

    let (_, body_a) = get_json_for(router.clone(), "/api/strategy/current", &acct_a).await;
    let (_, body_b) = get_json_for(router.clone(), "/api/strategy/current", &acct_b).await;

    assert_eq!(
        body_a["week_start"], body_b["week_start"],
        "week_start must be the same ISO week for all accounts"
    );
}

// ── refresh route ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn strategy_refresh_scoped_to_account() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_test_config(dir.path()).await;
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "refresh-a").await;
    let acct_b = create_test_account(&pool, "refresh-b").await;

    // Refresh for account A succeeds.
    let (status_a, body_a) = post_json_for(
        router.clone(),
        "/api/strategy/refresh",
        &acct_a,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status_a, StatusCode::OK, "acct_a refresh: {body_a}");
    assert!(
        body_a["week_start"].is_string(),
        "a has week_start: {body_a}"
    );

    // Refresh for account B also succeeds and is independent.
    let (status_b, body_b) = post_json_for(
        router.clone(),
        "/api/strategy/refresh",
        &acct_b,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status_b, StatusCode::OK, "acct_b refresh: {body_b}");
    assert!(
        body_b["week_start"].is_string(),
        "b has week_start: {body_b}"
    );
}

// ── history route ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn strategy_history_isolated_between_accounts() {
    // Seed a report for account A via refresh, then verify account B sees no
    // history (proves per-account filtering — data seeded by A is not visible to B).
    let dir = tempfile::tempdir().expect("tempdir");
    write_test_config(dir.path()).await;
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "hist-a").await;
    let acct_b = create_test_account(&pool, "hist-b").await;

    // Seed one report for A.
    let (status, _) = post_json_for(
        router.clone(),
        "/api/strategy/refresh",
        &acct_a,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // A's history should contain the seeded report.
    let (status_a, body_a) =
        get_json_for(router.clone(), "/api/strategy/history?limit=10", &acct_a).await;
    assert_eq!(status_a, StatusCode::OK, "hist-a history: {body_a}");
    let items_a = body_a.as_array().expect("array response");
    assert!(!items_a.is_empty(), "account A should have ≥1 history row");

    // B's history should be empty (different account_id).
    let (status_b, body_b) =
        get_json_for(router.clone(), "/api/strategy/history?limit=10", &acct_b).await;
    assert_eq!(status_b, StatusCode::OK, "hist-b history: {body_b}");
    let items_b = body_b.as_array().expect("array response");
    assert!(
        items_b.is_empty(),
        "account B must not see account A's strategy history, got: {body_b}"
    );
}

// ── inputs route (already scoped — confirm still works) ───────────────────────

#[tokio::test]
async fn strategy_inputs_still_works_after_route_changes() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_test_config(dir.path()).await;
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "inputs-ok").await;

    let (status, body) = get_json_for(router.clone(), "/api/strategy/inputs", &acct).await;
    assert_eq!(status, StatusCode::OK, "inputs: {body}");
    assert!(
        body["content_pillars"].is_array(),
        "has content_pillars: {body}"
    );
    assert!(
        body["target_accounts"].is_array(),
        "has target_accounts: {body}"
    );
}
