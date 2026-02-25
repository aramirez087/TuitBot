//! CRUD operations for analytics tables.
//!
//! Manages follower snapshots, reply/tweet performance metrics,
//! and content score running averages.

use super::DbPool;
use crate::error::StorageError;
use chrono::{NaiveDate, Utc};

// ============================================================================
// Follower snapshots
// ============================================================================

/// A daily follower snapshot.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FollowerSnapshot {
    pub snapshot_date: String,
    pub follower_count: i64,
    pub following_count: i64,
    pub tweet_count: i64,
}

/// Upsert a follower snapshot for today.
pub async fn upsert_follower_snapshot(
    pool: &DbPool,
    follower_count: i64,
    following_count: i64,
    tweet_count: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO follower_snapshots (snapshot_date, follower_count, following_count, tweet_count) \
         VALUES (date('now'), ?, ?, ?) \
         ON CONFLICT(snapshot_date) DO UPDATE SET \
         follower_count = excluded.follower_count, \
         following_count = excluded.following_count, \
         tweet_count = excluded.tweet_count",
    )
    .bind(follower_count)
    .bind(following_count)
    .bind(tweet_count)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Get the most recent N follower snapshots, newest first.
pub async fn get_follower_snapshots(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<FollowerSnapshot>, StorageError> {
    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        "SELECT snapshot_date, follower_count, following_count, tweet_count \
         FROM follower_snapshots ORDER BY snapshot_date DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| FollowerSnapshot {
            snapshot_date: r.0,
            follower_count: r.1,
            following_count: r.2,
            tweet_count: r.3,
        })
        .collect())
}

// ============================================================================
// Reply performance
// ============================================================================

/// Store or update reply performance metrics.
pub async fn upsert_reply_performance(
    pool: &DbPool,
    reply_id: &str,
    likes: i64,
    replies: i64,
    impressions: i64,
    score: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO reply_performance (reply_id, likes_received, replies_received, impressions, performance_score) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(reply_id) DO UPDATE SET \
         likes_received = excluded.likes_received, \
         replies_received = excluded.replies_received, \
         impressions = excluded.impressions, \
         performance_score = excluded.performance_score, \
         measured_at = datetime('now')",
    )
    .bind(reply_id)
    .bind(likes)
    .bind(replies)
    .bind(impressions)
    .bind(score)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

// ============================================================================
// Tweet performance
// ============================================================================

/// Store or update tweet performance metrics.
pub async fn upsert_tweet_performance(
    pool: &DbPool,
    tweet_id: &str,
    likes: i64,
    retweets: i64,
    replies: i64,
    impressions: i64,
    score: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO tweet_performance (tweet_id, likes_received, retweets_received, replies_received, impressions, performance_score) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON CONFLICT(tweet_id) DO UPDATE SET \
         likes_received = excluded.likes_received, \
         retweets_received = excluded.retweets_received, \
         replies_received = excluded.replies_received, \
         impressions = excluded.impressions, \
         performance_score = excluded.performance_score, \
         measured_at = datetime('now')",
    )
    .bind(tweet_id)
    .bind(likes)
    .bind(retweets)
    .bind(replies)
    .bind(impressions)
    .bind(score)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

// ============================================================================
// Content scores
// ============================================================================

/// A topic/format performance score.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContentScore {
    pub topic: String,
    pub format: String,
    pub total_posts: i64,
    pub avg_performance: f64,
}

/// Update the running average for a topic/format pair.
///
/// Uses incremental mean: new_avg = old_avg + (score - old_avg) / new_count.
pub async fn update_content_score(
    pool: &DbPool,
    topic: &str,
    format: &str,
    new_score: f64,
) -> Result<(), StorageError> {
    // Insert or update with incremental average
    sqlx::query(
        "INSERT INTO content_scores (topic, format, total_posts, avg_performance) \
         VALUES (?, ?, 1, ?) \
         ON CONFLICT(topic, format) DO UPDATE SET \
         total_posts = content_scores.total_posts + 1, \
         avg_performance = content_scores.avg_performance + \
         (? - content_scores.avg_performance) / (content_scores.total_posts + 1)",
    )
    .bind(topic)
    .bind(format)
    .bind(new_score)
    .bind(new_score)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Get top-performing topics ordered by average performance descending.
pub async fn get_top_topics(pool: &DbPool, limit: u32) -> Result<Vec<ContentScore>, StorageError> {
    let rows: Vec<(String, String, i64, f64)> = sqlx::query_as(
        "SELECT topic, format, total_posts, avg_performance \
         FROM content_scores \
         ORDER BY avg_performance DESC \
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| ContentScore {
            topic: r.0,
            format: r.1,
            total_posts: r.2,
            avg_performance: r.3,
        })
        .collect())
}

/// Get average reply engagement rate (avg performance_score across all measured replies).
pub async fn get_avg_reply_engagement(pool: &DbPool) -> Result<f64, StorageError> {
    let row: (f64,) =
        sqlx::query_as("SELECT COALESCE(AVG(performance_score), 0.0) FROM reply_performance")
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get average tweet engagement rate (avg performance_score across all measured tweets).
pub async fn get_avg_tweet_engagement(pool: &DbPool) -> Result<f64, StorageError> {
    let row: (f64,) =
        sqlx::query_as("SELECT COALESCE(AVG(performance_score), 0.0) FROM tweet_performance")
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get total count of measured replies and tweets.
pub async fn get_performance_counts(pool: &DbPool) -> Result<(i64, i64), StorageError> {
    let reply_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reply_performance")
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    let tweet_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tweet_performance")
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok((reply_count.0, tweet_count.0))
}

/// Compute the performance score for a piece of content.
///
/// Formula: `(likes * 3 + replies * 5 + retweets * 4) / max(impressions, 1) * 1000`
pub fn compute_performance_score(likes: i64, replies: i64, retweets: i64, impressions: i64) -> f64 {
    let numerator = (likes * 3 + replies * 5 + retweets * 4) as f64;
    let denominator = impressions.max(1) as f64;
    numerator / denominator * 1000.0
}

// ============================================================================
// Analytics summary (aggregated dashboard data)
// ============================================================================

/// Follower growth metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FollowerSummary {
    pub current: i64,
    pub change_7d: i64,
    pub change_30d: i64,
}

/// Today's action breakdown.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActionsSummary {
    pub replies: i64,
    pub tweets: i64,
    pub threads: i64,
}

/// Engagement overview.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EngagementSummary {
    pub avg_reply_score: f64,
    pub avg_tweet_score: f64,
    pub total_replies_sent: i64,
    pub total_tweets_posted: i64,
}

/// Combined analytics summary for the dashboard.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalyticsSummary {
    pub followers: FollowerSummary,
    pub actions_today: ActionsSummary,
    pub engagement: EngagementSummary,
    pub top_topics: Vec<ContentScore>,
}

/// Get a combined analytics summary for the dashboard.
///
/// Aggregates follower deltas, today's action counts, and engagement stats
/// into a single struct to minimise round-trips from the frontend.
pub async fn get_analytics_summary(pool: &DbPool) -> Result<AnalyticsSummary, StorageError> {
    // --- Follower data ---
    let snapshots = get_follower_snapshots(pool, 90).await?;
    let current = snapshots.first().map_or(0, |s| s.follower_count);

    // Find the first snapshot whose date is at least N days ago (handles gaps from
    // downtime or weekends).  Snapshots are ordered newest-first.
    let today = Utc::now().date_naive();
    let follower_at_or_before = |days: i64| -> i64 {
        snapshots
            .iter()
            .find(|s| {
                NaiveDate::parse_from_str(&s.snapshot_date, "%Y-%m-%d")
                    .map(|d| (today - d).num_days() >= days)
                    .unwrap_or(false)
            })
            .map_or(current, |s| s.follower_count)
    };

    let change_7d = if snapshots.len() >= 2 {
        current - follower_at_or_before(7)
    } else {
        0
    };
    let change_30d = if snapshots.len() >= 2 {
        current - follower_at_or_before(30)
    } else {
        0
    };

    // --- Today's actions (from action_log) ---
    let today = Utc::now().format("%Y-%m-%dT00:00:00Z").to_string();
    let counts = super::action_log::get_action_counts_since(pool, &today).await?;
    let actions_today = ActionsSummary {
        replies: *counts.get("reply").unwrap_or(&0),
        tweets: *counts.get("tweet").unwrap_or(&0),
        threads: *counts.get("thread").unwrap_or(&0),
    };

    // --- Engagement ---
    let avg_reply_score = get_avg_reply_engagement(pool).await?;
    let avg_tweet_score = get_avg_tweet_engagement(pool).await?;
    let (total_replies_sent, total_tweets_posted) = get_performance_counts(pool).await?;

    // --- Top topics ---
    let top_topics = get_top_topics(pool, 5).await?;

    Ok(AnalyticsSummary {
        followers: FollowerSummary {
            current,
            change_7d,
            change_30d,
        },
        actions_today,
        engagement: EngagementSummary {
            avg_reply_score,
            avg_tweet_score,
            total_replies_sent,
            total_tweets_posted,
        },
        top_topics,
    })
}

// ============================================================================
// Recent performance (joined with content for preview)
// ============================================================================

/// A recent content item with performance metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceItem {
    /// "reply", "tweet", or "thread"
    pub content_type: String,
    /// Truncated content preview
    pub content_preview: String,
    /// Likes received
    pub likes: i64,
    /// Replies received
    pub replies_received: i64,
    /// Retweets (0 for replies)
    pub retweets: i64,
    /// Impressions
    pub impressions: i64,
    /// Computed performance score
    pub performance_score: f64,
    /// When the content was posted (ISO-8601)
    pub posted_at: String,
}

/// Row type returned by the recent-performance UNION query.
type PerformanceRow = (String, String, i64, i64, i64, i64, f64, String);

/// Get recent content performance items, newest first.
///
/// Unions reply and tweet performance joined with their content tables
/// so the dashboard can show a content preview alongside metrics.
pub async fn get_recent_performance_items(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<PerformanceItem>, StorageError> {
    let rows: Vec<PerformanceRow> = sqlx::query_as(
        "SELECT 'reply' as content_type, \
                SUBSTR(rs.reply_content, 1, 120) as content_preview, \
                rp.likes_received, rp.replies_received, 0 as retweets, \
                rp.impressions, rp.performance_score, rs.created_at as posted_at \
         FROM reply_performance rp \
         JOIN replies_sent rs ON rs.reply_tweet_id = rp.reply_id \
         UNION ALL \
         SELECT 'tweet' as content_type, \
                SUBSTR(ot.content, 1, 120) as content_preview, \
                tp.likes_received, tp.replies_received, tp.retweets_received, \
                tp.impressions, tp.performance_score, ot.created_at as posted_at \
         FROM tweet_performance tp \
         JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
         ORDER BY posted_at DESC \
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| PerformanceItem {
            content_type: r.0,
            content_preview: r.1,
            likes: r.2,
            replies_received: r.3,
            retweets: r.4,
            impressions: r.5,
            performance_score: r.6,
            posted_at: r.7,
        })
        .collect())
}

/// Hourly posting performance data.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HourlyPerformance {
    /// Hour of day (0-23).
    pub hour: i64,
    /// Average engagement score for posts in this hour.
    pub avg_engagement: f64,
    /// Number of posts in this hour.
    pub post_count: i64,
}

/// Get optimal posting times based on historical performance.
pub async fn get_optimal_posting_times(
    pool: &DbPool,
) -> Result<Vec<HourlyPerformance>, StorageError> {
    let rows: Vec<(i64, f64, i64)> = sqlx::query_as(
        "SELECT
            CAST(strftime('%H', ot.created_at) AS INTEGER) as hour,
            COALESCE(AVG(tp.performance_score), 0.0) as avg_engagement,
            COUNT(*) as post_count
         FROM original_tweets ot
         LEFT JOIN tweet_performance tp ON tp.tweet_id = ot.tweet_id
         WHERE ot.status = 'sent' AND ot.tweet_id IS NOT NULL
         GROUP BY hour
         ORDER BY avg_engagement DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|(hour, avg_engagement, post_count)| HourlyPerformance {
            hour,
            avg_engagement,
            post_count,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
