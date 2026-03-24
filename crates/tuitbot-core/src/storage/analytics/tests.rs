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

// ============================================================================
// Ancestors: reply engagement score + keyword-filtered queries
// ============================================================================

#[tokio::test]
async fn update_reply_engagement_score_works() {
    let pool = init_test_db().await.expect("init db");

    upsert_reply_performance(&pool, "r1", 10, 5, 1000, 67.0)
        .await
        .expect("upsert");

    update_reply_engagement_score(&pool, "r1", 0.72)
        .await
        .expect("update");

    let row: (Option<f64>,) =
        sqlx::query_as("SELECT engagement_score FROM reply_performance WHERE reply_id = ?")
            .bind("r1")
            .fetch_one(&pool)
            .await
            .expect("query");
    assert!((row.0.unwrap() - 0.72).abs() < 0.001);
}

#[tokio::test]
async fn get_scored_ancestors_with_topic_keywords() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    // Insert two tweets with different topics
    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES (?, 'tw-rust', 'Rust is great for systems', 'rust', 'sent', '2026-02-27T10:00:00Z')",
    )
    .bind(acct)
    .execute(&pool)
    .await
    .expect("insert rust tweet");

    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES (?, 'tw-python', 'Python is great for AI', 'python', 'sent', '2026-02-27T11:00:00Z')",
    )
    .bind(acct)
    .execute(&pool)
    .await
    .expect("insert python tweet");

    upsert_tweet_performance(&pool, "tw-rust", 20, 10, 5, 1000, 90.0)
        .await
        .expect("perf rust");
    update_tweet_engagement_score(&pool, "tw-rust", 0.9)
        .await
        .expect("score rust");

    upsert_tweet_performance(&pool, "tw-python", 15, 8, 3, 800, 70.0)
        .await
        .expect("perf python");
    update_tweet_engagement_score(&pool, "tw-python", 0.7)
        .await
        .expect("score python");

    // Filter to rust only
    let ancestors = get_scored_ancestors(&pool, acct, &["rust".to_string()], 0.1, 10)
        .await
        .expect("query");
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0].id, "tw-rust");
}

#[tokio::test]
async fn get_scored_ancestors_with_reply_keyword_match() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    // Insert a reply that contains keyword in content
    sqlx::query(
        "INSERT INTO replies_sent \
         (account_id, target_tweet_id, reply_tweet_id, reply_content, status, created_at) \
         VALUES (?, 't1', 'reply-ml', 'Machine learning is transforming the field', 'sent', '2026-02-28T10:00:00Z')",
    )
    .bind(acct)
    .execute(&pool)
    .await
    .expect("insert reply");

    upsert_reply_performance(&pool, "reply-ml", 30, 15, 2000, 85.0)
        .await
        .expect("perf");
    update_reply_engagement_score(&pool, "reply-ml", 0.88)
        .await
        .expect("score");

    // Search with "learning" keyword — should match the reply content via LIKE
    let ancestors = get_scored_ancestors(&pool, acct, &["learning".to_string()], 0.1, 10)
        .await
        .expect("query");
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0].content_type, "reply");
    assert_eq!(ancestors[0].id, "reply-ml");
}

#[tokio::test]
async fn get_max_performance_score_picks_highest_across_tables() {
    let pool = init_test_db().await.expect("init db");

    // Tweet with lower score
    upsert_tweet_performance(&pool, "tw1", 5, 2, 1, 500, 40.0)
        .await
        .expect("upsert tweet");
    // Reply with higher score
    upsert_reply_performance(&pool, "r1", 30, 15, 2000, 120.0)
        .await
        .expect("upsert reply");

    let max = get_max_performance_score(&pool).await.expect("max");
    assert!((max - 120.0).abs() < 0.01);

    // Now add an even higher tweet
    upsert_tweet_performance(&pool, "tw2", 100, 50, 20, 5000, 200.0)
        .await
        .expect("upsert tweet 2");

    let max = get_max_performance_score(&pool).await.expect("max");
    assert!((max - 200.0).abs() < 0.01);
}

// ============================================================================
// Additional analytics coverage tests
// ============================================================================

#[tokio::test]
async fn avg_tweet_engagement_with_data() {
    let pool = init_test_db().await.expect("init db");
    upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");
    upsert_tweet_performance(&pool, "tw2", 20, 10, 5, 1000, 90.0)
        .await
        .expect("upsert");

    let avg = get_avg_tweet_engagement(&pool).await.expect("avg");
    // (82 + 90) / 2 = 86
    assert!((avg - 86.0).abs() < 0.01);
}

#[tokio::test]
async fn optimal_posting_times_empty() {
    let pool = init_test_db().await.expect("init db");
    let times = get_optimal_posting_times(&pool).await.expect("get");
    assert!(times.is_empty());
}

#[tokio::test]
async fn optimal_posting_times_with_data() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    // Insert tweets at different hours
    sqlx::query(
        "INSERT INTO original_tweets \
         (account_id, tweet_id, content, topic, status, created_at) \
         VALUES (?, 'tw-morning', 'Morning tweet', 'rust', 'sent', '2026-02-27T09:00:00Z')",
    )
    .bind(acct)
    .execute(&pool)
    .await
    .expect("insert morning tweet");

    sqlx::query(
        "INSERT INTO original_tweets \
         (account_id, tweet_id, content, topic, status, created_at) \
         VALUES (?, 'tw-afternoon', 'Afternoon tweet', 'rust', 'sent', '2026-02-27T15:00:00Z')",
    )
    .bind(acct)
    .execute(&pool)
    .await
    .expect("insert afternoon tweet");

    upsert_tweet_performance(&pool, "tw-morning", 20, 10, 5, 1000, 90.0)
        .await
        .expect("perf morning");
    upsert_tweet_performance(&pool, "tw-afternoon", 5, 2, 1, 500, 30.0)
        .await
        .expect("perf afternoon");

    let times = get_optimal_posting_times(&pool).await.expect("get");
    assert_eq!(times.len(), 2);
    // Best hour should be first (ordered by avg_engagement DESC)
    assert!(times[0].avg_engagement >= times[1].avg_engagement);
    assert_eq!(times[0].hour, 9);
}

#[tokio::test]
async fn content_score_incremental_mean() {
    let pool = init_test_db().await.expect("init db");

    // Insert 3 scores for the same topic/format
    update_content_score(&pool, "testing", "tip", 60.0)
        .await
        .expect("score 1");
    update_content_score(&pool, "testing", "tip", 80.0)
        .await
        .expect("score 2");
    update_content_score(&pool, "testing", "tip", 100.0)
        .await
        .expect("score 3");

    let top = get_top_topics(&pool, 10).await.expect("get");
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].total_posts, 3);
    // Incremental mean: (60 + 80 + 100) / 3 = 80 (approximately)
    assert!(
        (top[0].avg_performance - 80.0).abs() < 1.0,
        "expected avg ~80, got {}",
        top[0].avg_performance
    );
}

#[tokio::test]
async fn top_topics_respects_limit() {
    let pool = init_test_db().await.expect("init db");

    update_content_score(&pool, "topic1", "tip", 90.0)
        .await
        .expect("score");
    update_content_score(&pool, "topic2", "list", 80.0)
        .await
        .expect("score");
    update_content_score(&pool, "topic3", "thread", 70.0)
        .await
        .expect("score");

    let top = get_top_topics(&pool, 2).await.expect("get");
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].topic, "topic1");
    assert_eq!(top[1].topic, "topic2");
}

#[tokio::test]
async fn upsert_follower_snapshot_for_updates_correctly() {
    let pool = init_test_db().await.expect("init db");

    // Upsert twice on the same day — should update, not duplicate
    upsert_follower_snapshot(&pool, 500, 100, 200)
        .await
        .expect("upsert 1");
    upsert_follower_snapshot(&pool, 750, 150, 300)
        .await
        .expect("upsert 2");

    let snaps = get_follower_snapshots(&pool, 10).await.expect("get");
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].follower_count, 750);
    assert_eq!(snaps[0].following_count, 150);
}

#[tokio::test]
async fn upsert_reply_performance_for_account_scoped() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-reply-perf";

    crate::storage::accounts::create_account(&pool, acct, "RP")
        .await
        .expect("create");

    upsert_reply_performance_for(&pool, acct, "r-scoped", 15, 7, 500, 72.0)
        .await
        .expect("upsert");

    // Verify via avg engagement for that account
    let avg = get_avg_reply_engagement_for(&pool, acct)
        .await
        .expect("avg");
    assert!((avg - 72.0).abs() < 0.01);
}

#[tokio::test]
async fn upsert_tweet_performance_for_account_scoped() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-tweet-perf";

    crate::storage::accounts::create_account(&pool, acct, "TP")
        .await
        .expect("create");

    upsert_tweet_performance_for(&pool, acct, "tw-scoped", 25, 12, 6, 2000, 88.0)
        .await
        .expect("upsert");

    let avg = get_avg_tweet_engagement_for(&pool, acct)
        .await
        .expect("avg");
    assert!((avg - 88.0).abs() < 0.01);

    let (replies, tweets) = get_performance_counts_for(&pool, acct)
        .await
        .expect("counts");
    assert_eq!(replies, 0);
    assert_eq!(tweets, 1);
}

#[tokio::test]
async fn content_score_for_account_scoped() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-cscore";

    crate::storage::accounts::create_account(&pool, acct, "CS")
        .await
        .expect("create");

    update_content_score_for(&pool, acct, "go", "tip", 75.0)
        .await
        .expect("score");

    let top = get_top_topics_for(&pool, acct, 10).await.expect("get");
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].topic, "go");

    // Default account should have no scores
    let top_default = get_top_topics(&pool, 10).await.expect("get default");
    assert!(top_default.is_empty());
}

#[tokio::test]
async fn compute_performance_score_large_engagement() {
    // High engagement relative to impressions
    let score = compute_performance_score(100, 50, 30, 100);
    // (100*3 + 50*5 + 30*4) / 100 * 1000 = (300+250+120)/100*1000 = 6700
    assert!((score - 6700.0).abs() < 0.01);
}

#[tokio::test]
async fn recent_performance_items_with_tweets_and_replies() {
    let pool = init_test_db().await.expect("init db");
    let acct = DEFAULT_ACCOUNT_ID;

    // Insert a tweet with performance
    sqlx::query(
        "INSERT INTO original_tweets \
         (account_id, tweet_id, content, topic, status, created_at) \
         VALUES (?, 'tw-perf', 'Great tips for testing', 'testing', 'sent', '2026-03-01T10:00:00Z')",
    )
    .bind(acct)
    .execute(&pool)
    .await
    .expect("insert tweet");

    upsert_tweet_performance(&pool, "tw-perf", 30, 15, 8, 2000, 91.0)
        .await
        .expect("upsert tweet perf");

    // Insert a reply with performance
    let reply = crate::storage::replies::ReplySent {
        id: 0,
        target_tweet_id: "t-target".to_string(),
        reply_tweet_id: Some("r-perf".to_string()),
        reply_content: "Excellent analysis here".to_string(),
        llm_provider: Some("anthropic".to_string()),
        llm_model: Some("claude-3".to_string()),
        created_at: "2026-03-01T12:00:00Z".to_string(),
        status: "sent".to_string(),
        error_message: None,
    };
    crate::storage::replies::insert_reply(&pool, &reply)
        .await
        .expect("insert reply");
    upsert_reply_performance(&pool, "r-perf", 20, 10, 1500, 78.0)
        .await
        .expect("upsert reply perf");

    let items = get_recent_performance_items(&pool, 10).await.expect("get");
    assert_eq!(items.len(), 2);
    // Items are ordered by posted_at DESC
    // The reply is at 12:00, tweet at 10:00
    assert_eq!(items[0].content_type, "reply");
    assert_eq!(items[1].content_type, "tweet");
    assert_eq!(items[0].likes, 20);
    assert_eq!(items[1].likes, 30);
}

// ============================================================================
// Tweet performance: percentiles, multi-get, and account-scoped queries
// ============================================================================

#[tokio::test]
async fn compute_percentiles_insufficient_data() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-perc-few";
    crate::storage::accounts::create_account(&pool, acct, "PF")
        .await
        .expect("create");

    // Insert only 5 rows — below the 10-row threshold
    for i in 0..5 {
        upsert_tweet_performance_for(
            &pool,
            acct,
            &format!("tw-p{i}"),
            i,
            0,
            0,
            (i + 1) * 100,
            10.0,
        )
        .await
        .expect("upsert");
    }

    let p = compute_performance_percentiles_for(&pool, acct)
        .await
        .expect("percentiles");
    assert!(!p.has_sufficient_data);
    assert_eq!(p.p50_impressions, 0);
    assert_eq!(p.p90_impressions, 0);
}

#[tokio::test]
async fn compute_percentiles_empty_account() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-perc-empty";
    crate::storage::accounts::create_account(&pool, acct, "PE")
        .await
        .expect("create");

    let p = compute_performance_percentiles_for(&pool, acct)
        .await
        .expect("percentiles");
    assert!(!p.has_sufficient_data);
}

#[tokio::test]
async fn compute_percentiles_sufficient_data() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-perc-ok";
    crate::storage::accounts::create_account(&pool, acct, "PO")
        .await
        .expect("create");

    // Insert 20 rows with impressions 100, 200, ..., 2000
    for i in 1..=20 {
        upsert_tweet_performance_for(&pool, acct, &format!("tw-pc{i}"), i, 0, 0, i * 100, 10.0)
            .await
            .expect("upsert");
    }

    let p = compute_performance_percentiles_for(&pool, acct)
        .await
        .expect("percentiles");
    assert!(p.has_sufficient_data);
    // Sorted: [100, 200, ..., 2000]. count=20.
    // p50 = impressions[20/2] = impressions[10] = 1100
    // p90 = impressions[20*9/10] = impressions[18] = 1900
    assert_eq!(p.p50_impressions, 1100);
    assert_eq!(p.p90_impressions, 1900);
}

#[tokio::test]
async fn compute_percentiles_exactly_ten_rows() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-perc-ten";
    crate::storage::accounts::create_account(&pool, acct, "PT")
        .await
        .expect("create");

    // Insert exactly 10 rows with impressions 10, 20, ..., 100
    for i in 1..=10 {
        upsert_tweet_performance_for(&pool, acct, &format!("tw-p10-{i}"), i, 0, 0, i * 10, 10.0)
            .await
            .expect("upsert");
    }

    let p = compute_performance_percentiles_for(&pool, acct)
        .await
        .expect("percentiles");
    assert!(p.has_sufficient_data);
    // p50 = impressions[5] = 60, p90 = impressions[9] = 100
    assert_eq!(p.p50_impressions, 60);
    assert_eq!(p.p90_impressions, 100);
}

#[tokio::test]
async fn get_tweet_performances_for_empty_ids() {
    let pool = init_test_db().await.expect("init db");
    let result = get_tweet_performances_for(&pool, DEFAULT_ACCOUNT_ID, &[])
        .await
        .expect("get");
    assert!(result.is_empty());
}

#[tokio::test]
async fn get_tweet_performances_for_returns_matching() {
    let pool = init_test_db().await.expect("init db");

    upsert_tweet_performance(&pool, "tp-a", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");
    upsert_tweet_performance(&pool, "tp-b", 20, 10, 5, 1000, 95.0)
        .await
        .expect("upsert");
    upsert_tweet_performance(&pool, "tp-c", 5, 2, 1, 200, 40.0)
        .await
        .expect("upsert");

    let ids = vec!["tp-a".to_string(), "tp-c".to_string()];
    let result = get_tweet_performances_for(&pool, DEFAULT_ACCOUNT_ID, &ids)
        .await
        .expect("get");
    assert_eq!(result.len(), 2);
    let tweet_ids: Vec<&str> = result.iter().map(|r| r.tweet_id.as_str()).collect();
    assert!(tweet_ids.contains(&"tp-a"));
    assert!(tweet_ids.contains(&"tp-c"));
}

#[tokio::test]
async fn get_all_tweet_performances_for_account() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-all-perf";
    crate::storage::accounts::create_account(&pool, acct, "AP")
        .await
        .expect("create");

    upsert_tweet_performance_for(&pool, acct, "all-1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert");
    upsert_tweet_performance_for(&pool, acct, "all-2", 20, 10, 5, 1000, 95.0)
        .await
        .expect("upsert");

    let result = get_all_tweet_performances_for(&pool, acct)
        .await
        .expect("get");
    assert_eq!(result.len(), 2);

    // Default account should not see these
    let default_result = get_all_tweet_performances_for(&pool, DEFAULT_ACCOUNT_ID)
        .await
        .expect("get default");
    // May have data from other tests, but should not contain our acct's rows
    assert!(default_result
        .iter()
        .all(|r| r.tweet_id != "all-1" && r.tweet_id != "all-2"));
}

// ============================================================================
// Heatmap + content breakdown + aggregation tests
// ============================================================================

#[tokio::test]
async fn heatmap_empty() {
    let pool = init_test_db().await.expect("init db");
    let grid = get_heatmap(&pool).await.expect("get");
    // Full 7×24 grid even when empty
    assert_eq!(grid.len(), 168);
    assert!(grid.iter().all(|c| c.avg_engagement == 0.0));
    assert!(grid.iter().all(|c| c.sample_size == 0));
}

#[tokio::test]
async fn heatmap_after_aggregation() {
    let pool = init_test_db().await.expect("init db");
    let acct = DEFAULT_ACCOUNT_ID;

    // Insert engagement metrics with known posted_at times
    // Sunday 10:00 UTC
    let input = crate::storage::analytics::UpsertEngagementInput {
        post_id: "hm-1",
        impressions: 100,
        likes: 10,
        retweets: 5,
        replies: 2,
        bookmarks: 1,
        posted_at: Some("2026-03-22T10:00:00Z"), // Sunday
    };
    upsert_engagement_metric_for(&pool, acct, input)
        .await
        .expect("upsert");

    // Run aggregation to populate best_times
    aggregate_best_times_for(&pool, acct).await.expect("agg");

    let grid = get_heatmap(&pool).await.expect("get");
    assert_eq!(grid.len(), 168);

    // At least one cell should have data
    assert!(grid.iter().any(|c| c.avg_engagement > 0.0));
}

#[tokio::test]
async fn content_breakdown_empty() {
    let pool = init_test_db().await.expect("init db");
    let breakdown = get_content_breakdown(&pool).await.expect("get");
    // No data → empty (zero-count rows are filtered)
    assert!(breakdown.is_empty());
}

#[tokio::test]
async fn content_breakdown_with_data() {
    let pool = init_test_db().await.expect("init db");

    // Insert reply + tweet performance
    upsert_reply_performance(&pool, "cb-r1", 10, 5, 1000, 67.0)
        .await
        .expect("upsert");
    upsert_reply_performance(&pool, "cb-r2", 20, 10, 2000, 80.0)
        .await
        .expect("upsert");
    upsert_tweet_performance(&pool, "cb-tw1", 30, 15, 8, 3000, 90.0)
        .await
        .expect("upsert");

    let breakdown = get_content_breakdown(&pool).await.expect("get");
    assert_eq!(breakdown.len(), 2);

    let reply_row = breakdown
        .iter()
        .find(|b| b.content_type == "reply")
        .unwrap();
    assert_eq!(reply_row.count, 2);
    assert!(reply_row.avg_performance > 0.0);

    let tweet_row = breakdown
        .iter()
        .find(|b| b.content_type == "tweet")
        .unwrap();
    assert_eq!(tweet_row.count, 1);
    assert!(tweet_row.avg_performance > 0.0);
}

#[tokio::test]
async fn aggregate_best_times_populates_table() {
    let pool = init_test_db().await.expect("init db");
    let acct = DEFAULT_ACCOUNT_ID;

    // Insert two engagement metrics at different times
    let input1 = crate::storage::analytics::UpsertEngagementInput {
        post_id: "abt-1",
        impressions: 200,
        likes: 20,
        retweets: 10,
        replies: 5,
        bookmarks: 2,
        posted_at: Some("2026-03-20T09:00:00Z"),
    };
    upsert_engagement_metric_for(&pool, acct, input1)
        .await
        .expect("upsert");

    let input2 = crate::storage::analytics::UpsertEngagementInput {
        post_id: "abt-2",
        impressions: 300,
        likes: 30,
        retweets: 15,
        replies: 8,
        bookmarks: 3,
        posted_at: Some("2026-03-20T15:00:00Z"),
    };
    upsert_engagement_metric_for(&pool, acct, input2)
        .await
        .expect("upsert");

    aggregate_best_times_for(&pool, acct).await.expect("agg");

    let slots = get_best_times(&pool).await.expect("get");
    assert_eq!(slots.len(), 2);
    // Both should have confidence based on sample_size = 1
    assert!(slots.iter().all(|s| s.sample_size == 1));
}

#[tokio::test]
async fn aggregate_reach_creates_snapshot() {
    let pool = init_test_db().await.expect("init db");
    let acct = DEFAULT_ACCOUNT_ID;

    // Insert engagement metric for today
    let today = chrono::Utc::now().format("%Y-%m-%dT12:00:00Z").to_string();
    let input = crate::storage::analytics::UpsertEngagementInput {
        post_id: "reach-1",
        impressions: 500,
        likes: 50,
        retweets: 25,
        replies: 10,
        bookmarks: 5,
        posted_at: Some(&today),
    };
    upsert_engagement_metric_for(&pool, acct, input)
        .await
        .expect("upsert");

    aggregate_reach_for(&pool, acct).await.expect("agg");

    let reach = get_reach(&pool, 7).await.expect("get");
    assert!(!reach.is_empty());
    assert_eq!(reach[0].total_reach, 500);
    assert_eq!(reach[0].post_count, 1);
}
