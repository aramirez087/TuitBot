//! CRUD operations for original tweets and educational threads.
//!
//! Provides functions to insert and query original tweets and threads,
//! supporting the content and thread automation loops.

use super::DbPool;
use crate::error::StorageError;

/// An educational tweet generated and posted by the agent.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct OriginalTweet {
    /// Internal auto-generated ID.
    pub id: i64,
    /// X tweet ID after posting (None if failed).
    pub tweet_id: Option<String>,
    /// Tweet text.
    pub content: String,
    /// Industry topic this covers.
    pub topic: Option<String>,
    /// Which LLM generated this.
    pub llm_provider: Option<String>,
    /// ISO-8601 UTC timestamp when tweet was posted.
    pub created_at: String,
    /// Status: sent or failed.
    pub status: String,
    /// Error details if failed.
    pub error_message: Option<String>,
}

/// A series of connected tweets posted as a thread.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct Thread {
    /// Internal auto-generated ID.
    pub id: i64,
    /// Thread topic.
    pub topic: String,
    /// Number of tweets in thread.
    pub tweet_count: i64,
    /// X tweet ID of first tweet.
    pub root_tweet_id: Option<String>,
    /// ISO-8601 UTC timestamp when thread was posted.
    pub created_at: String,
    /// Status: sent, partial, or failed.
    pub status: String,
}

/// An individual tweet within a thread.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ThreadTweet {
    /// Internal auto-generated ID.
    pub id: i64,
    /// Parent thread ID.
    pub thread_id: i64,
    /// 0-indexed position in thread.
    pub position: i64,
    /// X tweet ID after posting.
    pub tweet_id: Option<String>,
    /// Tweet text.
    pub content: String,
    /// ISO-8601 UTC timestamp.
    pub created_at: String,
}

/// Insert a new original tweet. Returns the auto-generated ID.
pub async fn insert_original_tweet(
    pool: &DbPool,
    tweet: &OriginalTweet,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO original_tweets \
         (tweet_id, content, topic, llm_provider, created_at, status, error_message) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&tweet.tweet_id)
    .bind(&tweet.content)
    .bind(&tweet.topic)
    .bind(&tweet.llm_provider)
    .bind(&tweet.created_at)
    .bind(&tweet.status)
    .bind(&tweet.error_message)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Get the timestamp of the most recent successfully posted original tweet.
pub async fn get_last_original_tweet_time(pool: &DbPool) -> Result<Option<String>, StorageError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM original_tweets WHERE status = 'sent' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| r.0))
}

/// Count original tweets posted today (UTC).
pub async fn count_tweets_today(pool: &DbPool) -> Result<i64, StorageError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM original_tweets WHERE date(created_at) = date('now')")
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Insert a new thread record. Returns the auto-generated ID.
pub async fn insert_thread(pool: &DbPool, thread: &Thread) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO threads (topic, tweet_count, root_tweet_id, created_at, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&thread.topic)
    .bind(thread.tweet_count)
    .bind(&thread.root_tweet_id)
    .bind(&thread.created_at)
    .bind(&thread.status)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Insert all tweets for a thread atomically using a transaction.
///
/// Either all tweets are inserted or none are (rollback on failure).
pub async fn insert_thread_tweets(
    pool: &DbPool,
    thread_id: i64,
    tweets: &[ThreadTweet],
) -> Result<(), StorageError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    for tweet in tweets {
        sqlx::query(
            "INSERT INTO thread_tweets (thread_id, position, tweet_id, content, created_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(thread_id)
        .bind(tweet.position)
        .bind(&tweet.tweet_id)
        .bind(&tweet.content)
        .bind(&tweet.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    }

    tx.commit()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    Ok(())
}

/// Get the timestamp of the most recent successfully posted thread.
pub async fn get_last_thread_time(pool: &DbPool) -> Result<Option<String>, StorageError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM threads WHERE status = 'sent' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| r.0))
}

/// Get the timestamps of all successfully posted original tweets today (UTC).
pub async fn get_todays_tweet_times(pool: &DbPool) -> Result<Vec<String>, StorageError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT created_at FROM original_tweets \
         WHERE status = 'sent' AND date(created_at) = date('now') \
         ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Count threads posted in the current ISO week (Monday-Sunday, UTC).
pub async fn count_threads_this_week(pool: &DbPool) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM threads \
         WHERE strftime('%Y-%W', created_at) = strftime('%Y-%W', 'now')",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get original tweets within a date range, ordered by creation time.
pub async fn get_tweets_in_range(
    pool: &DbPool,
    from: &str,
    to: &str,
) -> Result<Vec<OriginalTweet>, StorageError> {
    sqlx::query_as::<_, OriginalTweet>(
        "SELECT * FROM original_tweets \
         WHERE created_at BETWEEN ? AND ? \
         ORDER BY created_at ASC",
    )
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Get threads within a date range, ordered by creation time.
pub async fn get_threads_in_range(
    pool: &DbPool,
    from: &str,
    to: &str,
) -> Result<Vec<Thread>, StorageError> {
    sqlx::query_as::<_, Thread>(
        "SELECT * FROM threads \
         WHERE created_at BETWEEN ? AND ? \
         ORDER BY created_at ASC",
    )
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Get the most recent original tweets, newest first.
pub async fn get_recent_original_tweets(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<OriginalTweet>, StorageError> {
    sqlx::query_as::<_, OriginalTweet>(
        "SELECT * FROM original_tweets ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Get the most recent threads, newest first.
pub async fn get_recent_threads(pool: &DbPool, limit: u32) -> Result<Vec<Thread>, StorageError> {
    sqlx::query_as::<_, Thread>("SELECT * FROM threads ORDER BY created_at DESC LIMIT ?")
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    fn now_iso() -> String {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    fn sample_original_tweet() -> OriginalTweet {
        OriginalTweet {
            id: 0,
            tweet_id: Some("ot_123".to_string()),
            content: "Educational tweet about Rust".to_string(),
            topic: Some("rust".to_string()),
            llm_provider: Some("openai".to_string()),
            created_at: now_iso(),
            status: "sent".to_string(),
            error_message: None,
        }
    }

    fn sample_thread() -> Thread {
        Thread {
            id: 0,
            topic: "Rust async patterns".to_string(),
            tweet_count: 3,
            root_tweet_id: Some("root_456".to_string()),
            created_at: now_iso(),
            status: "sent".to_string(),
        }
    }

    fn sample_thread_tweets(thread_id: i64) -> Vec<ThreadTweet> {
        (0..3)
            .map(|i| ThreadTweet {
                id: 0,
                thread_id,
                position: i,
                tweet_id: Some(format!("tt_{i}")),
                content: format!("Thread tweet {i}"),
                created_at: now_iso(),
            })
            .collect()
    }

    #[tokio::test]
    async fn insert_and_query_original_tweet() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_original_tweet();

        let id = insert_original_tweet(&pool, &tweet).await.expect("insert");
        assert!(id > 0);

        let time = get_last_original_tweet_time(&pool).await.expect("get time");
        assert!(time.is_some());
    }

    #[tokio::test]
    async fn count_tweets_today_works() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_original_tweet();

        insert_original_tweet(&pool, &tweet).await.expect("insert");
        let count = count_tweets_today(&pool).await.expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn insert_thread_with_tweets() {
        let pool = init_test_db().await.expect("init db");
        let thread = sample_thread();

        let thread_id = insert_thread(&pool, &thread).await.expect("insert thread");
        let tweets = sample_thread_tweets(thread_id);
        insert_thread_tweets(&pool, thread_id, &tweets)
            .await
            .expect("insert tweets");

        // Verify all tweets were inserted
        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT position FROM thread_tweets WHERE thread_id = ? ORDER BY position",
        )
        .bind(thread_id)
        .fetch_all(&pool)
        .await
        .expect("query");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, 0);
        assert_eq!(rows[1].0, 1);
        assert_eq!(rows[2].0, 2);
    }

    #[tokio::test]
    async fn thread_tweet_duplicate_position_fails() {
        let pool = init_test_db().await.expect("init db");
        let thread = sample_thread();

        let thread_id = insert_thread(&pool, &thread).await.expect("insert thread");

        // Two tweets with same position should fail the UNIQUE constraint
        let duplicate_tweets = vec![
            ThreadTweet {
                id: 0,
                thread_id,
                position: 0,
                tweet_id: Some("a".to_string()),
                content: "First".to_string(),
                created_at: now_iso(),
            },
            ThreadTweet {
                id: 0,
                thread_id,
                position: 0, // duplicate position
                tweet_id: Some("b".to_string()),
                content: "Second".to_string(),
                created_at: now_iso(),
            },
        ];

        let result = insert_thread_tweets(&pool, thread_id, &duplicate_tweets).await;
        assert!(result.is_err());

        // Verify transaction rolled back (no partial data)
        let rows: Vec<(i64,)> =
            sqlx::query_as("SELECT COUNT(*) FROM thread_tweets WHERE thread_id = ?")
                .bind(thread_id)
                .fetch_all(&pool)
                .await
                .expect("query");

        assert_eq!(rows[0].0, 0, "transaction should have rolled back");
    }

    #[tokio::test]
    async fn count_threads_this_week_works() {
        let pool = init_test_db().await.expect("init db");
        let thread = sample_thread();

        insert_thread(&pool, &thread).await.expect("insert");
        let count = count_threads_this_week(&pool).await.expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn last_thread_time_empty() {
        let pool = init_test_db().await.expect("init db");
        let time = get_last_thread_time(&pool).await.expect("get time");
        assert!(time.is_none());
    }

    #[tokio::test]
    async fn get_tweets_in_range_filters() {
        let pool = init_test_db().await.expect("init db");

        let mut tweet = sample_original_tweet();
        tweet.created_at = "2026-02-20T10:00:00Z".to_string();
        insert_original_tweet(&pool, &tweet).await.expect("insert");

        let mut tweet2 = sample_original_tweet();
        tweet2.created_at = "2026-02-25T10:00:00Z".to_string();
        tweet2.tweet_id = Some("ot_456".to_string());
        insert_original_tweet(&pool, &tweet2).await.expect("insert");

        let in_range = get_tweets_in_range(&pool, "2026-02-19T00:00:00Z", "2026-02-21T00:00:00Z")
            .await
            .expect("range");
        assert_eq!(in_range.len(), 1);
        assert_eq!(in_range[0].tweet_id, Some("ot_123".to_string()));

        let all = get_tweets_in_range(&pool, "2026-02-01T00:00:00Z", "2026-02-28T00:00:00Z")
            .await
            .expect("range");
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn get_threads_in_range_filters() {
        let pool = init_test_db().await.expect("init db");

        let mut thread = sample_thread();
        thread.created_at = "2026-02-20T10:00:00Z".to_string();
        insert_thread(&pool, &thread).await.expect("insert");

        let mut thread2 = sample_thread();
        thread2.created_at = "2026-02-25T10:00:00Z".to_string();
        insert_thread(&pool, &thread2).await.expect("insert");

        let in_range = get_threads_in_range(&pool, "2026-02-19T00:00:00Z", "2026-02-21T00:00:00Z")
            .await
            .expect("range");
        assert_eq!(in_range.len(), 1);

        let all = get_threads_in_range(&pool, "2026-02-01T00:00:00Z", "2026-02-28T00:00:00Z")
            .await
            .expect("range");
        assert_eq!(all.len(), 2);
    }
}
