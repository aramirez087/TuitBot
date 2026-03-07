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
