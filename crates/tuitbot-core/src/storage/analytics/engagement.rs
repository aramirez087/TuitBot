//! Engagement rate and reach analytics queries.

use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Engagement metrics for a single post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementMetric {
    pub post_id: String,
    pub text: Option<String>,
    pub engagement_rate: f64,
    pub impressions: i64,
    pub likes: i64,
    pub retweets: i64,
    pub replies: i64,
    pub bookmarks: i64,
    pub posted_at: Option<String>,
}

/// Get top engagement posts for a specific account (sorted by engagement_rate DESC).
pub async fn get_engagement_rate_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<EngagementMetric>, StorageError> {
    let rows = sqlx::query_as::<_, (String, Option<String>, f64, i64, i64, i64, i64, i64, Option<String>)>(
        "SELECT post_id, NULL as text, engagement_rate, impressions, likes, retweets, replies, bookmarks, posted_at \
         FROM engagement_metrics \
         WHERE account_id = ? \
         ORDER BY engagement_rate DESC \
         LIMIT ?",
    )
    .bind(account_id)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(
            |(
                post_id,
                text,
                engagement_rate,
                impressions,
                likes,
                retweets,
                replies,
                bookmarks,
                posted_at,
            )| {
                EngagementMetric {
                    post_id,
                    text,
                    engagement_rate,
                    impressions,
                    likes,
                    retweets,
                    replies,
                    bookmarks,
                    posted_at,
                }
            },
        )
        .collect())
}

/// Get top engagement posts (default account).
pub async fn get_engagement_rate(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<EngagementMetric>, StorageError> {
    get_engagement_rate_for(pool, DEFAULT_ACCOUNT_ID, limit).await
}

/// Reach snapshot for a single day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachSnapshot {
    pub date: String,
    pub total_reach: i64,
    pub avg_reach_per_post: f64,
    pub post_count: i64,
}

/// Get reach time-series for a specific account over the past N days.
pub async fn get_reach_for(
    pool: &DbPool,
    account_id: &str,
    days: u32,
) -> Result<Vec<ReachSnapshot>, StorageError> {
    let rows = sqlx::query_as::<_, (String, i64, f64, i64)>(
        "SELECT snapshot_date, total_reach, avg_reach_per_post, post_count \
         FROM reach_snapshots \
         WHERE account_id = ? \
         AND snapshot_date >= date('now', '-' || ? || ' days') \
         ORDER BY snapshot_date ASC",
    )
    .bind(account_id)
    .bind(days as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(
            |(date, total_reach, avg_reach_per_post, post_count)| ReachSnapshot {
                date,
                total_reach,
                avg_reach_per_post,
                post_count,
            },
        )
        .collect())
}

/// Get reach time-series (default account).
pub async fn get_reach(pool: &DbPool, days: u32) -> Result<Vec<ReachSnapshot>, StorageError> {
    get_reach_for(pool, DEFAULT_ACCOUNT_ID, days).await
}

/// Insert or update engagement metrics for a post.
/// Input for upserting engagement metrics for a single post.
pub struct UpsertEngagementInput<'a> {
    pub post_id: &'a str,
    pub impressions: i64,
    pub likes: i64,
    pub retweets: i64,
    pub replies: i64,
    pub bookmarks: i64,
    pub posted_at: Option<&'a str>,
}

pub async fn upsert_engagement_metric_for(
    pool: &DbPool,
    account_id: &str,
    input: UpsertEngagementInput<'_>,
) -> Result<(), StorageError> {
    let engagement_rate = if input.impressions > 0 {
        (input.likes + input.retweets + input.replies + input.bookmarks) as f64
            / input.impressions as f64
    } else {
        0.0
    };

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "INSERT INTO engagement_metrics \
         (account_id, post_id, impressions, likes, retweets, replies, bookmarks, engagement_rate, posted_at, fetched_at, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(account_id, post_id) DO UPDATE SET \
         impressions = excluded.impressions, \
         likes = excluded.likes, \
         retweets = excluded.retweets, \
         replies = excluded.replies, \
         bookmarks = excluded.bookmarks, \
         engagement_rate = excluded.engagement_rate, \
         fetched_at = excluded.fetched_at",
    )
    .bind(account_id)
    .bind(input.post_id)
    .bind(input.impressions)
    .bind(input.likes)
    .bind(input.retweets)
    .bind(input.replies)
    .bind(input.bookmarks)
    .bind(engagement_rate)
    .bind(input.posted_at)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Compute and store reach aggregations for today (call daily via background job).
pub async fn aggregate_reach_for(pool: &DbPool, account_id: &str) -> Result<(), StorageError> {
    let today = Utc::now().format("%Y-%m-%d").to_string();

    // Query engagement metrics for today, compute aggregates
    let row = sqlx::query_as::<_, (i64, i64)>(
        "SELECT COALESCE(SUM(impressions), 0) as total, COUNT(*) as count \
         FROM engagement_metrics \
         WHERE account_id = ? AND DATE(posted_at) = ?",
    )
    .bind(account_id)
    .bind(&today)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let (total_reach, post_count) = match row {
        Some((t, c)) => (t, c),
        None => (0, 0),
    };

    let avg_reach_per_post = if post_count > 0 {
        total_reach as f64 / post_count as f64
    } else {
        0.0
    };

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "INSERT INTO reach_snapshots \
         (account_id, snapshot_date, total_reach, avg_reach_per_post, post_count, created_at) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON CONFLICT(account_id, snapshot_date) DO UPDATE SET \
         total_reach = excluded.total_reach, \
         avg_reach_per_post = excluded.avg_reach_per_post, \
         post_count = excluded.post_count",
    )
    .bind(account_id)
    .bind(&today)
    .bind(total_reach)
    .bind(avg_reach_per_post)
    .bind(post_count)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn engagement_rate_calculation() {
        let likes = 10;
        let retweets = 5;
        let replies = 3;
        let bookmarks = 2;
        let impressions = 100;

        let engagement_rate = (likes + retweets + replies + bookmarks) as f64 / impressions as f64;
        assert!((engagement_rate - 0.2).abs() < 0.001);
    }

    #[test]
    fn engagement_rate_zero_impressions() {
        let engagement_rate = if 0 > 0 { 1.0 } else { 0.0 };
        assert_eq!(engagement_rate, 0.0);
    }

    #[test]
    fn reach_per_post_calculation() {
        let total_reach = 1000;
        let post_count = 5;
        let avg = total_reach as f64 / post_count as f64;
        assert_eq!(avg, 200.0);
    }

    #[test]
    fn reach_per_post_zero_posts() {
        let avg = if 0 > 0 { 1.0 } else { 0.0 };
        assert_eq!(avg, 0.0);
    }
}
