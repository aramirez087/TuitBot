//! CRUD operations for analytics tables.
//!
//! Manages follower snapshots, reply/tweet performance metrics,
//! and content score running averages.

use super::DbPool;
use crate::error::StorageError;

// ============================================================================
// Follower snapshots
// ============================================================================

/// A daily follower snapshot.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
}
