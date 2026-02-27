//! File-based bearer token management.
//!
//! On first start, generates a random 256-bit token (hex-encoded) and writes it
//! to `~/.tuitbot/api_token`. Tauri and CLI clients read this file directly.

use std::path::Path;

use rand::RngCore;

/// Ensure the API token file exists, creating one if needed.
///
/// Returns the token string. The file is written with restrictive permissions
/// so only the current user can read it.
pub fn ensure_api_token(config_dir: &Path) -> anyhow::Result<String> {
    let token_path = config_dir.join("api_token");

    if token_path.exists() {
        let token = std::fs::read_to_string(&token_path)?.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // Generate a random 256-bit (32-byte) token and hex-encode it.
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = hex::encode(bytes);

    // Ensure the directory exists.
    std::fs::create_dir_all(config_dir)?;

    std::fs::write(&token_path, &token)?;

    // Set file permissions to 0600 (owner read/write only) on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600))?;
    }

    tracing::info!(path = %token_path.display(), "Generated new API token");

    Ok(token)
}
