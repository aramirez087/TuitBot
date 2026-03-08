//! Query functions for the approval queue.

use super::{ApprovalItem, ApprovalRow, ApprovalStats, ReviewAction};
use crate::error::StorageError;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::DbPool;

/// Standard SELECT columns for approval queue queries.
const SELECT_COLS: &str = "id, action_type, target_tweet_id, target_author, \
    generated_content, topic, archetype, score, status, created_at, \
    COALESCE(media_paths, '[]') AS media_paths, reviewed_by, review_notes, reason, \
    COALESCE(detected_risks, '[]') AS detected_risks, COALESCE(qa_report, '{}') AS qa_report, \
    COALESCE(qa_hard_flags, '[]') AS qa_hard_flags, COALESCE(qa_soft_flags, '[]') AS qa_soft_flags, \
    COALESCE(qa_recommendations, '[]') AS qa_recommendations, COALESCE(qa_score, 0) AS qa_score, \
    COALESCE(qa_requires_override, 0) AS qa_requires_override, qa_override_by, qa_override_note, qa_override_at, \
    source_node_id, source_seed_id, COALESCE(source_chunks_json, '[]') AS source_chunks_json";

/// Insert a new item into the approval queue for a specific account.
#[allow(clippy::too_many_arguments)]
pub async fn enqueue_for(
    pool: &DbPool,
    account_id: &str,
    action_type: &str,
    target_tweet_id: &str,
    target_author: &str,
    generated_content: &str,
    topic: &str,
    archetype: &str,
    score: f64,
    media_paths: &str,
) -> Result<i64, StorageError> {
    enqueue_with_context_for(
        pool,
        account_id,
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
    enqueue_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        action_type,
        target_tweet_id,
        target_author,
        generated_content,
        topic,
        archetype,
        score,
        media_paths,
    )
    .await
}

/// Insert a new item into the approval queue with optional reason and risks for a specific account.
#[allow(clippy::too_many_arguments)]
pub async fn enqueue_with_context_for(
    pool: &DbPool,
    account_id: &str,
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
        "INSERT INTO approval_queue (account_id, action_type, target_tweet_id, target_author, \
         generated_content, topic, archetype, score, media_paths, reason, detected_risks) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(account_id)
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
    enqueue_with_context_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        action_type,
        target_tweet_id,
        target_author,
        generated_content,
        topic,
        archetype,
        score,
        media_paths,
        reason,
        detected_risks,
    )
    .await
}

/// Get all pending approval items for a specific account, ordered by creation time (oldest first).
pub async fn get_pending_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<ApprovalItem>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM approval_queue \
         WHERE status = 'pending' AND account_id = ? ORDER BY created_at ASC"
    );
    let rows: Vec<ApprovalRow> = sqlx::query_as(&sql)
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ApprovalItem::from).collect())
}

/// Get all pending approval items, ordered by creation time (oldest first).
pub async fn get_pending(pool: &DbPool) -> Result<Vec<ApprovalItem>, StorageError> {
    get_pending_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Get the count of pending items for a specific account.
pub async fn pending_count_for(pool: &DbPool, account_id: &str) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM approval_queue WHERE status = 'pending' AND account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get the count of pending items.
pub async fn pending_count(pool: &DbPool) -> Result<i64, StorageError> {
    pending_count_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Update the status of an approval item for a specific account.
pub async fn update_status_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    status: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET status = ?, \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ? AND account_id = ?",
    )
    .bind(status)
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the status of an approval item.
pub async fn update_status(pool: &DbPool, id: i64, status: &str) -> Result<(), StorageError> {
    update_status_for(pool, DEFAULT_ACCOUNT_ID, id, status).await
}

/// Update the status of an approval item with review metadata for a specific account.
pub async fn update_status_with_review_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    status: &str,
    review: &ReviewAction,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET status = ?, \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), \
         reviewed_by = ?, review_notes = ? WHERE id = ? AND account_id = ?",
    )
    .bind(status)
    .bind(&review.actor)
    .bind(&review.notes)
    .bind(id)
    .bind(account_id)
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
    update_status_with_review_for(pool, DEFAULT_ACCOUNT_ID, id, status, review).await
}

/// Update the content and status of an approval item for a specific account (for edit-then-approve).
pub async fn update_content_and_approve_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    new_content: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET generated_content = ?, status = 'approved', \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ? AND account_id = ?",
    )
    .bind(new_content)
    .bind(id)
    .bind(account_id)
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
    update_content_and_approve_for(pool, DEFAULT_ACCOUNT_ID, id, new_content).await
}

/// Get a single approval item by ID for a specific account.
pub async fn get_by_id_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
) -> Result<Option<ApprovalItem>, StorageError> {
    let sql = format!("SELECT {SELECT_COLS} FROM approval_queue WHERE id = ? AND account_id = ?");
    let row: Option<ApprovalRow> = sqlx::query_as(&sql)
        .bind(id)
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ApprovalItem::from))
}

/// Get a single approval item by ID.
pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<Option<ApprovalItem>, StorageError> {
    get_by_id_for(pool, DEFAULT_ACCOUNT_ID, id).await
}

/// Get counts of items grouped by status for a specific account.
pub async fn get_stats_for(pool: &DbPool, account_id: &str) -> Result<ApprovalStats, StorageError> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT \
            COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END), 0) \
         FROM approval_queue WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(ApprovalStats {
        pending: row.0,
        approved: row.1,
        rejected: row.2,
    })
}

/// Get counts of items grouped by status.
pub async fn get_stats(pool: &DbPool) -> Result<ApprovalStats, StorageError> {
    get_stats_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Get approval items filtered by one or more statuses for a specific account,
/// with optional action type filter.
pub async fn get_by_statuses_for(
    pool: &DbPool,
    account_id: &str,
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
             WHERE account_id = ? AND status IN ({in_clause}) AND action_type = ? \
             ORDER BY created_at ASC"
        );
        let mut q = sqlx::query_as::<_, ApprovalRow>(&sql);
        q = q.bind(account_id);
        for s in statuses {
            q = q.bind(*s);
        }
        q = q.bind(at);
        q.fetch_all(pool).await
    } else {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM approval_queue \
             WHERE account_id = ? AND status IN ({in_clause}) \
             ORDER BY created_at ASC"
        );
        let mut q = sqlx::query_as::<_, ApprovalRow>(&sql);
        q = q.bind(account_id);
        for s in statuses {
            q = q.bind(*s);
        }
        q.fetch_all(pool).await
    };

    let rows = query.map_err(|e| StorageError::Query { source: e })?;
    Ok(rows.into_iter().map(ApprovalItem::from).collect())
}

/// Get approval items filtered by one or more statuses, with optional action type filter.
pub async fn get_by_statuses(
    pool: &DbPool,
    statuses: &[&str],
    action_type: Option<&str>,
) -> Result<Vec<ApprovalItem>, StorageError> {
    get_by_statuses_for(pool, DEFAULT_ACCOUNT_ID, statuses, action_type).await
}

/// Get approval items with optional filters for a specific account.
pub async fn get_filtered_for(
    pool: &DbPool,
    account_id: &str,
    statuses: &[&str],
    action_type: Option<&str>,
    reviewed_by: Option<&str>,
    since: Option<&str>,
) -> Result<Vec<ApprovalItem>, StorageError> {
    if statuses.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = statuses.iter().map(|_| "?").collect();
    let in_clause = placeholders.join(", ");

    let mut sql = format!(
        "SELECT {SELECT_COLS} FROM approval_queue \
         WHERE account_id = ? AND status IN ({in_clause})"
    );
    if action_type.is_some() {
        sql.push_str(" AND action_type = ?");
    }
    if reviewed_by.is_some() {
        sql.push_str(" AND reviewed_by = ?");
    }
    if since.is_some() {
        sql.push_str(" AND created_at >= ?");
    }
    sql.push_str(" ORDER BY created_at ASC");

    let mut q = sqlx::query_as::<_, ApprovalRow>(&sql);
    q = q.bind(account_id);
    for s in statuses {
        q = q.bind(*s);
    }
    if let Some(at) = action_type {
        q = q.bind(at);
    }
    if let Some(rb) = reviewed_by {
        q = q.bind(rb);
    }
    if let Some(s) = since {
        q = q.bind(s);
    }

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(rows.into_iter().map(ApprovalItem::from).collect())
}

/// Get approval items with optional filters for reviewer, date range, statuses, and action type.
pub async fn get_filtered(
    pool: &DbPool,
    statuses: &[&str],
    action_type: Option<&str>,
    reviewed_by: Option<&str>,
    since: Option<&str>,
) -> Result<Vec<ApprovalItem>, StorageError> {
    get_filtered_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        statuses,
        action_type,
        reviewed_by,
        since,
    )
    .await
}

/// Update the generated content of an item for a specific account without changing its status.
pub async fn update_content_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    new_content: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE approval_queue SET generated_content = ? WHERE id = ? AND account_id = ?")
        .bind(new_content)
        .bind(id)
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the generated content of an item without changing its status.
pub async fn update_content(pool: &DbPool, id: i64, new_content: &str) -> Result<(), StorageError> {
    update_content_for(pool, DEFAULT_ACCOUNT_ID, id, new_content).await
}

/// Update the media paths of an approval item for a specific account.
pub async fn update_media_paths_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    media_paths: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE approval_queue SET media_paths = ? WHERE id = ? AND account_id = ?")
        .bind(media_paths)
        .bind(id)
        .bind(account_id)
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
    update_media_paths_for(pool, DEFAULT_ACCOUNT_ID, id, media_paths).await
}

/// Update QA fields for an approval item for a specific account.
#[allow(clippy::too_many_arguments)]
pub async fn update_qa_fields_for(
    pool: &DbPool,
    account_id: &str,
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
         qa_recommendations = ?, qa_score = ?, qa_requires_override = ? \
         WHERE id = ? AND account_id = ?",
    )
    .bind(qa_report)
    .bind(qa_hard_flags)
    .bind(qa_soft_flags)
    .bind(qa_recommendations)
    .bind(qa_score)
    .bind(if qa_requires_override { 1 } else { 0 })
    .bind(id)
    .bind(account_id)
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
    update_qa_fields_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        id,
        qa_report,
        qa_hard_flags,
        qa_soft_flags,
        qa_recommendations,
        qa_score,
        qa_requires_override,
    )
    .await
}

/// Record an explicit QA override action for a specific account.
pub async fn set_qa_override_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    actor: &str,
    note: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET qa_override_by = ?, qa_override_note = ?, \
         qa_override_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE id = ? AND account_id = ?",
    )
    .bind(actor)
    .bind(note)
    .bind(id)
    .bind(account_id)
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
    set_qa_override_for(pool, DEFAULT_ACCOUNT_ID, id, actor, note).await
}

/// Clear QA override metadata for a specific account (used when content changes and QA is re-run).
pub async fn clear_qa_override_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET qa_override_by = NULL, qa_override_note = NULL, \
         qa_override_at = NULL WHERE id = ? AND account_id = ?",
    )
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Clear QA override metadata (used when content changes and QA is re-run).
pub async fn clear_qa_override(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    clear_qa_override_for(pool, DEFAULT_ACCOUNT_ID, id).await
}

/// Fetch the next approved item ready for posting for a specific account.
pub async fn get_next_approved_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Option<ApprovalItem>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_COLS} FROM approval_queue \
         WHERE status = 'approved' AND account_id = ? ORDER BY reviewed_at ASC LIMIT 1"
    );
    let row: Option<ApprovalRow> = sqlx::query_as(&sql)
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ApprovalItem::from))
}

/// Fetch the next approved item ready for posting.
pub async fn get_next_approved(pool: &DbPool) -> Result<Option<ApprovalItem>, StorageError> {
    get_next_approved_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Mark an approved item as posted for a specific account, storing the returned tweet ID.
pub async fn mark_posted_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    tweet_id: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE approval_queue SET status = 'posted', posted_tweet_id = ? \
         WHERE id = ? AND account_id = ?",
    )
    .bind(tweet_id)
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Mark an approved item as posted, storing the returned tweet ID.
pub async fn mark_posted(pool: &DbPool, id: i64, tweet_id: &str) -> Result<(), StorageError> {
    mark_posted_for(pool, DEFAULT_ACCOUNT_ID, id, tweet_id).await
}

/// Expire old pending items for a specific account (older than the specified hours).
pub async fn expire_old_items_for(
    pool: &DbPool,
    account_id: &str,
    hours: u32,
) -> Result<u64, StorageError> {
    let result = sqlx::query(
        "UPDATE approval_queue SET status = 'expired', \
         reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         WHERE status = 'pending' AND account_id = ? \
         AND created_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?)",
    )
    .bind(account_id)
    .bind(format!("-{hours} hours"))
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

/// Expire old pending items (older than the specified hours).
pub async fn expire_old_items(pool: &DbPool, hours: u32) -> Result<u64, StorageError> {
    expire_old_items_for(pool, DEFAULT_ACCOUNT_ID, hours).await
}

/// Batch-approve the oldest N pending items for a specific account, returning their IDs.
pub async fn batch_approve_for(
    pool: &DbPool,
    account_id: &str,
    max_batch: usize,
    review: &ReviewAction,
) -> Result<Vec<i64>, StorageError> {
    let pending = get_pending_for(pool, account_id).await?;
    let to_approve: Vec<&ApprovalItem> = pending.iter().take(max_batch).collect();
    let mut approved_ids = Vec::with_capacity(to_approve.len());

    for item in to_approve {
        update_status_with_review_for(pool, account_id, item.id, "approved", review).await?;
        approved_ids.push(item.id);
    }

    Ok(approved_ids)
}

/// Batch-approve the oldest N pending items, returning their IDs.
pub async fn batch_approve(
    pool: &DbPool,
    max_batch: usize,
    review: &ReviewAction,
) -> Result<Vec<i64>, StorageError> {
    batch_approve_for(pool, DEFAULT_ACCOUNT_ID, max_batch, review).await
}
