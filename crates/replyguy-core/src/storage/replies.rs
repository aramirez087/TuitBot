//! CRUD operations for replies sent by the agent.
//!
//! Provides functions to insert replies, check for duplicates,
//! count daily usage, and retrieve recent reply content.

use super::DbPool;
use crate::error::StorageError;

/// A reply generated and posted by the agent.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ReplySent {
    /// Internal auto-generated ID.
    pub id: i64,
    /// Tweet ID that was replied to.
    pub target_tweet_id: String,
    /// Our reply's X tweet ID (None if post failed).
    pub reply_tweet_id: Option<String>,
    /// Generated reply text.
    pub reply_content: String,
    /// Which LLM generated this reply.
    pub llm_provider: Option<String>,
    /// Which model was used.
    pub llm_model: Option<String>,
    /// ISO-8601 UTC timestamp when reply was sent.
    pub created_at: String,
    /// Status: sent, failed, or deleted.
    pub status: String,
    /// Error details if failed.
    pub error_message: Option<String>,
}

/// Insert a new reply record. Returns the auto-generated ID.
pub async fn insert_reply(pool: &DbPool, reply: &ReplySent) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO replies_sent \
         (target_tweet_id, reply_tweet_id, reply_content, llm_provider, llm_model, \
          created_at, status, error_message) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&reply.target_tweet_id)
    .bind(&reply.reply_tweet_id)
    .bind(&reply.reply_content)
    .bind(&reply.llm_provider)
    .bind(&reply.llm_model)
    .bind(&reply.created_at)
    .bind(&reply.status)
    .bind(&reply.error_message)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Fetch all replies with `created_at >= since`.
pub async fn get_replies_since(pool: &DbPool, since: &str) -> Result<Vec<ReplySent>, StorageError> {
    sqlx::query_as::<_, ReplySent>(
        "SELECT * FROM replies_sent WHERE created_at >= ? ORDER BY created_at ASC",
    )
    .bind(since)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Check if a reply has already been sent to a given tweet (deduplication).
pub async fn has_replied_to(pool: &DbPool, tweet_id: &str) -> Result<bool, StorageError> {
    let row: (i64,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM replies_sent WHERE target_tweet_id = ?)")
            .bind(tweet_id)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0 == 1)
}

/// Get recent reply contents for phrasing deduplication.
pub async fn get_recent_reply_contents(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<String>, StorageError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT reply_content FROM replies_sent ORDER BY created_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Count replies sent today (UTC).
pub async fn count_replies_today(pool: &DbPool) -> Result<i64, StorageError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM replies_sent WHERE date(created_at) = date('now')")
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    fn sample_reply(target_id: &str) -> ReplySent {
        ReplySent {
            id: 0, // ignored on insert
            target_tweet_id: target_id.to_string(),
            reply_tweet_id: Some("reply_123".to_string()),
            reply_content: "Great point! Here's my take...".to_string(),
            llm_provider: Some("openai".to_string()),
            llm_model: Some("gpt-4o-mini".to_string()),
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            status: "sent".to_string(),
            error_message: None,
        }
    }

    #[tokio::test]
    async fn insert_and_check_reply() {
        let pool = init_test_db().await.expect("init db");
        let reply = sample_reply("tweet_abc");

        let id = insert_reply(&pool, &reply).await.expect("insert");
        assert!(id > 0);

        assert!(has_replied_to(&pool, "tweet_abc").await.expect("check"));
        assert!(!has_replied_to(&pool, "tweet_xyz").await.expect("check"));
    }

    #[tokio::test]
    async fn count_replies_today_works() {
        let pool = init_test_db().await.expect("init db");
        let reply = sample_reply("tweet_count");

        insert_reply(&pool, &reply).await.expect("insert");
        let count = count_replies_today(&pool).await.expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn get_recent_contents() {
        let pool = init_test_db().await.expect("init db");

        let mut r1 = sample_reply("t1");
        r1.reply_content = "Reply one".to_string();
        let mut r2 = sample_reply("t2");
        r2.reply_content = "Reply two".to_string();

        insert_reply(&pool, &r1).await.expect("ins1");
        insert_reply(&pool, &r2).await.expect("ins2");

        let contents = get_recent_reply_contents(&pool, 5).await.expect("get");
        assert_eq!(contents.len(), 2);
    }

    #[tokio::test]
    async fn get_replies_since_filters() {
        let pool = init_test_db().await.expect("init db");
        let reply = sample_reply("tweet_since");

        insert_reply(&pool, &reply).await.expect("insert");

        let all = get_replies_since(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("get");
        assert_eq!(all.len(), 1);

        let none = get_replies_since(&pool, "2099-01-01T00:00:00Z")
            .await
            .expect("get");
        assert!(none.is_empty());
    }
}
