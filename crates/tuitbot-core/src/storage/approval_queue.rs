//! Storage operations for the approval queue.
//!
//! Provides CRUD operations for queuing posts for human review
//! when `approval_mode` is enabled.

use super::DbPool;
use crate::error::StorageError;

/// Row type for approval queue queries.
type ApprovalRow = (
    i64,
    String,
    String,
    String,
    String,
    String,
    String,
    f64,
    String,
    String,
    String,
);

/// A pending item in the approval queue.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ApprovalItem {
    pub id: i64,
    pub action_type: String,
    pub target_tweet_id: String,
    pub target_author: String,
    pub generated_content: String,
    pub topic: String,
    pub archetype: String,
    pub score: f64,
    pub status: String,
    pub created_at: String,
    /// JSON-encoded list of local media file paths.
    /// Serialized as a raw JSON array (not a string) for API consumers.
    #[serde(serialize_with = "serialize_json_string")]
    pub media_paths: String,
}

/// Serialize a JSON-encoded string as a raw JSON value.
///
/// The database stores `media_paths` as a JSON string (e.g. `"[\"/path/to/img.jpg\"]"`).
/// This serializer emits it as an actual JSON array in the API response.
fn serialize_json_string<S: serde::Serializer>(
    value: &str,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    let parsed: serde_json::Value =
        serde_json::from_str(value).unwrap_or(serde_json::Value::Array(vec![]));
    parsed.serialize(serializer)
}

impl From<ApprovalRow> for ApprovalItem {
    fn from(r: ApprovalRow) -> Self {
        Self {
            id: r.0,
            action_type: r.1,
            target_tweet_id: r.2,
            target_author: r.3,
            generated_content: r.4,
            topic: r.5,
            archetype: r.6,
            score: r.7,
            status: r.8,
            created_at: r.9,
            media_paths: r.10,
        }
    }
}

/// Insert a new item into the approval queue.
#[allow(clippy::too_many_arguments)]
pub async fn enqueue(
    pool: &DbPool,
    action_type: &str,
    target_tweet_id: &str,
    target_author: &str,
    generated_content: &str,
    topic: &str,
    archetype: &str,
    score: f64,
    media_paths: &str,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO approval_queue (action_type, target_tweet_id, target_author, generated_content, topic, archetype, score, media_paths)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(action_type)
    .bind(target_tweet_id)
    .bind(target_author)
    .bind(generated_content)
    .bind(topic)
    .bind(archetype)
    .bind(score)
    .bind(media_paths)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Get all pending approval items, ordered by creation time (oldest first).
pub async fn get_pending(pool: &DbPool) -> Result<Vec<ApprovalItem>, StorageError> {
    let rows: Vec<ApprovalRow> = sqlx::query_as(
        "SELECT id, action_type, target_tweet_id, target_author, generated_content, topic, archetype, score, status, created_at, COALESCE(media_paths, '[]')
         FROM approval_queue
         WHERE status = 'pending'
         ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ApprovalItem::from).collect())
}

/// Get the count of pending items.
pub async fn pending_count(pool: &DbPool) -> Result<i64, StorageError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM approval_queue WHERE status = 'pending'")
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Update the status of an approval item.
pub async fn update_status(pool: &DbPool, id: i64, status: &str) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET status = ?, reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(status)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the content and status of an approval item (for edit-then-approve).
pub async fn update_content_and_approve(
    pool: &DbPool,
    id: i64,
    new_content: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET generated_content = ?, status = 'approved', reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(new_content)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Get a single approval item by ID.
pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<Option<ApprovalItem>, StorageError> {
    let row: Option<ApprovalRow> = sqlx::query_as(
        "SELECT id, action_type, target_tweet_id, target_author, generated_content, topic, archetype, score, status, created_at, COALESCE(media_paths, '[]')
         FROM approval_queue
         WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ApprovalItem::from))
}

/// Counts of approval items grouped by status.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ApprovalStats {
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
}

/// Get counts of items grouped by status.
pub async fn get_stats(pool: &DbPool) -> Result<ApprovalStats, StorageError> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END), 0)
         FROM approval_queue",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(ApprovalStats {
        pending: row.0,
        approved: row.1,
        rejected: row.2,
    })
}

/// Get approval items filtered by one or more statuses, with optional action type filter.
///
/// Returns items ordered by creation time (oldest first).
/// If `statuses` is empty, returns an empty `Vec`.
pub async fn get_by_statuses(
    pool: &DbPool,
    statuses: &[&str],
    action_type: Option<&str>,
) -> Result<Vec<ApprovalItem>, StorageError> {
    if statuses.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = statuses.iter().map(|_| "?").collect();
    let in_clause = placeholders.join(", ");

    let query = if let Some(at) = action_type {
        let sql = format!(
            "SELECT id, action_type, target_tweet_id, target_author, generated_content, topic, archetype, score, status, created_at, COALESCE(media_paths, '[]')
             FROM approval_queue
             WHERE status IN ({in_clause}) AND action_type = ?
             ORDER BY created_at ASC"
        );
        let mut q = sqlx::query_as::<_, ApprovalRow>(&sql);
        for s in statuses {
            q = q.bind(*s);
        }
        q = q.bind(at);
        q.fetch_all(pool).await
    } else {
        let sql = format!(
            "SELECT id, action_type, target_tweet_id, target_author, generated_content, topic, archetype, score, status, created_at, COALESCE(media_paths, '[]')
             FROM approval_queue
             WHERE status IN ({in_clause})
             ORDER BY created_at ASC"
        );
        let mut q = sqlx::query_as::<_, ApprovalRow>(&sql);
        for s in statuses {
            q = q.bind(*s);
        }
        q.fetch_all(pool).await
    };

    let rows = query.map_err(|e| StorageError::Query { source: e })?;
    Ok(rows.into_iter().map(ApprovalItem::from).collect())
}

/// Update the generated content of an item without changing its status.
pub async fn update_content(pool: &DbPool, id: i64, new_content: &str) -> Result<(), StorageError> {
    sqlx::query("UPDATE approval_queue SET generated_content = ? WHERE id = ?")
        .bind(new_content)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the media paths of an approval item.
pub async fn update_media_paths(
    pool: &DbPool,
    id: i64,
    media_paths: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE approval_queue SET media_paths = ? WHERE id = ?")
        .bind(media_paths)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Fetch the next approved item ready for posting.
///
/// Returns the oldest item with `status='approved'`, ordered by `reviewed_at`.
pub async fn get_next_approved(pool: &DbPool) -> Result<Option<ApprovalItem>, StorageError> {
    let row: Option<ApprovalRow> = sqlx::query_as(
        "SELECT id, action_type, target_tweet_id, target_author, generated_content, topic, archetype, score, status, created_at, COALESCE(media_paths, '[]')
         FROM approval_queue
         WHERE status = 'approved'
         ORDER BY reviewed_at ASC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ApprovalItem::from))
}

/// Mark an approved item as posted, storing the returned tweet ID.
pub async fn mark_posted(pool: &DbPool, id: i64, tweet_id: &str) -> Result<(), StorageError> {
    sqlx::query("UPDATE approval_queue SET status = 'posted', posted_tweet_id = ? WHERE id = ?")
        .bind(tweet_id)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Expire old pending items (older than the specified hours).
pub async fn expire_old_items(pool: &DbPool, hours: u32) -> Result<u64, StorageError> {
    let result = sqlx::query(
        "UPDATE approval_queue SET status = 'expired', reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE status = 'pending'
         AND created_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?)",
    )
    .bind(format!("-{hours} hours"))
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn enqueue_and_get_pending() {
        let pool = init_test_db().await.expect("init db");

        let id = enqueue(
            &pool,
            "reply",
            "tweet123",
            "@testuser",
            "Great point about Rust!",
            "Rust",
            "AgreeAndExpand",
            85.0,
            "[]",
        )
        .await
        .expect("enqueue");

        assert!(id > 0);

        let pending = get_pending(&pool).await.expect("get pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action_type, "reply");
        assert_eq!(pending[0].target_tweet_id, "tweet123");
        assert_eq!(pending[0].generated_content, "Great point about Rust!");
    }

    #[tokio::test]
    async fn pending_count_works() {
        let pool = init_test_db().await.expect("init db");

        assert_eq!(pending_count(&pool).await.expect("count"), 0);

        enqueue(
            &pool,
            "tweet",
            "",
            "",
            "Hello world",
            "General",
            "",
            0.0,
            "[]",
        )
        .await
        .expect("enqueue");
        enqueue(&pool, "reply", "t1", "@u", "Nice!", "Rust", "", 50.0, "[]")
            .await
            .expect("enqueue");

        assert_eq!(pending_count(&pool).await.expect("count"), 2);
    }

    #[tokio::test]
    async fn update_status_marks_approved() {
        let pool = init_test_db().await.expect("init db");

        let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");

        update_status(&pool, id, "approved").await.expect("update");

        let pending = get_pending(&pool).await.expect("get pending");
        assert!(pending.is_empty());

        let item = get_by_id(&pool, id).await.expect("get").expect("found");
        assert_eq!(item.status, "approved");
    }

    #[tokio::test]
    async fn update_status_marks_rejected() {
        let pool = init_test_db().await.expect("init db");

        let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");

        update_status(&pool, id, "rejected").await.expect("update");

        let item = get_by_id(&pool, id).await.expect("get").expect("found");
        assert_eq!(item.status, "rejected");
    }

    #[tokio::test]
    async fn update_content_and_approve_works() {
        let pool = init_test_db().await.expect("init db");

        let id = enqueue(&pool, "tweet", "", "", "Draft", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");

        update_content_and_approve(&pool, id, "Final version")
            .await
            .expect("update");

        let item = get_by_id(&pool, id).await.expect("get").expect("found");
        assert_eq!(item.status, "approved");
        assert_eq!(item.generated_content, "Final version");
    }

    #[tokio::test]
    async fn get_by_id_not_found() {
        let pool = init_test_db().await.expect("init db");
        let item = get_by_id(&pool, 99999).await.expect("get");
        assert!(item.is_none());
    }

    #[tokio::test]
    async fn pending_ordered_by_creation_time() {
        let pool = init_test_db().await.expect("init db");

        enqueue(&pool, "tweet", "", "", "First", "A", "", 0.0, "[]")
            .await
            .expect("enqueue");
        enqueue(&pool, "tweet", "", "", "Second", "B", "", 0.0, "[]")
            .await
            .expect("enqueue");
        enqueue(&pool, "tweet", "", "", "Third", "C", "", 0.0, "[]")
            .await
            .expect("enqueue");

        let pending = get_pending(&pool).await.expect("get pending");
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].generated_content, "First");
        assert_eq!(pending[1].generated_content, "Second");
        assert_eq!(pending[2].generated_content, "Third");
    }

    #[tokio::test]
    async fn get_stats_counts_correctly() {
        let pool = init_test_db().await.expect("init db");

        // Empty table.
        let stats = get_stats(&pool).await.expect("stats");
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.approved, 0);
        assert_eq!(stats.rejected, 0);

        // Add items and change statuses.
        let id1 = enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");
        enqueue(&pool, "tweet", "", "", "B", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");
        let id3 = enqueue(&pool, "reply", "t1", "@u", "C", "Rust", "", 50.0, "[]")
            .await
            .expect("enqueue");

        update_status(&pool, id1, "approved").await.expect("update");
        update_status(&pool, id3, "rejected").await.expect("update");

        let stats = get_stats(&pool).await.expect("stats");
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.approved, 1);
        assert_eq!(stats.rejected, 1);
    }

    #[tokio::test]
    async fn get_by_statuses_filters_correctly() {
        let pool = init_test_db().await.expect("init db");

        let id1 = enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");
        enqueue(&pool, "tweet", "", "", "B", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");
        let id3 = enqueue(&pool, "reply", "t1", "@u", "C", "Rust", "", 50.0, "[]")
            .await
            .expect("enqueue");

        update_status(&pool, id1, "approved").await.expect("update");
        update_status(&pool, id3, "rejected").await.expect("update");

        // Only pending.
        let items = get_by_statuses(&pool, &["pending"], None)
            .await
            .expect("query");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].generated_content, "B");

        // Pending + approved.
        let items = get_by_statuses(&pool, &["pending", "approved"], None)
            .await
            .expect("query");
        assert_eq!(items.len(), 2);

        // All three.
        let items = get_by_statuses(&pool, &["pending", "approved", "rejected"], None)
            .await
            .expect("query");
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn get_by_statuses_with_action_type() {
        let pool = init_test_db().await.expect("init db");

        enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");
        enqueue(&pool, "reply", "t1", "@u", "B", "Rust", "", 50.0, "[]")
            .await
            .expect("enqueue");

        let items = get_by_statuses(&pool, &["pending"], Some("reply"))
            .await
            .expect("query");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].action_type, "reply");

        let items = get_by_statuses(&pool, &["pending"], Some("tweet"))
            .await
            .expect("query");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].action_type, "tweet");
    }

    #[tokio::test]
    async fn get_by_statuses_empty_returns_empty() {
        let pool = init_test_db().await.expect("init db");

        enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");

        let items = get_by_statuses(&pool, &[], None).await.expect("query");
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn update_content_preserves_status() {
        let pool = init_test_db().await.expect("init db");

        let id = enqueue(&pool, "tweet", "", "", "Original", "General", "", 0.0, "[]")
            .await
            .expect("enqueue");

        update_content(&pool, id, "Edited version")
            .await
            .expect("update");

        let item = get_by_id(&pool, id).await.expect("get").expect("found");
        assert_eq!(item.generated_content, "Edited version");
        assert_eq!(item.status, "pending");
    }
}
