//! Implementation of the `tuitbot uninstall` command.
//!
//! Discovers and removes Tuitbot data, tokens, database, and binaries
//! so users can cleanly uninstall or start fresh.

use std::fs;
use std::io;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use tuitbot_core::startup::data_dir;

// ============================================================================
// Inventory
// ============================================================================

/// A file found in the data directory.
struct DataFile {
    name: String,
    size: u64,
}

/// Summary of a subdirectory (e.g. `backups/`).
struct DataSubdir {
    name: String,
    file_count: usize,
    total_size: u64,
}

/// Everything we found on disk.
struct UninstallInventory {
    data_dir: PathBuf,
    data_dir_exists: bool,
    files: Vec<DataFile>,
    subdirs: Vec<DataSubdir>,
    cli_binary: Option<PathBuf>,
    server_binary: Option<PathBuf>,
    server_running: bool,
}

// ============================================================================
// Discovery
// ============================================================================

fn discover() -> UninstallInventory {
    let data = data_dir();
    let data_dir_exists = data.is_dir();

    let mut files = Vec::new();
    let mut subdirs = Vec::new();

    if data_dir_exists {
        if let Ok(entries) = fs::read_dir(&data) {
            for entry in entries.flatten() {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let name = entry.file_name().to_string_lossy().to_string();

                if meta.is_file() {
                    files.push(DataFile {
                        name,
                        size: meta.len(),
                    });
                } else if meta.is_dir() {
                    let (count, size) = dir_size(&entry.path());
                    subdirs.push(DataSubdir {
                        name: format!("{name}/"),
                        file_count: count,
                        total_size: size,
                    });
                }
            }
        }
        files.sort_by(|a, b| a.name.cmp(&b.name));
        subdirs.sort_by(|a, b| a.name.cmp(&b.name));
    }

    let cli_binary = std::env::current_exe()
        .ok()
        .filter(|p| p.exists() && is_installed_path(p));
    let server_binary = detect_server_path();
    let server_running = is_server_running();

    UninstallInventory {
        data_dir: data,
        data_dir_exists,
        files,
        subdirs,
        cli_binary,
        server_binary,
        server_running,
    }
}

/// Recursively compute (file_count, total_bytes) for a directory.
fn dir_size(path: &Path) -> (usize, u64) {
    let mut count = 0usize;
    let mut size = 0u64;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_file() {
                count += 1;
                size += meta.len();
            } else if meta.is_dir() {
                let (c, s) = dir_size(&entry.path());
                count += c;
                size += s;
            }
        }
    }

    (count, size)
}

/// Check whether a binary path looks like a standard install location
/// rather than a temporary/ad-hoc extraction directory.
///
/// Prevents uninstall from deleting locally extracted release artifacts
/// or binaries run from random working directories.
fn is_installed_path(path: &Path) -> bool {
    let s = path.to_string_lossy();

    // Standard cargo/system install roots
    if s.contains("/.cargo/bin/")
        || s.contains("/usr/local/bin/")
        || s.contains("/usr/bin/")
        || s.contains("/opt/")
        || s.contains("/bin/tuitbot")
    {
        return true;
    }

    // Windows: typical cargo install path
    #[cfg(windows)]
    if s.contains("\\.cargo\\bin\\") || s.contains("\\Program Files") {
        return true;
    }

    // macOS: Homebrew
    if s.contains("/Cellar/") || s.contains("/homebrew/") {
        return true;
    }

    false
}

/// Walk PATH to find `tuitbot-server` binary.
fn detect_server_path() -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join("tuitbot-server");
        if candidate.is_file() {
            return Some(candidate);
        }
        // Windows: check with .exe extension
        let candidate_exe = dir.join("tuitbot-server.exe");
        if candidate_exe.is_file() {
            return Some(candidate_exe);
        }
    }
    None
}

/// Quick TCP probe on 127.0.0.1:3001 to check if the server is running.
fn is_server_running() -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], 3001).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

// ============================================================================
// Display
// ============================================================================

fn print_inventory(inv: &UninstallInventory, data_only: bool) {
    use console::style;

    println!();
    println!("{}", style("Tuitbot Uninstall").bold());
    println!();

    if !inv.data_dir_exists && inv.cli_binary.is_none() && inv.server_binary.is_none() {
        println!("Nothing to remove — Tuitbot does not appear to be installed.");
        return;
    }

    println!("The following will be permanently removed:");
    println!();

    if inv.data_dir_exists {
        let display_dir = shorten_home(&inv.data_dir);
        println!("  {}  {display_dir}", style("Data:").bold());
        for f in &inv.files {
            println!("    {:<24}{}", f.name, format_size(f.size));
        }
        for d in &inv.subdirs {
            let label = format!("{} ({} files)", d.name, d.file_count);
            println!("    {:<24}{}", label, format_size(d.total_size));
        }
    } else {
        println!("  {}  ~/.tuitbot/ not found", style("Data:").bold());
    }

    if !data_only {
        println!();
        println!("  {}", style("Binaries:").bold());
        if let Some(ref p) = inv.server_binary {
            println!("    {}", shorten_home(p));
        }
        if let Some(ref p) = inv.cli_binary {
            println!("    {}", shorten_home(p));
        }
        if inv.server_binary.is_none() && inv.cli_binary.is_none() {
            println!("    (none found in PATH)");
        }
    }

    if inv.server_running {
        println!();
        println!(
            "  {} The tuitbot-server appears to be running on port 3001.",
            style("⚠").yellow()
        );
        println!("  Stop it first if you want a clean removal.");
    }

    println!();
}

/// Replace the user's home directory with `~` for display.
fn shorten_home(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(suffix) = path.strip_prefix(&home) {
            return format!("~/{}", suffix.display());
        }
    }
    path.display().to_string()
}

/// Human-readable byte sizes.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{} KB", bytes / KB)
    } else {
        format!("{bytes} B")
    }
}

// ============================================================================
// Removal
// ============================================================================

fn remove_data_dir(path: &Path) -> io::Result<()> {
    fs::remove_dir_all(path)
}

/// Remove a binary file. On Unix, `remove_file` unlinks the inode; the
/// running process keeps its file handle. On Windows, rename first then
/// best-effort delete.
fn remove_binary(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        fs::remove_file(path)
    }

    #[cfg(windows)]
    {
        let old_path = path.with_extension("old");
        fs::rename(path, &old_path)?;
        // Best-effort delete — the rename already made the original name available.
        let _ = fs::remove_file(&old_path);
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        fs::remove_file(path)
    }
}

// ============================================================================
// Main entry point
// ============================================================================

/// Execute the `tuitbot uninstall` command.
pub fn execute(force: bool, data_only: bool, out: crate::output::CliOutput) -> anyhow::Result<()> {
    let inv = discover();

    if !out.is_json() && !out.quiet {
        print_inventory(&inv, data_only);
    }

    if !inv.data_dir_exists && inv.cli_binary.is_none() && inv.server_binary.is_none() {
        if out.is_json() {
            out.json(&serde_json::json!({
                "status": "noop",
                "message": "Nothing to remove"
            }))?;
        } else {
            out.info("Nothing to remove — Tuitbot does not appear to be installed.");
        }
        return Ok(());
    }

    // Confirmation
    if !force {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt("This action is irreversible. Remove everything?")
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            out.info("Aborted.");
            return Ok(());
        }
        out.info("");
    }

    let mut removed = Vec::new();
    let mut errors = Vec::new();

    // 1. Data directory
    if inv.data_dir_exists {
        match remove_data_dir(&inv.data_dir) {
            Ok(()) => {
                let display = shorten_home(&inv.data_dir);
                out.info(&format!(
                    "  {} Removed {display}",
                    console::style("✓").green()
                ));
                removed.push(display);
            }
            Err(e) => {
                let display = shorten_home(&inv.data_dir);
                if !out.quiet {
                    eprintln!(
                        "  {} Failed to remove {display}: {e}",
                        console::style("✗").red()
                    );
                    if e.kind() == io::ErrorKind::PermissionDenied {
                        eprintln!("    Hint: try running with sudo");
                    }
                }
                errors.push(format!("Failed to remove {display}: {e}"));
            }
        }
    }

    if !data_only {
        // 2. Server binary
        if let Some(ref server_path) = inv.server_binary {
            match remove_binary(server_path) {
                Ok(()) => {
                    out.info(&format!(
                        "  {} Removed tuitbot-server",
                        console::style("✓").green()
                    ));
                    removed.push("tuitbot-server".to_string());
                }
                Err(e) => {
                    if !out.quiet {
                        eprintln!(
                            "  {} Failed to remove tuitbot-server: {e}",
                            console::style("✗").red()
                        );
                    }
                    errors.push(format!("Failed to remove tuitbot-server: {e}"));
                }
            }
        }

        // 3. CLI binary (self — last)
        if let Some(ref cli_path) = inv.cli_binary {
            match remove_binary(cli_path) {
                Ok(()) => {
                    out.info(&format!(
                        "  {} Removed tuitbot (this binary)",
                        console::style("✓").green()
                    ));
                    removed.push("tuitbot".to_string());
                }
                Err(e) => {
                    if !out.quiet {
                        eprintln!(
                            "  {} Failed to remove tuitbot: {e}",
                            console::style("✗").red()
                        );
                    }
                    errors.push(format!("Failed to remove tuitbot: {e}"));
                }
            }
        }
    }

    if out.is_json() {
        out.json(&serde_json::json!({
            "status": if errors.is_empty() { "success" } else { "partial" },
            "removed": removed,
            "errors": errors,
            "data_only": data_only,
        }))?;
    } else {
        // Summary
        out.info("");
        if data_only {
            if inv.data_dir_exists {
                out.info("Tuitbot data has been removed.");
            }
            out.info("");
            out.info("To reinitialize:");
            out.info("  tuitbot init");
        } else {
            out.info("Tuitbot has been completely removed.");
            out.info("");
            out.info("To reinstall:");
            out.info("  cargo install tuitbot-cli tuitbot-server");
            out.info("  tuitbot init");
        }
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024), "1 KB");
        assert_eq!(format_size(2048), "2 KB");
        assert_eq!(format_size(1024 * 340), "340 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 3 + 1024 * 200), "3.2 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_dir_size_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let (count, size) = dir_size(tmp.path());
        assert_eq!(count, 0);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_dir_size_with_files() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a.txt"), "hello").unwrap();
        fs::write(tmp.path().join("b.txt"), "world!").unwrap();
        let (count, size) = dir_size(tmp.path());
        assert_eq!(count, 2);
        assert_eq!(size, 11); // 5 + 6
    }

    // ── is_installed_path ────────────────────────────────────────────

    #[test]
    fn is_installed_path_cargo_bin() {
        assert!(is_installed_path(Path::new(
            "/home/user/.cargo/bin/tuitbot"
        )));
    }

    #[test]
    fn is_installed_path_usr_local_bin() {
        assert!(is_installed_path(Path::new("/usr/local/bin/tuitbot")));
    }

    #[test]
    fn is_installed_path_usr_bin() {
        assert!(is_installed_path(Path::new("/usr/bin/tuitbot")));
    }

    #[test]
    fn is_installed_path_opt() {
        assert!(is_installed_path(Path::new("/opt/tuitbot/bin/tuitbot")));
    }

    #[test]
    fn is_installed_path_bin_tuitbot() {
        assert!(is_installed_path(Path::new("/bin/tuitbot")));
    }

    #[test]
    fn is_installed_path_homebrew() {
        assert!(is_installed_path(Path::new(
            "/usr/local/Cellar/tuitbot/0.1.0/bin/tuitbot"
        )));
        assert!(is_installed_path(Path::new("/opt/homebrew/bin/tuitbot")));
    }

    #[test]
    fn is_installed_path_random_dir_is_false() {
        assert!(!is_installed_path(Path::new("/tmp/tuitbot")));
        assert!(!is_installed_path(Path::new(
            "/home/user/downloads/tuitbot"
        )));
        assert!(!is_installed_path(Path::new(
            "/home/user/projects/tuitbot/target/release/tuitbot"
        )));
    }

    // ── shorten_home ──────────────────────────────────────────────────

    #[test]
    fn shorten_home_with_non_home_path() {
        let path = Path::new("/tmp/file.txt");
        let shortened = shorten_home(path);
        assert_eq!(shortened, "/tmp/file.txt");
    }

    #[test]
    fn shorten_home_uses_tilde() {
        if let Some(home) = dirs::home_dir() {
            let path = home.join("some/file.txt");
            let shortened = shorten_home(&path);
            assert!(shortened.starts_with("~/"));
            assert!(shortened.contains("some/file.txt"));
        }
    }

    // ── is_server_running ─────────────────────────────────────────────

    #[test]
    fn is_server_running_returns_bool() {
        // Just verify it doesn't panic. In test env, server is unlikely running.
        let _running = is_server_running();
    }

    // ── dir_size recursive ────────────────────────────────────────────

    #[test]
    fn dir_size_nested_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let sub1 = tmp.path().join("sub1");
        let sub2 = sub1.join("sub2");
        fs::create_dir_all(&sub2).unwrap();
        fs::write(sub1.join("a.txt"), "aaa").unwrap();
        fs::write(sub2.join("b.txt"), "bb").unwrap();
        let (count, size) = dir_size(tmp.path());
        assert_eq!(count, 2);
        assert_eq!(size, 5); // 3 + 2
    }

    #[test]
    fn dir_size_nonexistent_dir() {
        let (count, size) = dir_size(Path::new("/nonexistent/path/unlikely"));
        assert_eq!(count, 0);
        assert_eq!(size, 0);
    }

    // ── DataFile / DataSubdir struct access ────────────────────────────

    #[test]
    fn data_file_struct_fields() {
        let f = DataFile {
            name: "config.toml".to_string(),
            size: 1024,
        };
        assert_eq!(f.name, "config.toml");
        assert_eq!(f.size, 1024);
    }

    #[test]
    fn data_subdir_struct_fields() {
        let d = DataSubdir {
            name: "backups/".to_string(),
            file_count: 3,
            total_size: 4096,
        };
        assert_eq!(d.name, "backups/");
        assert_eq!(d.file_count, 3);
        assert_eq!(d.total_size, 4096);
    }

    // ── UninstallInventory struct access ───────────────────────────────

    #[test]
    fn uninstall_inventory_nothing_to_remove() {
        let inv = UninstallInventory {
            data_dir: PathBuf::from("/tmp/test"),
            data_dir_exists: false,
            files: vec![],
            subdirs: vec![],
            cli_binary: None,
            server_binary: None,
            server_running: false,
        };
        assert!(!inv.data_dir_exists);
        assert!(inv.cli_binary.is_none());
        assert!(inv.server_binary.is_none());
        assert!(!inv.server_running);
    }

    // ── format_size additional ────────────────────────────────────────

    #[test]
    fn format_size_exact_1kb() {
        assert_eq!(format_size(1024), "1 KB");
    }

    #[test]
    fn format_size_exact_1mb() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn format_size_exact_1gb() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn format_size_large_gb() {
        assert_eq!(format_size(10 * 1024 * 1024 * 1024), "10.0 GB");
    }

    #[test]
    fn format_size_fractional_mb() {
        // 1.5 MB
        let bytes = 1024 * 1024 + 512 * 1024;
        let result = format_size(bytes);
        assert!(result.contains("MB"));
    }

    // ── is_installed_path additional ────────────────────────────────

    #[test]
    fn is_installed_path_target_debug() {
        assert!(!is_installed_path(Path::new(
            "/home/user/project/target/debug/tuitbot"
        )));
    }

    #[test]
    fn is_installed_path_temp_dir() {
        let tmp = std::env::temp_dir().join("tuitbot");
        assert!(!is_installed_path(&tmp));
    }

    // ── shorten_home additional ────────────────────────────────────

    #[test]
    fn shorten_home_root_path() {
        let path = Path::new("/");
        let shortened = shorten_home(path);
        assert_eq!(shortened, "/");
    }

    // ── dir_size additional ─────────────────────────────────────────

    #[test]
    fn dir_size_single_file() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("test.txt"), "x").unwrap();
        let (count, size) = dir_size(tmp.path());
        assert_eq!(count, 1);
        assert_eq!(size, 1);
    }

    #[test]
    fn dir_size_deeply_nested() {
        let tmp = tempfile::tempdir().unwrap();
        let deep = tmp.path().join("a").join("b").join("c");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("file.txt"), "deep content").unwrap();
        let (count, size) = dir_size(tmp.path());
        assert_eq!(count, 1);
        assert_eq!(size, 12); // "deep content".len()
    }

    // ── UninstallInventory with data ────────────────────────────────

    #[test]
    fn uninstall_inventory_with_data() {
        let inv = UninstallInventory {
            data_dir: PathBuf::from(std::env::temp_dir()),
            data_dir_exists: true,
            files: vec![DataFile {
                name: "config.toml".to_string(),
                size: 1024,
            }],
            subdirs: vec![DataSubdir {
                name: "backups/".to_string(),
                file_count: 2,
                total_size: 4096,
            }],
            cli_binary: Some(PathBuf::from("/usr/local/bin/tuitbot")),
            server_binary: None,
            server_running: false,
        };
        assert!(inv.data_dir_exists);
        assert_eq!(inv.files.len(), 1);
        assert_eq!(inv.subdirs.len(), 1);
        assert!(inv.cli_binary.is_some());
    }

    #[test]
    fn test_inventory_data_files() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("config.toml"), "key = 1").unwrap();
        fs::write(tmp.path().join("tokens.json"), "{}").unwrap();
        fs::create_dir(tmp.path().join("backups")).unwrap();
        fs::write(tmp.path().join("backups/b1.tar.gz"), "data").unwrap();

        let mut files = Vec::new();
        let mut subdirs = Vec::new();

        for entry in fs::read_dir(tmp.path()).unwrap().flatten() {
            let meta = entry.metadata().unwrap();
            let name = entry.file_name().to_string_lossy().to_string();
            if meta.is_file() {
                files.push(DataFile {
                    name,
                    size: meta.len(),
                });
            } else if meta.is_dir() {
                let (count, size) = dir_size(&entry.path());
                subdirs.push(DataSubdir {
                    name: format!("{name}/"),
                    file_count: count,
                    total_size: size,
                });
            }
        }

        assert_eq!(files.len(), 2);
        assert_eq!(subdirs.len(), 1);
        assert_eq!(subdirs[0].file_count, 1);
        assert_eq!(subdirs[0].total_size, 4);
    }
}
