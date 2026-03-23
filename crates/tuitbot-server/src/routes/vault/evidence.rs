//! `GET /api/vault/evidence` — unified semantic evidence endpoint.
//!
//! Serves all evidence consumers (Ghostwriter, hook picker, thread editor,
//! selection review) through a single endpoint with optional scope filters
//! and mode selection. Falls back cleanly to keyword-only when the semantic
//! index is unavailable.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use tuitbot_core::context::hybrid_retrieval::{self, EvidenceResult};
use tuitbot_core::context::retrieval::MatchReason;
use tuitbot_core::context::semantic_search;
use tuitbot_core::storage::watchtower::embeddings;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

/// Default result limit.
const DEFAULT_LIMIT: u32 = 8;

/// Maximum result limit.
const MAX_LIMIT: u32 = 20;

#[derive(Deserialize)]
pub struct EvidenceQuery {
    /// Search query text (required, non-empty).
    pub q: Option<String>,
    /// Maximum results to return (default 8, max 20).
    pub limit: Option<u32>,
    /// Retrieval mode: "hybrid" (default), "semantic", "keyword".
    pub mode: Option<String>,
    /// Optional scope filter, e.g. "selection:{session_id}".
    pub scope: Option<String>,
}

#[derive(Serialize)]
pub struct EvidenceResponse {
    pub results: Vec<EvidenceResultItem>,
    pub query: String,
    pub mode: String,
    pub index_status: IndexStatusSummary,
}

#[derive(Serialize)]
pub struct EvidenceResultItem {
    pub chunk_id: i64,
    pub node_id: i64,
    pub heading_path: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<String>,
    pub match_reason: MatchReason,
    pub score: f64,
    pub node_title: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct IndexStatusSummary {
    pub total_chunks: i64,
    pub embedded_chunks: i64,
    pub freshness_pct: f64,
}

pub async fn search_evidence(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<EvidenceQuery>,
) -> Result<Json<EvidenceResponse>, ApiError> {
    // Validate query
    let query = params.q.unwrap_or_default();
    if query.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "query parameter 'q' is required and must be non-empty".to_string(),
        ));
    }

    let limit = params
        .limit
        .map(|l| if l == 0 { DEFAULT_LIMIT } else { l })
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT);

    let mode = params.mode.as_deref().unwrap_or("hybrid");

    let is_cloud = matches!(
        state.deployment_mode,
        tuitbot_core::config::DeploymentMode::Cloud
    );

    // Resolve scope for graph context
    let selected_node_ids: Option<Vec<i64>> = if let Some(scope) = &params.scope {
        if let Some(session_id) = scope.strip_prefix("selection:") {
            resolve_selection_node_ids(&state, &ctx.account_id, session_id).await
        } else {
            None
        }
    } else {
        None
    };

    // Semantic search (if mode != "keyword" and provider is available)
    let search_start = std::time::Instant::now();
    let semantic_hits = if mode != "keyword" {
        embed_and_search(&state, &query, limit).await
    } else {
        None
    };
    let semantic_elapsed = search_start.elapsed();
    let did_fallback =
        mode != "keyword" && semantic_hits.is_none() && state.embedding_provider.is_some();

    tracing::info!(
        latency_ms = semantic_elapsed.as_millis() as u64,
        fallback = did_fallback,
        mode = mode,
        result_count = semantic_hits.as_ref().map(|h| h.len()).unwrap_or(0),
        "evidence_search_completed"
    );

    // In semantic-only mode, skip keyword search (pass empty query to hybrid_search)
    let effective_query = if mode == "semantic" { "" } else { &query };

    let results = hybrid_retrieval::hybrid_search(
        &state.db,
        &ctx.account_id,
        effective_query,
        semantic_hits.as_deref(),
        selected_node_ids.as_deref(),
        limit,
    )
    .await?;

    // Fetch index stats for the response
    let index_status = get_index_status_summary(&state.db, &ctx.account_id).await;

    let items: Vec<EvidenceResultItem> = results
        .into_iter()
        .map(|r| evidence_to_item(r, is_cloud))
        .collect();

    Ok(Json(EvidenceResponse {
        results: items,
        query,
        mode: mode.to_string(),
        index_status,
    }))
}

fn evidence_to_item(r: EvidenceResult, is_cloud: bool) -> EvidenceResultItem {
    EvidenceResultItem {
        chunk_id: r.chunk_id,
        node_id: r.node_id,
        heading_path: r.heading_path,
        snippet: r.snippet,
        relative_path: if is_cloud { None } else { Some(r.source_path) },
        match_reason: r.match_reason,
        score: r.score,
        node_title: r.node_title,
    }
}

/// Embed the query and search the semantic index. Returns `None` on any failure.
async fn embed_and_search(
    state: &AppState,
    query: &str,
    limit: u32,
) -> Option<Vec<semantic_search::SemanticHit>> {
    let provider = state.embedding_provider.as_ref()?;
    let index_lock = state.semantic_index.as_ref()?;

    let response = match provider.embed(vec![query.to_string()]).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("evidence: embedding query failed: {e}");
            return None;
        }
    };

    let embedding = response.embeddings.into_iter().next()?;
    let index = index_lock.read().await;
    let hits = semantic_search::semantic_search(&index, &embedding, limit as usize);

    if hits.is_empty() {
        None
    } else {
        Some(hits)
    }
}

/// Resolve a selection session_id to its node_ids for graph context.
async fn resolve_selection_node_ids(
    state: &AppState,
    account_id: &str,
    session_id: &str,
) -> Option<Vec<i64>> {
    use tuitbot_core::storage::vault_selections;

    let selection = vault_selections::get_selection_by_session(&state.db, account_id, session_id)
        .await
        .ok()
        .flatten()?;

    selection.resolved_node_id.map(|nid| vec![nid])
}

/// Build index status summary from DB stats.
async fn get_index_status_summary(
    pool: &tuitbot_core::storage::DbPool,
    account_id: &str,
) -> IndexStatusSummary {
    match embeddings::get_index_stats_for(pool, account_id).await {
        Ok(stats) => IndexStatusSummary {
            total_chunks: stats.total_chunks,
            embedded_chunks: stats.embedded_chunks,
            freshness_pct: stats.freshness_pct,
        },
        Err(_) => IndexStatusSummary {
            total_chunks: 0,
            embedded_chunks: 0,
            freshness_pct: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::path::PathBuf;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
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
            config_path: PathBuf::from("test-config.toml"),
            data_dir: PathBuf::from(std::env::temp_dir()),
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
            .route("/vault/evidence", get(search_evidence))
            .with_state(state)
    }

    #[tokio::test]
    async fn empty_query_returns_400() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn missing_query_returns_400() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn keyword_mode_works_without_embedding_provider() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&mode=keyword")
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
        assert_eq!(body["mode"], "keyword");
        assert!(body["results"].as_array().unwrap().is_empty());
        assert!(body["index_status"]["freshness_pct"].as_f64().is_some());
    }

    #[tokio::test]
    async fn hybrid_mode_falls_back_without_provider() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&mode=hybrid")
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
        assert_eq!(body["mode"], "hybrid");
    }

    #[tokio::test]
    async fn semantic_mode_returns_empty_without_provider() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&mode=semantic")
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
        assert_eq!(body["mode"], "semantic");
        assert!(body["results"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn default_limit_applied() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn cloud_mode_omits_relative_path() {
        let db = tuitbot_core::storage::init_test_db()
            .await
            .expect("init test db");
        let (event_tx, _) = broadcast::channel::<AccountWsEvent>(16);
        let state = Arc::new(AppState {
            db,
            config_path: PathBuf::from("test-config.toml"),
            data_dir: PathBuf::from(std::env::temp_dir()),
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
            deployment_mode: tuitbot_core::config::DeploymentMode::Cloud,
            pending_oauth: Mutex::new(HashMap::new()),
            token_managers: Mutex::new(HashMap::new()),
            x_client_id: String::new(),
            semantic_index: None,
            embedding_provider: None,
        });
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&mode=keyword")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn limit_zero_defaults_to_8() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&limit=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn limit_over_max_clamped_to_20() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&limit=50")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_scope_returns_ok_with_empty_results() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&mode=keyword&scope=selection:nonexistent-session")
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
        assert!(body["results"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn index_status_included_in_response() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&mode=keyword")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 64)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body["index_status"].is_object());
        assert_eq!(body["index_status"]["total_chunks"], 0);
        assert_eq!(body["index_status"]["embedded_chunks"], 0);
    }

    // --- evidence_to_item tests ---

    fn make_evidence_result() -> EvidenceResult {
        EvidenceResult {
            chunk_id: 42,
            node_id: 7,
            heading_path: "# Intro > Details".to_string(),
            source_path: "notes/test.md".to_string(),
            source_title: Some("Test Note".to_string()),
            snippet: "some snippet".to_string(),
            score: 0.5,
            match_reason: MatchReason::Semantic,
            node_title: Some("Test Note".to_string()),
        }
    }

    #[test]
    fn evidence_to_item_local_mode_includes_path() {
        let r = make_evidence_result();
        let item = evidence_to_item(r, false);
        assert_eq!(item.relative_path, Some("notes/test.md".to_string()));
        assert_eq!(item.chunk_id, 42);
        assert_eq!(item.node_id, 7);
        assert_eq!(item.heading_path, "# Intro > Details");
        assert_eq!(item.snippet, "some snippet");
        assert!((item.score - 0.5).abs() < f64::EPSILON);
        assert_eq!(item.match_reason, MatchReason::Semantic);
        assert_eq!(item.node_title, Some("Test Note".to_string()));
    }

    #[test]
    fn evidence_to_item_cloud_mode_omits_path() {
        let r = make_evidence_result();
        let item = evidence_to_item(r, true);
        assert!(item.relative_path.is_none());
    }

    #[test]
    fn evidence_to_item_no_node_title() {
        let mut r = make_evidence_result();
        r.node_title = None;
        let item = evidence_to_item(r, false);
        assert!(item.node_title.is_none());
    }

    // --- DTO serialization tests ---

    #[test]
    fn evidence_result_item_serializes_correctly() {
        let item = EvidenceResultItem {
            chunk_id: 1,
            node_id: 2,
            heading_path: "# H".to_string(),
            snippet: "text".to_string(),
            relative_path: Some("path.md".to_string()),
            match_reason: MatchReason::Keyword,
            score: 0.123,
            node_title: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["chunk_id"], 1);
        assert_eq!(json["node_id"], 2);
        assert_eq!(json["heading_path"], "# H");
        assert_eq!(json["snippet"], "text");
        assert_eq!(json["relative_path"], "path.md");
        assert_eq!(json["score"], 0.123);
        // node_title should be present as null
        assert!(json["node_title"].is_null());
    }

    #[test]
    fn evidence_result_item_skips_none_relative_path() {
        let item = EvidenceResultItem {
            chunk_id: 1,
            node_id: 2,
            heading_path: "# H".to_string(),
            snippet: "text".to_string(),
            relative_path: None,
            match_reason: MatchReason::Graph,
            score: 0.5,
            node_title: Some("Title".to_string()),
        };
        let json = serde_json::to_value(&item).unwrap();
        // skip_serializing_if = "Option::is_none" means the key is absent
        assert!(json.get("relative_path").is_none());
        assert_eq!(json["node_title"], "Title");
    }

    #[test]
    fn index_status_summary_serializes() {
        let status = IndexStatusSummary {
            total_chunks: 100,
            embedded_chunks: 75,
            freshness_pct: 75.0,
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["total_chunks"], 100);
        assert_eq!(json["embedded_chunks"], 75);
        assert_eq!(json["freshness_pct"], 75.0);
    }

    #[test]
    fn evidence_response_serializes_all_fields() {
        let resp = EvidenceResponse {
            results: vec![],
            query: "test query".to_string(),
            mode: "hybrid".to_string(),
            index_status: IndexStatusSummary {
                total_chunks: 0,
                embedded_chunks: 0,
                freshness_pct: 0.0,
            },
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["query"], "test query");
        assert_eq!(json["mode"], "hybrid");
        assert!(json["results"].as_array().unwrap().is_empty());
        assert!(json["index_status"].is_object());
    }

    // --- Scope parsing edge cases ---

    #[tokio::test]
    async fn non_selection_scope_prefix_ignored() {
        let state = test_state().await;
        let app = test_router(state);

        // "folder:xyz" is not a recognized scope prefix — should be ignored, not error
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&mode=keyword&scope=folder:xyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn whitespace_only_query_returns_400() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=%20%20%20")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn explicit_limit_5_returns_ok() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test&limit=5&mode=keyword")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn default_mode_is_hybrid() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/evidence?q=test")
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
        assert_eq!(body["mode"], "hybrid");
    }
}
