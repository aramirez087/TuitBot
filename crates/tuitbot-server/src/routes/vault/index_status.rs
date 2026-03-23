//! `GET /api/vault/index-status` — semantic index health and stats.
//!
//! Returns the current state of the semantic index for the requesting account,
//! including freshness, provider configuration, and in-memory index size.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Serialize;

use tuitbot_core::storage::watchtower::embeddings;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct IndexStatusResponse {
    pub total_chunks: i64,
    pub embedded_chunks: i64,
    pub dirty_chunks: i64,
    pub freshness_pct: f64,
    pub last_indexed_at: Option<String>,
    pub model_id: Option<String>,
    pub provider_configured: bool,
    pub index_loaded: bool,
    pub index_size: usize,
    /// Deployment mode for privacy envelope display.
    pub deployment_mode: String,
    /// Whether semantic search is available right now.
    pub search_available: bool,
    /// Provider name (e.g., "openai", "ollama") if configured.
    pub provider_name: Option<String>,
}

pub async fn get_index_status(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<IndexStatusResponse>, ApiError> {
    let stats = embeddings::get_index_stats_for(&state.db, &ctx.account_id).await?;

    let provider_configured = state.embedding_provider.is_some();

    let (index_loaded, index_size) = if let Some(idx) = &state.semantic_index {
        let guard = idx.read().await;
        (true, guard.len())
    } else {
        (false, 0)
    };

    let search_available = provider_configured && index_loaded;
    let provider_name = state
        .embedding_provider
        .as_ref()
        .map(|p| p.name().to_string());

    Ok(Json(IndexStatusResponse {
        total_chunks: stats.total_chunks,
        embedded_chunks: stats.embedded_chunks,
        dirty_chunks: stats.dirty_chunks,
        freshness_pct: stats.freshness_pct,
        last_indexed_at: stats.last_indexed_at,
        model_id: stats.model_id,
        provider_configured,
        index_loaded,
        index_size,
        deployment_mode: state.deployment_mode.to_string(),
        search_available,
        provider_name,
    }))
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
            .route("/vault/index-status", get(get_index_status))
            .with_state(state)
    }

    #[tokio::test]
    async fn returns_zero_stats_when_no_embeddings() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/index-status")
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
        assert_eq!(body["total_chunks"], 0);
        assert_eq!(body["embedded_chunks"], 0);
        assert_eq!(body["dirty_chunks"], 0);
        assert_eq!(body["provider_configured"], false);
        assert_eq!(body["index_loaded"], false);
        assert_eq!(body["index_size"], 0);
        assert_eq!(body["deployment_mode"], "desktop");
        assert_eq!(body["search_available"], false);
        assert!(body["provider_name"].is_null());
    }

    #[tokio::test]
    async fn provider_configured_reflects_state() {
        let state = test_state().await;
        // No embedding_provider set → false
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/index-status")
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
        assert_eq!(body["provider_configured"], false);
    }

    #[tokio::test]
    async fn search_unavailable_without_provider() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/index-status")
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
        assert_eq!(body["search_available"], false);
        assert_eq!(body["deployment_mode"], "desktop");
    }

    #[tokio::test]
    async fn index_loaded_reflects_state() {
        let state = test_state().await;
        let app = test_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/vault/index-status")
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
        // semantic_index is None → false
        assert_eq!(body["index_loaded"], false);
    }
}
