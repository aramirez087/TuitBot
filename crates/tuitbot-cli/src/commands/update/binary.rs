use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::github::{
    available_asset_names, download_asset, download_asset_text, parse_sha256sums, verify_sha256,
    GitHubRelease,
};
use super::platform::platform_asset_name;
use super::CURRENT_VERSION;

// ---------------------------------------------------------------------------
// Public update entry points
// ---------------------------------------------------------------------------

/// Download, verify, extract, and replace the CLI binary (the running executable).
pub(super) async fn update_cli_binary(release: &GitHubRelease) -> Result<()> {
    let asset_name =
        platform_asset_name().context("Unsupported platform for binary self-update")?;

    let current_exe =
        std::env::current_exe().context("Failed to determine current executable path")?;

    let current_exe = current_exe
        .canonicalize()
        .unwrap_or_else(|_| current_exe.clone());

    update_target_binary(release, "tuitbot", &asset_name, &current_exe).await
}

/// Download, verify, extract, and replace an arbitrary binary at `target_path`.
pub(super) async fn update_target_binary(
    release: &GitHubRelease,
    binary_name: &str,
    asset_name: &str,
    target_path: &Path,
) -> Result<()> {
    // Find the archive asset
    let archive_asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .with_context(|| {
            format!(
                "Release '{}' has no asset named '{}'. Available assets: {}",
                release.tag_name,
                asset_name,
                available_asset_names(release)
            )
        })?;

    // Find the SHA256SUMS asset
    let checksums_asset = release
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS")
        .context("Release has no SHA256SUMS asset")?;

    let client = reqwest::Client::builder()
        .user_agent(format!("tuitbot/{CURRENT_VERSION}"))
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    // Download both in parallel
    eprintln!("  Downloading {asset_name}...");
    let (archive_bytes, checksums_text) = tokio::try_join!(
        download_asset(&client, &archive_asset.browser_download_url),
        download_asset_text(&client, &checksums_asset.browser_download_url),
    )?;

    // Verify SHA256
    let expected_hash = parse_sha256sums(&checksums_text, asset_name)
        .context("Could not find checksum for asset in SHA256SUMS")?;

    verify_sha256(&archive_bytes, &expected_hash)?;
    eprintln!("  SHA256 verified.");

    // Extract binary from archive
    let binary_bytes = extract_named_binary(&archive_bytes, binary_name)?;

    // Replace binary at target path
    replace_binary_at(&binary_bytes, target_path)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Server detection
// ---------------------------------------------------------------------------

/// Walk `PATH` looking for a `tuitbot-server` (or `tuitbot-server.exe`) binary.
///
/// Returns `None` if the server is not installed anywhere on `PATH`.
pub(super) fn detect_server_path() -> Option<PathBuf> {
    let binary_name = if cfg!(target_os = "windows") {
        "tuitbot-server.exe"
    } else {
        "tuitbot-server"
    };

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Archive extraction
// ---------------------------------------------------------------------------

/// Extract a named binary from a tar.gz archive (Unix) or zip (Windows).
pub(super) fn extract_named_binary(archive_bytes: &[u8], binary_name: &str) -> Result<Vec<u8>> {
    #[cfg(not(target_os = "windows"))]
    {
        extract_from_tar_gz(archive_bytes, binary_name)
    }
    #[cfg(target_os = "windows")]
    {
        extract_from_zip(archive_bytes, binary_name)
    }
}

#[cfg(not(target_os = "windows"))]
fn extract_from_tar_gz(archive_bytes: &[u8], binary_name: &str) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tar::Archive;

    let gz = GzDecoder::new(archive_bytes);
    let mut archive = Archive::new(gz);

    let target_name = if cfg!(target_os = "windows") {
        format!("{binary_name}.exe")
    } else {
        binary_name.to_string()
    };

    for entry in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry.context("Failed to read tar entry")?;
        let path = entry.path().context("Failed to read entry path")?;

        // Match the binary by filename (may be nested in a directory)
        if path
            .file_name()
            .is_some_and(|name| name == target_name.as_str())
        {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .context("Failed to read binary from archive")?;
            return Ok(buf);
        }
    }

    bail!("Archive does not contain '{target_name}'")
}

#[cfg(target_os = "windows")]
fn extract_from_zip(archive_bytes: &[u8], binary_name: &str) -> Result<Vec<u8>> {
    use std::io::Read;

    let target_name = format!("{binary_name}.exe");

    let cursor = std::io::Cursor::new(archive_bytes);
    let mut archive = zip::ZipArchive::new(cursor).context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Failed to read zip entry")?;

        if file.name().ends_with(&target_name) {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .context("Failed to read binary from zip")?;
            return Ok(buf);
        }
    }

    bail!("Archive does not contain '{target_name}'")
}

// ---------------------------------------------------------------------------
// Binary replacement
// ---------------------------------------------------------------------------

/// Atomically replace a binary at `target_path` with new bytes.
pub(super) fn replace_binary_at(new_binary: &[u8], target_path: &Path) -> Result<()> {
    let parent = target_path
        .parent()
        .context("Target binary has no parent directory")?;

    let stem = target_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "binary".to_string());

    // Write to temp file in the same directory (same filesystem for rename)
    let temp_path = parent.join(format!(".{stem}-update-tmp"));
    let old_path = parent.join(format!(".{stem}-old"));

    // Write new binary to temp file
    fs::write(&temp_path, new_binary).with_context(|| {
        format!(
            "Failed to write temporary file: {}\nHint: Check file permissions or try running with sudo.",
            temp_path.display()
        )
    })?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&temp_path, perms).context("Failed to set executable permissions")?;
    }

    // Atomic-ish replace: rename current → old, rename temp → current
    // If the first rename fails, nothing has changed.
    // If the second rename fails, we try to restore the old binary.
    if let Err(e) = fs::rename(target_path, &old_path) {
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
        return Err(e).with_context(|| {
            format!(
                "Failed to rename current binary.\nHint: You may need elevated permissions to update {}",
                target_path.display()
            )
        });
    }

    if let Err(e) = fs::rename(&temp_path, target_path) {
        // Try to restore old binary
        let _ = fs::rename(&old_path, target_path);
        let _ = fs::remove_file(&temp_path);
        return Err(e).context("Failed to install new binary (old binary restored)");
    }

    // Best-effort cleanup of old binary
    let _ = fs::remove_file(&old_path);

    Ok(())
}
