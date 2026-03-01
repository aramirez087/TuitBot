//! Factory reset: clear all user data from the database.
//!
//! Deletes rows from all 30 user tables in FK-safe order within a single
//! transaction. Preserves the schema and `_sqlx_migrations` so the pool
//! and migration tracking remain usable.

use super::DbPool;
use crate::error::StorageError;

/// Statistics returned from a factory reset operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResetStats {
    /// Number of tables cleared.
    pub tables_cleared: u32,
    /// Total number of rows deleted across all tables.
    pub rows_deleted: u64,
}

/// FK-safe table deletion order (children before parents).
///
/// Derived from the 20 migration SQL files. Tables with foreign key
/// constraints appear first so their rows are deleted before the
/// referenced parent rows.
const TABLES_TO_CLEAR: &[&str] = &[
    // FK-constrained tables (children first)
    "draft_seeds",
    "original_tweets",
    "content_nodes",
    "thread_tweets",
    "account_roles",
    "target_tweets",
    "approval_edit_history",
    // No FK constraints below this line
    "reply_performance",
    "tweet_performance",
    "replies_sent",
    "discovered_tweets",
    "threads",
    "approval_queue",
    "scheduled_content",
    "target_accounts",
    "follower_snapshots",
    "content_scores",
    "strategy_reports",
    "rate_limits",
    "action_log",
    "cursors",
    "author_interactions",
    "media_uploads",
    "llm_usage",
    "x_api_usage",
    "mcp_telemetry",
    "mutation_audit",
    "source_contexts",
    "sessions",
    "accounts",
];

/// Clear all user data from the database within a single transaction.
///
/// Deletes all rows from 30 user tables in FK-safe order.
/// Preserves the schema (tables, indexes) and `_sqlx_migrations`.
///
/// Table names come from the compile-time `TABLES_TO_CLEAR` constant --
/// not from user input -- so the `format!` is safe from injection.
pub async fn factory_reset(pool: &DbPool) -> Result<ResetStats, StorageError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let mut rows_deleted: u64 = 0;
    let mut tables_cleared: u32 = 0;

    for table in TABLES_TO_CLEAR {
        let query = format!("DELETE FROM {table}");
        let result = sqlx::query(&query)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::Query { source: e })?;
        rows_deleted += result.rows_affected();
        tables_cleared += 1;
    }

    tx.commit()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    Ok(ResetStats {
        tables_cleared,
        rows_deleted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn factory_reset_clears_all_tables() {
        let pool = init_test_db().await.expect("init test db");

        // Insert additional sample data beyond migration seeds.
        sqlx::query("INSERT INTO accounts (id, label) VALUES ('acc1', 'Extra')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO account_roles (account_id, actor, role) \
             VALUES ('acc1', 'dashboard', 'composer')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO target_accounts (account_id, username) VALUES ('t1', 'target1')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO discovered_tweets (id, author_id, author_username, content) \
             VALUES ('tw1', 'auth1', 'someone', 'hello world')",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Run factory reset.
        let stats = factory_reset(&pool).await.expect("factory reset");
        assert_eq!(stats.tables_cleared, 30);
        // Migration seeds 1 account + 2 account_roles = 3 rows, plus our 4 = 7.
        assert!(stats.rows_deleted >= 7);

        // Verify all tables are empty.
        for table in TABLES_TO_CLEAR {
            let count: (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(count.0, 0, "table {table} should be empty after reset");
        }
    }

    #[tokio::test]
    async fn factory_reset_preserves_migrations() {
        let pool = init_test_db().await.expect("init test db");

        let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(before.0 > 0, "migrations table should have entries");

        factory_reset(&pool).await.expect("factory reset");

        let after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(before.0, after.0, "migrations should be untouched");
    }

    #[tokio::test]
    async fn factory_reset_returns_accurate_stats() {
        let pool = init_test_db().await.expect("init test db");

        // Clear migration-seeded data first so we start from a known state.
        factory_reset(&pool).await.expect("pre-clear");

        // Insert exactly 2 rows.
        sqlx::query("INSERT INTO accounts (id, label) VALUES ('a1', 'U1')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO accounts (id, label) VALUES ('a2', 'U2')")
            .execute(&pool)
            .await
            .unwrap();

        let stats = factory_reset(&pool).await.expect("factory reset");
        assert_eq!(stats.tables_cleared, 30);
        assert_eq!(stats.rows_deleted, 2);
    }

    #[tokio::test]
    async fn factory_reset_idempotent() {
        let pool = init_test_db().await.expect("init test db");

        // First reset clears migration-seeded rows.
        let stats1 = factory_reset(&pool).await.expect("first reset");
        assert_eq!(stats1.tables_cleared, 30);
        // Migration seeds 1 account + 2 account_roles = 3 rows.
        assert_eq!(stats1.rows_deleted, 3);

        // Second reset on now-empty DB succeeds with 0 rows.
        let stats2 = factory_reset(&pool).await.expect("second reset");
        assert_eq!(stats2.tables_cleared, 30);
        assert_eq!(stats2.rows_deleted, 0);
    }

    #[tokio::test]
    async fn tables_to_clear_covers_all_user_tables() {
        let pool = init_test_db().await.expect("init test db");

        // Query sqlite_master for all user tables (excluding internal ones).
        let all_tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master \
             WHERE type='table' \
             AND name NOT LIKE 'sqlite_%' \
             AND name != '_sqlx_migrations' \
             ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let table_names: Vec<&str> = all_tables.iter().map(|t| t.0.as_str()).collect();

        // Every table in the DB should be in TABLES_TO_CLEAR.
        for name in &table_names {
            assert!(
                TABLES_TO_CLEAR.contains(name),
                "table '{name}' exists in DB but is missing from TABLES_TO_CLEAR"
            );
        }

        // Every table in TABLES_TO_CLEAR should exist in the DB.
        for name in TABLES_TO_CLEAR {
            assert!(
                table_names.contains(name),
                "table '{name}' is in TABLES_TO_CLEAR but does not exist in DB"
            );
        }
    }
}
