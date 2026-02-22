//! Append-only action log for auditing and status reporting.
//!
//! Records every action taken by the agent with timestamps,
//! status, and optional metadata in JSON format.

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
    sqlx::query(
        "INSERT INTO action_log (action_type, status, message, metadata) VALUES (?, ?, ?, ?)",
    )
    .bind(action_type)
    .bind(status)
    .bind(message)
    .bind(metadata)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Fetch action log entries since a given timestamp, optionally filtered by type.
///
/// Results are ordered by `created_at` ascending.
pub async fn get_actions_since(
    pool: &DbPool,
    since: &str,
    action_type: Option<&str>,
) -> Result<Vec<ActionLogEntry>, StorageError> {
    match action_type {
        Some(at) => sqlx::query_as::<_, ActionLogEntry>(
            "SELECT * FROM action_log WHERE created_at >= ? AND action_type = ? \
                 ORDER BY created_at ASC",
        )
        .bind(since)
        .bind(at)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e }),
        None => sqlx::query_as::<_, ActionLogEntry>(
            "SELECT * FROM action_log WHERE created_at >= ? ORDER BY created_at ASC",
        )
        .bind(since)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e }),
    }
}

/// Get counts of each action type since a given timestamp.
///
/// Returns a HashMap mapping action types to their counts.
pub async fn get_action_counts_since(
    pool: &DbPool,
    since: &str,
) -> Result<HashMap<String, i64>, StorageError> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT action_type, COUNT(*) as count FROM action_log \
         WHERE created_at >= ? GROUP BY action_type",
    )
    .bind(since)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().collect())
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
}
