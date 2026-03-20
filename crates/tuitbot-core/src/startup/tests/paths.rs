//! Tests for path helpers: expand_tilde, data_dir, token_file_path,
//! validate_db_path, resolve_db_path, and extract_callback_state.

use std::path::PathBuf;

use crate::startup::db::{
    data_dir, expand_tilde, resolve_db_path, token_file_path, validate_db_path,
};
use crate::startup::services::extract_callback_state;

// ============================================================================
// Path Helpers
// ============================================================================

#[test]
fn expand_tilde_works() {
    let expanded = expand_tilde("~/.tuitbot/config.toml");
    assert!(!expanded.to_string_lossy().starts_with('~'));
}

#[test]
fn expand_tilde_no_tilde() {
    let expanded = expand_tilde("/absolute/path");
    assert_eq!(expanded, PathBuf::from("/absolute/path"));
}

#[test]
fn data_dir_under_home() {
    let dir = data_dir();
    assert!(dir.to_string_lossy().contains(".tuitbot"));
}

#[test]
fn token_file_path_under_data_dir() {
    let path = token_file_path();
    assert!(path.to_string_lossy().contains("tokens.json"));
    assert!(path.to_string_lossy().contains(".tuitbot"));
}

// ============================================================================
// extract_callback_state
// ============================================================================

#[test]
fn extract_callback_state_from_url() {
    let state =
        extract_callback_state("http://127.0.0.1:8080/callback?code=abc123&state=mystate456");
    assert_eq!(state, "mystate456");
}

#[test]
fn extract_callback_state_no_state() {
    let state = extract_callback_state("http://127.0.0.1:8080/callback?code=abc123");
    assert_eq!(state, "");
}

#[test]
fn extract_callback_state_state_only() {
    let state = extract_callback_state("state=xyz789");
    assert_eq!(state, "xyz789");
}

#[test]
fn extract_callback_state_with_http_suffix() {
    let state = extract_callback_state("/callback?code=abc&state=test123 HTTP/1.1");
    assert_eq!(state, "test123");
}

// ============================================================================
// validate_db_path
// ============================================================================

#[test]
fn validate_db_path_empty_rejected() {
    let result = validate_db_path("");
    assert!(result.is_err());
}

#[test]
fn validate_db_path_whitespace_rejected() {
    let result = validate_db_path("   ");
    assert!(result.is_err());
}

#[test]
fn validate_db_path_valid_path() {
    let result = validate_db_path("/tmp/test.db");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("/tmp/test.db"));
}

#[test]
fn validate_db_path_tilde_expansion() {
    let result = validate_db_path("~/.tuitbot/test.db");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(!path.to_string_lossy().starts_with('~'));
    assert!(path.to_string_lossy().contains("test.db"));
}

#[test]
fn validate_db_path_directory_rejected() {
    let tmp = std::env::temp_dir();
    let result = validate_db_path(tmp.to_str().unwrap());
    assert!(result.is_err());
}

// ============================================================================
// expand_tilde edge cases
// ============================================================================

#[test]
fn expand_tilde_bare_tilde() {
    let expanded = expand_tilde("~");
    if let Some(home) = dirs::home_dir() {
        assert_eq!(expanded, home);
    }
}

#[test]
fn expand_tilde_relative_path() {
    let expanded = expand_tilde("relative/path");
    assert_eq!(expanded, PathBuf::from("relative/path"));
}

// ============================================================================
// resolve_db_path
// ============================================================================

#[test]
fn resolve_db_path_nonexistent_config_falls_back() {
    let result = resolve_db_path("/nonexistent/path/to/config.toml");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("tuitbot.db"));
}

#[test]
fn resolve_db_path_valid_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        "[storage]\ndb_path = \"~/.tuitbot/custom.db\"\n",
    )
    .expect("write config");
    let result = resolve_db_path(config_path.to_str().unwrap());
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("custom.db"));
}

#[test]
fn resolve_db_path_empty_db_path_in_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[storage]\ndb_path = \"\"\n").expect("write config");
    let result = resolve_db_path(config_path.to_str().unwrap());
    assert!(result.is_err());
}
