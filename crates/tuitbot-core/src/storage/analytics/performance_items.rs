use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;

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

/// Get recent content performance items for a specific account, newest first.
///
/// Unions reply and tweet performance joined with their content tables
/// so the dashboard can show a content preview alongside metrics.
pub async fn get_recent_performance_items_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<PerformanceItem>, StorageError> {
    let rows: Vec<PerformanceRow> = sqlx::query_as(
        "SELECT 'reply' as content_type, \
                SUBSTR(rs.reply_content, 1, 120) as content_preview, \
                rp.likes_received, rp.replies_received, 0 as retweets, \
                rp.impressions, rp.performance_score, rs.created_at as posted_at \
         FROM reply_performance rp \
         JOIN replies_sent rs ON rs.reply_tweet_id = rp.reply_id \
         WHERE rp.account_id = ? \
         UNION ALL \
         SELECT 'tweet' as content_type, \
                SUBSTR(ot.content, 1, 120) as content_preview, \
                tp.likes_received, tp.replies_received, tp.retweets_received, \
                tp.impressions, tp.performance_score, ot.created_at as posted_at \
         FROM tweet_performance tp \
         JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
         WHERE tp.account_id = ? \
         ORDER BY posted_at DESC \
         LIMIT ?",
    )
    .bind(account_id)
    .bind(account_id)
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

/// Get recent content performance items, newest first.
///
/// Unions reply and tweet performance joined with their content tables
/// so the dashboard can show a content preview alongside metrics.
pub async fn get_recent_performance_items(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<PerformanceItem>, StorageError> {
    get_recent_performance_items_for(pool, DEFAULT_ACCOUNT_ID, limit).await
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

/// Get optimal posting times based on historical performance for a specific account.
pub async fn get_optimal_posting_times_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<HourlyPerformance>, StorageError> {
    let rows: Vec<(i64, f64, i64)> = sqlx::query_as(
        "SELECT
            CAST(strftime('%H', ot.created_at) AS INTEGER) as hour,
            COALESCE(AVG(tp.performance_score), 0.0) as avg_engagement,
            COUNT(*) as post_count
         FROM original_tweets ot
         LEFT JOIN tweet_performance tp ON tp.tweet_id = ot.tweet_id
         WHERE ot.account_id = ? AND ot.status = 'sent' AND ot.tweet_id IS NOT NULL
         GROUP BY hour
         ORDER BY avg_engagement DESC",
    )
    .bind(account_id)
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

/// Get optimal posting times based on historical performance.
pub async fn get_optimal_posting_times(
    pool: &DbPool,
) -> Result<Vec<HourlyPerformance>, StorageError> {
    get_optimal_posting_times_for(pool, DEFAULT_ACCOUNT_ID).await
}
