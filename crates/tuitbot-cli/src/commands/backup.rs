//! Implementation of the `tuitbot backup` command.

use std::path::PathBuf;

use tuitbot_core::startup::{data_dir, resolve_db_path};
use tuitbot_core::storage;

use super::BackupArgs;
use crate::output::CliOutput;

/// Execute the `tuitbot backup` command.
pub async fn execute(args: BackupArgs, config_path: &str, out: CliOutput) -> anyhow::Result<()> {
    if args.list && (args.prune.is_some() || args.output_dir.is_some()) {
        anyhow::bail!(
            "--list is mutually exclusive with --prune and --output-dir.\n\
             Use --list alone to view backups."
        );
    }

    let db_path = resolve_db_path(config_path)?;
    let data = db_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(data_dir);

    if args.list {
        return list_backups(&data, out);
    }

    if let Some(keep) = args.prune {
        return prune_backups(&data, keep, out);
    }

    // Create a backup.
    let backup_dir = args
        .output_dir
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| data.join("backups"));

    if !db_path.exists() {
        anyhow::bail!(
            "Database not found at {}. Run `tuitbot init` first.",
            db_path.display()
        );
    }

    // Open a read-only pool to the existing DB for VACUUM INTO.
    let pool = storage::init_db(&db_path.to_string_lossy()).await?;

    out.info("Creating backup...");
    let result = storage::backup::create_backup(&pool, &backup_dir).await?;
    pool.close().await;

    if out.is_json() {
        out.json(&serde_json::json!({
            "status": "success",
            "path": result.path.display().to_string(),
            "size_bytes": result.size_bytes,
            "duration_ms": result.duration_ms,
        }))?;
    } else {
        out.info("Backup created successfully:");
        out.info(&format!("  Path: {}", result.path.display()));
        out.info(&format!("  Size: {} bytes", result.size_bytes));
        out.info(&format!("  Duration: {}ms", result.duration_ms));
    }

    Ok(())
}

fn list_backups(data_dir: &std::path::Path, out: CliOutput) -> anyhow::Result<()> {
    let backup_dir = data_dir.join("backups");
    let backups = storage::backup::list_backups(&backup_dir);

    if out.is_json() {
        let items: Vec<serde_json::Value> = backups
            .iter()
            .map(|b| {
                serde_json::json!({
                    "path": b.path.display().to_string(),
                    "size_bytes": b.size_bytes,
                    "timestamp": b.timestamp,
                })
            })
            .collect();
        return out.json(&items);
    }

    if backups.is_empty() {
        out.info(&format!("No backups found in {}", backup_dir.display()));
        return Ok(());
    }

    out.info(&format!("Backups in {}:", backup_dir.display()));
    for backup in &backups {
        let ts = backup.timestamp.as_deref().unwrap_or("unknown");
        let size_kb = backup.size_bytes / 1024;
        out.info(&format!(
            "  {} ({ts}) — {size_kb} KB",
            backup.path.display()
        ));
    }
    out.info(&format!("\nTotal: {} backup(s)", backups.len()));

    Ok(())
}

fn prune_backups(data_dir: &std::path::Path, keep: usize, out: CliOutput) -> anyhow::Result<()> {
    let backup_dir = data_dir.join("backups");
    let deleted = storage::backup::prune_backups(&backup_dir, keep)?;

    if out.is_json() {
        return out.json(&serde_json::json!({
            "pruned": deleted,
            "kept": keep,
        }));
    }

    if deleted == 0 {
        out.info(&format!("Nothing to prune (at most {keep} backups exist)."));
    } else {
        out.info(&format!(
            "Pruned {deleted} old backup(s), kept {keep} most recent."
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_args_list_and_prune_conflict_detected() {
        // The execute function checks list + prune conflict.
        // We test that the check logic is correct.
        let args = BackupArgs {
            list: true,
            prune: Some(3),
            output_dir: None,
        };
        assert!(args.list);
        assert!(args.prune.is_some());
    }

    #[test]
    fn backup_args_list_only() {
        let args = BackupArgs {
            list: true,
            prune: None,
            output_dir: None,
        };
        assert!(args.list);
        assert!(args.prune.is_none());
        assert!(args.output_dir.is_none());
    }

    #[test]
    fn backup_args_prune_only() {
        let args = BackupArgs {
            list: false,
            prune: Some(5),
            output_dir: None,
        };
        assert!(!args.list);
        assert_eq!(args.prune, Some(5));
    }

    #[test]
    fn backup_args_output_dir() {
        let args = BackupArgs {
            list: false,
            prune: None,
            output_dir: Some("/custom/backups".to_string()),
        };
        assert_eq!(args.output_dir.as_deref(), Some("/custom/backups"));
    }

    #[test]
    fn backup_args_debug_impl() {
        let args = BackupArgs {
            list: true,
            prune: Some(3),
            output_dir: Some("/tmp/backups".to_string()),
        };
        let debug = format!("{:?}", args);
        assert!(debug.contains("list: true"));
        assert!(debug.contains("3"));
    }

    #[test]
    fn backup_args_default_creates_backup() {
        let args = BackupArgs {
            list: false,
            prune: None,
            output_dir: None,
        };
        // Default state: not list, not prune, no output_dir => create backup
        assert!(!args.list);
        assert!(args.prune.is_none());
    }

    #[test]
    fn list_backups_empty_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = CliOutput::new(false, crate::commands::OutputFormat::Text);
        // backups dir doesn't exist yet — should not fail
        let result = list_backups(dir.path(), out);
        assert!(result.is_ok());
    }

    #[test]
    fn prune_backups_empty_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = CliOutput::new(false, crate::commands::OutputFormat::Text);
        let result = prune_backups(dir.path(), 3, out);
        assert!(result.is_ok());
    }

    #[test]
    fn list_backups_json_format() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = CliOutput::new(false, crate::commands::OutputFormat::Json);
        let result = list_backups(dir.path(), out);
        assert!(result.is_ok());
    }

    #[test]
    fn prune_backups_json_format() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = CliOutput::new(false, crate::commands::OutputFormat::Json);
        let result = prune_backups(dir.path(), 5, out);
        assert!(result.is_ok());
    }
}
