//! CRUD operations for manually composed and scheduled content.
//!
//! Provides functions to insert, query, update, and cancel content
//! that users create through the dashboard composer. Extended by
//! Draft Studio submodules for archive, revisions, activity, and tags.

pub mod activity;
pub mod drafts;
pub mod revisions;
pub mod tags;

pub use activity::*;
pub use drafts::*;
pub use revisions::*;
pub use tags::*;

use super::accounts::DEFAULT_ACCOUNT_ID;
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
    /// Full QA report payload as JSON.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_report: String,
    /// JSON-encoded hard QA flags.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_hard_flags: String,
    /// JSON-encoded soft QA flags.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_soft_flags: String,
    /// JSON-encoded QA recommendations.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_recommendations: String,
    /// QA score summary (0-100).
    pub qa_score: f64,
    /// Optional draft title (shown in rail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Free-form notes (internal, not posted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Soft-delete timestamp (NULL = not archived).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
}

/// A revision snapshot of content at a meaningful event.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ContentRevision {
    pub id: i64,
    pub content_id: i64,
    pub account_id: String,
    pub content: String,
    pub content_type: String,
    pub trigger_kind: String,
    pub created_at: String,
}

/// A user-defined tag for organizing content.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ContentTag {
    pub id: i64,
    pub account_id: String,
    pub name: String,
    pub color: Option<String>,
}

/// An activity log entry for content lifecycle events.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ContentActivity {
    pub id: i64,
    pub content_id: i64,
    pub account_id: String,
    pub action: String,
    pub detail: Option<String>,
    pub created_at: String,
}

/// Serialize a JSON-encoded string as a raw JSON value.
fn serialize_json_string<S: serde::Serializer>(
    value: &str,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    let parsed: serde_json::Value =
        serde_json::from_str(value).unwrap_or(serde_json::Value::Array(vec![]));
    parsed.serialize(serializer)
}

/// Insert a new scheduled content item for a specific account. Returns the auto-generated ID.
pub async fn insert_for(
    pool: &DbPool,
    account_id: &str,
    content_type: &str,
    content: &str,
    scheduled_for: Option<&str>,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO scheduled_content (account_id, content_type, content, scheduled_for) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(account_id)
    .bind(content_type)
    .bind(content)
    .bind(scheduled_for)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Insert a new scheduled content item. Returns the auto-generated ID.
pub async fn insert(
    pool: &DbPool,
    content_type: &str,
    content: &str,
    scheduled_for: Option<&str>,
) -> Result<i64, StorageError> {
    insert_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        content_type,
        content,
        scheduled_for,
    )
    .await
}

/// Fetch a scheduled content item by ID for a specific account.
pub async fn get_by_id_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
) -> Result<Option<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content WHERE id = ? AND account_id = ?",
    )
    .bind(id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Fetch a scheduled content item by ID.
pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<Option<ScheduledContent>, StorageError> {
    get_by_id_for(pool, DEFAULT_ACCOUNT_ID, id).await
}

/// Fetch all scheduled content items within a date range for a specific account.
///
/// Matches items where either `scheduled_for` or `created_at` falls within the range.
pub async fn get_in_range_for(
    pool: &DbPool,
    account_id: &str,
    from: &str,
    to: &str,
) -> Result<Vec<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content \
         WHERE account_id = ? \
           AND ((scheduled_for BETWEEN ? AND ?) \
            OR (scheduled_for IS NULL AND created_at BETWEEN ? AND ?)) \
         ORDER BY COALESCE(scheduled_for, created_at) ASC",
    )
    .bind(account_id)
    .bind(from)
    .bind(to)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
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
    get_in_range_for(pool, DEFAULT_ACCOUNT_ID, from, to).await
}

/// Fetch scheduled items that are due for posting for a specific account.
///
/// Returns items with status = 'scheduled' and scheduled_for <= now.
pub async fn get_due_items_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content \
         WHERE status = 'scheduled' AND scheduled_for IS NOT NULL \
           AND scheduled_for <= datetime('now') AND account_id = ? \
         ORDER BY scheduled_for ASC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Fetch scheduled items that are due for posting.
///
/// Returns items with status = 'scheduled' and scheduled_for <= now.
pub async fn get_due_items(pool: &DbPool) -> Result<Vec<ScheduledContent>, StorageError> {
    get_due_items_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Update the status of a scheduled content item for a specific account.
pub async fn update_status_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    status: &str,
    posted_tweet_id: Option<&str>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content \
         SET status = ?, posted_tweet_id = ?, updated_at = datetime('now') \
         WHERE id = ? AND account_id = ?",
    )
    .bind(status)
    .bind(posted_tweet_id)
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the status of a scheduled content item.
pub async fn update_status(
    pool: &DbPool,
    id: i64,
    status: &str,
    posted_tweet_id: Option<&str>,
) -> Result<(), StorageError> {
    update_status_for(pool, DEFAULT_ACCOUNT_ID, id, status, posted_tweet_id).await
}

/// Cancel a scheduled content item for a specific account (set status to 'cancelled').
pub async fn cancel_for(pool: &DbPool, account_id: &str, id: i64) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content \
         SET status = 'cancelled', updated_at = datetime('now') \
         WHERE id = ? AND status = 'scheduled' AND account_id = ?",
    )
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Cancel a scheduled content item (set status to 'cancelled').
pub async fn cancel(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    cancel_for(pool, DEFAULT_ACCOUNT_ID, id).await
}

/// Update the content and/or scheduled time of a scheduled item for a specific account.
///
/// Only allowed when the item is still in 'scheduled' status.
pub async fn update_content_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    content: &str,
    scheduled_for: Option<&str>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content \
         SET content = ?, scheduled_for = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = 'scheduled' AND account_id = ?",
    )
    .bind(content)
    .bind(scheduled_for)
    .bind(id)
    .bind(account_id)
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
    update_content_for(pool, DEFAULT_ACCOUNT_ID, id, content, scheduled_for).await
}

/// Update QA fields for a content item for a specific account.
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
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content SET qa_report = ?, qa_hard_flags = ?, qa_soft_flags = ?, \
         qa_recommendations = ?, qa_score = ?, updated_at = datetime('now') \
         WHERE id = ? AND account_id = ?",
    )
    .bind(qa_report)
    .bind(qa_hard_flags)
    .bind(qa_soft_flags)
    .bind(qa_recommendations)
    .bind(qa_score)
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update QA fields for a content item.
#[allow(clippy::too_many_arguments)]
pub async fn update_qa_fields(
    pool: &DbPool,
    id: i64,
    qa_report: &str,
    qa_hard_flags: &str,
    qa_soft_flags: &str,
    qa_recommendations: &str,
    qa_score: f64,
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
    )
    .await
}

// ============================================================================
// Draft operations
// ============================================================================

/// Insert a new draft for a specific account (status = 'draft', no scheduled_for).
pub async fn insert_draft_for(
    pool: &DbPool,
    account_id: &str,
    content_type: &str,
    content: &str,
    source: &str,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO scheduled_content (account_id, content_type, content, status, source) \
         VALUES (?, ?, ?, 'draft', ?)",
    )
    .bind(account_id)
    .bind(content_type)
    .bind(content)
    .bind(source)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Insert a new draft (status = 'draft', no scheduled_for).
pub async fn insert_draft(
    pool: &DbPool,
    content_type: &str,
    content: &str,
    source: &str,
) -> Result<i64, StorageError> {
    insert_draft_for(pool, DEFAULT_ACCOUNT_ID, content_type, content, source).await
}

/// List all draft items for a specific account, ordered by creation time (newest first).
/// Excludes archived drafts.
pub async fn list_drafts_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content \
         WHERE status = 'draft' AND account_id = ? AND archived_at IS NULL \
         ORDER BY created_at DESC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// List all draft items, ordered by creation time (newest first).
pub async fn list_drafts(pool: &DbPool) -> Result<Vec<ScheduledContent>, StorageError> {
    list_drafts_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Update a draft's content for a specific account.
pub async fn update_draft_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    content: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content SET content = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = 'draft' AND account_id = ?",
    )
    .bind(content)
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update a draft's content.
pub async fn update_draft(pool: &DbPool, id: i64, content: &str) -> Result<(), StorageError> {
    update_draft_for(pool, DEFAULT_ACCOUNT_ID, id, content).await
}

/// Delete a draft for a specific account (set status to 'cancelled').
pub async fn delete_draft_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content SET status = 'cancelled', updated_at = datetime('now') \
         WHERE id = ? AND status = 'draft' AND account_id = ?",
    )
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Delete a draft (set status to 'cancelled').
pub async fn delete_draft(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    delete_draft_for(pool, DEFAULT_ACCOUNT_ID, id).await
}

/// Promote a draft to scheduled for a specific account (set status to 'scheduled' with a scheduled_for time).
pub async fn schedule_draft_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    scheduled_for: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE scheduled_content SET status = 'scheduled', scheduled_for = ?, \
         updated_at = datetime('now') WHERE id = ? AND status = 'draft' AND account_id = ?",
    )
    .bind(scheduled_for)
    .bind(id)
    .bind(account_id)
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
    schedule_draft_for(pool, DEFAULT_ACCOUNT_ID, id, scheduled_for).await
}

#[cfg(test)]
mod tests;
