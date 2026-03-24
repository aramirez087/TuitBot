//! Write operations for target account monitoring.

use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;

/// Upsert a target account (insert or update username if exists) for a specific owner account.
pub async fn upsert_target_account_for(
    pool: &DbPool,
    owner_account_id: &str,
    account_id: &str,
    username: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO target_accounts (owner_account_id, account_id, username) \
         VALUES (?, ?, ?) \
         ON CONFLICT(account_id) DO UPDATE SET username = excluded.username",
    )
    .bind(owner_account_id)
    .bind(account_id)
    .bind(username)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Upsert a target account (insert or update username if exists).
pub async fn upsert_target_account(
    pool: &DbPool,
    account_id: &str,
    username: &str,
) -> Result<(), StorageError> {
    upsert_target_account_for(pool, DEFAULT_ACCOUNT_ID, account_id, username).await
}

/// Record a reply to a target account's tweet for a specific owner account.
pub async fn record_target_reply_for(
    pool: &DbPool,
    owner_account_id: &str,
    account_id: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE target_accounts \
         SET total_replies_sent = total_replies_sent + 1, \
             last_reply_at = datetime('now'), \
             first_engagement_at = COALESCE(first_engagement_at, datetime('now')) \
         WHERE account_id = ? AND owner_account_id = ?",
    )
    .bind(account_id)
    .bind(owner_account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Record a reply to a target account's tweet.
pub async fn record_target_reply(pool: &DbPool, account_id: &str) -> Result<(), StorageError> {
    record_target_reply_for(pool, DEFAULT_ACCOUNT_ID, account_id).await
}

/// Get the number of target replies sent today for a specific owner account.
pub async fn count_target_replies_today_for(
    pool: &DbPool,
    owner_account_id: &str,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM target_tweets \
         WHERE replied_to = 1 AND date(discovered_at) = date('now') AND owner_account_id = ?",
    )
    .bind(owner_account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(row.0)
}

/// Get the number of target replies sent today.
pub async fn count_target_replies_today(pool: &DbPool) -> Result<i64, StorageError> {
    count_target_replies_today_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Store a discovered target tweet for a specific owner account.
#[allow(clippy::too_many_arguments)]
pub async fn store_target_tweet_for(
    pool: &DbPool,
    owner_account_id: &str,
    tweet_id: &str,
    account_id: &str,
    content: &str,
    created_at: &str,
    reply_count: i64,
    like_count: i64,
    relevance_score: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO target_tweets \
         (owner_account_id, id, account_id, content, created_at, reply_count, like_count, relevance_score) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(owner_account_id)
    .bind(tweet_id)
    .bind(account_id)
    .bind(content)
    .bind(created_at)
    .bind(reply_count)
    .bind(like_count)
    .bind(relevance_score)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Store a discovered target tweet.
#[allow(clippy::too_many_arguments)]
pub async fn store_target_tweet(
    pool: &DbPool,
    tweet_id: &str,
    account_id: &str,
    content: &str,
    created_at: &str,
    reply_count: i64,
    like_count: i64,
    relevance_score: f64,
) -> Result<(), StorageError> {
    store_target_tweet_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        tweet_id,
        account_id,
        content,
        created_at,
        reply_count,
        like_count,
        relevance_score,
    )
    .await
}

/// Mark a target tweet as replied to for a specific owner account.
pub async fn mark_target_tweet_replied_for(
    pool: &DbPool,
    owner_account_id: &str,
    tweet_id: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE target_tweets SET replied_to = 1 WHERE id = ? AND owner_account_id = ?")
        .bind(tweet_id)
        .bind(owner_account_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Mark a target tweet as replied to.
pub async fn mark_target_tweet_replied(pool: &DbPool, tweet_id: &str) -> Result<(), StorageError> {
    mark_target_tweet_replied_for(pool, DEFAULT_ACCOUNT_ID, tweet_id).await
}

/// Deactivate a target account by username (soft delete) for a specific owner account.
pub async fn deactivate_target_account_for(
    pool: &DbPool,
    owner_account_id: &str,
    username: &str,
) -> Result<bool, StorageError> {
    let result = sqlx::query(
        "UPDATE target_accounts SET status = 'inactive' \
         WHERE username = ? AND status = 'active' AND owner_account_id = ?",
    )
    .bind(username)
    .bind(owner_account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(result.rows_affected() > 0)
}

/// Deactivate a target account by username (soft delete).
pub async fn deactivate_target_account(
    pool: &DbPool,
    username: &str,
) -> Result<bool, StorageError> {
    deactivate_target_account_for(pool, DEFAULT_ACCOUNT_ID, username).await
}
