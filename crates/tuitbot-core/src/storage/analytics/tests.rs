use super::*;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::init_test_db;

#[tokio::test]
async fn upsert_and_get_follower_snapshot() {
    let pool = init_test_db().await.expect("init db");

    upsert_follower_snapshot(&pool, 1000, 200, 500)
        .await
        .expect("upsert");

    let snapshots = get_follower_snapshots(&pool, 10).await.expect("get");
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].follower_count, 1000);
    assert_eq!(snapshots[0].following_count, 200);
    assert_eq!(snapshots[0].tweet_count, 500);
}

#[tokio::test]
async fn upsert_follower_snapshot_updates_existing() {
    let pool = init_test_db().await.expect("init db");

    upsert_follower_snapshot(&pool, 1000, 200, 500)
        .await
        .expect("upsert");

    upsert_follower_snapshot(&pool, 1050, 201, 510)
        .await
        .expect("upsert again");

    let snapshots = get_follower_snapshots(&pool, 10).await.expect("get");
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].follower_count, 1050);
}

#[tokio::test]
async fn upsert_reply_performance_works() {
    let pool = init_test_db().await.expect("init db");

    upsert_reply_performance(&pool, "r1", 5, 2, 100, 55.0)
        .await
        .expect("upsert");

    // Update
    upsert_reply_performance(&pool, "r1", 10, 3, 200, 75.0)
        .await
        .expect("update");
}

#[tokio::test]
async fn upsert_tweet_performance_works() {
    let pool = init_test_db().await.expect("init db");

    upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");

    // Update
    upsert_tweet_performance(&pool, "tw1", 20, 10, 5, 1000, 95.0)
        .await
        .expect("update");
}

#[tokio::test]
async fn update_and_get_content_scores() {
    let pool = init_test_db().await.expect("init db");

    update_content_score(&pool, "rust", "tip", 80.0)
        .await
        .expect("update");
    update_content_score(&pool, "rust", "tip", 90.0)
        .await
        .expect("update");
    update_content_score(&pool, "python", "list", 60.0)
        .await
        .expect("update");

    let top = get_top_topics(&pool, 10).await.expect("get");
    assert_eq!(top.len(), 2);
    // Rust should be higher (avg ~85) than Python (60)
    assert_eq!(top[0].topic, "rust");
    assert_eq!(top[0].total_posts, 2);
    assert!(top[0].avg_performance > 80.0);
}

#[test]
fn compute_performance_score_basic() {
    let score = compute_performance_score(10, 5, 3, 1000);
    // (10*3 + 5*5 + 3*4) / 1000 * 1000 = (30 + 25 + 12) = 67
    assert!((score - 67.0).abs() < 0.01);
}

#[test]
fn compute_performance_score_zero_impressions() {
    let score = compute_performance_score(10, 5, 3, 0);
    // Denominator clamped to 1: (30 + 25 + 12) / 1 * 1000 = 67000
    assert!((score - 67000.0).abs() < 0.01);
}

#[test]
fn compute_performance_score_all_zero() {
    let score = compute_performance_score(0, 0, 0, 0);
    assert!((score - 0.0).abs() < 0.01);
}

#[tokio::test]
async fn avg_reply_engagement_empty() {
    let pool = init_test_db().await.expect("init db");
    let avg = get_avg_reply_engagement(&pool).await.expect("avg");
    assert!((avg - 0.0).abs() < 0.01);
}

#[tokio::test]
async fn avg_reply_engagement_with_data() {
    let pool = init_test_db().await.expect("init db");
    upsert_reply_performance(&pool, "r1", 10, 5, 1000, 67.0)
        .await
        .expect("upsert");
    upsert_reply_performance(&pool, "r2", 20, 10, 2000, 80.0)
        .await
        .expect("upsert");

    let avg = get_avg_reply_engagement(&pool).await.expect("avg");
    // (67 + 80) / 2 = 73.5
    assert!((avg - 73.5).abs() < 0.01);
}

#[tokio::test]
async fn avg_tweet_engagement_empty() {
    let pool = init_test_db().await.expect("init db");
    let avg = get_avg_tweet_engagement(&pool).await.expect("avg");
    assert!((avg - 0.0).abs() < 0.01);
}

#[tokio::test]
async fn performance_counts_empty() {
    let pool = init_test_db().await.expect("init db");
    let (replies, tweets) = get_performance_counts(&pool).await.expect("counts");
    assert_eq!(replies, 0);
    assert_eq!(tweets, 0);
}

#[tokio::test]
async fn performance_counts_with_data() {
    let pool = init_test_db().await.expect("init db");
    upsert_reply_performance(&pool, "r1", 10, 5, 1000, 67.0)
        .await
        .expect("upsert");
    upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");
    upsert_tweet_performance(&pool, "tw2", 20, 10, 5, 1000, 95.0)
        .await
        .expect("upsert");

    let (replies, tweets) = get_performance_counts(&pool).await.expect("counts");
    assert_eq!(replies, 1);
    assert_eq!(tweets, 2);
}

#[tokio::test]
async fn analytics_summary_empty() {
    let pool = init_test_db().await.expect("init db");
    let summary = get_analytics_summary(&pool).await.expect("summary");
    assert_eq!(summary.followers.current, 0);
    assert_eq!(summary.followers.change_7d, 0);
    assert_eq!(summary.followers.change_30d, 0);
    assert_eq!(summary.actions_today.replies, 0);
    assert!((summary.engagement.avg_reply_score - 0.0).abs() < 0.01);
    assert!(summary.top_topics.is_empty());
}

#[tokio::test]
async fn analytics_summary_with_data() {
    let pool = init_test_db().await.expect("init db");

    // Insert follower snapshot (only today since test db is in-memory)
    upsert_follower_snapshot(&pool, 1000, 200, 500)
        .await
        .expect("upsert");

    // Insert some performance data
    upsert_reply_performance(&pool, "r1", 10, 5, 1000, 67.0)
        .await
        .expect("upsert");

    // Insert content scores so top_topics is populated
    update_content_score(&pool, "rust", "tip", 80.0)
        .await
        .expect("score");
    update_content_score(&pool, "ai", "thread", 60.0)
        .await
        .expect("score");

    let summary = get_analytics_summary(&pool).await.expect("summary");
    assert_eq!(summary.followers.current, 1000);
    assert!(summary.engagement.avg_reply_score > 0.0);
    assert_eq!(summary.engagement.total_replies_sent, 1);
    assert_eq!(summary.top_topics.len(), 2);
    assert_eq!(summary.top_topics[0].topic, "rust");
}

#[tokio::test]
async fn recent_performance_items_empty() {
    let pool = init_test_db().await.expect("init db");
    let items = get_recent_performance_items(&pool, 10).await.expect("get");
    assert!(items.is_empty());
}

#[tokio::test]
async fn recent_performance_items_with_data() {
    let pool = init_test_db().await.expect("init db");

    // Insert a reply and its performance
    let reply = crate::storage::replies::ReplySent {
        id: 0,
        target_tweet_id: "t1".to_string(),
        reply_tweet_id: Some("r1".to_string()),
        reply_content: "Great point about testing!".to_string(),
        llm_provider: Some("openai".to_string()),
        llm_model: Some("gpt-4o".to_string()),
        created_at: "2026-02-23T12:00:00Z".to_string(),
        status: "sent".to_string(),
        error_message: None,
    };
    crate::storage::replies::insert_reply(&pool, &reply)
        .await
        .expect("insert reply");
    upsert_reply_performance(&pool, "r1", 10, 5, 1000, 67.0)
        .await
        .expect("upsert perf");

    let items = get_recent_performance_items(&pool, 10).await.expect("get");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].content_type, "reply");
    assert!(items[0].content_preview.contains("testing"));
    assert_eq!(items[0].likes, 10);
}

// ============================================================================
// Winning DNA storage tests
// ============================================================================

#[tokio::test]
async fn update_and_get_tweet_archetype() {
    let pool = init_test_db().await.expect("init db");

    upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");

    update_tweet_archetype(&pool, "tw1", "list")
        .await
        .expect("update");

    let row: (Option<String>,) =
        sqlx::query_as("SELECT archetype_vibe FROM tweet_performance WHERE tweet_id = ?")
            .bind("tw1")
            .fetch_one(&pool)
            .await
            .expect("query");
    assert_eq!(row.0.as_deref(), Some("list"));
}

#[tokio::test]
async fn update_and_get_reply_archetype() {
    let pool = init_test_db().await.expect("init db");

    upsert_reply_performance(&pool, "r1", 10, 5, 1000, 67.0)
        .await
        .expect("upsert");

    update_reply_archetype(&pool, "r1", "ask_question")
        .await
        .expect("update");

    let row: (Option<String>,) =
        sqlx::query_as("SELECT archetype_vibe FROM reply_performance WHERE reply_id = ?")
            .bind("r1")
            .fetch_one(&pool)
            .await
            .expect("query");
    assert_eq!(row.0.as_deref(), Some("ask_question"));
}

#[tokio::test]
async fn update_and_get_engagement_score() {
    let pool = init_test_db().await.expect("init db");

    upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");

    update_tweet_engagement_score(&pool, "tw1", 0.85)
        .await
        .expect("update");

    let row: (Option<f64>,) =
        sqlx::query_as("SELECT engagement_score FROM tweet_performance WHERE tweet_id = ?")
            .bind("tw1")
            .fetch_one(&pool)
            .await
            .expect("query");
    assert!((row.0.unwrap() - 0.85).abs() < 0.001);
}

#[tokio::test]
async fn get_max_performance_score_empty() {
    let pool = init_test_db().await.expect("init db");
    let max = get_max_performance_score(&pool).await.expect("max");
    assert!((max - 0.0).abs() < 0.01);
}

#[tokio::test]
async fn get_max_performance_score_with_data() {
    let pool = init_test_db().await.expect("init db");

    upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");
    upsert_reply_performance(&pool, "r1", 20, 10, 2000, 95.0)
        .await
        .expect("upsert");

    let max = get_max_performance_score(&pool).await.expect("max");
    assert!((max - 95.0).abs() < 0.01);
}

#[tokio::test]
async fn get_scored_ancestors_empty() {
    let pool = init_test_db().await.expect("init db");
    let ancestors =
        get_scored_ancestors(&pool, "00000000-0000-0000-0000-000000000000", &[], 0.1, 10)
            .await
            .expect("query");
    assert!(ancestors.is_empty());
}

#[tokio::test]
async fn get_scored_ancestors_returns_scored_items() {
    let pool = init_test_db().await.expect("init db");

    // Insert a tweet with performance + engagement_score
    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES ('00000000-0000-0000-0000-000000000000', 'tw1', 'Great Rust testing tips', 'rust', 'sent', '2026-02-27T10:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("insert tweet");

    upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert perf");
    update_tweet_engagement_score(&pool, "tw1", 0.85)
        .await
        .expect("update score");

    let ancestors =
        get_scored_ancestors(&pool, "00000000-0000-0000-0000-000000000000", &[], 0.1, 10)
            .await
            .expect("query");
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0].content_type, "tweet");
    assert_eq!(ancestors[0].id, "tw1");
    assert!((ancestors[0].engagement_score.unwrap() - 0.85).abs() < 0.001);
}

#[tokio::test]
async fn get_scored_ancestors_filters_low_engagement() {
    let pool = init_test_db().await.expect("init db");

    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES ('00000000-0000-0000-0000-000000000000', 'tw1', 'Low performer', 'rust', 'sent', '2026-02-27T10:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("insert tweet");

    upsert_tweet_performance(&pool, "tw1", 1, 0, 0, 500, 5.0)
        .await
        .expect("upsert perf");
    update_tweet_engagement_score(&pool, "tw1", 0.05)
        .await
        .expect("update score");

    // min_score = 0.1, so this ancestor should be filtered out
    let ancestors =
        get_scored_ancestors(&pool, "00000000-0000-0000-0000-000000000000", &[], 0.1, 10)
            .await
            .expect("query");
    assert!(ancestors.is_empty());
}

#[tokio::test]
async fn malformed_snapshot_date_excluded() {
    let pool = init_test_db().await.expect("init db");

    // Insert a valid snapshot.
    upsert_follower_snapshot(&pool, 1000, 200, 500)
        .await
        .expect("upsert");

    // Insert a row with a malformed snapshot_date via raw SQL.
    sqlx::query(
        "INSERT INTO follower_snapshots (account_id, snapshot_date, follower_count, following_count, tweet_count) \
         VALUES (?, 'not-a-date', 999, 99, 9)",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .execute(&pool)
    .await
    .expect("insert malformed");

    let snapshots = get_follower_snapshots(&pool, 10).await.expect("get");
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].follower_count, 1000);
}
