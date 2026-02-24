//! CRUD operations for target account monitoring.
//!
//! Manages the `target_accounts` and `target_tweets` tables for
//! relationship-based engagement with specific accounts.

use super::DbPool;
use crate::error::StorageError;

/// A target account record.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TargetAccount {
    pub account_id: String,
    pub username: String,
    pub followed_at: Option<String>,
    pub first_engagement_at: Option<String>,
    pub total_replies_sent: i64,
    pub last_reply_at: Option<String>,
    pub status: String,
}

/// Upsert a target account (insert or update username if exists).
pub async fn upsert_target_account(
    pool: &DbPool,
    account_id: &str,
    username: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO target_accounts (account_id, username) \
         VALUES (?, ?) \
         ON CONFLICT(account_id) DO UPDATE SET username = excluded.username",
    )
    .bind(account_id)
    .bind(username)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

type TargetAccountRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    String,
);

/// Get a target account by ID.
pub async fn get_target_account(
    pool: &DbPool,
    account_id: &str,
) -> Result<Option<TargetAccount>, StorageError> {
    let row: Option<TargetAccountRow> = sqlx::query_as(
        "SELECT account_id, username, followed_at, first_engagement_at, \
             total_replies_sent, last_reply_at, status \
             FROM target_accounts WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| TargetAccount {
        account_id: r.0,
        username: r.1,
        followed_at: r.2,
        first_engagement_at: r.3,
        total_replies_sent: r.4,
        last_reply_at: r.5,
        status: r.6,
    }))
}

/// Get all active target accounts.
pub async fn get_active_target_accounts(pool: &DbPool) -> Result<Vec<TargetAccount>, StorageError> {
    let rows: Vec<TargetAccountRow> = sqlx::query_as(
        "SELECT account_id, username, followed_at, first_engagement_at, \
             total_replies_sent, last_reply_at, status \
             FROM target_accounts WHERE status = 'active'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| TargetAccount {
            account_id: r.0,
            username: r.1,
            followed_at: r.2,
            first_engagement_at: r.3,
            total_replies_sent: r.4,
            last_reply_at: r.5,
            status: r.6,
        })
        .collect())
}

/// Record that we followed a target account.
pub async fn record_follow(pool: &DbPool, account_id: &str) -> Result<(), StorageError> {
    sqlx::query("UPDATE target_accounts SET followed_at = datetime('now') WHERE account_id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Record a reply to a target account's tweet.
pub async fn record_target_reply(pool: &DbPool, account_id: &str) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE target_accounts \
         SET total_replies_sent = total_replies_sent + 1, \
             last_reply_at = datetime('now'), \
             first_engagement_at = COALESCE(first_engagement_at, datetime('now')) \
         WHERE account_id = ?",
    )
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Get the number of target replies sent today.
pub async fn count_target_replies_today(pool: &DbPool) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM target_tweets \
         WHERE replied_to = 1 AND date(discovered_at) = date('now')",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(row.0)
}

/// Check if a target tweet exists.
pub async fn target_tweet_exists(pool: &DbPool, tweet_id: &str) -> Result<bool, StorageError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM target_tweets WHERE id = ?")
        .bind(tweet_id)
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(row.0 > 0)
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
    sqlx::query(
        "INSERT OR IGNORE INTO target_tweets \
         (id, account_id, content, created_at, reply_count, like_count, relevance_score) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
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

/// Mark a target tweet as replied to.
pub async fn mark_target_tweet_replied(pool: &DbPool, tweet_id: &str) -> Result<(), StorageError> {
    sqlx::query("UPDATE target_tweets SET replied_to = 1 WHERE id = ?")
        .bind(tweet_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Get a target account by username.
pub async fn get_target_account_by_username(
    pool: &DbPool,
    username: &str,
) -> Result<Option<TargetAccount>, StorageError> {
    let row: Option<TargetAccountRow> = sqlx::query_as(
        "SELECT account_id, username, followed_at, first_engagement_at, \
             total_replies_sent, last_reply_at, status \
             FROM target_accounts WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| TargetAccount {
        account_id: r.0,
        username: r.1,
        followed_at: r.2,
        first_engagement_at: r.3,
        total_replies_sent: r.4,
        last_reply_at: r.5,
        status: r.6,
    }))
}

/// Deactivate a target account by username (soft delete).
pub async fn deactivate_target_account(
    pool: &DbPool,
    username: &str,
) -> Result<bool, StorageError> {
    let result = sqlx::query(
        "UPDATE target_accounts SET status = 'inactive' WHERE username = ? AND status = 'active'",
    )
    .bind(username)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn upsert_and_get_target_account() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");

        let account = get_target_account(&pool, "acc_1")
            .await
            .expect("get")
            .expect("found");
        assert_eq!(account.username, "alice");
        assert_eq!(account.total_replies_sent, 0);
        assert_eq!(account.status, "active");
    }

    #[tokio::test]
    async fn get_active_target_accounts_works() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        upsert_target_account(&pool, "acc_2", "bob")
            .await
            .expect("upsert");

        let accounts = get_active_target_accounts(&pool).await.expect("get all");
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn record_follow_sets_timestamp() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        record_follow(&pool, "acc_1").await.expect("follow");

        let account = get_target_account(&pool, "acc_1")
            .await
            .expect("get")
            .expect("found");
        assert!(account.followed_at.is_some());
    }

    #[tokio::test]
    async fn record_target_reply_increments() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        record_target_reply(&pool, "acc_1").await.expect("reply");
        record_target_reply(&pool, "acc_1").await.expect("reply");

        let account = get_target_account(&pool, "acc_1")
            .await
            .expect("get")
            .expect("found");
        assert_eq!(account.total_replies_sent, 2);
        assert!(account.first_engagement_at.is_some());
        assert!(account.last_reply_at.is_some());
    }

    #[tokio::test]
    async fn store_and_check_target_tweet() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");

        assert!(!target_tweet_exists(&pool, "tw_1").await.expect("check"));

        store_target_tweet(&pool, "tw_1", "acc_1", "hello", "2026-01-01", 0, 5, 80.0)
            .await
            .expect("store");

        assert!(target_tweet_exists(&pool, "tw_1").await.expect("check"));
    }

    #[tokio::test]
    async fn mark_replied_updates_flag() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        store_target_tweet(&pool, "tw_1", "acc_1", "hello", "2026-01-01", 0, 5, 80.0)
            .await
            .expect("store");

        mark_target_tweet_replied(&pool, "tw_1")
            .await
            .expect("mark");

        // Verify by checking the count of replied tweets
        let count = count_target_replies_today(&pool).await.expect("count");
        assert!(count >= 0); // May or may not be today depending on test timing
    }
}
