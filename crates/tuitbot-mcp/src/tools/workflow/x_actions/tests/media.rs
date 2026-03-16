//! Media upload tool tests.

use super::*;

/// Create a unique temp directory for tests (no tempfile crate needed).
fn test_temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tuitbot_mcp_test_{name}_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

// ── Not configured tests ─────────────────────────────────────────────

#[tokio::test]
async fn upload_media_not_configured() {
    let state = make_state(None, None).await;
    let result = upload_media(&state, "photo.jpg", None, false).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn upload_media_unsupported_ext_with_client() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = upload_media(&state, "file.bmp", None, false).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "unsupported_media_type");
}

#[tokio::test]
async fn upload_media_unsupported_ext_no_ext() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = upload_media(&state, "noext", None, false).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "unsupported_media_type");
}

// ── Dry-run tests ────────────────────────────────────────────────────

#[tokio::test]
async fn upload_media_dry_run_no_client_unsupported() {
    let state = make_state(None, None).await;
    let result = upload_media(&state, "file.bmp", None, true).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "unsupported_media_type");
}

#[tokio::test]
async fn upload_media_dry_run_no_client_nonexistent_file() {
    let state = make_state(None, None).await;
    let result = upload_media(&state, "/nonexistent/photo.jpg", None, true).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "file_read_error");
}

#[tokio::test]
async fn upload_media_dry_run_valid_file() {
    let dir = test_temp_dir("dry_run_valid");
    let file_path = dir.join("test.jpg");
    std::fs::write(&file_path, vec![0u8; 100]).expect("write file");

    let state = make_state(None, None).await;
    let result = upload_media(&state, file_path.to_str().unwrap(), None, true).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true, "dry-run should succeed: {parsed}");
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["valid"], true);
    assert_eq!(parsed["data"]["media_type"], "image/jpeg");
    assert_eq!(parsed["data"]["file_size_bytes"], 100);

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn upload_media_dry_run_with_alt_text() {
    let dir = test_temp_dir("dry_run_alt");
    let file_path = dir.join("test.png");
    std::fs::write(&file_path, vec![0u8; 200]).expect("write file");

    let state = make_state(None, None).await;
    let result = upload_media(&state, file_path.to_str().unwrap(), Some("a picture"), true).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["alt_text"], "a picture");

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Read error tests ─────────────────────────────────────────────────

#[tokio::test]
async fn upload_media_file_not_found() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = upload_media(&state, "/nonexistent/photo.jpg", None, false).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "file_read_error");
}

// ── Dry-run with client present (exercises different path) ───────────

#[tokio::test]
async fn upload_media_dry_run_with_client() {
    let dir = test_temp_dir("dry_run_client");
    let file_path = dir.join("test.gif");
    std::fs::write(&file_path, vec![0u8; 500]).expect("write file");

    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = upload_media(&state, file_path.to_str().unwrap(), None, true).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true, "dry-run with client: {parsed}");
    assert_eq!(parsed["data"]["dry_run"], true);

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Scraper mutation guard tests ─────────────────────────────────────

#[tokio::test]
async fn upload_media_scraper_mutations_blocked() {
    let mut config = Config::default();
    config.x_api.provider_backend = "scraper".to_string();
    config.x_api.scraper_allow_mutations = false;
    let state =
        make_state_with_config(Some(Box::new(MockXApiClient)), Some("u1".into()), config).await;
    let result = upload_media(&state, "photo.jpg", None, false).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
}
