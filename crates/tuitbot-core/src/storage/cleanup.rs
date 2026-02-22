//! Data retention cleanup for bounded database growth.
//!
//! Prunes old records according to retention rules while preserving
//! rate limit counters and records needed for deduplication.

use super::DbPool;
use crate::error::StorageError;
use chrono::Utc;

/// Statistics from a cleanup run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CleanupStats {
    /// Number of discovered tweets deleted.
    pub discovered_tweets_deleted: u64,
    /// Number of reply records deleted.
    pub replies_deleted: u64,
    /// Number of original tweet records deleted.
    pub original_tweets_deleted: u64,
    /// Number of thread records deleted (thread_tweets cascade).
    pub threads_deleted: u64,
    /// Number of action log entries deleted.
    pub action_log_deleted: u64,
    /// Total records deleted across all tables.
    pub total_deleted: u64,
    /// Whether VACUUM was run to reclaim disk space.
    pub vacuum_run: bool,
}

/// Run data retention cleanup, pruning old records per retention rules.
///
/// Retention rules:
/// - Unreplied discovered tweets: 7 days (fixed).
/// - Replied discovered tweets: `retention_days` (configurable, default 90).
/// - Replies: `retention_days`.
/// - Original tweets: `retention_days`.
/// - Threads: `retention_days` (CASCADE deletes thread_tweets).
/// - Action log: 14 days (fixed).
/// - Rate limits: NEVER deleted.
///
/// Runs VACUUM if more than 1000 total rows were deleted.
pub async fn run_cleanup(pool: &DbPool, retention_days: u32) -> Result<CleanupStats, StorageError> {
    let now = Utc::now();

    let unreplied_cutoff = (now - chrono::Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let replied_cutoff = (now - chrono::Duration::days(i64::from(retention_days)))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let action_log_cutoff = (now - chrono::Duration::days(14))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    // Delete child records before parent records for FK constraints.

    // 1. Delete old replies first (before their parent discovered_tweets).
    let replies_result = sqlx::query("DELETE FROM replies_sent WHERE created_at < ?")
        .bind(&replied_cutoff)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    let replies_deleted = replies_result.rows_affected();

    // 2. Delete unreplied discovered tweets older than 7 days.
    let unreplied_result =
        sqlx::query("DELETE FROM discovered_tweets WHERE replied_to = 0 AND discovered_at < ?")
            .bind(&unreplied_cutoff)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    // 3. Delete replied discovered tweets older than retention_days.
    let replied_result =
        sqlx::query("DELETE FROM discovered_tweets WHERE replied_to = 1 AND discovered_at < ?")
            .bind(&replied_cutoff)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    let discovered_tweets_deleted =
        unreplied_result.rows_affected() + replied_result.rows_affected();

    // 4. Delete old original tweets.
    let originals_result = sqlx::query("DELETE FROM original_tweets WHERE created_at < ?")
        .bind(&replied_cutoff)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    let original_tweets_deleted = originals_result.rows_affected();

    // 5. Delete old threads (CASCADE deletes thread_tweets).
    let threads_result = sqlx::query("DELETE FROM threads WHERE created_at < ?")
        .bind(&replied_cutoff)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    let threads_deleted = threads_result.rows_affected();

    // 6. Delete old action log entries.
    let action_log_result = sqlx::query("DELETE FROM action_log WHERE created_at < ?")
        .bind(&action_log_cutoff)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    let action_log_deleted = action_log_result.rows_affected();

    let total_deleted = discovered_tweets_deleted
        + replies_deleted
        + original_tweets_deleted
        + threads_deleted
        + action_log_deleted;

    let vacuum_run = if total_deleted > 1000 {
        sqlx::query("VACUUM")
            .execute(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;
        true
    } else {
        false
    };

    let stats = CleanupStats {
        discovered_tweets_deleted,
        replies_deleted,
        original_tweets_deleted,
        threads_deleted,
        action_log_deleted,
        total_deleted,
        vacuum_run,
    };

    tracing::info!(
        discovered_tweets = stats.discovered_tweets_deleted,
        replies = stats.replies_deleted,
        original_tweets = stats.original_tweets_deleted,
        threads = stats.threads_deleted,
        action_log = stats.action_log_deleted,
        total = stats.total_deleted,
        vacuum = stats.vacuum_run,
        "Cleanup completed"
    );

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    /// Insert a discovered tweet with a specific timestamp and replied_to status.
    async fn insert_tweet_at(pool: &DbPool, id: &str, discovered_at: &str, replied_to: i64) {
        sqlx::query(
            "INSERT INTO discovered_tweets \
             (id, author_id, author_username, content, discovered_at, replied_to) \
             VALUES (?, 'u1', 'user1', 'content', ?, ?)",
        )
        .bind(id)
        .bind(discovered_at)
        .bind(replied_to)
        .execute(pool)
        .await
        .expect("insert tweet");
    }

    /// Insert a reply with a specific timestamp.
    async fn insert_reply_at(pool: &DbPool, target_id: &str, created_at: &str) {
        sqlx::query(
            "INSERT INTO replies_sent (target_tweet_id, reply_content, created_at) \
             VALUES (?, 'reply text', ?)",
        )
        .bind(target_id)
        .bind(created_at)
        .execute(pool)
        .await
        .expect("insert reply");
    }

    /// Insert an action log entry with a specific timestamp.
    async fn insert_action_at(pool: &DbPool, created_at: &str) {
        sqlx::query(
            "INSERT INTO action_log (action_type, status, created_at) \
             VALUES ('search', 'success', ?)",
        )
        .bind(created_at)
        .execute(pool)
        .await
        .expect("insert action");
    }

    #[tokio::test]
    async fn cleanup_deletes_unreplied_tweets_older_than_7_days() {
        let pool = init_test_db().await.expect("init db");

        // Old unreplied tweet (10 days ago)
        insert_tweet_at(&pool, "old_unreplied", "2020-01-01T00:00:00Z", 0).await;
        // Recent unreplied tweet (now)
        insert_tweet_at(
            &pool,
            "recent_unreplied",
            &Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            0,
        )
        .await;

        let stats = run_cleanup(&pool, 90).await.expect("cleanup");
        assert_eq!(stats.discovered_tweets_deleted, 1);

        // Verify the recent one still exists
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM discovered_tweets WHERE id = 'recent_unreplied'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn cleanup_deletes_replied_tweets_older_than_retention() {
        let pool = init_test_db().await.expect("init db");

        // Old replied tweet (100 days ago)
        insert_tweet_at(&pool, "old_replied", "2020-01-01T00:00:00Z", 1).await;
        // Recent replied tweet (now)
        insert_tweet_at(
            &pool,
            "recent_replied",
            &Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            1,
        )
        .await;

        let stats = run_cleanup(&pool, 90).await.expect("cleanup");
        assert_eq!(stats.discovered_tweets_deleted, 1);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM discovered_tweets WHERE id = 'recent_replied'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn cleanup_deletes_old_replies() {
        let pool = init_test_db().await.expect("init db");

        insert_reply_at(&pool, "t1", "2020-01-01T00:00:00Z").await;
        insert_reply_at(
            &pool,
            "t2",
            &Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        )
        .await;

        let stats = run_cleanup(&pool, 90).await.expect("cleanup");
        assert_eq!(stats.replies_deleted, 1);
    }

    #[tokio::test]
    async fn cleanup_deletes_old_action_log_entries() {
        let pool = init_test_db().await.expect("init db");

        // 15 days ago
        insert_action_at(&pool, "2020-01-01T00:00:00Z").await;
        // Recent
        insert_action_at(&pool, &Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()).await;

        let stats = run_cleanup(&pool, 90).await.expect("cleanup");
        assert_eq!(stats.action_log_deleted, 1);
    }

    #[tokio::test]
    async fn cleanup_never_deletes_rate_limits() {
        let pool = init_test_db().await.expect("init db");

        // Insert a rate limit row
        sqlx::query(
            "INSERT INTO rate_limits (action_type, request_count, period_start, max_requests, period_seconds) \
             VALUES ('reply', 5, '2020-01-01T00:00:00Z', 20, 86400)",
        )
        .execute(&pool)
        .await
        .expect("insert rate limit");

        run_cleanup(&pool, 90).await.expect("cleanup");

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rate_limits")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count.0, 1, "rate_limits should never be deleted");
    }

    #[tokio::test]
    async fn cleanup_empty_database_returns_zero_stats() {
        let pool = init_test_db().await.expect("init db");

        let stats = run_cleanup(&pool, 90).await.expect("cleanup");
        assert_eq!(stats.total_deleted, 0);
        assert!(!stats.vacuum_run);
    }

    #[tokio::test]
    async fn cleanup_deletes_old_threads_with_cascade() {
        let pool = init_test_db().await.expect("init db");

        // Insert an old thread
        sqlx::query(
            "INSERT INTO threads (topic, tweet_count, created_at, status) \
             VALUES ('old topic', 3, '2020-01-01T00:00:00Z', 'sent')",
        )
        .execute(&pool)
        .await
        .expect("insert thread");

        // Insert thread tweets (should cascade delete)
        sqlx::query(
            "INSERT INTO thread_tweets (thread_id, position, content, created_at) \
             VALUES (1, 0, 'tweet 0', '2020-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert thread tweet");

        let stats = run_cleanup(&pool, 90).await.expect("cleanup");
        assert_eq!(stats.threads_deleted, 1);

        // Verify thread_tweets were cascaded
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM thread_tweets")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count.0, 0);
    }
}
