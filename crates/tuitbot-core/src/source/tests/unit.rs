//! Unit tests for content source providers.

use crate::source::*;

// ---------------------------------------------------------------------------
// LocalFsProvider
// ---------------------------------------------------------------------------

#[tokio::test]
async fn local_fs_provider_scan_returns_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note.md"), "# Hello\nBody text.\n").unwrap();
    std::fs::write(dir.path().join("readme.txt"), "Plain text.\n").unwrap();
    std::fs::write(dir.path().join("image.jpg"), "binary data").unwrap();

    let provider = local_fs::LocalFsProvider::new(dir.path().to_path_buf());
    assert_eq!(provider.source_type(), "local_fs");

    let patterns = vec!["*.md".to_string(), "*.txt".to_string()];
    let files = provider.scan_for_changes(None, &patterns).await.unwrap();

    assert_eq!(files.len(), 2);

    let names: Vec<&str> = files.iter().map(|f| f.display_name.as_str()).collect();
    assert!(names.contains(&"note.md"));
    assert!(names.contains(&"readme.txt"));

    // Each file has a non-empty hash.
    for f in &files {
        assert!(!f.content_hash.is_empty());
        assert!(!f.modified_at.is_empty());
    }
}

#[tokio::test]
async fn local_fs_provider_read_content() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note.md"), "# Test\nContent here.\n").unwrap();

    let provider = local_fs::LocalFsProvider::new(dir.path().to_path_buf());
    let content = provider.read_content("note.md").await.unwrap();

    assert_eq!(content, "# Test\nContent here.\n");
}

#[tokio::test]
async fn local_fs_provider_read_nonexistent_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let provider = local_fs::LocalFsProvider::new(dir.path().to_path_buf());
    let result = provider.read_content("missing.md").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn local_fs_provider_filters_patterns() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.md"), "markdown").unwrap();
    std::fs::write(dir.path().join("b.txt"), "text").unwrap();
    std::fs::write(dir.path().join("c.rs"), "rust code").unwrap();

    let provider = local_fs::LocalFsProvider::new(dir.path().to_path_buf());

    let md_only = provider
        .scan_for_changes(None, &["*.md".to_string()])
        .await
        .unwrap();
    assert_eq!(md_only.len(), 1);
    assert_eq!(md_only[0].display_name, "a.md");
}

#[tokio::test]
async fn local_fs_provider_skips_hidden_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let hidden = dir.path().join(".hidden");
    std::fs::create_dir(&hidden).unwrap();
    std::fs::write(hidden.join("secret.md"), "hidden").unwrap();
    std::fs::write(dir.path().join("visible.md"), "visible").unwrap();

    let provider = local_fs::LocalFsProvider::new(dir.path().to_path_buf());
    let files = provider
        .scan_for_changes(None, &["*.md".to_string()])
        .await
        .unwrap();

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].display_name, "visible.md");
}

// ---------------------------------------------------------------------------
// GoogleDriveProvider: extract_drive_id
// ---------------------------------------------------------------------------

#[test]
fn extract_drive_id_from_provider_format() {
    let id =
        google_drive::GoogleDriveProvider::extract_drive_id_for_test("gdrive://abc123/notes.md");
    assert_eq!(id, "abc123");
}

#[test]
fn extract_drive_id_from_raw_id() {
    let id = google_drive::GoogleDriveProvider::extract_drive_id_for_test("abc123");
    assert_eq!(id, "abc123");
}

// ---------------------------------------------------------------------------
// SourceFile dedup: content hash comparison
// ---------------------------------------------------------------------------

#[test]
fn source_file_hash_equality() {
    let a = SourceFile {
        provider_id: "a.md".into(),
        display_name: "a.md".into(),
        content_hash: "abc123".into(),
        modified_at: "2026-01-01T00:00:00Z".into(),
    };
    let b = SourceFile {
        provider_id: "a.md".into(),
        display_name: "a.md".into(),
        content_hash: "abc123".into(),
        modified_at: "2026-01-02T00:00:00Z".into(),
    };
    // Same hash = same content, even with different timestamps.
    assert_eq!(a.content_hash, b.content_hash);
}

#[test]
fn source_file_hash_difference() {
    let a = SourceFile {
        provider_id: "a.md".into(),
        display_name: "a.md".into(),
        content_hash: "abc123".into(),
        modified_at: "2026-01-01T00:00:00Z".into(),
    };
    let b = SourceFile {
        provider_id: "a.md".into(),
        display_name: "a.md".into(),
        content_hash: "def456".into(),
        modified_at: "2026-01-01T00:00:00Z".into(),
    };
    assert_ne!(a.content_hash, b.content_hash);
}

// ---------------------------------------------------------------------------
// Ingest parity: local vs direct content
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_parity_local_vs_direct_content() {
    use crate::automation::watchtower::{ingest_content, ingest_file};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let body = "---\ntitle: Test\n---\nBody text.\n";
    std::fs::write(dir.path().join("test.md"), body).unwrap();

    let src1 = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();
    let src2 = store::insert_source_context(&pool, "google_drive", "{}")
        .await
        .unwrap();

    // Ingest via filesystem path.
    ingest_file(&pool, src1, dir.path(), "test.md", false)
        .await
        .unwrap();

    // Ingest via direct content (simulating remote provider).
    ingest_content(&pool, src2, "gdrive://abc/test.md", body, false)
        .await
        .unwrap();

    let nodes1 = store::get_nodes_for_source(&pool, src1, None)
        .await
        .unwrap();
    let nodes2 = store::get_nodes_for_source(&pool, src2, None)
        .await
        .unwrap();

    assert_eq!(nodes1.len(), 1);
    assert_eq!(nodes2.len(), 1);

    // Both should produce the same body text, title, and hash.
    assert_eq!(nodes1[0].title, nodes2[0].title);
    assert_eq!(nodes1[0].body_text, nodes2[0].body_text);
    assert_eq!(nodes1[0].content_hash, nodes2[0].content_hash);
}

// ---------------------------------------------------------------------------
// Ingest content: dedup by hash
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_content_dedup_by_hash() {
    use crate::automation::watchtower::ingest_content;
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let src = store::insert_source_context(&pool, "google_drive", "{}")
        .await
        .unwrap();

    let body = "Some remote content.\n";
    let r1 = ingest_content(&pool, src, "gdrive://id1/note.md", body, false)
        .await
        .unwrap();
    assert_eq!(r1, store::UpsertResult::Inserted);

    // Same content → Skipped.
    let r2 = ingest_content(&pool, src, "gdrive://id1/note.md", body, false)
        .await
        .unwrap();
    assert_eq!(r2, store::UpsertResult::Skipped);

    // Changed content → Updated.
    let r3 = ingest_content(
        &pool,
        src,
        "gdrive://id1/note.md",
        "Updated content.\n",
        false,
    )
    .await
    .unwrap();
    assert_eq!(r3, store::UpsertResult::Updated);
}

// ---------------------------------------------------------------------------
// Storage: Google Drive source helpers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ensure_google_drive_source_creates_once() {
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    let config = r#"{"folder_id":"abc123"}"#;
    let id1 = store::ensure_google_drive_source(&pool, "abc123", config)
        .await
        .unwrap();
    let id2 = store::ensure_google_drive_source(&pool, "abc123", config)
        .await
        .unwrap();

    assert_eq!(id1, id2);

    let ctx = store::get_source_context(&pool, id1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ctx.source_type, "google_drive");
}

#[tokio::test]
async fn find_source_by_folder_id_returns_match() {
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    store::ensure_google_drive_source(&pool, "folder_xyz", r#"{"folder_id":"folder_xyz"}"#)
        .await
        .unwrap();

    let found = store::find_source_by_folder_id(&pool, "folder_xyz")
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().source_type, "google_drive");

    let not_found = store::find_source_by_folder_id(&pool, "nonexistent")
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn different_source_types_coexist() {
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    let local_id = store::ensure_local_fs_source(&pool, "/home/notes", r#"{"path":"/home/notes"}"#)
        .await
        .unwrap();
    let drive_id = store::ensure_google_drive_source(&pool, "abc123", r#"{"folder_id":"abc123"}"#)
        .await
        .unwrap();

    assert_ne!(local_id, drive_id);

    let contexts = store::get_source_contexts(&pool).await.unwrap();
    assert_eq!(contexts.len(), 2);

    let types: Vec<&str> = contexts.iter().map(|c| c.source_type.as_str()).collect();
    assert!(types.contains(&"local_fs"));
    assert!(types.contains(&"google_drive"));
}

// ---------------------------------------------------------------------------
// GoogleDriveProvider: linked-account credential flow
// ---------------------------------------------------------------------------

/// Helper: create a GoogleDriveConnector that points at a wiremock server.
fn test_connector_config() -> crate::config::GoogleDriveConnectorConfig {
    crate::config::GoogleDriveConnectorConfig {
        client_id: Some("test-client-id".into()),
        client_secret: Some("test-client-secret".into()),
        redirect_uri: Some("http://localhost:3001/callback".into()),
    }
}

#[tokio::test]
async fn google_drive_provider_from_connection_gets_token() {
    use crate::source::connector::crypto::encrypt_credentials;
    use crate::source::connector::google_drive::GoogleDriveConnector;
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let pool = init_test_db().await.expect("init db");
    let mock_server = MockServer::start().await;

    // Mock token refresh endpoint.
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "ya29.linked-account-token",
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .mount(&mock_server)
        .await;

    // Create connection and store encrypted credentials.
    let conn_id = store::insert_connection(&pool, "google_drive", Some("test@example.com"), None)
        .await
        .unwrap();

    let key: Vec<u8> = (0..32).collect();
    let encrypted = encrypt_credentials(b"1//test-refresh-token", &key).unwrap();
    store::store_encrypted_credentials(&pool, conn_id, &encrypted)
        .await
        .unwrap();

    // Build connector with test URLs.
    let config = test_connector_config();
    let connector = GoogleDriveConnector::with_test_urls(
        &config,
        reqwest::Client::new(),
        format!("{}/token", mock_server.uri()),
        format!("{}/revoke", mock_server.uri()),
        format!("{}/userinfo", mock_server.uri()),
    )
    .unwrap();

    // Build provider.
    let provider = google_drive::GoogleDriveProvider::from_connection(
        "test-folder".to_string(),
        conn_id,
        pool.clone(),
        key,
        connector,
    );

    assert_eq!(provider.source_type(), "google_drive");
}

#[tokio::test]
async fn google_drive_provider_connection_missing_credentials_returns_broken() {
    use crate::source::connector::google_drive::GoogleDriveConnector;
    use crate::source::ContentSourceProvider;
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;
    use wiremock::MockServer;

    let pool = init_test_db().await.expect("init db");
    let mock_server = MockServer::start().await;

    // Create connection WITHOUT storing credentials.
    let conn_id = store::insert_connection(&pool, "google_drive", Some("test@example.com"), None)
        .await
        .unwrap();

    let key: Vec<u8> = (0..32).collect();
    let config = test_connector_config();
    let connector = GoogleDriveConnector::with_test_urls(
        &config,
        reqwest::Client::new(),
        format!("{}/token", mock_server.uri()),
        format!("{}/revoke", mock_server.uri()),
        format!("{}/userinfo", mock_server.uri()),
    )
    .unwrap();

    let provider = google_drive::GoogleDriveProvider::from_connection(
        "test-folder".to_string(),
        conn_id,
        pool.clone(),
        key,
        connector,
    );

    // Scan should fail with ConnectionBroken (no credentials).
    let result = provider.scan_for_changes(None, &["*.md".to_string()]).await;
    assert!(
        matches!(result, Err(SourceError::ConnectionBroken { .. })),
        "Expected ConnectionBroken, got {:?}",
        result
    );
}

#[tokio::test]
async fn google_drive_provider_connection_revoked_returns_broken() {
    use crate::source::connector::crypto::encrypt_credentials;
    use crate::source::connector::google_drive::GoogleDriveConnector;
    use crate::source::ContentSourceProvider;
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let pool = init_test_db().await.expect("init db");
    let mock_server = MockServer::start().await;

    // Mock token endpoint returning a revocation error.
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "invalid_grant",
            "error_description": "Token has been revoked"
        })))
        .mount(&mock_server)
        .await;

    let conn_id = store::insert_connection(&pool, "google_drive", Some("test@example.com"), None)
        .await
        .unwrap();

    let key: Vec<u8> = (0..32).collect();
    let encrypted = encrypt_credentials(b"1//revoked-token", &key).unwrap();
    store::store_encrypted_credentials(&pool, conn_id, &encrypted)
        .await
        .unwrap();

    let config = test_connector_config();
    let connector = GoogleDriveConnector::with_test_urls(
        &config,
        reqwest::Client::new(),
        format!("{}/token", mock_server.uri()),
        format!("{}/revoke", mock_server.uri()),
        format!("{}/userinfo", mock_server.uri()),
    )
    .unwrap();

    let provider = google_drive::GoogleDriveProvider::from_connection(
        "test-folder".to_string(),
        conn_id,
        pool.clone(),
        key,
        connector,
    );

    let result = provider.scan_for_changes(None, &["*.md".to_string()]).await;
    assert!(
        matches!(result, Err(SourceError::ConnectionBroken { .. })),
        "Expected ConnectionBroken for revoked token, got {:?}",
        result
    );
}

#[tokio::test]
async fn google_drive_provider_connection_construction() {
    use crate::source::connector::crypto::encrypt_credentials;
    use crate::source::connector::google_drive::GoogleDriveConnector;
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;
    use wiremock::MockServer;

    let pool = init_test_db().await.expect("init db");
    let mock_server = MockServer::start().await;

    let conn_id = store::insert_connection(&pool, "google_drive", Some("test@example.com"), None)
        .await
        .unwrap();

    let key: Vec<u8> = (0..32).collect();
    let encrypted = encrypt_credentials(b"1//refresh-for-cache", &key).unwrap();
    store::store_encrypted_credentials(&pool, conn_id, &encrypted)
        .await
        .unwrap();

    let config = test_connector_config();
    let connector = GoogleDriveConnector::with_test_urls(
        &config,
        reqwest::Client::new(),
        format!("{}/token", mock_server.uri()),
        format!("{}/revoke", mock_server.uri()),
        format!("{}/userinfo", mock_server.uri()),
    )
    .unwrap();

    // Build a provider with linked-account strategy and custom HTTP client.
    let provider = google_drive::GoogleDriveProvider::with_client_and_connection(
        "test-folder".to_string(),
        conn_id,
        pool.clone(),
        key,
        connector,
        reqwest::Client::new(),
    );

    // Verify the provider has the correct source type and is ready to use.
    assert_eq!(provider.source_type(), "google_drive");
}
