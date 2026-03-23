use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;

/// Store or update tweet performance metrics for a specific account.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_tweet_performance_for(
    pool: &DbPool,
    account_id: &str,
    tweet_id: &str,
    likes: i64,
    retweets: i64,
    replies: i64,
    impressions: i64,
    score: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO tweet_performance (account_id, tweet_id, likes_received, retweets_received, replies_received, impressions, performance_score) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(tweet_id) DO UPDATE SET \
         likes_received = excluded.likes_received, \
         retweets_received = excluded.retweets_received, \
         replies_received = excluded.replies_received, \
         impressions = excluded.impressions, \
         performance_score = excluded.performance_score, \
         measured_at = datetime('now')",
    )
    .bind(account_id)
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
    upsert_tweet_performance_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        tweet_id,
        likes,
        retweets,
        replies,
        impressions,
        score,
    )
    .await
}

/// Performance metrics row from the tweet_performance table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TweetPerformanceRow {
    pub tweet_id: String,
    pub likes_received: i64,
    pub retweets_received: i64,
    pub replies_received: i64,
    pub impressions: i64,
    pub performance_score: f64,
}

/// Get performance metrics for multiple tweet IDs in one query.
pub async fn get_tweet_performances_for(
    pool: &DbPool,
    account_id: &str,
    tweet_ids: &[String],
) -> Result<Vec<TweetPerformanceRow>, StorageError> {
    if tweet_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = tweet_ids.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT tweet_id, likes_received, retweets_received, replies_received, \
         impressions, performance_score \
         FROM tweet_performance \
         WHERE account_id = ? AND tweet_id IN ({})",
        placeholders.join(", ")
    );

    let mut query = sqlx::query_as::<_, TweetPerformanceRow>(&sql).bind(account_id);
    for id in tweet_ids {
        query = query.bind(id);
    }

    query
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Get all tweet performance rows for an account.
pub async fn get_all_tweet_performances_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<TweetPerformanceRow>, StorageError> {
    sqlx::query_as::<_, TweetPerformanceRow>(
        "SELECT tweet_id, likes_received, retweets_received, replies_received, \
         impressions, performance_score \
         FROM tweet_performance \
         WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}
