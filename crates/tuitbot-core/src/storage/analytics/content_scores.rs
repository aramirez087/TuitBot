use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;

/// A topic/format performance score.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContentScore {
    pub topic: String,
    pub format: String,
    pub total_posts: i64,
    pub avg_performance: f64,
}

/// Update the running average for a topic/format pair for a specific account.
///
/// Uses incremental mean: new_avg = old_avg + (score - old_avg) / new_count.
pub async fn update_content_score_for(
    pool: &DbPool,
    account_id: &str,
    topic: &str,
    format: &str,
    new_score: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO content_scores (account_id, topic, format, total_posts, avg_performance) \
         VALUES (?, ?, ?, 1, ?) \
         ON CONFLICT(topic, format) DO UPDATE SET \
         account_id = excluded.account_id, \
         total_posts = content_scores.total_posts + 1, \
         avg_performance = content_scores.avg_performance + \
         (? - content_scores.avg_performance) / (content_scores.total_posts + 1)",
    )
    .bind(account_id)
    .bind(topic)
    .bind(format)
    .bind(new_score)
    .bind(new_score)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
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
    update_content_score_for(pool, DEFAULT_ACCOUNT_ID, topic, format, new_score).await
}

/// Get top-performing topics for a specific account ordered by average performance descending.
pub async fn get_top_topics_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<ContentScore>, StorageError> {
    let rows: Vec<(String, String, i64, f64)> = sqlx::query_as(
        "SELECT topic, format, total_posts, avg_performance \
         FROM content_scores \
         WHERE account_id = ? \
         ORDER BY avg_performance DESC \
         LIMIT ?",
    )
    .bind(account_id)
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

/// Get top-performing topics ordered by average performance descending.
pub async fn get_top_topics(pool: &DbPool, limit: u32) -> Result<Vec<ContentScore>, StorageError> {
    get_top_topics_for(pool, DEFAULT_ACCOUNT_ID, limit).await
}

/// Get average reply engagement rate for a specific account (avg performance_score across all measured replies).
pub async fn get_avg_reply_engagement_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<f64, StorageError> {
    let row: (f64,) = sqlx::query_as(
        "SELECT COALESCE(AVG(performance_score), 0.0) FROM reply_performance WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get average reply engagement rate (avg performance_score across all measured replies).
pub async fn get_avg_reply_engagement(pool: &DbPool) -> Result<f64, StorageError> {
    get_avg_reply_engagement_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Get average tweet engagement rate for a specific account (avg performance_score across all measured tweets).
pub async fn get_avg_tweet_engagement_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<f64, StorageError> {
    let row: (f64,) = sqlx::query_as(
        "SELECT COALESCE(AVG(performance_score), 0.0) FROM tweet_performance WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get average tweet engagement rate (avg performance_score across all measured tweets).
pub async fn get_avg_tweet_engagement(pool: &DbPool) -> Result<f64, StorageError> {
    get_avg_tweet_engagement_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Get total count of measured replies and tweets for a specific account.
pub async fn get_performance_counts_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<(i64, i64), StorageError> {
    let reply_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM reply_performance WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    let tweet_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM tweet_performance WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok((reply_count.0, tweet_count.0))
}

/// Get total count of measured replies and tweets.
pub async fn get_performance_counts(pool: &DbPool) -> Result<(i64, i64), StorageError> {
    get_performance_counts_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Compute the performance score for a piece of content.
///
/// Formula: `(likes * 3 + replies * 5 + retweets * 4) / max(impressions, 1) * 1000`
pub fn compute_performance_score(likes: i64, replies: i64, retweets: i64, impressions: i64) -> f64 {
    let numerator = (likes * 3 + replies * 5 + retweets * 4) as f64;
    let denominator = impressions.max(1) as f64;
    numerator / denominator * 1000.0
}
