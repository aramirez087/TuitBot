//! Date-ranged metric queries over existing tables.
//!
//! All functions query existing tables (`action_log`, `follower_snapshots`,
//! `reply_performance`, `tweet_performance`, `original_tweets`, `replies_sent`)
//! with date bounds. No new data collection is needed.

use crate::error::StorageError;
use crate::storage::DbPool;

/// Action counts for a date range.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ActionCounts {
    pub replies: i64,
    pub tweets: i64,
    pub threads: i64,
    pub target_replies: i64,
}

/// A topic's performance within a date range.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopicPerformance {
    pub topic: String,
    pub format: String,
    pub avg_score: f64,
    pub post_count: i64,
}

/// A top-performing content item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentHighlight {
    pub content_type: String,
    pub content_preview: String,
    pub performance_score: f64,
    pub likes: i64,
    pub replies_received: i64,
}

/// Count actions by type within a date range.
///
/// Queries the `action_log` table. The range is `[start, end)`.
pub async fn count_actions_in_range(
    pool: &DbPool,
    start: &str,
    end: &str,
) -> Result<ActionCounts, StorageError> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT action_type, COUNT(*) as cnt FROM action_log \
         WHERE created_at >= ? AND created_at < ? AND status = 'success' \
         GROUP BY action_type",
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let mut counts = ActionCounts::default();
    for (action_type, count) in rows {
        match action_type.as_str() {
            "reply" => counts.replies = count,
            "tweet" => counts.tweets = count,
            "thread" => counts.threads = count,
            "target_reply" => counts.target_replies = count,
            _ => {}
        }
    }
    Ok(counts)
}

/// Get the follower count at or before a given date.
///
/// Returns the nearest snapshot whose date is `<= date`.
pub async fn get_follower_at_date(pool: &DbPool, date: &str) -> Result<Option<i64>, StorageError> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT follower_count FROM follower_snapshots \
         WHERE snapshot_date <= ? ORDER BY snapshot_date DESC LIMIT 1",
    )
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| r.0))
}

/// Average reply performance score in a date range.
pub async fn avg_reply_score_in_range(
    pool: &DbPool,
    start: &str,
    end: &str,
) -> Result<f64, StorageError> {
    let row: (f64,) = sqlx::query_as(
        "SELECT COALESCE(AVG(rp.performance_score), 0.0) \
         FROM reply_performance rp \
         JOIN replies_sent rs ON rs.reply_tweet_id = rp.reply_id \
         WHERE rs.created_at >= ? AND rs.created_at < ?",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Average tweet performance score in a date range.
pub async fn avg_tweet_score_in_range(
    pool: &DbPool,
    start: &str,
    end: &str,
) -> Result<f64, StorageError> {
    let row: (f64,) = sqlx::query_as(
        "SELECT COALESCE(AVG(tp.performance_score), 0.0) \
         FROM tweet_performance tp \
         JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
         WHERE ot.created_at >= ? AND ot.created_at < ?",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Reply acceptance rate: fraction of replies that received at least one reply back.
pub async fn reply_acceptance_rate(
    pool: &DbPool,
    start: &str,
    end: &str,
) -> Result<f64, StorageError> {
    let row: (i64, i64) = sqlx::query_as(
        "SELECT \
            COUNT(*) as total, \
            SUM(CASE WHEN rp.replies_received > 0 THEN 1 ELSE 0 END) as accepted \
         FROM replies_sent rs \
         JOIN reply_performance rp ON rp.reply_id = rs.reply_tweet_id \
         WHERE rs.created_at >= ? AND rs.created_at < ?",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    if row.0 == 0 {
        return Ok(0.0);
    }
    Ok(row.1 as f64 / row.0 as f64)
}

/// Top topics by average performance score in a date range.
pub async fn top_topics_in_range(
    pool: &DbPool,
    start: &str,
    end: &str,
    limit: u32,
) -> Result<Vec<TopicPerformance>, StorageError> {
    let rows: Vec<(String, String, f64, i64)> = sqlx::query_as(
        "SELECT ot.topic, COALESCE(ot.topic, '') as format, \
                AVG(tp.performance_score) as avg_score, COUNT(*) as post_count \
         FROM tweet_performance tp \
         JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
         WHERE ot.created_at >= ? AND ot.created_at < ? AND ot.topic IS NOT NULL \
         GROUP BY ot.topic \
         HAVING post_count >= 1 \
         ORDER BY avg_score DESC \
         LIMIT ?",
    )
    .bind(start)
    .bind(end)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| TopicPerformance {
            topic: r.0,
            format: r.1,
            avg_score: r.2,
            post_count: r.3,
        })
        .collect())
}

/// Bottom topics by average performance score in a date range (minimum 3 posts).
pub async fn bottom_topics_in_range(
    pool: &DbPool,
    start: &str,
    end: &str,
    limit: u32,
) -> Result<Vec<TopicPerformance>, StorageError> {
    let rows: Vec<(String, String, f64, i64)> = sqlx::query_as(
        "SELECT ot.topic, COALESCE(ot.topic, '') as format, \
                AVG(tp.performance_score) as avg_score, COUNT(*) as post_count \
         FROM tweet_performance tp \
         JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
         WHERE ot.created_at >= ? AND ot.created_at < ? AND ot.topic IS NOT NULL \
         GROUP BY ot.topic \
         HAVING post_count >= 3 \
         ORDER BY avg_score ASC \
         LIMIT ?",
    )
    .bind(start)
    .bind(end)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| TopicPerformance {
            topic: r.0,
            format: r.1,
            avg_score: r.2,
            post_count: r.3,
        })
        .collect())
}

/// Top-performing content items (UNION of replies + tweets) in a date range.
pub async fn top_content_in_range(
    pool: &DbPool,
    start: &str,
    end: &str,
    limit: u32,
) -> Result<Vec<ContentHighlight>, StorageError> {
    let rows: Vec<(String, String, f64, i64, i64)> = sqlx::query_as(
        "SELECT content_type, content_preview, performance_score, likes, replies_received FROM ( \
            SELECT 'reply' as content_type, \
                   SUBSTR(rs.reply_content, 1, 120) as content_preview, \
                   rp.performance_score, rp.likes_received as likes, \
                   rp.replies_received, rs.created_at as posted_at \
            FROM reply_performance rp \
            JOIN replies_sent rs ON rs.reply_tweet_id = rp.reply_id \
            WHERE rs.created_at >= ? AND rs.created_at < ? \
            UNION ALL \
            SELECT 'tweet' as content_type, \
                   SUBSTR(ot.content, 1, 120) as content_preview, \
                   tp.performance_score, tp.likes_received as likes, \
                   tp.replies_received, ot.created_at as posted_at \
            FROM tweet_performance tp \
            JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
            WHERE ot.created_at >= ? AND ot.created_at < ? \
         ) ORDER BY performance_score DESC LIMIT ?",
    )
    .bind(start)
    .bind(end)
    .bind(start)
    .bind(end)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| ContentHighlight {
            content_type: r.0,
            content_preview: r.1,
            performance_score: r.2,
            likes: r.3,
            replies_received: r.4,
        })
        .collect())
}

/// Count distinct topics posted in a date range.
pub async fn distinct_topic_count(
    pool: &DbPool,
    start: &str,
    end: &str,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT topic) FROM original_tweets \
         WHERE created_at >= ? AND created_at < ? AND topic IS NOT NULL AND topic != ''",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn count_actions_empty() {
        let pool = init_test_db().await.expect("init db");
        let counts = count_actions_in_range(&pool, "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z")
            .await
            .expect("count");
        assert_eq!(counts.replies, 0);
        assert_eq!(counts.tweets, 0);
    }

    #[tokio::test]
    async fn follower_at_date_empty() {
        let pool = init_test_db().await.expect("init db");
        let count = get_follower_at_date(&pool, "2026-12-31")
            .await
            .expect("get");
        assert!(count.is_none());
    }

    #[tokio::test]
    async fn avg_reply_score_empty() {
        let pool = init_test_db().await.expect("init db");
        let score = avg_reply_score_in_range(&pool, "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z")
            .await
            .expect("avg");
        assert!((score - 0.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn reply_acceptance_rate_empty() {
        let pool = init_test_db().await.expect("init db");
        let rate = reply_acceptance_rate(&pool, "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z")
            .await
            .expect("rate");
        assert!((rate - 0.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn top_topics_empty() {
        let pool = init_test_db().await.expect("init db");
        let topics = top_topics_in_range(&pool, "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z", 5)
            .await
            .expect("topics");
        assert!(topics.is_empty());
    }

    #[tokio::test]
    async fn top_content_empty() {
        let pool = init_test_db().await.expect("init db");
        let items = top_content_in_range(&pool, "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z", 5)
            .await
            .expect("content");
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn distinct_topic_count_empty() {
        let pool = init_test_db().await.expect("init db");
        let count = distinct_topic_count(&pool, "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z")
            .await
            .expect("count");
        assert_eq!(count, 0);
    }
}
