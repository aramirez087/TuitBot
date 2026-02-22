//! SQLite storage layer for Tuitbot.
//!
//! Provides database initialization, connection pooling, and CRUD operations
//! for all persistent entities. Uses SQLx with WAL mode for concurrent access.

pub mod action_log;
pub mod analytics;
pub mod approval_queue;
pub mod author_interactions;
pub mod cleanup;
pub mod cursors;
pub mod rate_limits;
pub mod replies;
pub mod target_accounts;
pub mod threads;
pub mod tweets;

use crate::error::StorageError;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::str::FromStr;
use std::time::Duration;

/// Type alias for the SQLite connection pool.
pub type DbPool = sqlx::SqlitePool;

/// Initialize the SQLite database with optimal settings for a background daemon.
///
/// Creates the database file and parent directories if they don't exist,
/// configures WAL mode for concurrent read/write performance, runs embedded
/// migrations, and returns a connection pool.
pub async fn init_db(db_path: &str) -> Result<DbPool, StorageError> {
    let expanded = expand_tilde(db_path);

    // Create parent directories if needed
    if let Some(parent) = std::path::Path::new(&expanded).parent() {
        std::fs::create_dir_all(parent).map_err(|e| StorageError::Connection {
            source: sqlx::Error::Configuration(
                format!("failed to create directory {}: {e}", parent.display()).into(),
            ),
        })?;
    }

    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{expanded}"))
        .map_err(|e| StorageError::Connection { source: e })?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5))
        .optimize_on_close(true, None)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .min_connections(1)
        .idle_timeout(Duration::from_secs(300))
        .connect_with(connect_options)
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| StorageError::Migration { source: e })?;

    Ok(pool)
}

/// Initialize an in-memory SQLite database for testing.
///
/// Uses a shared cache so multiple connections can access the same in-memory database.
#[cfg(any(test, feature = "test-helpers"))]
pub async fn init_test_db() -> Result<DbPool, StorageError> {
    let connect_options = SqliteConnectOptions::from_str("sqlite::memory:")
        .map_err(|e| StorageError::Connection { source: e })?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(connect_options)
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| StorageError::Migration { source: e })?;

    Ok(pool)
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().to_string();
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn init_test_db_creates_all_tables() {
        let pool = init_test_db().await.expect("init test db");

        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("query tables");

        let table_names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
        assert!(table_names.contains(&"discovered_tweets"));
        assert!(table_names.contains(&"replies_sent"));
        assert!(table_names.contains(&"original_tweets"));
        assert!(table_names.contains(&"threads"));
        assert!(table_names.contains(&"thread_tweets"));
        assert!(table_names.contains(&"rate_limits"));
        assert!(table_names.contains(&"action_log"));
        assert!(table_names.contains(&"target_accounts"));
        assert!(table_names.contains(&"target_tweets"));
        assert!(table_names.contains(&"follower_snapshots"));
        assert!(table_names.contains(&"reply_performance"));
        assert!(table_names.contains(&"tweet_performance"));
        assert!(table_names.contains(&"content_scores"));
        assert!(table_names.contains(&"approval_queue"));
    }

    #[tokio::test]
    async fn init_test_db_idempotent() {
        let pool = init_test_db().await.expect("first init");
        // Running migrations again should not fail (SQLx tracks applied migrations)
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("second migration run");
    }

    #[tokio::test]
    async fn init_db_creates_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db_path_str = db_path.to_string_lossy().to_string();

        let pool = init_db(&db_path_str).await.expect("init db");
        assert!(db_path.exists());
        pool.close().await;
    }
}
