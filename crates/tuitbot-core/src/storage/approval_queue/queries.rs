//! Query functions for the approval queue.

use super::{ApprovalItem, ApprovalRow, ApprovalStats, ReviewAction};
use crate::error::StorageError;
use crate::storage::DbPool;

/// Standard SELECT columns for approval queue queries.
const SELECT_COLS: &str = "id, action_type, target_tweet_id, target_author, \
    generated_content, topic, archetype, score, status, created_at, \
    COALESCE(media_paths, '[]') AS media_paths, reviewed_by, review_notes, reason, \
    COALESCE(detected_risks, '[]') AS detected_risks, COALESCE(qa_report, '{}') AS qa_report, \
    COALESCE(qa_hard_flags, '[]') AS qa_hard_flags, COALESCE(qa_soft_flags, '[]') AS qa_soft_flags, \
    COALESCE(qa_recommendations, '[]') AS qa_recommendations, COALESCE(qa_score, 0) AS qa_score, \
    COALESCE(qa_requires_override, 0) AS qa_requires_override, qa_override_by, qa_override_note, qa_override_at";

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
    enqueue_with_context(
        pool,
        action_type,
        target_tweet_id,
        target_author,
        generated_content,
        topic,
        archetype,
        score,
        media_paths,
        None,
        None,
    )
    .await
}

/// Insert a new item into the approval queue with optional reason and risks.
#[allow(clippy::too_many_arguments)]
pub async fn enqueue_with_context(
    pool: &DbPool,
    action_type: &str,
    target_tweet_id: &str,
    target_author: &str,
    generated_content: &str,
    topic: &str,
    archetype: &str,
    score: f64,
    media_paths: &str,
    reason: Option<&str>,
    detected_risks: Option<&str>,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO approval_queue (action_type, target_tweet_id, target_author, \
         generated_content, topic, archetype, score, media_paths, reason, detected_risks) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(action_type)
    .bind(target_tweet_id)
    .bind(target_author)
    .bind(generated_content)
    .bind(topic)
    .bind(archetype)
    .bind(score)
    .bind(media_paths)
    .bind(reason)
    .bind(detected_risks.unwrap_or("[]"))
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Get all pending approval items, ordered by creation time (oldest first).
pub async fn get_pending(pool: &DbPool) -> Result<Vec<ApprovalItem>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM approval_queue \
         WHERE status = 'pending' ORDER BY created_at ASC"
    );
    let rows: Vec<ApprovalRow> = sqlx::query_as(&sql)
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
        "UPDATE approval_queue SET status = ?, \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(status)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the status of an approval item with review metadata.
pub async fn update_status_with_review(
    pool: &DbPool,
    id: i64,
    status: &str,
    review: &ReviewAction,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET status = ?, \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), \
         reviewed_by = ?, review_notes = ? WHERE id = ?",
    )
    .bind(status)
    .bind(&review.actor)
    .bind(&review.notes)
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
        "UPDATE approval_queue SET generated_content = ?, status = 'approved', \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
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
    let sql = format!("SELECT {SELECT_COLS} FROM approval_queue WHERE id = ?");
    let row: Option<ApprovalRow> = sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ApprovalItem::from))
}

/// Get counts of items grouped by status.
pub async fn get_stats(pool: &DbPool) -> Result<ApprovalStats, StorageError> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT \
            COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END), 0) \
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
            "SELECT {SELECT_COLS} FROM approval_queue \
             WHERE status IN ({in_clause}) AND action_type = ? \
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
            "SELECT {SELECT_COLS} FROM approval_queue \
             WHERE status IN ({in_clause}) \
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

/// Update QA fields for an approval item.
#[allow(clippy::too_many_arguments)]
pub async fn update_qa_fields(
    pool: &DbPool,
    id: i64,
    qa_report: &str,
    qa_hard_flags: &str,
    qa_soft_flags: &str,
    qa_recommendations: &str,
    qa_score: f64,
    qa_requires_override: bool,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET qa_report = ?, qa_hard_flags = ?, qa_soft_flags = ?, \
         qa_recommendations = ?, qa_score = ?, qa_requires_override = ? WHERE id = ?",
    )
    .bind(qa_report)
    .bind(qa_hard_flags)
    .bind(qa_soft_flags)
    .bind(qa_recommendations)
    .bind(qa_score)
    .bind(if qa_requires_override { 1 } else { 0 })
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Record an explicit QA override action.
pub async fn set_qa_override(
    pool: &DbPool,
    id: i64,
    actor: &str,
    note: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET qa_override_by = ?, qa_override_note = ?, \
         qa_override_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(actor)
    .bind(note)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Clear QA override metadata (used when content changes and QA is re-run).
pub async fn clear_qa_override(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET qa_override_by = NULL, qa_override_note = NULL, \
         qa_override_at = NULL WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Fetch the next approved item ready for posting.
pub async fn get_next_approved(pool: &DbPool) -> Result<Option<ApprovalItem>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM approval_queue \
         WHERE status = 'approved' ORDER BY reviewed_at ASC LIMIT 1"
    );
    let row: Option<ApprovalRow> = sqlx::query_as(&sql)
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
        "UPDATE approval_queue SET status = 'expired', \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE status = 'pending' \
         AND created_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?)",
    )
    .bind(format!("-{hours} hours"))
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

/// Batch-approve the oldest N pending items, returning their IDs.
pub async fn batch_approve(
    pool: &DbPool,
    max_batch: usize,
    review: &ReviewAction,
) -> Result<Vec<i64>, StorageError> {
    let pending = get_pending(pool).await?;
    let to_approve: Vec<&ApprovalItem> = pending.iter().take(max_batch).collect();
    let mut approved_ids = Vec::with_capacity(to_approve.len());

    for item in to_approve {
        update_status_with_review(pool, item.id, "approved", review).await?;
        approved_ids.push(item.id);
    }

    Ok(approved_ids)
}
