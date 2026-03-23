//! Server integration tests for the graph neighbor API endpoint.
//!
//! Tests the `GET /api/vault/notes/{id}/neighbors` endpoint and the
//! graph_neighbors field in `GET /api/vault/selection/{session_id}`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::{broadcast, Mutex, RwLock};
use tower::ServiceExt;

use tuitbot_core::storage;
use tuitbot_core::storage::watchtower;
use tuitbot_core::storage::watchtower::edges::NewEdge;
use tuitbot_core::storage::DbPool;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::AccountWsEvent;

const TEST_TOKEN: &str = "test-token-graph-abc";
const DEFAULT_ACCOUNT_ID: &str = "00000000-0000-0000-0000-000000000000";

// ============================================================================
// Helpers
// ============================================================================

async fn build_test_state() -> (Arc<AppState>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[business]\nproduct_name = \"TestApp\"\n").unwrap();

    let pool = storage::init_test_db().await.expect("init db");
    let (event_tx, _) = broadcast::channel::<AccountWsEvent>(16);

    let state = Arc::new(AppState {
        db: pool,
        config_path,
        data_dir: dir.path().to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: RwLock::new(None),
        passphrase_hash_mtime: RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(HashMap::new()),
        content_generators: Mutex::new(HashMap::new()),
        runtimes: Mutex::new(HashMap::new()),
        circuit_breaker: None,
        scraper_health: None,
        watchtower_cancel: RwLock::new(None),
        content_sources: RwLock::new(Default::default()),
        connector_config: Default::default(),
        deployment_mode: Default::default(),
        pending_oauth: Mutex::new(HashMap::new()),
        token_managers: Mutex::new(HashMap::new()),
        x_client_id: String::new(),
        semantic_index: None,
        embedding_provider: None,
    });

    (state, dir)
}

fn build_router(state: Arc<AppState>) -> axum::Router {
    tuitbot_server::build_router(state)
}

async fn get_json(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .uri(format!("/api{path}"))
        .header("authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
    (status, json)
}

/// Seed two content nodes with an edge between them.
async fn seed_graph_data(pool: &DbPool) -> (i64, i64) {
    let source_id =
        watchtower::insert_source_context_for(pool, DEFAULT_ACCOUNT_ID, "local_fs", "{}")
            .await
            .expect("insert source");

    watchtower::upsert_content_node_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        source_id,
        "notes/main.md",
        "hash-main",
        Some("Main Note"),
        "This is the main note body",
        None,
        None,
    )
    .await
    .expect("upsert main node");

    watchtower::upsert_content_node_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        source_id,
        "notes/linked.md",
        "hash-linked",
        Some("Linked Note"),
        "This is a linked note body",
        None,
        None,
    )
    .await
    .expect("upsert linked node");

    // Get node IDs.
    let nodes = watchtower::get_nodes_for_source_for(pool, DEFAULT_ACCOUNT_ID, source_id, 10)
        .await
        .expect("get nodes");
    assert_eq!(nodes.len(), 2);

    let main_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/main.md")
        .unwrap()
        .id;
    let linked_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/linked.md")
        .unwrap()
        .id;

    // Insert a chunk for the linked note (for snippet enrichment).
    watchtower::insert_chunk(
        pool,
        DEFAULT_ACCOUNT_ID,
        linked_id,
        "# Linked Note",
        "This is content from the linked note about Rust async patterns",
        "hash-linked-chunk",
        0,
    )
    .await
    .expect("insert chunk");

    // Insert an edge: main → linked (wikilink).
    watchtower::insert_edge(
        pool,
        DEFAULT_ACCOUNT_ID,
        &NewEdge {
            source_node_id: main_id,
            target_node_id: linked_id,
            edge_type: "wikilink".to_string(),
            edge_label: Some("see also".to_string()),
            source_chunk_id: None,
        },
    )
    .await
    .expect("insert edge");

    (main_id, linked_id)
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn neighbors_returns_results_for_seeded_graph() {
    let (state, _dir) = build_test_state().await;
    let (main_id, _linked_id) = seed_graph_data(&state.db).await;
    let router = build_router(state);

    let (status, body) = get_json(router, &format!("/vault/notes/{main_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["graph_state"], "available");
    assert_eq!(body["node_id"], main_id);

    let neighbors = body["neighbors"].as_array().unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0]["reason"], "linked_note");
    assert_eq!(neighbors[0]["reason_label"], "linked note");
    assert!(neighbors[0]["snippet"].is_string());
    assert!(neighbors[0]["relative_path"].is_string());
}

#[tokio::test]
async fn neighbors_node_not_found_returns_not_indexed() {
    let (state, _dir) = build_test_state().await;
    let router = build_router(state);

    let (status, body) = get_json(router, "/vault/notes/99999/neighbors").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["graph_state"], "node_not_indexed");
    assert!(body["neighbors"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn neighbors_no_edges_returns_no_related_notes() {
    let (state, _dir) = build_test_state().await;

    // Create a node with no edges.
    let source_id =
        watchtower::insert_source_context_for(&state.db, DEFAULT_ACCOUNT_ID, "local_fs", "{}")
            .await
            .expect("insert source");

    watchtower::upsert_content_node_for(
        &state.db,
        DEFAULT_ACCOUNT_ID,
        source_id,
        "notes/isolated.md",
        "hash-isolated",
        Some("Isolated Note"),
        "No links here",
        None,
        None,
    )
    .await
    .expect("upsert");

    let nodes = watchtower::get_nodes_for_source_for(&state.db, DEFAULT_ACCOUNT_ID, source_id, 10)
        .await
        .unwrap();
    let node_id = nodes[0].id;

    let router = build_router(state);
    let (status, body) = get_json(router, &format!("/vault/notes/{node_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["graph_state"], "no_related_notes");
    assert!(body["neighbors"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn neighbors_account_isolation() {
    let (state, _dir) = build_test_state().await;

    // Seed data for default account.
    let (main_id, _) = seed_graph_data(&state.db).await;

    // Create a different account's node.
    let other_account = "11111111-1111-1111-1111-111111111111";
    let source_id =
        watchtower::insert_source_context_for(&state.db, other_account, "local_fs", "{}")
            .await
            .expect("insert source");

    watchtower::upsert_content_node_for(
        &state.db,
        other_account,
        source_id,
        "notes/other.md",
        "hash-other",
        Some("Other Account Note"),
        "Body",
        None,
        None,
    )
    .await
    .expect("upsert");

    // The default account's main_id should still show neighbors.
    let router = build_router(state);
    let (status, body) = get_json(router, &format!("/vault/notes/{main_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["graph_state"], "available");
    // Only the linked note from the same account should appear.
    let neighbors = body["neighbors"].as_array().unwrap();
    assert_eq!(neighbors.len(), 1);
}

#[tokio::test]
async fn neighbors_max_parameter_respected() {
    let (state, _dir) = build_test_state().await;
    let (main_id, _) = seed_graph_data(&state.db).await;
    let router = build_router(state);

    let (status, body) = get_json(router, &format!("/vault/notes/{main_id}/neighbors?max=0")).await;
    assert_eq!(status, StatusCode::OK);
    // max=0 gets clamped to DEFAULT_MAX_NEIGHBORS, so it still returns results.
    assert_eq!(body["graph_state"], "available");
}

#[tokio::test]
async fn neighbors_cloud_mode_omits_relative_path() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[business]\nproduct_name = \"TestApp\"\n").unwrap();

    let pool = storage::init_test_db().await.expect("init db");
    let (event_tx, _) = broadcast::channel::<AccountWsEvent>(16);

    let state = Arc::new(AppState {
        db: pool,
        config_path,
        data_dir: dir.path().to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: RwLock::new(None),
        passphrase_hash_mtime: RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(HashMap::new()),
        content_generators: Mutex::new(HashMap::new()),
        runtimes: Mutex::new(HashMap::new()),
        circuit_breaker: None,
        scraper_health: None,
        watchtower_cancel: RwLock::new(None),
        content_sources: RwLock::new(Default::default()),
        connector_config: Default::default(),
        deployment_mode: tuitbot_core::config::DeploymentMode::Cloud,
        pending_oauth: Mutex::new(HashMap::new()),
        token_managers: Mutex::new(HashMap::new()),
        x_client_id: String::new(),
        semantic_index: None,
        embedding_provider: None,
    });

    let (main_id, _) = seed_graph_data(&state.db).await;
    let router = build_router(state);

    let (status, body) = get_json(router, &format!("/vault/notes/{main_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);
    let neighbors = body["neighbors"].as_array().unwrap();
    assert_eq!(neighbors.len(), 1);
    // Cloud mode should omit relative_path.
    assert!(neighbors[0].get("relative_path").is_none());
}

#[tokio::test]
async fn neighbors_mutual_link_reason() {
    let (state, _dir) = build_test_state().await;
    let (main_id, linked_id) = seed_graph_data(&state.db).await;

    // Add a backlink: linked → main.
    watchtower::insert_edge(
        &state.db,
        DEFAULT_ACCOUNT_ID,
        &NewEdge {
            source_node_id: linked_id,
            target_node_id: main_id,
            edge_type: "wikilink".to_string(),
            edge_label: None,
            source_chunk_id: None,
        },
    )
    .await
    .expect("insert backlink");

    let router = build_router(state);
    let (status, body) = get_json(router, &format!("/vault/notes/{main_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);

    let neighbors = body["neighbors"].as_array().unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0]["reason"], "mutual_link");
    assert_eq!(neighbors[0]["reason_label"], "mutual link");
}

#[tokio::test]
async fn neighbors_score_ordering() {
    let (state, _dir) = build_test_state().await;

    let source_id =
        watchtower::insert_source_context_for(&state.db, DEFAULT_ACCOUNT_ID, "local_fs", "{}")
            .await
            .unwrap();

    // Create 3 nodes: main, high-score (direct link), low-score (shared tag only).
    for (path, title) in [
        ("notes/main.md", "Main"),
        ("notes/high.md", "High Score"),
        ("notes/low.md", "Low Score"),
    ] {
        watchtower::upsert_content_node_for(
            &state.db,
            DEFAULT_ACCOUNT_ID,
            source_id,
            path,
            &format!("hash-{path}"),
            Some(title),
            "body",
            None,
            None,
        )
        .await
        .unwrap();
    }

    let nodes = watchtower::get_nodes_for_source_for(&state.db, DEFAULT_ACCOUNT_ID, source_id, 10)
        .await
        .unwrap();
    let main_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/main.md")
        .unwrap()
        .id;
    let high_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/high.md")
        .unwrap()
        .id;
    let low_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/low.md")
        .unwrap()
        .id;

    // Direct link to high_id (score: 3.0).
    watchtower::insert_edge(
        &state.db,
        DEFAULT_ACCOUNT_ID,
        &NewEdge {
            source_node_id: main_id,
            target_node_id: high_id,
            edge_type: "wikilink".to_string(),
            edge_label: None,
            source_chunk_id: None,
        },
    )
    .await
    .unwrap();

    // Shared tag edge to low_id (score: 1.0).
    watchtower::insert_edge(
        &state.db,
        DEFAULT_ACCOUNT_ID,
        &NewEdge {
            source_node_id: main_id,
            target_node_id: low_id,
            edge_type: "shared_tag".to_string(),
            edge_label: Some("rust".to_string()),
            source_chunk_id: None,
        },
    )
    .await
    .unwrap();

    let router = build_router(state);
    let (status, body) = get_json(router, &format!("/vault/notes/{main_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);

    let neighbors = body["neighbors"].as_array().unwrap();
    assert_eq!(neighbors.len(), 2);
    // High-score neighbor should be first.
    assert_eq!(neighbors[0]["node_id"], high_id);
    assert_eq!(neighbors[1]["node_id"], low_id);
    // Scores should be ordered DESC.
    let score_0 = neighbors[0]["score"].as_f64().unwrap();
    let score_1 = neighbors[1]["score"].as_f64().unwrap();
    assert!(score_0 > score_1);
}

#[tokio::test]
async fn neighbors_tag_only_returns_shared_tag_reason() {
    let (state, _dir) = build_test_state().await;

    let source_id =
        watchtower::insert_source_context_for(&state.db, DEFAULT_ACCOUNT_ID, "local_fs", "{}")
            .await
            .unwrap();

    // Create two nodes with no direct links.
    for (path, title) in [
        ("notes/alpha.md", "Alpha Note"),
        ("notes/beta.md", "Beta Note"),
    ] {
        watchtower::upsert_content_node_for(
            &state.db,
            DEFAULT_ACCOUNT_ID,
            source_id,
            path,
            &format!("hash-{path}"),
            Some(title),
            "body content",
            None,
            None,
        )
        .await
        .unwrap();
    }

    let nodes = watchtower::get_nodes_for_source_for(&state.db, DEFAULT_ACCOUNT_ID, source_id, 10)
        .await
        .unwrap();
    let alpha_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/alpha.md")
        .unwrap()
        .id;
    let beta_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/beta.md")
        .unwrap()
        .id;

    // Insert shared_tag edges (simulating what graph_ingest creates).
    watchtower::insert_edge(
        &state.db,
        DEFAULT_ACCOUNT_ID,
        &NewEdge {
            source_node_id: alpha_id,
            target_node_id: beta_id,
            edge_type: "shared_tag".to_string(),
            edge_label: Some("rust".to_string()),
            source_chunk_id: None,
        },
    )
    .await
    .unwrap();

    let router = build_router(state);
    let (status, body) = get_json(router, &format!("/vault/notes/{alpha_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["graph_state"], "available");

    let neighbors = body["neighbors"].as_array().unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0]["reason"], "shared_tag");
    assert!(neighbors[0]["reason_label"]
        .as_str()
        .unwrap()
        .contains("rust"));
}

#[tokio::test]
async fn neighbors_response_fields_complete() {
    let (state, _dir) = build_test_state().await;
    let (main_id, _) = seed_graph_data(&state.db).await;
    let router = build_router(state);

    let (status, body) = get_json(router, &format!("/vault/notes/{main_id}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);

    // Verify top-level contract fields.
    assert!(body.get("node_id").is_some());
    assert!(body.get("neighbors").is_some());
    assert!(body.get("total_edges").is_some());
    assert!(body.get("graph_state").is_some());

    // Verify neighbor item contract fields.
    let neighbor = &body["neighbors"][0];
    assert!(neighbor.get("node_id").is_some());
    assert!(neighbor.get("node_title").is_some());
    assert!(neighbor.get("reason").is_some());
    assert!(neighbor.get("reason_label").is_some());
    assert!(neighbor.get("intent").is_some());
    assert!(neighbor.get("matched_tags").is_some());
    assert!(neighbor.get("score").is_some());
    assert!(neighbor.get("best_chunk_id").is_some());
    assert!(neighbor.get("relative_path").is_some());
}

#[tokio::test]
async fn neighbors_intent_field_from_edge_label() {
    let (state, _dir) = build_test_state().await;

    let source_id =
        watchtower::insert_source_context_for(&state.db, DEFAULT_ACCOUNT_ID, "local_fs", "{}")
            .await
            .unwrap();

    for (path, title) in [
        ("notes/source.md", "Source Note"),
        ("notes/tip.md", "Tip Note"),
    ] {
        watchtower::upsert_content_node_for(
            &state.db,
            DEFAULT_ACCOUNT_ID,
            source_id,
            path,
            &format!("hash-{path}"),
            Some(title),
            "body",
            None,
            None,
        )
        .await
        .unwrap();
    }

    let nodes = watchtower::get_nodes_for_source_for(&state.db, DEFAULT_ACCOUNT_ID, source_id, 10)
        .await
        .unwrap();
    let source_id_node = nodes
        .iter()
        .find(|n| n.relative_path == "notes/source.md")
        .unwrap()
        .id;
    let tip_id = nodes
        .iter()
        .find(|n| n.relative_path == "notes/tip.md")
        .unwrap()
        .id;

    // Edge with "tip" in the label → should classify as pro_tip.
    watchtower::insert_edge(
        &state.db,
        DEFAULT_ACCOUNT_ID,
        &NewEdge {
            source_node_id: source_id_node,
            target_node_id: tip_id,
            edge_type: "wikilink".to_string(),
            edge_label: Some("quick tip for setup".to_string()),
            source_chunk_id: None,
        },
    )
    .await
    .unwrap();

    let router = build_router(state);
    let (status, body) =
        get_json(router, &format!("/vault/notes/{source_id_node}/neighbors")).await;
    assert_eq!(status, StatusCode::OK);

    let neighbors = body["neighbors"].as_array().unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0]["intent"], "pro_tip");
}
