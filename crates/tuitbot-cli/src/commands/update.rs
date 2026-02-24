/// `tuitbot update` — unified binary self-update + config upgrade.
///
/// Phase 1: Check GitHub releases for a newer binary, download, verify SHA256,
///          and atomically replace the current binary.
/// Phase 2: Run config upgrade (reuses `upgrade.rs` logic) to patch missing
///          feature groups into the user's `config.toml`.
use std::fs;
use std::io::IsTerminal;

use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::Confirm;
use semver::Version;
use sha2::{Digest, Sha256};

use super::upgrade;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/aramirez087/TuitBot/releases/latest";

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Execute the `update` command.
pub async fn execute(
    non_interactive: bool,
    check_only: bool,
    config_only: bool,
    config_path_str: &str,
) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    let green = Style::new().green().bold();

    // Phase 1: Binary update
    if !config_only {
        eprintln!("{}", bold.apply_to("Checking for updates..."));
        eprintln!();

        match check_latest_release().await {
            Ok(release) => {
                let current =
                    Version::parse(CURRENT_VERSION).context("Failed to parse current version")?;

                match parse_version_from_tag(&release.tag_name) {
                    Some(latest) if is_newer(&latest, &current) => {
                        eprintln!(
                            "  {} {} → {}",
                            green.apply_to("New version available:"),
                            current,
                            latest
                        );

                        if check_only {
                            eprintln!();
                            eprintln!(
                                "{}",
                                dim.apply_to("Run 'tuitbot update' to install the update.")
                            );
                            return Ok(());
                        }

                        // Confirm with user in interactive mode
                        if !non_interactive && std::io::stdin().is_terminal() {
                            eprintln!();
                            let proceed = Confirm::new()
                                .with_prompt(format!("Update to v{latest}?"))
                                .default(true)
                                .interact()?;

                            if !proceed {
                                eprintln!("{}", dim.apply_to("Update skipped."));
                                eprintln!();
                                // Fall through to config upgrade
                                return run_config_upgrade(
                                    non_interactive,
                                    config_path_str,
                                    &bold,
                                    &dim,
                                );
                            }
                        }

                        // Download and replace binary
                        match update_binary(&release).await {
                            Ok(()) => {
                                eprintln!();
                                eprintln!("  {} Updated to v{}", green.apply_to("✓"), latest);
                            }
                            Err(e) => {
                                eprintln!();
                                eprintln!(
                                    "  {} Binary update failed: {e}",
                                    Style::new().red().bold().apply_to("✗"),
                                );
                                eprintln!(
                                    "  {}",
                                    dim.apply_to(
                                        "You can download manually from: https://github.com/aramirez087/TuitBot/releases"
                                    )
                                );
                            }
                        }
                    }
                    Some(latest) => {
                        eprintln!("  Already up to date (v{current}).");
                        if latest == current {
                            // exact match
                        } else {
                            eprintln!("  {}", dim.apply_to(format!("(latest release: v{latest})")));
                        }

                        if check_only {
                            return Ok(());
                        }
                    }
                    None => {
                        eprintln!(
                            "  {} Could not parse version from tag: {}",
                            Style::new().yellow().bold().apply_to("⚠"),
                            release.tag_name,
                        );

                        if check_only {
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "  {} Could not check for updates: {e}",
                    Style::new().yellow().bold().apply_to("⚠"),
                );
                eprintln!(
                    "  {}",
                    dim.apply_to("Skipping binary update, continuing with config upgrade...")
                );
            }
        }

        eprintln!();
    } else if check_only {
        bail!("--check and --config-only cannot be used together.");
    }

    // Phase 2: Config upgrade
    run_config_upgrade(non_interactive, config_path_str, &bold, &dim)
}

/// Pre-run check: hint about `tuitbot update` when config has missing features.
pub async fn check_before_run(config_path_str: &str) -> Result<()> {
    let config_path = upgrade::expand_tilde(config_path_str);

    if !config_path.exists() {
        return Ok(());
    }

    let missing = upgrade::detect_missing_features(&config_path)?;
    if missing.is_empty() {
        return Ok(());
    }

    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!(
        "{}",
        bold.apply_to("New features available in your config:")
    );
    for group in &missing {
        eprintln!("  • {} — {}", group.display_name(), group.description());
    }
    eprintln!();

    let configure_now = Confirm::new()
        .with_prompt("Configure new features now?")
        .default(false)
        .interact()?;

    if !configure_now {
        eprintln!(
            "{}",
            dim.apply_to("Tip: Run 'tuitbot update' any time to configure new features.")
        );
        eprintln!();
        return Ok(());
    }

    upgrade::run_upgrade_wizard(&config_path, &missing)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Config upgrade (Phase 2)
// ---------------------------------------------------------------------------

fn run_config_upgrade(
    non_interactive: bool,
    config_path_str: &str,
    bold: &Style,
    dim: &Style,
) -> Result<()> {
    let config_path = upgrade::expand_tilde(config_path_str);

    if !config_path.exists() {
        eprintln!(
            "  {}",
            dim.apply_to("No config file found — run 'tuitbot init' to create one.")
        );
        return Ok(());
    }

    eprintln!("{}", bold.apply_to("Checking configuration..."));

    let missing = upgrade::detect_missing_features(&config_path)?;

    if missing.is_empty() {
        eprintln!("  Config is up to date.");
        return Ok(());
    }

    eprintln!("  New feature groups to configure:");
    for group in &missing {
        eprintln!("    • {} — {}", group.display_name(), group.description());
    }
    eprintln!();

    if non_interactive {
        upgrade::apply_defaults(&config_path, &missing)?;
    } else if std::io::stdin().is_terminal() {
        upgrade::run_upgrade_wizard(&config_path, &missing)?;
    } else {
        eprintln!(
            "  {}",
            dim.apply_to(
                "Non-interactive terminal detected. Use --non-interactive to apply defaults."
            )
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// GitHub release API
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

async fn check_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::Client::builder()
        .user_agent(format!("tuitbot/{CURRENT_VERSION}"))
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = client
        .get(GITHUB_RELEASES_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("Failed to reach GitHub API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        bail!("GitHub API returned {status}");
    }

    resp.json::<GitHubRelease>()
        .await
        .context("Failed to parse GitHub release response")
}

// ---------------------------------------------------------------------------
// Version parsing and comparison
// ---------------------------------------------------------------------------

/// Extract a semver `Version` from a tag like `tuitbot-cli-v0.2.0`.
fn parse_version_from_tag(tag: &str) -> Option<Version> {
    // Try stripping known prefixes
    let version_str = tag
        .strip_prefix("tuitbot-cli-v")
        .or_else(|| tag.strip_prefix("v"))
        .unwrap_or(tag);

    Version::parse(version_str).ok()
}

/// Returns true if `latest` is strictly newer than `current`.
fn is_newer(latest: &Version, current: &Version) -> bool {
    latest > current
}

// ---------------------------------------------------------------------------
// Platform detection
// ---------------------------------------------------------------------------

/// Returns the platform target triple for asset name construction.
fn platform_target() -> Option<&'static str> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Some("x86_64-unknown-linux-gnu")
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        Some("aarch64-unknown-linux-gnu")
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        Some("x86_64-apple-darwin")
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Some("aarch64-apple-darwin")
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Some("x86_64-pc-windows-msvc")
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    {
        None
    }
}

/// Build the expected asset filename for this platform.
fn platform_asset_name() -> Option<String> {
    let target = platform_target()?;
    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    Some(format!("tuitbot-{target}.{ext}"))
}

// ---------------------------------------------------------------------------
// Binary update
// ---------------------------------------------------------------------------

async fn update_binary(release: &GitHubRelease) -> Result<()> {
    let asset_name =
        platform_asset_name().context("Unsupported platform for binary self-update")?;

    // Find the archive asset
    let archive_asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .with_context(|| format!("Release has no asset named '{asset_name}'"))?;

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
    let expected_hash = parse_sha256sums(&checksums_text, &asset_name)
        .context("Could not find checksum for asset in SHA256SUMS")?;

    verify_sha256(&archive_bytes, &expected_hash)?;
    eprintln!("  SHA256 verified.");

    // Extract binary from archive
    let binary_bytes = extract_binary(&archive_bytes)?;

    // Replace current binary atomically
    replace_binary(&binary_bytes)?;

    Ok(())
}

async fn download_asset(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let resp = client
        .get(url)
        .send()
        .await
        .context("Failed to download asset")?;

    if !resp.status().is_success() {
        bail!("Download failed with status {}", resp.status());
    }

    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .context("Failed to read asset bytes")
}

async fn download_asset_text(client: &reqwest::Client, url: &str) -> Result<String> {
    let resp = client
        .get(url)
        .send()
        .await
        .context("Failed to download checksums")?;

    if !resp.status().is_success() {
        bail!("Checksum download failed with status {}", resp.status());
    }

    resp.text().await.context("Failed to read checksum text")
}

/// Parse a SHA256SUMS file to find the hash for a specific filename.
///
/// Expected format: `<hex_hash>  <filename>` (two spaces between hash and name).
fn parse_sha256sums(content: &str, filename: &str) -> Option<String> {
    for line in content.lines() {
        // Format: "<hash>  <filename>" or "<hash> <filename>"
        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() == 2 {
            let hash = parts[0].trim();
            let name = parts[1].trim();
            if name == filename {
                return Some(hash.to_lowercase());
            }
        }
    }
    None
}

/// Verify that the SHA256 hash of `data` matches `expected_hex`.
fn verify_sha256(data: &[u8], expected_hex: &str) -> Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual = format!("{:x}", hasher.finalize());

    if actual != expected_hex.to_lowercase() {
        bail!(
            "SHA256 mismatch!\n  Expected: {expected_hex}\n  Actual:   {actual}\n\
             The downloaded file may be corrupted. Aborting update."
        );
    }

    Ok(())
}

/// Extract the `tuitbot` binary from a tar.gz archive (Unix) or zip (Windows).
fn extract_binary(archive_bytes: &[u8]) -> Result<Vec<u8>> {
    #[cfg(not(target_os = "windows"))]
    {
        extract_from_tar_gz(archive_bytes)
    }
    #[cfg(target_os = "windows")]
    {
        extract_from_zip(archive_bytes)
    }
}

#[cfg(not(target_os = "windows"))]
fn extract_from_tar_gz(archive_bytes: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tar::Archive;

    let gz = GzDecoder::new(archive_bytes);
    let mut archive = Archive::new(gz);

    let binary_name = if cfg!(target_os = "windows") {
        "tuitbot.exe"
    } else {
        "tuitbot"
    };

    for entry in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry.context("Failed to read tar entry")?;
        let path = entry.path().context("Failed to read entry path")?;

        // Match the binary by filename (may be nested in a directory)
        if path.file_name().is_some_and(|name| name == binary_name) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .context("Failed to read binary from archive")?;
            return Ok(buf);
        }
    }

    bail!("Archive does not contain '{binary_name}'")
}

#[cfg(target_os = "windows")]
fn extract_from_zip(archive_bytes: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;

    let cursor = std::io::Cursor::new(archive_bytes);
    let mut archive = zip::ZipArchive::new(cursor).context("Failed to read zip archive")?;

    let binary_name = "tuitbot.exe";

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Failed to read zip entry")?;

        if file.name().ends_with(binary_name) {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .context("Failed to read binary from zip")?;
            return Ok(buf);
        }
    }

    bail!("Archive does not contain '{binary_name}'")
}

/// Atomically replace the current binary with new bytes.
fn replace_binary(new_binary: &[u8]) -> Result<()> {
    let current_exe =
        std::env::current_exe().context("Failed to determine current executable path")?;

    let current_exe = current_exe
        .canonicalize()
        .unwrap_or_else(|_| current_exe.clone());

    let parent = current_exe
        .parent()
        .context("Current executable has no parent directory")?;

    // Write to temp file in the same directory (same filesystem for rename)
    let temp_path = parent.join(".tuitbot-update-tmp");
    let old_path = parent.join(".tuitbot-old");

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
    if let Err(e) = fs::rename(&current_exe, &old_path) {
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
        return Err(e).with_context(|| {
            format!(
                "Failed to rename current binary.\nHint: You may need elevated permissions to update {}",
                current_exe.display()
            )
        });
    }

    if let Err(e) = fs::rename(&temp_path, &current_exe) {
        // Try to restore old binary
        let _ = fs::rename(&old_path, &current_exe);
        let _ = fs::remove_file(&temp_path);
        return Err(e).context("Failed to install new binary (old binary restored)");
    }

    // Best-effort cleanup of old binary
    let _ = fs::remove_file(&old_path);

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_from_cli_tag() {
        let v = parse_version_from_tag("tuitbot-cli-v0.2.0").unwrap();
        assert_eq!(v, Version::new(0, 2, 0));
    }

    #[test]
    fn parse_version_from_v_tag() {
        let v = parse_version_from_tag("v1.0.0").unwrap();
        assert_eq!(v, Version::new(1, 0, 0));
    }

    #[test]
    fn parse_version_from_bare_semver() {
        let v = parse_version_from_tag("0.3.1").unwrap();
        assert_eq!(v, Version::new(0, 3, 1));
    }

    #[test]
    fn parse_version_from_prerelease_tag() {
        let v = parse_version_from_tag("tuitbot-cli-v1.0.0-rc.1").unwrap();
        assert_eq!(v.major, 1);
        assert!(!v.pre.is_empty());
    }

    #[test]
    fn parse_version_invalid_tag() {
        assert!(parse_version_from_tag("not-a-version").is_none());
        assert!(parse_version_from_tag("").is_none());
        assert!(parse_version_from_tag("tuitbot-cli-vgarbage").is_none());
    }

    #[test]
    fn is_newer_returns_true_for_higher_version() {
        let latest = Version::new(0, 2, 0);
        let current = Version::new(0, 1, 0);
        assert!(is_newer(&latest, &current));
    }

    #[test]
    fn is_newer_returns_false_for_same_version() {
        let v = Version::new(0, 1, 0);
        assert!(!is_newer(&v, &v));
    }

    #[test]
    fn is_newer_returns_false_for_older_version() {
        let latest = Version::new(0, 1, 0);
        let current = Version::new(0, 2, 0);
        assert!(!is_newer(&latest, &current));
    }

    #[test]
    fn is_newer_handles_prerelease() {
        let release = Version::new(1, 0, 0);
        let prerelease = Version::parse("1.0.0-rc.1").unwrap();
        // A release is "newer" than its own prerelease
        assert!(is_newer(&release, &prerelease));
    }

    #[test]
    fn platform_asset_name_returns_some_on_supported() {
        // This test runs on the host platform; it should return Some on CI
        let name = platform_asset_name();
        if platform_target().is_some() {
            let name = name.unwrap();
            assert!(name.starts_with("tuitbot-"));
            assert!(name.ends_with(".tar.gz") || name.ends_with(".zip"));
        }
    }

    #[test]
    fn verify_sha256_valid() {
        let data = b"hello world";
        let hash = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_sha256(data, hash).is_ok());
    }

    #[test]
    fn verify_sha256_uppercase_expected() {
        let data = b"hello world";
        let hash = "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9";
        assert!(verify_sha256(data, hash).is_ok());
    }

    #[test]
    fn verify_sha256_invalid() {
        let data = b"hello world";
        let hash = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(verify_sha256(data, hash).is_err());
    }

    #[test]
    fn parse_sha256sums_finds_entry() {
        let content = "\
abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890  tuitbot-x86_64-unknown-linux-gnu.tar.gz
1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef  tuitbot-aarch64-apple-darwin.tar.gz
";
        let hash = parse_sha256sums(content, "tuitbot-aarch64-apple-darwin.tar.gz");
        assert_eq!(
            hash.as_deref(),
            Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
        );
    }

    #[test]
    fn parse_sha256sums_missing_entry() {
        let content = "abcdef1234567890  tuitbot-x86_64-unknown-linux-gnu.tar.gz\n";
        assert!(parse_sha256sums(content, "tuitbot-windows.zip").is_none());
    }

    #[test]
    fn parse_sha256sums_empty() {
        assert!(parse_sha256sums("", "anything").is_none());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn extract_binary_from_tar_gz() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::{Builder, Header};

        // Create a minimal tar.gz with a "tuitbot" file
        let mut gz_buf = Vec::new();
        {
            let gz = GzEncoder::new(&mut gz_buf, Compression::fast());
            let mut builder = Builder::new(gz);

            let content = b"#!/bin/sh\necho hello";
            let mut header = Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();

            builder
                .append_data(&mut header, "tuitbot", &content[..])
                .unwrap();
            builder.finish().unwrap();
            let gz = builder.into_inner().unwrap();
            gz.finish().unwrap();
        }

        let result = extract_from_tar_gz(&gz_buf).unwrap();
        assert_eq!(result, b"#!/bin/sh\necho hello");
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn extract_binary_missing_from_tar_gz() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::{Builder, Header};

        let mut gz_buf = Vec::new();
        {
            let gz = GzEncoder::new(&mut gz_buf, Compression::fast());
            let mut builder = Builder::new(gz);

            let content = b"not the binary";
            let mut header = Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            builder
                .append_data(&mut header, "README.md", &content[..])
                .unwrap();
            builder.finish().unwrap();
            let gz = builder.into_inner().unwrap();
            gz.finish().unwrap();
        }

        assert!(extract_from_tar_gz(&gz_buf).is_err());
    }
}
