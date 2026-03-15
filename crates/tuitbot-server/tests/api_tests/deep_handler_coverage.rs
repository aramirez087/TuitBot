//! Deep server integration tests that exercise happy paths through
//! large handler functions. Each test creates a real config via
//! `test_router_with_dir`, seeds the database, and asserts on 200
//! responses from the full handler logic.

use axum::http::StatusCode;
use serde_json::json;
use tuitbot_core::storage::{accounts::DEFAULT_ACCOUNT_ID, action_log, approval_queue, tweets};

use super::*;

// ============================================================
// Settings deep tests (settings.rs)
// ============================================================

#[tokio::test]
async fn init_then_patch_business_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let _ = std::fs::remove_file(dir.path().join("config.toml"));
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    // Config already exists from test_router_with_dir, so use PATCH.
    let (status, body) = patch_json(
        router.clone(),
        "/api/settings",
        json!({ "business": { "product_name": "PatchedName" } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "patch business name: {body}");

    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK, "get after patch: {body}");
    assert_eq!(
        body["business"]["product_name"], "PatchedName",
        "name should be updated: {body}"
    );
}

#[tokio::test]
async fn init_then_get_settings_returns_all_sections() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK, "get settings: {body}");
    assert!(body.is_object(), "expected object: {body}");
    assert!(body.get("business").is_some(), "missing business: {body}");
    assert!(body.get("scoring").is_some(), "missing scoring: {body}");
    assert!(body.get("limits").is_some(), "missing limits: {body}");
    assert!(body.get("schedule").is_some(), "missing schedule: {body}");
    assert!(body.get("llm").is_some(), "missing llm: {body}");
}

#[tokio::test]
async fn patch_scoring_threshold() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = patch_json(
        router.clone(),
        "/api/settings",
        json!({ "scoring": { "threshold": 80 } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "patch scoring: {body}");

    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["scoring"]["threshold"], 80, "threshold: {body}");
}

#[tokio::test]
async fn patch_limits_max_replies() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = patch_json(
        router.clone(),
        "/api/settings",
        json!({ "limits": { "max_replies_per_day": 10 } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "patch limits: {body}");

    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["limits"]["max_replies_per_day"], 10,
        "max replies: {body}"
    );
}

#[tokio::test]
async fn patch_schedule_timezone() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = patch_json(
        router.clone(),
        "/api/settings",
        json!({ "schedule": { "timezone": "America/New_York" } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "patch schedule: {body}");

    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["schedule"]["timezone"], "America/New_York",
        "timezone: {body}"
    );
}

#[tokio::test]
async fn get_settings_capabilities_after_init() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = get_json(router, "/api/settings/status").await;
    assert_eq!(status, StatusCode::OK, "status: {body}");
    assert_eq!(body["configured"], true, "configured: {body}");
    assert!(
        body.get("capabilities").is_some(),
        "missing capabilities: {body}"
    );
    assert!(
        body.get("deployment_mode").is_some(),
        "missing deployment_mode: {body}"
    );
}

#[tokio::test]
async fn validate_after_init() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/settings/validate",
        json!({ "business": { "product_name": "StillValid" } }),
    )
    .await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "validate: {code}: {body}");
    if code == 200 {
        assert!(body.get("valid").is_some(), "expected valid key: {body}");
    }
}

// ============================================================
// Approval deep tests (approval.rs)
// ============================================================

#[tokio::test]
async fn seed_and_list_approval_items() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    // Seed two approval items.
    approval_queue::enqueue_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "reply",
        "tweet_1",
        "alice",
        "Great post!",
        "tech",
        "helpful",
        85.0,
        "[]",
    )
    .await
    .expect("enqueue 1");
    approval_queue::enqueue_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "tweet",
        "tweet_2",
        "bob",
        "Check this out",
        "news",
        "witty",
        70.0,
        "[]",
    )
    .await
    .expect("enqueue 2");

    let (status, body) = get_json(router, "/api/approval?status=pending").await;
    assert_eq!(status, StatusCode::OK, "list approval: {body}");
    let items = body.as_array().expect("expected array");
    assert_eq!(items.len(), 2, "should have 2 pending items: {body}");
    assert_eq!(items[0]["target_author"], "alice");
    assert_eq!(items[1]["target_author"], "bob");
}

#[tokio::test]
async fn seed_and_reject_item() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let id = approval_queue::enqueue_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "reply",
        "t1",
        "carol",
        "Nice!",
        "topic",
        "arch",
        60.0,
        "[]",
    )
    .await
    .expect("enqueue");

    let (status, body) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/reject"),
        json!({ "actor": "reviewer", "notes": "not relevant" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reject: {body}");
    assert_eq!(body["status"], "rejected", "status: {body}");

    // Verify via storage.
    let item = approval_queue::get_by_id_for(&pool, DEFAULT_ACCOUNT_ID, id)
        .await
        .expect("get")
        .expect("item");
    assert_eq!(item.status, "rejected");
}

#[tokio::test]
async fn seed_and_edit_item() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let id = approval_queue::enqueue_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "reply",
        "t1",
        "dave",
        "Original content",
        "topic",
        "arch",
        75.0,
        "[]",
    )
    .await
    .expect("enqueue");

    let (status, body) = patch_json(
        router,
        &format!("/api/approval/{id}"),
        json!({ "content": "Updated content", "editor": "tester" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "edit: {body}");
    assert_eq!(
        body["generated_content"], "Updated content",
        "content: {body}"
    );
}

#[tokio::test]
async fn seed_approval_stats() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    // Seed a mix of items.
    for i in 0..3 {
        approval_queue::enqueue_for(
            &pool,
            DEFAULT_ACCOUNT_ID,
            "reply",
            &format!("t_pending_{i}"),
            "author",
            "content",
            "topic",
            "arch",
            50.0,
            "[]",
        )
        .await
        .expect("enqueue pending");
    }
    let reject_id = approval_queue::enqueue_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "reply",
        "t_reject",
        "author",
        "content",
        "topic",
        "arch",
        50.0,
        "[]",
    )
    .await
    .expect("enqueue to-reject");
    approval_queue::update_status_for(&pool, DEFAULT_ACCOUNT_ID, reject_id, "rejected")
        .await
        .expect("reject");

    let (status, body) = get_json(router, "/api/approval/stats").await;
    assert_eq!(status, StatusCode::OK, "stats: {body}");
    assert_eq!(body["pending"], 3, "pending count: {body}");
    assert_eq!(body["rejected"], 1, "rejected count: {body}");
}

// ============================================================
// Content / Draft Studio deep tests (draft_studio.rs)
// ============================================================

#[tokio::test]
async fn create_draft_and_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router.clone(),
        "/api/drafts",
        json!({
            "content_type": "tweet",
            "content": "My first draft",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create draft: {body}");
    let draft_id = body["id"].as_i64().expect("id");
    assert!(draft_id > 0, "draft id should be positive");

    let (status, body) = get_json(router, "/api/drafts").await;
    assert_eq!(status, StatusCode::OK, "list drafts: {body}");
    let drafts = body.as_array().expect("array");
    assert!(!drafts.is_empty(), "should have at least 1 draft");
}

#[tokio::test]
async fn create_draft_and_schedule() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    // Create a draft.
    let (status, body) = post_json(
        router.clone(),
        "/api/drafts",
        json!({
            "content_type": "tweet",
            "content": "Scheduled draft",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create: {body}");
    let draft_id = body["id"].as_i64().expect("id");

    // Schedule it for the future.
    let future_time = "2099-12-31T23:59:00Z";
    let (status, body) = post_json(
        router,
        &format!("/api/drafts/{draft_id}/schedule"),
        json!({ "scheduled_for": future_time }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "schedule: {body}");
    assert_eq!(body["status"], "scheduled", "status: {body}");
}

#[tokio::test]
async fn create_draft_autosave_and_get() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    // Create draft.
    let (status, create_body) = post_json(
        router.clone(),
        "/api/drafts",
        json!({
            "content_type": "tweet",
            "content": "Initial content",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create: {create_body}");
    let draft_id = create_body["id"].as_i64().expect("id");
    let updated_at = create_body["updated_at"]
        .as_str()
        .expect("updated_at")
        .to_string();

    // Autosave with updated content.
    let (status, body) = patch_json(
        router.clone(),
        &format!("/api/drafts/{draft_id}"),
        json!({
            "content": "Autosaved content",
            "content_type": "tweet",
            "updated_at": updated_at
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "autosave: {body}");

    // Get and verify updated content.
    let (status, body) = get_json(router, &format!("/api/drafts/{draft_id}")).await;
    assert_eq!(status, StatusCode::OK, "get: {body}");
    assert_eq!(body["content"], "Autosaved content", "content: {body}");
}

#[tokio::test]
async fn create_draft_archive_and_restore() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    // Create draft.
    let (status, body) = post_json(
        router.clone(),
        "/api/drafts",
        json!({
            "content_type": "tweet",
            "content": "Archive me",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create: {body}");
    let draft_id = body["id"].as_i64().expect("id");

    // Archive.
    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{draft_id}/archive"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "archive: {body}");

    // Non-archived list should be empty.
    let (status, body) = get_json(router.clone(), "/api/drafts").await;
    assert_eq!(status, StatusCode::OK, "list: {body}");
    let drafts = body.as_array().expect("array");
    assert!(
        drafts.is_empty(),
        "archived draft should not appear in list: {body}"
    );

    // Restore.
    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{draft_id}/restore"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "restore: {body}");

    // Should be back in list.
    let (status, body) = get_json(router, "/api/drafts").await;
    assert_eq!(status, StatusCode::OK, "list after restore: {body}");
    let drafts = body.as_array().expect("array");
    assert_eq!(drafts.len(), 1, "restored draft should be in list: {body}");
}

#[tokio::test]
async fn create_draft_duplicate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    // Create draft.
    let (status, body) = post_json(
        router.clone(),
        "/api/drafts",
        json!({
            "content_type": "tweet",
            "content": "Original draft",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create: {body}");
    let draft_id = body["id"].as_i64().expect("id");

    // Duplicate.
    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{draft_id}/duplicate"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "duplicate: {body}");
    let new_id = body["id"].as_i64().expect("new id");
    assert_ne!(new_id, draft_id, "duplicate should have different id");

    // List should have 2 drafts.
    let (status, body) = get_json(router, "/api/drafts").await;
    assert_eq!(status, StatusCode::OK, "list: {body}");
    let drafts = body.as_array().expect("array");
    assert_eq!(drafts.len(), 2, "should have 2 drafts: {body}");
}

// ============================================================
// Activity deep tests (activity.rs)
// ============================================================

#[tokio::test]
async fn log_action_and_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    action_log::log_action_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "search",
        "success",
        Some("Found 5 tweets"),
        None,
    )
    .await
    .expect("log");

    let (status, body) = get_json(router, "/api/activity").await;
    assert_eq!(status, StatusCode::OK, "list activity: {body}");
    assert!(body["actions"].is_array(), "actions is array: {body}");
    let actions = body["actions"].as_array().expect("array");
    assert_eq!(actions.len(), 1, "should have 1 action: {body}");
    assert_eq!(actions[0]["action_type"], "search");
    assert_eq!(body["total"], 1, "total count: {body}");
}

#[tokio::test]
async fn log_multiple_actions_filter_by_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    action_log::log_action_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "search",
        "success",
        Some("s1"),
        None,
    )
    .await
    .expect("log");
    action_log::log_action_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "reply",
        "success",
        Some("r1"),
        None,
    )
    .await
    .expect("log");
    action_log::log_action_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "search",
        "failure",
        Some("s2"),
        None,
    )
    .await
    .expect("log");

    // Filter by type=search.
    let (status, body) = get_json(router.clone(), "/api/activity?type=search").await;
    assert_eq!(status, StatusCode::OK, "filter search: {body}");
    let actions = body["actions"].as_array().expect("array");
    assert_eq!(actions.len(), 2, "should have 2 search actions: {body}");

    // Filter by type=reply.
    let (status, body) = get_json(router, "/api/activity?type=reply").await;
    assert_eq!(status, StatusCode::OK, "filter reply: {body}");
    let actions = body["actions"].as_array().expect("array");
    assert_eq!(actions.len(), 1, "should have 1 reply action: {body}");
}

// ============================================================
// Discovery deep tests (discovery.rs)
// ============================================================

#[tokio::test]
async fn seed_tweets_and_get_discovery_feed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    // Seed discovered tweets.
    let tweet1 = tweets::DiscoveredTweet {
        id: "disc_001".to_string(),
        author_id: "user_1".to_string(),
        author_username: "techguru".to_string(),
        content: "Rust is amazing for systems programming".to_string(),
        like_count: 42,
        retweet_count: 10,
        reply_count: 5,
        impression_count: Some(1000),
        relevance_score: Some(90.0),
        matched_keyword: Some("rust".to_string()),
        discovered_at: "2025-01-15T10:00:00Z".to_string(),
        replied_to: 0,
    };
    let tweet2 = tweets::DiscoveredTweet {
        id: "disc_002".to_string(),
        author_id: "user_2".to_string(),
        author_username: "devnews".to_string(),
        content: "New programming paradigm released".to_string(),
        like_count: 100,
        retweet_count: 30,
        reply_count: 15,
        impression_count: Some(5000),
        relevance_score: Some(75.0),
        matched_keyword: Some("programming".to_string()),
        discovered_at: "2025-01-15T11:00:00Z".to_string(),
        replied_to: 0,
    };

    tweets::insert_discovered_tweet_for(&pool, DEFAULT_ACCOUNT_ID, &tweet1)
        .await
        .expect("insert tweet 1");
    tweets::insert_discovered_tweet_for(&pool, DEFAULT_ACCOUNT_ID, &tweet2)
        .await
        .expect("insert tweet 2");

    let (status, body) = get_json(router, "/api/discovery/feed?min_score=50").await;
    assert_eq!(status, StatusCode::OK, "discovery feed: {body}");
    let feed = body.as_array().expect("expected array");
    assert_eq!(feed.len(), 2, "should have 2 tweets: {body}");
    // Feed is ordered by discovered_at DESC
    assert_eq!(feed[0]["author_username"], "devnews");
    assert_eq!(feed[1]["author_username"], "techguru");
}

#[tokio::test]
async fn seed_tweets_and_get_discovery_keywords() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let tweet = tweets::DiscoveredTweet {
        id: "kw_001".to_string(),
        author_id: "u1".to_string(),
        author_username: "user1".to_string(),
        content: "test".to_string(),
        like_count: 0,
        retweet_count: 0,
        reply_count: 0,
        impression_count: None,
        relevance_score: Some(80.0),
        matched_keyword: Some("blockchain".to_string()),
        discovered_at: "2025-01-15T10:00:00Z".to_string(),
        replied_to: 0,
    };
    tweets::insert_discovered_tweet_for(&pool, DEFAULT_ACCOUNT_ID, &tweet)
        .await
        .expect("insert");

    let (status, body) = get_json(router, "/api/discovery/keywords").await;
    assert_eq!(status, StatusCode::OK, "keywords: {body}");
    let kws = body.as_array().expect("array");
    assert!(kws.contains(&json!("blockchain")), "keywords: {body}");
}
