//! Implementation of the `tuitbot restore` command.

use std::io::IsTerminal;
use std::path::PathBuf;

use tuitbot_core::startup::data_dir;
use tuitbot_core::storage;

use super::RestoreArgs;
use crate::output::CliOutput;

/// Execute the `tuitbot restore` command.
pub async fn execute(args: RestoreArgs, out: CliOutput) -> anyhow::Result<()> {
    let backup_path = PathBuf::from(&args.backup_path);

    if !backup_path.exists() {
        anyhow::bail!("Backup file not found: {}", backup_path.display());
    }

    // Validate.
    out.info(&format!("Validating backup: {}", backup_path.display()));
    let validation = storage::backup::validate_backup(&backup_path).await?;

    for msg in &validation.messages {
        out.info(&format!("  {msg}"));
    }

    if !validation.valid {
        anyhow::bail!("Backup validation failed. Aborting restore.");
    }

    out.info(&format!("  Tables: {}", validation.tables.join(", ")));

    if args.validate_only {
        if out.is_json() {
            out.json(&serde_json::json!({
                "status": "valid",
                "tables": validation.tables,
                "messages": validation.messages,
            }))?;
        } else {
            out.info("\nValidation passed. Use without --validate-only to restore.");
        }
        return Ok(());
    }

    // Confirm unless --force.
    let target = data_dir().join("tuitbot.db");
    if !args.force && std::io::stdin().is_terminal() {
        eprintln!("\nThis will replace the database at {}", target.display());
        eprintln!("A safety backup of the current database will be created first.");
        eprint!("Continue? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            out.info("Aborted.");
            return Ok(());
        }
    }

    // Restore.
    out.info("Restoring...");
    storage::backup::restore_from_backup(&backup_path, &target).await?;

    if out.is_json() {
        out.json(&serde_json::json!({
            "status": "restored",
            "target": target.display().to_string(),
        }))?;
    } else {
        out.info(&format!("Restore complete: {}", target.display()));
        out.info("Restart the server or run `tuitbot test` to verify connectivity.");
    }

    Ok(())
}
