//! Scraper session management — load/save cookie-based auth sessions.
//!
//! A scraper session stores the two browser cookies (`auth_token` and `ct0`)
//! needed to authenticate with X's internal API endpoints. Sessions are
//! persisted as JSON in the data directory.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Cookie-based authentication session extracted from a browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperSession {
    /// The `auth_token` cookie value.
    pub auth_token: String,
    /// The `ct0` cookie value (CSRF token).
    pub ct0: String,
    /// Optional X username associated with this session.
    #[serde(default)]
    pub username: Option<String>,
    /// ISO 8601 timestamp when this session was created/imported.
    #[serde(default)]
    pub created_at: Option<String>,
}

impl ScraperSession {
    /// Load a session from a JSON file. Returns `None` if the file does not exist.
    pub fn load(path: &Path) -> Result<Option<Self>, std::io::Error> {
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(path)?;
        let session: Self = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        if session.auth_token.is_empty() || session.ct0.is_empty() {
            return Ok(None);
        }
        Ok(Some(session))
    }

    /// Save this session to a JSON file with restrictive permissions.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        std::fs::write(path, &json)?;

        // Set 0600 permissions on Unix so only the owner can read cookies.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    /// Delete the session file if it exists.
    pub fn delete(path: &Path) -> Result<bool, std::io::Error> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_returns_none_when_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("session.json");
        assert!(ScraperSession::load(&path).unwrap().is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("session.json");
        let session = ScraperSession {
            auth_token: "abc123".to_string(),
            ct0: "csrf456".to_string(),
            username: Some("testuser".to_string()),
            created_at: Some("2026-03-05T12:00:00Z".to_string()),
        };
        session.save(&path).unwrap();
        let loaded = ScraperSession::load(&path).unwrap().unwrap();
        assert_eq!(loaded.auth_token, "abc123");
        assert_eq!(loaded.ct0, "csrf456");
        assert_eq!(loaded.username.as_deref(), Some("testuser"));
    }

    #[test]
    fn load_returns_none_for_empty_tokens() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("session.json");
        let session = ScraperSession {
            auth_token: String::new(),
            ct0: "csrf".to_string(),
            username: None,
            created_at: None,
        };
        session.save(&path).unwrap();
        assert!(ScraperSession::load(&path).unwrap().is_none());
    }

    #[test]
    fn delete_returns_false_when_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("session.json");
        assert!(!ScraperSession::delete(&path).unwrap());
    }

    #[test]
    fn delete_removes_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("session.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(ScraperSession::delete(&path).unwrap());
        assert!(!path.exists());
    }
}
