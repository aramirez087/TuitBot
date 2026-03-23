//! Vault API endpoints for searching notes, previewing fragments,
//! and resolving selected references from the dashboard.
//!
//! All endpoints are account-scoped via `AccountContext` and return
//! privacy-safe responses (no raw note bodies — only titles, paths,
//! tags, heading paths, and truncated snippets).

pub mod evidence;
pub mod index_status;
pub mod selections;

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use tuitbot_core::context::graph_expansion::{self, GraphState};
use tuitbot_core::context::retrieval::{self, VaultCitation};
use tuitbot_core::storage::watchtower;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

/// Maximum snippet length returned in API responses (characters).
const SNIPPET_MAX_LEN: usize = 120;

/// Default result limit for search endpoints.
const DEFAULT_LIMIT: u32 = 20;

/// Maximum result limit for search endpoints.
const MAX_LIMIT: u32 = 100;

fn clamp_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)
}

fn truncate_snippet(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let mut end = max_len.saturating_sub(3);
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}

// ---------------------------------------------------------------------------
// GET /api/vault/sources
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct VaultSourcesResponse {
    pub sources: Vec<VaultSourceStatusItem>,
    pub deployment_mode: String,
    pub privacy_envelope: String,
}

#[derive(Serialize)]
pub struct VaultSourceStatusItem {
    pub id: i64,
    pub source_type: String,
    pub status: String,
    pub error_message: Option<String>,
    pub node_count: i64,
    pub updated_at: String,
    /// For `local_fs` sources, the configured vault path.  Used by the
    /// desktop frontend to construct `obsidian://` deep-link URIs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

pub async fn vault_sources(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<VaultSourcesResponse>, ApiError> {
    let sources = watchtower::get_all_source_contexts_for(&state.db, &ctx.account_id).await?;

    let is_cloud = matches!(
        state.deployment_mode,
        tuitbot_core::config::DeploymentMode::Cloud
    );

    let mut items = Vec::with_capacity(sources.len());
    for src in sources {
        let count = watchtower::count_nodes_for_source(&state.db, &ctx.account_id, src.id)
            .await
            .unwrap_or(0);
        // Only expose local path for non-Cloud modes (defense in depth).
        let path = if src.source_type == "local_fs" && !is_cloud {
            serde_json::from_str::<serde_json::Value>(&src.config_json)
                .ok()
                .and_then(|v| v.get("path").and_then(|p| p.as_str().map(String::from)))
        } else {
            None
        };
        items.push(VaultSourceStatusItem {
            id: src.id,
            source_type: src.source_type,
            status: src.status,
            error_message: src.error_message,
            node_count: count,
            updated_at: src.updated_at,
            path,
        });
    }

    Ok(Json(VaultSourcesResponse {
        sources: items,
        deployment_mode: state.deployment_mode.to_string(),
        privacy_envelope: state.deployment_mode.privacy_envelope().to_string(),
    }))
}

// ---------------------------------------------------------------------------
// GET /api/vault/notes?q=&source_id=&limit=
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SearchNotesQuery {
    pub q: Option<String>,
    pub source_id: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct SearchNotesResponse {
    pub notes: Vec<VaultNoteItem>,
}

#[derive(Serialize)]
pub struct VaultNoteItem {
    pub node_id: i64,
    pub source_id: i64,
    pub title: Option<String>,
    pub relative_path: String,
    pub tags: Option<String>,
    pub status: String,
    pub chunk_count: i64,
    pub updated_at: String,
}

pub async fn search_notes(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<SearchNotesQuery>,
) -> Result<Json<SearchNotesResponse>, ApiError> {
    let limit = clamp_limit(params.limit);

    let nodes = match (&params.q, params.source_id) {
        (Some(q), _) if !q.is_empty() => {
            watchtower::search_nodes_for(&state.db, &ctx.account_id, q, limit).await?
        }
        (_, Some(sid)) => {
            watchtower::get_nodes_for_source_for(&state.db, &ctx.account_id, sid, limit).await?
        }
        _ => {
            // No query and no source_id — return recent nodes.
            watchtower::search_nodes_for(&state.db, &ctx.account_id, "", limit).await?
        }
    };

    let mut notes = Vec::with_capacity(nodes.len());
    for node in nodes {
        let chunk_count =
            watchtower::count_chunks_for_node(&state.db, &ctx.account_id, node.id).await?;
        notes.push(VaultNoteItem {
            node_id: node.id,
            source_id: node.source_id,
            title: node.title,
            relative_path: node.relative_path,
            tags: node.tags,
            status: node.status,
            chunk_count,
            updated_at: node.updated_at,
        });
    }

    Ok(Json(SearchNotesResponse { notes }))
}

// ---------------------------------------------------------------------------
// GET /api/vault/notes/{id}
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct VaultNoteDetail {
    pub node_id: i64,
    pub source_id: i64,
    pub title: Option<String>,
    pub relative_path: String,
    pub tags: Option<String>,
    pub status: String,
    pub ingested_at: String,
    pub updated_at: String,
    pub chunks: Vec<VaultChunkSummary>,
}

#[derive(Serialize)]
pub struct VaultChunkSummary {
    pub chunk_id: i64,
    pub heading_path: String,
    pub snippet: String,
    pub retrieval_boost: f64,
}

pub async fn note_detail(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<VaultNoteDetail>, ApiError> {
    let node = watchtower::get_content_node_for(&state.db, &ctx.account_id, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("note {id} not found")))?;

    let chunks = watchtower::get_chunks_for_node(&state.db, &ctx.account_id, id).await?;

    let chunk_summaries: Vec<VaultChunkSummary> = chunks
        .into_iter()
        .map(|c| VaultChunkSummary {
            chunk_id: c.id,
            heading_path: c.heading_path,
            snippet: truncate_snippet(&c.chunk_text, SNIPPET_MAX_LEN),
            retrieval_boost: c.retrieval_boost,
        })
        .collect();

    Ok(Json(VaultNoteDetail {
        node_id: node.id,
        source_id: node.source_id,
        title: node.title,
        relative_path: node.relative_path,
        tags: node.tags,
        status: node.status,
        ingested_at: node.ingested_at,
        updated_at: node.updated_at,
        chunks: chunk_summaries,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/vault/notes/{id}/neighbors?max=8
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct NoteNeighborsQuery {
    pub max: Option<u32>,
}

#[derive(Serialize)]
pub struct NoteNeighborsResponse {
    pub node_id: i64,
    pub neighbors: Vec<NeighborItem>,
    pub total_edges: u32,
    pub graph_state: GraphState,
}

#[derive(Serialize)]
pub struct NeighborItem {
    pub node_id: i64,
    pub node_title: Option<String>,
    pub reason: String,
    pub reason_label: String,
    pub intent: String,
    pub matched_tags: Vec<String>,
    pub score: f64,
    pub snippet: Option<String>,
    pub best_chunk_id: Option<i64>,
    pub heading_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<String>,
}

impl NeighborItem {
    fn from_graph_neighbor(n: graph_expansion::GraphNeighbor, is_cloud: bool) -> Self {
        Self {
            node_id: n.node_id,
            node_title: n.node_title,
            reason: serde_json::to_value(&n.reason)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "related".to_string()),
            reason_label: n.reason_label,
            intent: serde_json::to_value(&n.intent)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "related".to_string()),
            matched_tags: n.matched_tags,
            score: n.score,
            snippet: n.snippet,
            best_chunk_id: n.best_chunk_id,
            heading_path: n.heading_path,
            relative_path: if is_cloud {
                None
            } else {
                Some(n.relative_path)
            },
        }
    }
}

pub async fn note_neighbors(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Query(params): Query<NoteNeighborsQuery>,
) -> Result<Json<NoteNeighborsResponse>, ApiError> {
    let max = params
        .max
        .unwrap_or(graph_expansion::DEFAULT_MAX_NEIGHBORS)
        .min(MAX_LIMIT);
    let is_cloud = matches!(
        state.deployment_mode,
        tuitbot_core::config::DeploymentMode::Cloud
    );

    // Verify node exists and is account-scoped.
    let node = watchtower::get_content_node_for(&state.db, &ctx.account_id, id).await?;
    if node.is_none() {
        return Ok(Json(NoteNeighborsResponse {
            node_id: id,
            neighbors: Vec::new(),
            total_edges: 0,
            graph_state: GraphState::NodeNotIndexed,
        }));
    }

    // Expand graph neighbors (fail-open).
    let result =
        crate::routes::rag_helpers::resolve_graph_suggestions(&state, &ctx.account_id, id, max)
            .await;

    let total_edges: u32 = result.neighbors.iter().map(|n| n.edge_count).sum();
    let items: Vec<NeighborItem> = result
        .neighbors
        .into_iter()
        .map(|n| NeighborItem::from_graph_neighbor(n, is_cloud))
        .collect();

    Ok(Json(NoteNeighborsResponse {
        node_id: id,
        neighbors: items,
        total_edges,
        graph_state: result.graph_state,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/vault/search?q=&limit=
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SearchFragmentsQuery {
    pub q: String,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct SearchFragmentsResponse {
    pub fragments: Vec<VaultCitation>,
}

pub async fn search_fragments(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<SearchFragmentsQuery>,
) -> Result<Json<SearchFragmentsResponse>, ApiError> {
    let limit = clamp_limit(params.limit);

    if params.q.is_empty() {
        return Ok(Json(SearchFragmentsResponse { fragments: vec![] }));
    }

    let keywords: Vec<String> = params.q.split_whitespace().map(|s| s.to_string()).collect();

    let fragments =
        retrieval::retrieve_vault_fragments(&state.db, &ctx.account_id, &keywords, None, limit)
            .await?;

    let citations = retrieval::build_citations(&fragments);

    Ok(Json(SearchFragmentsResponse {
        fragments: citations,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/vault/resolve-refs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ResolveRefsRequest {
    pub node_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct ResolveRefsResponse {
    pub citations: Vec<VaultCitation>,
}

pub async fn resolve_refs(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<ResolveRefsRequest>,
) -> Result<Json<ResolveRefsResponse>, ApiError> {
    if body.node_ids.is_empty() {
        return Ok(Json(ResolveRefsResponse { citations: vec![] }));
    }

    let fragments = retrieval::retrieve_vault_fragments(
        &state.db,
        &ctx.account_id,
        &[],
        Some(&body.node_ids),
        MAX_LIMIT,
    )
    .await?;

    let citations = retrieval::build_citations(&fragments);

    Ok(Json(ResolveRefsResponse { citations }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::path::PathBuf;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::{get, post};
    use axum::Router;
    use tokio::sync::{broadcast, Mutex, RwLock};
    use tower::ServiceExt;

    use crate::ws::AccountWsEvent;

    async fn test_state() -> Arc<AppState> {
        let db = tuitbot_core::storage::init_test_db()
            .await
            .expect("init test db");
        let (event_tx, _) = broadcast::channel::<AccountWsEvent>(16);
        Arc::new(AppState {
            db,
            config_path: PathBuf::from("/tmp/test-config.toml"),
            data_dir: PathBuf::from("/tmp"),
            event_tx,
            api_token: "test-token".to_string(),
            passphrase_hash: RwLock::new(None),
            passphrase_hash_mtime: RwLock::new(None),
            bind_host: "127.0.0.1".to_string(),
            bind_port: 3001,
            login_attempts: Mutex::new(HashMap::new()),
            runtimes: Mutex::new(HashMap::new()),
            content_generators: Mutex::new(HashMap::new()),
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
        })
    }

    fn test_router(state: Arc<AppState>) -> Router {
        Router::new()
            .route("/vault/sources", get(vault_sources))
            .route("/vault/notes", get(search_notes))
            .route("/vault/notes/{id}/neighbors", get(note_neighbors))
            .route("/vault/notes/{id}", get(note_detail))
            .route("/vault/search", get(search_fragments))
            .route("/vault/resolve-refs", post(resolve_refs))
            .with_state(state)
    }

    #[tokio::test]
    async fn vault_sources_returns_empty_when_no_sources() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/sources")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 64)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["sources"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn search_notes_returns_empty_for_no_matches() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/notes?q=nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 64)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["notes"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn note_detail_returns_404_for_missing_node() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/notes/999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn search_fragments_returns_empty_for_no_chunks() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/search?q=nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 64)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["fragments"].as_array().unwrap().len(), 0);
    }

    // --- clamp_limit tests ---

    #[test]
    fn clamp_limit_default() {
        assert_eq!(clamp_limit(None), DEFAULT_LIMIT);
    }

    #[test]
    fn clamp_limit_under_max() {
        assert_eq!(clamp_limit(Some(50)), 50);
    }

    #[test]
    fn clamp_limit_at_max() {
        assert_eq!(clamp_limit(Some(MAX_LIMIT)), MAX_LIMIT);
    }

    #[test]
    fn clamp_limit_over_max() {
        assert_eq!(clamp_limit(Some(500)), MAX_LIMIT);
    }

    // --- truncate_snippet tests ---

    #[test]
    fn truncate_snippet_short_text() {
        assert_eq!(truncate_snippet("hello", 120), "hello");
    }

    #[test]
    fn truncate_snippet_at_limit() {
        let text = "a".repeat(120);
        assert_eq!(truncate_snippet(&text, 120), text);
    }

    #[test]
    fn truncate_snippet_over_limit() {
        let text = "a".repeat(200);
        let result = truncate_snippet(&text, 120);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 120);
    }

    #[test]
    fn truncate_snippet_unicode_safe() {
        // Test with multi-byte chars
        let text = "a".repeat(115) + "\u{1F600}\u{1F600}\u{1F600}";
        let result = truncate_snippet(&text, 120);
        assert!(result.ends_with("..."));
        // Should not panic on char boundary
    }

    // --- deserialization tests ---

    #[test]
    fn search_notes_query_defaults() {
        let json = "{}";
        let q: SearchNotesQuery = serde_json::from_str(json).expect("deser");
        assert!(q.q.is_none());
        assert!(q.source_id.is_none());
        assert!(q.limit.is_none());
    }

    #[test]
    fn search_fragments_query_deserializes() {
        let json = r#"{"q":"rust","limit":10}"#;
        let q: SearchFragmentsQuery = serde_json::from_str(json).expect("deser");
        assert_eq!(q.q, "rust");
        assert_eq!(q.limit, Some(10));
    }

    #[test]
    fn resolve_refs_request_deserializes() {
        let json = r#"{"node_ids":[1,2,3]}"#;
        let req: ResolveRefsRequest = serde_json::from_str(json).expect("deser");
        assert_eq!(req.node_ids.len(), 3);
    }

    #[test]
    fn resolve_refs_request_empty_ids() {
        let json = r#"{"node_ids":[]}"#;
        let req: ResolveRefsRequest = serde_json::from_str(json).expect("deser");
        assert!(req.node_ids.is_empty());
    }

    #[tokio::test]
    async fn resolve_refs_returns_empty_for_empty_ids() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/vault/resolve-refs")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"node_ids":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 64)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["citations"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn vault_sources_includes_privacy_envelope() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/sources")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 64)
                .await
                .unwrap(),
        )
        .unwrap();
        // Default deployment_mode is Desktop
        assert_eq!(body["deployment_mode"], "desktop");
        assert_eq!(body["privacy_envelope"], "local_first");
    }

    async fn test_state_with_mode(mode: tuitbot_core::config::DeploymentMode) -> Arc<AppState> {
        let db = tuitbot_core::storage::init_test_db()
            .await
            .expect("init test db");
        let (event_tx, _) = broadcast::channel::<AccountWsEvent>(16);
        Arc::new(AppState {
            db,
            config_path: PathBuf::from("/tmp/test-config.toml"),
            data_dir: PathBuf::from("/tmp"),
            event_tx,
            api_token: "test-token".to_string(),
            passphrase_hash: RwLock::new(None),
            passphrase_hash_mtime: RwLock::new(None),
            bind_host: "127.0.0.1".to_string(),
            bind_port: 3001,
            login_attempts: Mutex::new(HashMap::new()),
            runtimes: Mutex::new(HashMap::new()),
            content_generators: Mutex::new(HashMap::new()),
            circuit_breaker: None,
            scraper_health: None,
            watchtower_cancel: RwLock::new(None),
            content_sources: RwLock::new(Default::default()),
            connector_config: Default::default(),
            deployment_mode: mode,
            pending_oauth: Mutex::new(HashMap::new()),
            token_managers: Mutex::new(HashMap::new()),
            x_client_id: String::new(),
            semantic_index: None,
            embedding_provider: None,
        })
    }

    #[tokio::test]
    async fn vault_sources_cloud_mode_privacy_envelope() {
        let state = test_state_with_mode(tuitbot_core::config::DeploymentMode::Cloud).await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/sources")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 64)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["deployment_mode"], "cloud");
        assert_eq!(body["privacy_envelope"], "provider_controlled");
    }
}
