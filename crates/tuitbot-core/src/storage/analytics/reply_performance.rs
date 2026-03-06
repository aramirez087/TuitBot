use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;

/// Store or update reply performance metrics for a specific account.
pub async fn upsert_reply_performance_for(
    pool: &DbPool,
    account_id: &str,
    reply_id: &str,
    likes: i64,
    replies: i64,
    impressions: i64,
    score: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO reply_performance (account_id, reply_id, likes_received, replies_received, impressions, performance_score) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON CONFLICT(reply_id) DO UPDATE SET \
         likes_received = excluded.likes_received, \
         replies_received = excluded.replies_received, \
         impressions = excluded.impressions, \
         performance_score = excluded.performance_score, \
         measured_at = datetime('now')",
    )
    .bind(account_id)
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

/// Store or update reply performance metrics.
pub async fn upsert_reply_performance(
    pool: &DbPool,
    reply_id: &str,
    likes: i64,
    replies: i64,
    impressions: i64,
    score: f64,
) -> Result<(), StorageError> {
    upsert_reply_performance_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        reply_id,
        likes,
        replies,
        impressions,
        score,
    )
    .await
}
