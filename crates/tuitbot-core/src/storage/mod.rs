//! SQLite storage layer for Tuitbot.
//!
//! Provides database initialization, connection pooling, and CRUD operations
//! for all persistent entities. Uses SQLx with WAL mode for concurrent access.

pub mod accounts;
pub mod action_log;
pub mod analytics;
pub mod approval_queue;
pub mod author_interactions;
pub mod backup;
pub mod cleanup;
pub mod cursors;
pub mod health;
pub mod llm_usage;
pub mod mcp_telemetry;
pub mod media;
pub mod mutation_audit;
pub mod provenance;
pub mod rate_limits;
pub mod replies;
pub mod reset;
pub mod scheduled_content;
pub mod strategy;
pub mod target_accounts;
pub mod threads;
pub mod tweets;
pub mod watchtower;
pub mod x_api_usage;

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
    let trimmed = db_path.trim();
    if trimmed.is_empty() {
        return Err(StorageError::Connection {
            source: sqlx::Error::Configuration(
                "storage.db_path must not be empty or whitespace-only".into(),
            ),
        });
    }

    let expanded = expand_tilde(trimmed);

    if std::path::Path::new(&expanded).is_dir() {
        return Err(StorageError::Connection {
            source: sqlx::Error::Configuration(
                format!(
                    "storage.db_path '{}' is a directory, must point to a file",
                    trimmed
                )
                .into(),
            ),
        });
    }

    // Create parent directories if needed
    if let Some(parent) = std::path::Path::new(&expanded).parent() {
        std::fs::create_dir_all(parent).map_err(|e| StorageError::Connection {
            source: sqlx::Error::Configuration(
                format!("failed to create directory {}: {e}", parent.display()).into(),
            ),
        })?;
    }

    // Pre-migration backup: snapshot existing DB before running migrations.
    let db_file = std::path::Path::new(&expanded);
    if db_file.exists()
        && std::fs::metadata(db_file)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    {
        match backup::preflight_migration_backup(db_file).await {
            Ok(Some(path)) => {
                tracing::info!(path = %path.display(), "Pre-migration backup created");
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, "Pre-migration backup failed (non-fatal, continuing)");
            }
        }
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
pub fn expand_tilde(path: &str) -> String {
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
        assert!(table_names.contains(&"accounts"));
        assert!(table_names.contains(&"account_roles"));
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
        assert!(table_names.contains(&"scheduled_content"));
        assert!(table_names.contains(&"llm_usage"));
        assert!(table_names.contains(&"x_api_usage"));
        assert!(table_names.contains(&"mcp_telemetry"));
        assert!(table_names.contains(&"approval_edit_history"));
        assert!(table_names.contains(&"media_uploads"));
        assert!(table_names.contains(&"mutation_audit"));
        assert!(table_names.contains(&"source_contexts"));
        assert!(table_names.contains(&"content_nodes"));
        assert!(table_names.contains(&"draft_seeds"));
        // Draft Studio tables
        assert!(table_names.contains(&"content_revisions"));
        assert!(table_names.contains(&"content_tags"));
        assert!(table_names.contains(&"content_tag_assignments"));
        assert!(table_names.contains(&"content_activity"));
        // Vault tables
        assert!(table_names.contains(&"content_chunks"));
        assert!(table_names.contains(&"vault_provenance_links"));
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

    #[test]
    fn expand_tilde_with_home_prefix() {
        let expanded = expand_tilde("~/some/path");
        // Should not start with ~ anymore (unless no home dir)
        if dirs::home_dir().is_some() {
            assert!(!expanded.starts_with('~'));
            assert!(expanded.ends_with("some/path"));
        }
    }

    #[test]
    fn expand_tilde_bare_tilde() {
        let expanded = expand_tilde("~");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home.to_string_lossy().to_string());
        }
    }

    #[test]
    fn expand_tilde_no_tilde() {
        let expanded = expand_tilde("/absolute/path");
        assert_eq!(expanded, "/absolute/path");
    }

    #[test]
    fn expand_tilde_relative_path() {
        let expanded = expand_tilde("relative/path");
        assert_eq!(expanded, "relative/path");
    }

    #[test]
    fn expand_tilde_tilde_not_at_start() {
        // ~ in the middle should not be expanded
        let expanded = expand_tilde("/home/~user/dir");
        assert_eq!(expanded, "/home/~user/dir");
    }

    #[tokio::test]
    async fn init_db_empty_path_errors() {
        let result = init_db("").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("must not be empty"));
    }

    #[tokio::test]
    async fn init_db_whitespace_path_errors() {
        let result = init_db("   ").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn init_db_directory_path_errors() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let result = init_db(&dir.path().to_string_lossy()).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("directory"));
    }

    #[tokio::test]
    async fn init_db_creates_parent_directories() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let nested_path = dir.path().join("a").join("b").join("c").join("test.db");
        let db_path_str = nested_path.to_string_lossy().to_string();

        let pool = init_db(&db_path_str).await.expect("init db");
        assert!(nested_path.exists());
        pool.close().await;
    }

    #[tokio::test]
    async fn init_db_idempotent_file_db() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("idempotent.db");
        let db_path_str = db_path.to_string_lossy().to_string();

        let pool1 = init_db(&db_path_str).await.expect("first init");
        pool1.close().await;

        // Second init on same path should succeed
        let pool2 = init_db(&db_path_str).await.expect("second init");
        pool2.close().await;
    }
}
