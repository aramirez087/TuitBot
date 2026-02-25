//! CRUD operations for manually composed and scheduled content.
//!
//! Provides functions to insert, query, update, and cancel content
//! that users create through the dashboard composer.

use super::DbPool;
use crate::error::StorageError;

/// A manually composed content item with optional scheduling.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ScheduledContent {
    /// Internal auto-generated ID.
    pub id: i64,
    /// Content type: "tweet" or "thread".
    pub content_type: String,
    /// Content text (string for tweet, JSON array for thread).
    pub content: String,
    /// Optional ISO-8601 scheduled time. NULL = next available slot.
    pub scheduled_for: Option<String>,
    /// Status: scheduled, posted, or cancelled.
    pub status: String,
    /// X tweet ID after posting (filled when posted).
    pub posted_tweet_id: Option<String>,
    /// ISO-8601 UTC timestamp when created.
    pub created_at: String,
    /// ISO-8601 UTC timestamp when last updated.
    pub updated_at: String,
}

/// Insert a new scheduled content item. Returns the auto-generated ID.
pub async fn insert(
    pool: &DbPool,
    content_type: &str,
    content: &str,
    scheduled_for: Option<&str>,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO scheduled_content (content_type, content, scheduled_for) \
         VALUES (?, ?, ?)",
    )
    .bind(content_type)
    .bind(content)
    .bind(scheduled_for)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Fetch a scheduled content item by ID.
pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<Option<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>("SELECT * FROM scheduled_content WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Fetch all scheduled content items within a date range.
///
/// Matches items where either `scheduled_for` or `created_at` falls within the range.
pub async fn get_in_range(
    pool: &DbPool,
    from: &str,
    to: &str,
) -> Result<Vec<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content \
         WHERE (scheduled_for BETWEEN ? AND ?) \
            OR (scheduled_for IS NULL AND created_at BETWEEN ? AND ?) \
         ORDER BY COALESCE(scheduled_for, created_at) ASC",
    )
    .bind(from)
    .bind(to)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Fetch scheduled items that are due for posting.
///
/// Returns items with status = 'scheduled' and scheduled_for <= now.
pub async fn get_due_items(pool: &DbPool) -> Result<Vec<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content \
         WHERE status = 'scheduled' AND scheduled_for IS NOT NULL \
           AND scheduled_for <= datetime('now') \
         ORDER BY scheduled_for ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Update the status of a scheduled content item.
pub async fn update_status(
    pool: &DbPool,
    id: i64,
    status: &str,
    posted_tweet_id: Option<&str>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content \
         SET status = ?, posted_tweet_id = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(status)
    .bind(posted_tweet_id)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Cancel a scheduled content item (set status to 'cancelled').
pub async fn cancel(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content \
         SET status = 'cancelled', updated_at = datetime('now') \
         WHERE id = ? AND status = 'scheduled'",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the content and/or scheduled time of a scheduled item.
///
/// Only allowed when the item is still in 'scheduled' status.
pub async fn update_content(
    pool: &DbPool,
    id: i64,
    content: &str,
    scheduled_for: Option<&str>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content \
         SET content = ?, scheduled_for = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = 'scheduled'",
    )
    .bind(content)
    .bind(scheduled_for)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

// ============================================================================
// Draft operations
// ============================================================================

/// Insert a new draft (status = 'draft', no scheduled_for).
pub async fn insert_draft(
    pool: &DbPool,
    content_type: &str,
    content: &str,
    source: &str,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO scheduled_content (content_type, content, status, source) \
         VALUES (?, ?, 'draft', ?)",
    )
    .bind(content_type)
    .bind(content)
    .bind(source)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// List all draft items, ordered by creation time (newest first).
pub async fn list_drafts(pool: &DbPool) -> Result<Vec<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content WHERE status = 'draft' ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Update a draft's content.
pub async fn update_draft(pool: &DbPool, id: i64, content: &str) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content SET content = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = 'draft'",
    )
    .bind(content)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Delete a draft (set status to 'cancelled').
pub async fn delete_draft(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content SET status = 'cancelled', updated_at = datetime('now') \
         WHERE id = ? AND status = 'draft'",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Promote a draft to scheduled (set status to 'scheduled' with a scheduled_for time).
pub async fn schedule_draft(
    pool: &DbPool,
    id: i64,
    scheduled_for: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content SET status = 'scheduled', scheduled_for = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = 'draft'",
    )
    .bind(scheduled_for)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn insert_and_retrieve() {
        let pool = init_test_db().await.expect("init db");

        let id = insert(&pool, "tweet", "Hello world!", Some("2026-02-24T09:15:00Z"))
            .await
            .expect("insert");
        assert!(id > 0);

        let item = get_by_id(&pool, id).await.expect("get").expect("exists");
        assert_eq!(item.content_type, "tweet");
        assert_eq!(item.content, "Hello world!");
        assert_eq!(item.scheduled_for.as_deref(), Some("2026-02-24T09:15:00Z"));
        assert_eq!(item.status, "scheduled");
        assert!(item.posted_tweet_id.is_none());
    }

    #[tokio::test]
    async fn insert_without_scheduled_time() {
        let pool = init_test_db().await.expect("init db");

        let id = insert(&pool, "tweet", "No time set", None)
            .await
            .expect("insert");
        let item = get_by_id(&pool, id).await.expect("get").expect("exists");
        assert!(item.scheduled_for.is_none());
    }

    #[tokio::test]
    async fn get_in_range_filters() {
        let pool = init_test_db().await.expect("init db");

        insert(&pool, "tweet", "In range", Some("2026-02-24T09:00:00Z"))
            .await
            .expect("insert");
        insert(&pool, "tweet", "Out of range", Some("2026-03-01T09:00:00Z"))
            .await
            .expect("insert");

        let items = get_in_range(&pool, "2026-02-23T00:00:00Z", "2026-02-25T00:00:00Z")
            .await
            .expect("range");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "In range");
    }

    #[tokio::test]
    async fn get_due_items_returns_past_scheduled() {
        let pool = init_test_db().await.expect("init db");

        // Insert an item scheduled in the past
        insert(&pool, "tweet", "Past tweet", Some("2020-01-01T09:00:00Z"))
            .await
            .expect("insert");

        // Insert a future item
        insert(&pool, "tweet", "Future tweet", Some("2099-01-01T09:00:00Z"))
            .await
            .expect("insert");

        // Insert an item with no schedule
        insert(&pool, "tweet", "No schedule", None)
            .await
            .expect("insert");

        let due = get_due_items(&pool).await.expect("due");
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].content, "Past tweet");
    }

    #[tokio::test]
    async fn update_status_marks_posted() {
        let pool = init_test_db().await.expect("init db");

        let id = insert(&pool, "tweet", "Will post", Some("2026-02-24T09:00:00Z"))
            .await
            .expect("insert");

        update_status(&pool, id, "posted", Some("x_tweet_123"))
            .await
            .expect("update");

        let item = get_by_id(&pool, id).await.expect("get").expect("exists");
        assert_eq!(item.status, "posted");
        assert_eq!(item.posted_tweet_id.as_deref(), Some("x_tweet_123"));
    }

    #[tokio::test]
    async fn cancel_sets_cancelled_status() {
        let pool = init_test_db().await.expect("init db");

        let id = insert(&pool, "tweet", "Will cancel", Some("2026-02-24T09:00:00Z"))
            .await
            .expect("insert");

        cancel(&pool, id).await.expect("cancel");

        let item = get_by_id(&pool, id).await.expect("get").expect("exists");
        assert_eq!(item.status, "cancelled");
    }

    #[tokio::test]
    async fn cancel_only_affects_scheduled_items() {
        let pool = init_test_db().await.expect("init db");

        let id = insert(&pool, "tweet", "Posted item", Some("2026-02-24T09:00:00Z"))
            .await
            .expect("insert");

        // Mark as posted first
        update_status(&pool, id, "posted", Some("x_123"))
            .await
            .expect("update");

        // Try to cancel — should not change status
        cancel(&pool, id).await.expect("cancel");

        let item = get_by_id(&pool, id).await.expect("get").expect("exists");
        assert_eq!(item.status, "posted"); // unchanged
    }

    #[tokio::test]
    async fn update_content_changes_text_and_time() {
        let pool = init_test_db().await.expect("init db");

        let id = insert(&pool, "tweet", "Original", Some("2026-02-24T09:00:00Z"))
            .await
            .expect("insert");

        update_content(&pool, id, "Updated text", Some("2026-02-25T12:00:00Z"))
            .await
            .expect("update");

        let item = get_by_id(&pool, id).await.expect("get").expect("exists");
        assert_eq!(item.content, "Updated text");
        assert_eq!(item.scheduled_for.as_deref(), Some("2026-02-25T12:00:00Z"));
    }

    #[tokio::test]
    async fn get_nonexistent_returns_none() {
        let pool = init_test_db().await.expect("init db");
        let item = get_by_id(&pool, 999).await.expect("get");
        assert!(item.is_none());
    }

    #[tokio::test]
    async fn insert_thread_content() {
        let pool = init_test_db().await.expect("init db");

        let thread_content =
            serde_json::to_string(&vec!["First tweet", "Second tweet", "Third tweet"])
                .expect("json");
        let id = insert(
            &pool,
            "thread",
            &thread_content,
            Some("2026-02-24T10:00:00Z"),
        )
        .await
        .expect("insert");

        let item = get_by_id(&pool, id).await.expect("get").expect("exists");
        assert_eq!(item.content_type, "thread");

        let tweets: Vec<String> = serde_json::from_str(&item.content).expect("parse");
        assert_eq!(tweets.len(), 3);
    }
}
