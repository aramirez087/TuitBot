//! Append-only action log for auditing and status reporting.
//!
//! Records every action taken by the agent with timestamps,
//! status, and optional metadata in JSON format.

use super::accounts::DEFAULT_ACCOUNT_ID;
use super::DbPool;
use crate::error::StorageError;
use std::collections::HashMap;

/// An entry in the action audit log.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ActionLogEntry {
    /// Internal auto-generated ID.
    pub id: i64,
    /// Action type: search, reply, tweet, thread, mention_check, cleanup, auth_refresh.
    pub action_type: String,
    /// Status: success, failure, or skipped.
    pub status: String,
    /// Human-readable description.
    pub message: Option<String>,
    /// JSON blob for flexible extra data.
    pub metadata: Option<String>,
    /// ISO-8601 UTC timestamp.
    pub created_at: String,
}

/// Insert a new action log entry for a specific account.
///
/// The `metadata` parameter is a pre-serialized JSON string; the caller
/// is responsible for serialization. The `created_at` field uses the SQL default.
pub async fn log_action_for(
    pool: &DbPool,
    account_id: &str,
    action_type: &str,
    status: &str,
    message: Option<&str>,
    metadata: Option<&str>,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO action_log (account_id, action_type, status, message, metadata) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(account_id)
    .bind(action_type)
    .bind(status)
    .bind(message)
    .bind(metadata)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Insert a new action log entry.
///
/// The `metadata` parameter is a pre-serialized JSON string; the caller
/// is responsible for serialization. The `created_at` field uses the SQL default.
pub async fn log_action(
    pool: &DbPool,
    action_type: &str,
    status: &str,
    message: Option<&str>,
    metadata: Option<&str>,
) -> Result<(), StorageError> {
    log_action_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        action_type,
        status,
        message,
        metadata,
    )
    .await
}

/// Fetch action log entries since a given timestamp for a specific account,
/// optionally filtered by type.
///
/// Results are ordered by `created_at` ascending.
pub async fn get_actions_since_for(
    pool: &DbPool,
    account_id: &str,
    since: &str,
    action_type: Option<&str>,
) -> Result<Vec<ActionLogEntry>, StorageError> {
    match action_type {
        Some(at) => sqlx::query_as::<_, ActionLogEntry>(
            "SELECT * FROM action_log WHERE created_at >= ? AND action_type = ? \
                 AND account_id = ? ORDER BY created_at ASC",
        )
        .bind(since)
        .bind(at)
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e }),
        None => sqlx::query_as::<_, ActionLogEntry>(
            "SELECT * FROM action_log WHERE created_at >= ? \
                 AND account_id = ? ORDER BY created_at ASC",
        )
        .bind(since)
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e }),
    }
}

/// Fetch action log entries since a given timestamp, optionally filtered by type.
///
/// Results are ordered by `created_at` ascending.
pub async fn get_actions_since(
    pool: &DbPool,
    since: &str,
    action_type: Option<&str>,
) -> Result<Vec<ActionLogEntry>, StorageError> {
    get_actions_since_for(pool, DEFAULT_ACCOUNT_ID, since, action_type).await
}

/// Get counts of each action type since a given timestamp for a specific account.
///
/// Returns a HashMap mapping action types to their counts.
pub async fn get_action_counts_since_for(
    pool: &DbPool,
    account_id: &str,
    since: &str,
) -> Result<HashMap<String, i64>, StorageError> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT action_type, COUNT(*) as count FROM action_log \
         WHERE created_at >= ? AND account_id = ? GROUP BY action_type",
    )
    .bind(since)
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().collect())
}

/// Get counts of each action type since a given timestamp.
///
/// Returns a HashMap mapping action types to their counts.
pub async fn get_action_counts_since(
    pool: &DbPool,
    since: &str,
) -> Result<HashMap<String, i64>, StorageError> {
    get_action_counts_since_for(pool, DEFAULT_ACCOUNT_ID, since).await
}

/// Get the most recent action log entries for a specific account, newest first.
pub async fn get_recent_actions_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<ActionLogEntry>, StorageError> {
    sqlx::query_as::<_, ActionLogEntry>(
        "SELECT * FROM action_log WHERE account_id = ? ORDER BY created_at DESC LIMIT ?",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Get the most recent action log entries, newest first.
pub async fn get_recent_actions(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<ActionLogEntry>, StorageError> {
    get_recent_actions_for(pool, DEFAULT_ACCOUNT_ID, limit).await
}

/// Fetch paginated action log entries for a specific account with optional
/// type and status filters.
///
/// Results are ordered by `created_at` descending (newest first).
pub async fn get_actions_paginated_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
    offset: u32,
    action_type: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<ActionLogEntry>, StorageError> {
    let mut sql = String::from("SELECT * FROM action_log WHERE 1=1 AND account_id = ?");
    if action_type.is_some() {
        sql.push_str(" AND action_type = ?");
    }
    if status.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

    let mut query = sqlx::query_as::<_, ActionLogEntry>(&sql);
    query = query.bind(account_id);
    if let Some(at) = action_type {
        query = query.bind(at);
    }
    if let Some(st) = status {
        query = query.bind(st);
    }
    query = query.bind(limit).bind(offset);

    query
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Fetch paginated action log entries with optional type and status filters.
///
/// Results are ordered by `created_at` descending (newest first).
pub async fn get_actions_paginated(
    pool: &DbPool,
    limit: u32,
    offset: u32,
    action_type: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<ActionLogEntry>, StorageError> {
    get_actions_paginated_for(pool, DEFAULT_ACCOUNT_ID, limit, offset, action_type, status).await
}

/// Get total count of action log entries for a specific account with optional
/// type and status filters.
pub async fn get_actions_count_for(
    pool: &DbPool,
    account_id: &str,
    action_type: Option<&str>,
    status: Option<&str>,
) -> Result<i64, StorageError> {
    let mut sql = String::from("SELECT COUNT(*) FROM action_log WHERE 1=1 AND account_id = ?");
    if action_type.is_some() {
        sql.push_str(" AND action_type = ?");
    }
    if status.is_some() {
        sql.push_str(" AND status = ?");
    }

    let mut query = sqlx::query_as::<_, (i64,)>(&sql);
    query = query.bind(account_id);
    if let Some(at) = action_type {
        query = query.bind(at);
    }
    if let Some(st) = status {
        query = query.bind(st);
    }

    let (count,) = query
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(count)
}

/// Get total count of action log entries with optional type and status filters.
pub async fn get_actions_count(
    pool: &DbPool,
    action_type: Option<&str>,
    status: Option<&str>,
) -> Result<i64, StorageError> {
    get_actions_count_for(pool, DEFAULT_ACCOUNT_ID, action_type, status).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn log_and_retrieve_action() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", Some("Found 10 tweets"), None)
            .await
            .expect("log");

        let actions = get_actions_since(&pool, "2000-01-01T00:00:00Z", None)
            .await
            .expect("get");

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, "search");
        assert_eq!(actions[0].status, "success");
        assert_eq!(actions[0].message.as_deref(), Some("Found 10 tweets"));
    }

    #[tokio::test]
    async fn filter_by_action_type() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "search", "failure", None, None)
            .await
            .expect("log");

        let searches = get_actions_since(&pool, "2000-01-01T00:00:00Z", Some("search"))
            .await
            .expect("get");
        assert_eq!(searches.len(), 2);

        let replies = get_actions_since(&pool, "2000-01-01T00:00:00Z", Some("reply"))
            .await
            .expect("get");
        assert_eq!(replies.len(), 1);
    }

    #[tokio::test]
    async fn action_counts_aggregation() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "tweet", "failure", None, None)
            .await
            .expect("log");

        let counts = get_action_counts_since(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("get counts");

        assert_eq!(counts.get("search"), Some(&2));
        assert_eq!(counts.get("reply"), Some(&1));
        assert_eq!(counts.get("tweet"), Some(&1));
    }

    #[tokio::test]
    async fn log_with_metadata() {
        let pool = init_test_db().await.expect("init db");

        let metadata = r#"{"tweet_id": "123", "score": 85}"#;
        log_action(
            &pool,
            "reply",
            "success",
            Some("Replied to tweet"),
            Some(metadata),
        )
        .await
        .expect("log");

        let actions = get_actions_since(&pool, "2000-01-01T00:00:00Z", Some("reply"))
            .await
            .expect("get");

        assert_eq!(actions[0].metadata.as_deref(), Some(metadata));
    }

    #[tokio::test]
    async fn empty_counts_returns_empty_map() {
        let pool = init_test_db().await.expect("init db");

        let counts = get_action_counts_since(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("get counts");

        assert!(counts.is_empty());
    }

    #[tokio::test]
    async fn paginated_actions_with_offset() {
        let pool = init_test_db().await.expect("init db");

        for i in 0..10 {
            log_action(
                &pool,
                "search",
                "success",
                Some(&format!("Action {i}")),
                None,
            )
            .await
            .expect("log");
        }

        let page1 = get_actions_paginated(&pool, 3, 0, None, None)
            .await
            .expect("page 1");
        assert_eq!(page1.len(), 3);

        let page2 = get_actions_paginated(&pool, 3, 3, None, None)
            .await
            .expect("page 2");
        assert_eq!(page2.len(), 3);

        // Pages should not overlap
        let ids1: Vec<i64> = page1.iter().map(|a| a.id).collect();
        let ids2: Vec<i64> = page2.iter().map(|a| a.id).collect();
        assert!(ids1.iter().all(|id| !ids2.contains(id)));
    }

    #[tokio::test]
    async fn paginated_actions_with_type_filter() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");

        let searches = get_actions_paginated(&pool, 10, 0, Some("search"), None)
            .await
            .expect("get");
        assert_eq!(searches.len(), 2);

        let count = get_actions_count(&pool, Some("search"), None)
            .await
            .expect("count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn paginated_actions_with_status_filter() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "failure", Some("Rate limited"), None)
            .await
            .expect("log");
        log_action(&pool, "tweet", "failure", Some("API error"), None)
            .await
            .expect("log");

        let failures = get_actions_paginated(&pool, 10, 0, None, Some("failure"))
            .await
            .expect("get");
        assert_eq!(failures.len(), 2);

        let count = get_actions_count(&pool, None, Some("failure"))
            .await
            .expect("count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn paginated_actions_combined_filters() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "failure", None, None)
            .await
            .expect("log");
        log_action(&pool, "tweet", "failure", None, None)
            .await
            .expect("log");

        let reply_failures = get_actions_paginated(&pool, 10, 0, Some("reply"), Some("failure"))
            .await
            .expect("get");
        assert_eq!(reply_failures.len(), 1);

        let count = get_actions_count(&pool, Some("reply"), Some("failure"))
            .await
            .expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn actions_count_no_filter() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");

        let count = get_actions_count(&pool, None, None).await.expect("count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn actions_count_empty_db() {
        let pool = init_test_db().await.expect("init db");
        let count = get_actions_count(&pool, None, None).await.expect("count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn paginated_offset_beyond_data_returns_empty() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");

        let page = get_actions_paginated(&pool, 10, 100, None, None)
            .await
            .expect("page");
        assert!(page.is_empty(), "offset past data should return empty");
    }

    #[tokio::test]
    async fn get_recent_actions_returns_limited_set() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", Some("first"), None)
            .await
            .expect("log");
        log_action(&pool, "reply", "success", Some("second"), None)
            .await
            .expect("log");
        log_action(&pool, "tweet", "success", Some("third"), None)
            .await
            .expect("log");

        let recent = get_recent_actions(&pool, 2).await.expect("get");
        assert_eq!(recent.len(), 2, "should respect limit");

        let all = get_recent_actions(&pool, 10).await.expect("get all");
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn log_action_with_null_message_and_metadata() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "cleanup", "success", None, None)
            .await
            .expect("log");

        let actions = get_actions_since(&pool, "2000-01-01T00:00:00Z", None)
            .await
            .expect("get");
        assert_eq!(actions.len(), 1);
        assert!(actions[0].message.is_none());
        assert!(actions[0].metadata.is_none());
    }

    #[tokio::test]
    async fn action_counts_since_future_returns_empty() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "search", "success", None, None)
            .await
            .expect("log");

        let counts = get_action_counts_since(&pool, "2099-01-01T00:00:00Z")
            .await
            .expect("counts");
        assert!(counts.is_empty());
    }

    #[tokio::test]
    async fn paginated_type_and_status_combined_count() {
        let pool = init_test_db().await.expect("init db");

        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "failure", None, None)
            .await
            .expect("log");
        log_action(&pool, "reply", "success", None, None)
            .await
            .expect("log");
        log_action(&pool, "tweet", "success", None, None)
            .await
            .expect("log");

        let count = get_actions_count(&pool, Some("reply"), Some("success"))
            .await
            .expect("count");
        assert_eq!(count, 2);

        let page = get_actions_paginated(&pool, 10, 0, Some("reply"), Some("success"))
            .await
            .expect("page");
        assert_eq!(page.len(), 2);
    }

    #[tokio::test]
    async fn log_action_for_different_accounts() {
        let pool = init_test_db().await.expect("init db");

        log_action_for(&pool, "acct_a", "search", "success", Some("a"), None)
            .await
            .expect("log a");
        log_action_for(&pool, "acct_b", "search", "success", Some("b"), None)
            .await
            .expect("log b");
        log_action_for(&pool, "acct_a", "reply", "success", Some("a2"), None)
            .await
            .expect("log a2");

        let actions_a = get_actions_since_for(&pool, "acct_a", "2000-01-01T00:00:00Z", None)
            .await
            .expect("get a");
        assert_eq!(actions_a.len(), 2);

        let actions_b = get_actions_since_for(&pool, "acct_b", "2000-01-01T00:00:00Z", None)
            .await
            .expect("get b");
        assert_eq!(actions_b.len(), 1);

        let count_a = get_actions_count_for(&pool, "acct_a", None, None)
            .await
            .expect("count a");
        assert_eq!(count_a, 2);

        let count_b = get_actions_count_for(&pool, "acct_b", None, None)
            .await
            .expect("count b");
        assert_eq!(count_b, 1);
    }

    #[tokio::test]
    async fn get_recent_actions_respects_limit() {
        let pool = init_test_db().await.expect("init db");

        for i in 0..5 {
            log_action(
                &pool,
                "search",
                "success",
                Some(&format!("Action {i}")),
                None,
            )
            .await
            .expect("log");
        }

        let recent = get_recent_actions(&pool, 0).await.expect("get");
        assert!(recent.is_empty(), "limit 0 should return empty");

        let recent = get_recent_actions(&pool, 1).await.expect("get");
        assert_eq!(recent.len(), 1);
    }
}
