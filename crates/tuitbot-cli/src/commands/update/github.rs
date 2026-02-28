use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use super::CURRENT_VERSION;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub(super) struct GitHubRelease {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

// ---------------------------------------------------------------------------
// API calls
// ---------------------------------------------------------------------------

const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/aramirez087/TuitBot/releases?per_page=50";

pub(super) async fn check_recent_releases() -> Result<Vec<GitHubRelease>> {
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

    resp.json::<Vec<GitHubRelease>>()
        .await
        .context("Failed to parse GitHub release response")
}

// ---------------------------------------------------------------------------
// Download helpers
// ---------------------------------------------------------------------------

pub(super) async fn download_asset(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
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

pub(super) async fn download_asset_text(client: &reqwest::Client, url: &str) -> Result<String> {
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

// ---------------------------------------------------------------------------
// Checksum utilities
// ---------------------------------------------------------------------------

/// Parse a SHA256SUMS file to find the hash for a specific filename.
///
/// Expected format: `<hex_hash>  <filename>` (two spaces between hash and name).
pub(super) fn parse_sha256sums(content: &str, filename: &str) -> Option<String> {
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
pub(super) fn verify_sha256(data: &[u8], expected_hex: &str) -> Result<()> {
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

/// Returns true when a release contains both the platform archive and SHA256SUMS.
pub(super) fn has_update_assets(release: &GitHubRelease, asset_name: &str) -> bool {
    release.assets.iter().any(|a| a.name == asset_name)
        && release.assets.iter().any(|a| a.name == "SHA256SUMS")
}

pub(super) fn available_asset_names(release: &GitHubRelease) -> String {
    if release.assets.is_empty() {
        return "(none)".to_string();
    }

    release
        .assets
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
