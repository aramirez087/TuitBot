//! Token file I/O and filesystem path helpers.

use std::path::PathBuf;

use super::config::{StartupError, StoredTokens};

// ============================================================================
// Token File I/O
// ============================================================================

/// Default directory for Tuitbot data files (`~/.tuitbot/`).
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tuitbot")
}

/// Path to the token storage file (`~/.tuitbot/tokens.json`).
pub fn token_file_path() -> PathBuf {
    data_dir().join("tokens.json")
}

/// Load OAuth tokens from the default file path.
pub fn load_tokens_from_file() -> Result<StoredTokens, StartupError> {
    let path = token_file_path();
    let contents = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StartupError::AuthRequired
        } else {
            StartupError::Io(e)
        }
    })?;
    serde_json::from_str(&contents)
        .map_err(|e| StartupError::Other(format!("failed to parse tokens file: {e}")))
}

/// Save OAuth tokens to the default file path with secure permissions.
///
/// Creates the `~/.tuitbot/` directory if it does not exist.
/// On Unix, sets file permissions to 0600 (owner read/write only).
pub fn save_tokens_to_file(tokens: &StoredTokens) -> Result<(), StartupError> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;

    let path = token_file_path();
    let json = serde_json::to_string_pretty(tokens)
        .map_err(|e| StartupError::Other(format!("failed to serialize tokens: {e}")))?;
    std::fs::write(&path, json)?;

    // Set file permissions to 0600 on Unix (owner read/write only).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

// ============================================================================
// Path Helpers
// ============================================================================

/// Expand `~` at the start of a path to the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Resolve the database path by loading the config file and reading `storage.db_path`.
///
/// Falls back to `~/.tuitbot/tuitbot.db` if the config cannot be loaded.
/// Returns an error if the resolved `db_path` is empty, whitespace-only,
/// or points to an existing directory.
pub fn resolve_db_path(config_path: &str) -> Result<PathBuf, crate::error::ConfigError> {
    use crate::config::Config;
    let config = match Config::load(Some(config_path)) {
        Ok(c) => c,
        Err(_) => return Ok(data_dir().join("tuitbot.db")),
    };

    validate_db_path(&config.storage.db_path)
}

/// Validate and expand a `storage.db_path` value.
///
/// Rejects empty, whitespace-only, and directory paths with a clear error.
pub fn validate_db_path(raw: &str) -> Result<PathBuf, crate::error::ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(crate::error::ConfigError::InvalidValue {
            field: "storage.db_path".to_string(),
            message: "must not be empty or whitespace-only".to_string(),
        });
    }
    let expanded = expand_tilde(trimmed);
    if expanded.is_dir() {
        return Err(crate::error::ConfigError::InvalidValue {
            field: "storage.db_path".to_string(),
            message: format!("'{}' is a directory, must point to a file", trimmed),
        });
    }
    Ok(expanded)
}
