//! SQLite backup and restore using `VACUUM INTO`.
//!
//! Provides consistent backups even during active writes, validation
//! of backup files, and atomic restore with safety backup.

use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;

use super::DbPool;
use crate::error::StorageError;

/// Result of a successful backup operation.
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// Path to the backup file.
    pub path: PathBuf,
    /// Size of the backup file in bytes.
    pub size_bytes: u64,
    /// Duration of the backup operation in milliseconds.
    pub duration_ms: u64,
}

/// Information about an existing backup file.
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Path to the backup file.
    pub path: PathBuf,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Timestamp extracted from filename (if parseable).
    pub timestamp: Option<String>,
}

/// Result of validating a backup file.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the backup passed all checks.
    pub valid: bool,
    /// Tables found in the backup.
    pub tables: Vec<String>,
    /// Human-readable messages about the validation.
    pub messages: Vec<String>,
}

/// Create a consistent backup of the database using `VACUUM INTO`.
///
/// The backup is written to `backup_dir` with a timestamped filename.
/// Returns the backup result on success.
pub async fn create_backup(pool: &DbPool, backup_dir: &Path) -> Result<BackupResult, StorageError> {
    create_backup_with_prefix(pool, backup_dir, "tuitbot").await
}

/// Create a backup with a custom filename prefix.
async fn create_backup_with_prefix(
    pool: &DbPool,
    backup_dir: &Path,
    prefix: &str,
) -> Result<BackupResult, StorageError> {
    std::fs::create_dir_all(backup_dir).map_err(|e| StorageError::Connection {
        source: sqlx::Error::Configuration(
            format!(
                "failed to create backup directory {}: {e}",
                backup_dir.display()
            )
            .into(),
        ),
    })?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%3f");
    let filename = format!("{prefix}_{timestamp}.db");
    let backup_path = backup_dir.join(&filename);

    let start = Instant::now();

    let path_str = backup_path
        .to_str()
        .ok_or_else(|| StorageError::Connection {
            source: sqlx::Error::Configuration("backup path is not valid UTF-8".into()),
        })?
        .to_string();

    // VACUUM INTO creates a consistent copy of the database.
    let query = format!("VACUUM INTO '{}'", path_str.replace('\'', "''"));
    sqlx::query(&query)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    let duration_ms = start.elapsed().as_millis() as u64;

    let metadata = std::fs::metadata(&backup_path).map_err(|e| StorageError::Connection {
        source: sqlx::Error::Configuration(format!("failed to stat backup file: {e}").into()),
    })?;

    Ok(BackupResult {
        path: backup_path,
        size_bytes: metadata.len(),
        duration_ms,
    })
}

/// Validate a backup file by opening it read-only and checking expected tables.
pub async fn validate_backup(backup_path: &Path) -> Result<ValidationResult, StorageError> {
    if !backup_path.exists() {
        return Ok(ValidationResult {
            valid: false,
            tables: vec![],
            messages: vec![format!("File not found: {}", backup_path.display())],
        });
    }

    let path_str = backup_path.to_string_lossy();
    let options = SqliteConnectOptions::new()
        .filename(&*path_str)
        .read_only(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let mut messages = Vec::new();

    // Check tables.
    let rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' \
         AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' \
         ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let tables: Vec<String> = rows.iter().map(|r| r.get("name")).collect();

    // Check for expected core tables.
    let expected = [
        "action_log",
        "discovered_tweets",
        "replies_sent",
        "rate_limits",
    ];
    let mut missing = Vec::new();
    for table in &expected {
        if !tables.iter().any(|t| t == table) {
            missing.push(*table);
        }
    }

    let valid = missing.is_empty() && !tables.is_empty();

    if valid {
        messages.push(format!("Valid backup with {} tables", tables.len()));
    } else if tables.is_empty() {
        messages.push("No tables found in backup".to_string());
    } else {
        messages.push(format!("Missing expected tables: {}", missing.join(", ")));
    }

    // Integrity check.
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| "error".to_string());

    if integrity != "ok" {
        messages.push(format!("Integrity check failed: {integrity}"));
        return Ok(ValidationResult {
            valid: false,
            tables,
            messages,
        });
    }

    pool.close().await;

    Ok(ValidationResult {
        valid,
        tables,
        messages,
    })
}

/// Restore from a backup file to the target database path.
///
/// 1. Validates the backup.
/// 2. Creates a safety backup of the current database.
/// 3. Atomically copies the backup via temp file + rename.
pub async fn restore_from_backup(
    backup_path: &Path,
    target_path: &Path,
) -> Result<(), StorageError> {
    // 1. Validate.
    let validation = validate_backup(backup_path).await?;
    if !validation.valid {
        return Err(StorageError::Connection {
            source: sqlx::Error::Configuration(
                format!(
                    "Backup validation failed: {}",
                    validation.messages.join("; ")
                )
                .into(),
            ),
        });
    }

    // 2. Safety backup of current database (if it exists).
    if target_path.exists() {
        let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
        let safety_name = format!("pre_restore_{}.db", Utc::now().format("%Y%m%d_%H%M%S"));
        let safety_path = parent.join(safety_name);
        std::fs::copy(target_path, &safety_path).map_err(|e| StorageError::Connection {
            source: sqlx::Error::Configuration(
                format!("Failed to create safety backup: {e}").into(),
            ),
        })?;
        tracing::info!(
            path = %safety_path.display(),
            "Created safety backup of current database"
        );
    }

    // 3. Atomic copy via temp + rename.
    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
    let temp_path = parent.join(format!(
        ".tuitbot_restore_{}.tmp",
        Utc::now().timestamp_millis()
    ));

    std::fs::copy(backup_path, &temp_path).map_err(|e| StorageError::Connection {
        source: sqlx::Error::Configuration(format!("Failed to copy backup: {e}").into()),
    })?;

    std::fs::rename(&temp_path, target_path).map_err(|e| StorageError::Connection {
        source: sqlx::Error::Configuration(format!("Failed to rename temp to target: {e}").into()),
    })?;

    // Clean up WAL/SHM files from the old database.
    let wal_path = target_path.with_extension("db-wal");
    let shm_path = target_path.with_extension("db-shm");
    let _ = std::fs::remove_file(wal_path);
    let _ = std::fs::remove_file(shm_path);

    Ok(())
}

/// List backup files in a directory, sorted newest first.
pub fn list_backups(backup_dir: &Path) -> Vec<BackupInfo> {
    let mut backups = Vec::new();

    let entries = match std::fs::read_dir(backup_dir) {
        Ok(e) => e,
        Err(_) => return backups,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !name.starts_with("tuitbot_") || !name.ends_with(".db") {
            continue;
        }

        let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        // Extract timestamp from filename: tuitbot_YYYYMMDD_HHMMSS.db
        let timestamp = name
            .strip_prefix("tuitbot_")
            .and_then(|s| s.strip_suffix(".db"))
            .map(|s| s.to_string());

        backups.push(BackupInfo {
            path,
            size_bytes,
            timestamp,
        });
    }

    // Sort newest first by timestamp string.
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    backups
}

/// Delete the oldest backups, keeping `keep` most recent.
///
/// Returns the number of backups deleted.
pub fn prune_backups(backup_dir: &Path, keep: usize) -> Result<u32, StorageError> {
    let backups = list_backups(backup_dir);
    let mut deleted = 0u32;

    if backups.len() <= keep {
        return Ok(0);
    }

    for backup in backups.iter().skip(keep) {
        if let Err(e) = std::fs::remove_file(&backup.path) {
            tracing::warn!(
                path = %backup.path.display(),
                error = %e,
                "Failed to prune backup"
            );
        } else {
            deleted += 1;
        }
    }

    Ok(deleted)
}

/// Create a pre-migration backup of an existing database.
///
/// Skips if the database file doesn't exist or is empty (fresh init).
/// Creates the backup in a `backups/` sibling directory with a
/// `pre_migration_` prefix. Prunes old pre-migration backups (keep 3).
pub async fn preflight_migration_backup(db_path: &Path) -> Result<Option<PathBuf>, StorageError> {
    // Skip if DB doesn't exist or is empty.
    let metadata = match std::fs::metadata(db_path) {
        Ok(m) if m.len() > 0 => m,
        _ => return Ok(None),
    };

    tracing::info!(
        db = %db_path.display(),
        size_bytes = metadata.len(),
        "Creating pre-migration backup"
    );

    // Open a temporary read-only pool to the existing DB.
    let path_str = db_path.to_string_lossy();
    let options = SqliteConnectOptions::new()
        .filename(&*path_str)
        .read_only(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let backup_dir = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups");

    let result = create_backup_with_prefix(&pool, &backup_dir, "pre_migration").await?;

    pool.close().await;

    tracing::info!(
        path = %result.path.display(),
        size_bytes = result.size_bytes,
        duration_ms = result.duration_ms,
        "Pre-migration backup complete"
    );

    // Prune old pre-migration backups (keep 3).
    prune_preflight_backups(&backup_dir, 3);

    Ok(Some(result.path))
}

/// Prune old pre-migration backups, keeping `keep` most recent.
fn prune_preflight_backups(backup_dir: &Path, keep: usize) {
    let entries = match std::fs::read_dir(backup_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut pre_migration: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("pre_migration_") && n.ends_with(".db"))
        })
        .collect();

    // Sort newest first.
    pre_migration.sort_by(|a, b| b.cmp(a));

    for path in pre_migration.iter().skip(keep) {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_db;

    /// Create a file-based test DB (VACUUM INTO doesn't work with in-memory DBs).
    async fn file_test_db(dir: &std::path::Path) -> (DbPool, PathBuf) {
        let db_path = dir.join("test.db");
        let pool = init_db(&db_path.to_string_lossy())
            .await
            .expect("init file db");
        (pool, db_path)
    }

    #[tokio::test]
    async fn create_and_validate_backup() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let (pool, _db_path) = file_test_db(dir.path()).await;

        // Insert some data so the backup isn't empty.
        sqlx::query(
            "INSERT INTO action_log (action_type, status, message) \
             VALUES ('test', 'success', 'backup test')",
        )
        .execute(&pool)
        .await
        .expect("insert");

        let backup_dir = dir.path().join("backups");
        let result = create_backup(&pool, &backup_dir).await.expect("backup");

        assert!(result.path.exists());
        assert!(result.size_bytes > 0);
        assert!(result
            .path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("tuitbot_"));

        // Validate.
        let validation = validate_backup(&result.path).await.expect("validate");
        assert!(validation.valid);
        assert!(!validation.tables.is_empty());
        assert!(validation.tables.contains(&"action_log".to_string()));

        pool.close().await;
    }

    #[tokio::test]
    async fn validate_nonexistent_file() {
        let result = validate_backup(Path::new("/nonexistent/backup.db"))
            .await
            .expect("validate");
        assert!(!result.valid);
    }

    #[tokio::test]
    async fn list_and_prune_backups() {
        let dir = tempfile::tempdir().expect("create temp dir");

        // Create fake backup files.
        for i in 1..=5 {
            let name = format!("tuitbot_20240101_00000{i}.db");
            std::fs::write(dir.path().join(name), "fake").expect("write");
        }

        let backups = list_backups(dir.path());
        assert_eq!(backups.len(), 5);
        // Newest first.
        assert!(
            backups[0].timestamp.as_deref().unwrap() > backups[4].timestamp.as_deref().unwrap()
        );

        // Prune to keep 2.
        let pruned = prune_backups(dir.path(), 2).expect("prune");
        assert_eq!(pruned, 3);

        let remaining = list_backups(dir.path());
        assert_eq!(remaining.len(), 2);
    }

    #[tokio::test]
    async fn backup_and_restore() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let (pool, _db_path) = file_test_db(dir.path()).await;

        sqlx::query(
            "INSERT INTO action_log (action_type, status, message) \
             VALUES ('test', 'success', 'restore test')",
        )
        .execute(&pool)
        .await
        .expect("insert");

        let backup_dir = dir.path().join("backups");
        let result = create_backup(&pool, &backup_dir).await.expect("backup");
        pool.close().await;

        // Create a target path.
        let target = dir.path().join("restored.db");

        restore_from_backup(&result.path, &target)
            .await
            .expect("restore");

        assert!(target.exists());

        // Verify restored DB has the data.
        let options = SqliteConnectOptions::new()
            .filename(target.to_string_lossy().as_ref())
            .read_only(true);
        let restored_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("open restored");

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM action_log")
            .fetch_one(&restored_pool)
            .await
            .expect("count");
        assert_eq!(count.0, 1);
        restored_pool.close().await;
    }
}
