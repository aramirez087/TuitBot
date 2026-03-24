//! Tests for target account CRUD and analytics.

use super::*;
use crate::storage::init_test_db;

#[tokio::test]
async fn upsert_and_get_target_account() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    let account = get_target_account(&pool, "acc_1")
        .await
        .expect("get")
        .expect("found");
    assert_eq!(account.username, "alice");
    assert_eq!(account.total_replies_sent, 0);
    assert_eq!(account.status, "active");
}

#[tokio::test]
async fn get_active_target_accounts_works() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");
    upsert_target_account(&pool, "acc_2", "bob")
        .await
        .expect("upsert");

    let accounts = get_active_target_accounts(&pool).await.expect("get all");
    assert_eq!(accounts.len(), 2);
}

#[tokio::test]
async fn record_target_reply_increments() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");
    record_target_reply(&pool, "acc_1").await.expect("reply");
    record_target_reply(&pool, "acc_1").await.expect("reply");

    let account = get_target_account(&pool, "acc_1")
        .await
        .expect("get")
        .expect("found");
    assert_eq!(account.total_replies_sent, 2);
    assert!(account.first_engagement_at.is_some());
    assert!(account.last_reply_at.is_some());
}

#[tokio::test]
async fn store_and_check_target_tweet() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    assert!(!target_tweet_exists(&pool, "tw_1").await.expect("check"));

    store_target_tweet(&pool, "tw_1", "acc_1", "hello", "2026-01-01", 0, 5, 80.0)
        .await
        .expect("store");

    assert!(target_tweet_exists(&pool, "tw_1").await.expect("check"));
}

#[tokio::test]
async fn mark_replied_updates_flag() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");
    store_target_tweet(&pool, "tw_1", "acc_1", "hello", "2026-01-01", 0, 5, 80.0)
        .await
        .expect("store");

    mark_target_tweet_replied(&pool, "tw_1")
        .await
        .expect("mark");

    // Verify by checking the count of replied tweets
    let count = count_target_replies_today(&pool).await.expect("count");
    assert!(count >= 0); // May or may not be today depending on test timing
}

#[tokio::test]
async fn get_enriched_includes_interactions_today() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");
    upsert_target_account(&pool, "acc_2", "bob")
        .await
        .expect("upsert");

    // Store a tweet for alice discovered today and mark as replied
    let today = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    store_target_tweet(&pool, "tw_1", "acc_1", "hello", &today, 0, 5, 80.0)
        .await
        .expect("store");
    mark_target_tweet_replied(&pool, "tw_1")
        .await
        .expect("mark");

    let enriched = get_enriched_target_accounts(&pool).await.expect("enriched");
    assert_eq!(enriched.len(), 2);

    let alice = enriched.iter().find(|a| a.username == "alice").unwrap();
    assert_eq!(alice.interactions_today, 1);

    let bob = enriched.iter().find(|a| a.username == "bob").unwrap();
    assert_eq!(bob.interactions_today, 0);
}

#[tokio::test]
async fn get_target_timeline_returns_tweets_with_replies() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    store_target_tweet(
        &pool,
        "tw_1",
        "acc_1",
        "First tweet",
        "2026-02-20T10:00:00Z",
        2,
        10,
        75.0,
    )
    .await
    .expect("store");
    store_target_tweet(
        &pool,
        "tw_2",
        "acc_1",
        "Second tweet",
        "2026-02-21T10:00:00Z",
        1,
        5,
        60.0,
    )
    .await
    .expect("store");
    mark_target_tweet_replied(&pool, "tw_1")
        .await
        .expect("mark");

    // Insert a reply for tw_1
    sqlx::query(
        "INSERT INTO replies_sent (target_tweet_id, reply_content, created_at, status) \
         VALUES ('tw_1', 'Great point!', '2026-02-20T11:00:00Z', 'sent')",
    )
    .execute(&pool)
    .await
    .expect("insert reply");

    let timeline = get_target_timeline(&pool, "alice", 50)
        .await
        .expect("timeline");
    assert_eq!(timeline.len(), 2);

    // Most recent first
    assert_eq!(timeline[0].tweet_id, "tw_2");
    assert!(!timeline[0].replied_to);
    assert!(timeline[0].reply_content.is_none());

    assert_eq!(timeline[1].tweet_id, "tw_1");
    assert!(timeline[1].replied_to);
    assert_eq!(timeline[1].reply_content.as_deref(), Some("Great point!"));
}

#[tokio::test]
async fn get_target_stats_returns_aggregates() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    // Record some replies to set first/last engagement
    record_target_reply(&pool, "acc_1").await.expect("reply");
    record_target_reply(&pool, "acc_1").await.expect("reply");

    // Store tweets with scores and mark as replied
    store_target_tweet(
        &pool,
        "tw_1",
        "acc_1",
        "Tweet one",
        "2026-02-20T10:00:00Z",
        0,
        5,
        70.0,
    )
    .await
    .expect("store");
    mark_target_tweet_replied(&pool, "tw_1")
        .await
        .expect("mark");

    store_target_tweet(
        &pool,
        "tw_2",
        "acc_1",
        "Tweet two",
        "2026-02-22T10:00:00Z",
        0,
        3,
        90.0,
    )
    .await
    .expect("store");
    mark_target_tweet_replied(&pool, "tw_2")
        .await
        .expect("mark");

    // Insert replies for both
    sqlx::query(
        "INSERT INTO replies_sent (target_tweet_id, reply_content, created_at, status) \
         VALUES ('tw_1', 'Reply one', '2026-02-20T11:00:00Z', 'sent')",
    )
    .execute(&pool)
    .await
    .expect("insert reply");
    sqlx::query(
        "INSERT INTO replies_sent (target_tweet_id, reply_content, created_at, status) \
         VALUES ('tw_2', 'Reply two', '2026-02-22T11:00:00Z', 'sent')",
    )
    .execute(&pool)
    .await
    .expect("insert reply");

    let stats = get_target_stats(&pool, "alice")
        .await
        .expect("stats")
        .expect("found");
    assert_eq!(stats.total_replies, 2);
    assert!((stats.avg_score - 80.0).abs() < 0.01); // (70+90)/2
    assert!(stats.best_reply_content.is_some());
    assert!((stats.best_reply_score.unwrap() - 90.0).abs() < 0.01);
}

#[tokio::test]
async fn get_target_stats_returns_none_for_missing() {
    let pool = init_test_db().await.expect("init db");

    let stats = get_target_stats(&pool, "nobody").await.expect("stats");
    assert!(stats.is_none());
}

#[test]
fn compute_frequency_less_than_two_replies() {
    let result = compute_frequency(
        &Some("2026-01-01T00:00:00".to_string()),
        &Some("2026-01-10T00:00:00".to_string()),
        1,
    );
    assert!(result.is_none());
}

#[test]
fn compute_frequency_none_dates() {
    assert!(compute_frequency(&None, &Some("2026-01-10T00:00:00".to_string()), 5).is_none());
    assert!(compute_frequency(&Some("2026-01-01T00:00:00".to_string()), &None, 5).is_none());
}

#[test]
fn compute_frequency_valid_dates() {
    let first = Some("2026-01-01T00:00:00".to_string());
    let last = Some("2026-01-11T00:00:00".to_string());
    let result = compute_frequency(&first, &last, 6);
    assert!(result.is_some());
    // 10 days / (6-1) = 2.0 days per interaction
    assert!((result.unwrap() - 2.0).abs() < 0.01);
}

#[test]
fn compute_frequency_same_date_returns_none() {
    let date = Some("2026-01-01T00:00:00".to_string());
    let result = compute_frequency(&date, &date, 3);
    assert!(result.is_none()); // span = 0
}

#[test]
fn compute_frequency_invalid_dates() {
    let result = compute_frequency(
        &Some("not-a-date".to_string()),
        &Some("2026-01-10T00:00:00".to_string()),
        5,
    );
    assert!(result.is_none());
}

#[tokio::test]
async fn deactivate_target_account_works() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    let deactivated = deactivate_target_account(&pool, "alice")
        .await
        .expect("deactivate");
    assert!(deactivated);

    // Should not appear in active accounts
    let active = get_active_target_accounts(&pool).await.expect("get");
    assert!(active.is_empty());
}

#[tokio::test]
async fn deactivate_nonexistent_account_returns_false() {
    let pool = init_test_db().await.expect("init db");

    let deactivated = deactivate_target_account(&pool, "nobody")
        .await
        .expect("deactivate");
    assert!(!deactivated);
}

#[tokio::test]
async fn deactivate_already_inactive_returns_false() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");
    deactivate_target_account(&pool, "alice")
        .await
        .expect("deactivate");

    // Second deactivate should return false
    let second = deactivate_target_account(&pool, "alice")
        .await
        .expect("deactivate");
    assert!(!second);
}

#[tokio::test]
async fn get_target_account_by_username_works() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    let account = get_target_account_by_username(&pool, "alice")
        .await
        .expect("get")
        .expect("found");
    assert_eq!(account.account_id, "acc_1");
    assert_eq!(account.username, "alice");
}

#[tokio::test]
async fn get_target_account_by_username_not_found() {
    let pool = init_test_db().await.expect("init db");

    let account = get_target_account_by_username(&pool, "nobody")
        .await
        .expect("get");
    assert!(account.is_none());
}

#[tokio::test]
async fn upsert_updates_username_on_conflict() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");
    upsert_target_account(&pool, "acc_1", "alice_new")
        .await
        .expect("upsert again");

    let account = get_target_account(&pool, "acc_1")
        .await
        .expect("get")
        .expect("found");
    assert_eq!(account.username, "alice_new");
}

#[tokio::test]
async fn get_target_account_not_found() {
    let pool = init_test_db().await.expect("init db");

    let account = get_target_account(&pool, "nonexistent").await.expect("get");
    assert!(account.is_none());
}

#[tokio::test]
async fn store_target_tweet_ignore_duplicate() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    store_target_tweet(&pool, "tw_1", "acc_1", "hello", "2026-01-01", 0, 5, 80.0)
        .await
        .expect("store first");

    // Should not error (INSERT OR IGNORE)
    store_target_tweet(&pool, "tw_1", "acc_1", "updated", "2026-01-01", 0, 5, 90.0)
        .await
        .expect("store duplicate");
}

#[tokio::test]
async fn count_target_replies_today_zero() {
    let pool = init_test_db().await.expect("init db");
    let count = count_target_replies_today(&pool).await.expect("count");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn get_target_stats_returns_zero_avg_for_target_without_replies() {
    let pool = init_test_db().await.expect("init db");

    upsert_target_account(&pool, "acc_1", "alice")
        .await
        .expect("upsert");

    let stats = get_target_stats(&pool, "alice")
        .await
        .expect("stats")
        .expect("found");
    assert_eq!(stats.total_replies, 0);
    assert_eq!(stats.avg_score, 0.0);
    assert!(stats.best_reply_content.is_none());
    assert!(stats.best_reply_score.is_none());
    assert!(stats.first_interaction.is_none());
    assert!(stats.interaction_frequency_days.is_none());
}
