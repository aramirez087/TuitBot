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

    use tuitbot_core::context::semantic_index::SemanticIndex;
    use tuitbot_core::llm::embedding::{
        EmbeddingError, EmbeddingInput, EmbeddingProvider, EmbeddingResponse, EmbeddingUsage,
    };

    use crate::ws::AccountWsEvent;

    struct MockEmbeddingProvider;

    #[async_trait::async_trait]
    impl EmbeddingProvider for MockEmbeddingProvider {
        fn name(&self) -> &str {
            "mock-ollama"
        }
        fn dimension(&self) -> usize {
            768
        }
        fn model_id(&self) -> &str {
            "mock-model"
        }
        async fn embed(
            &self,
            _inputs: EmbeddingInput,
        ) -> Result<EmbeddingResponse, EmbeddingError> {
            Ok(EmbeddingResponse {
                embeddings: vec![],
                model: "mock-model".to_string(),
                dimension: 768,
                usage: EmbeddingUsage::default(),
            })
        }
        async fn health_check(&self) -> Result<(), EmbeddingError> {
            Ok(())
        }
    }

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

    async fn test_state_with_provider_and_index() -> Arc<AppState> {
        let db = tuitbot_core::storage::init_test_db()
            .await
            .expect("init test db");
        let (event_tx, _) = broadcast::channel::<AccountWsEvent>(16);
        let index = Arc::new(RwLock::new(SemanticIndex::new(
            768,
            "mock-model".to_string(),
            1000,
        )));
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
            semantic_index: Some(index),
            embedding_provider: Some(Arc::new(MockEmbeddingProvider)),
        })
    }

    #[tokio::test]
    async fn search_available_when_provider_and_index_configured() {
        let state = test_state_with_provider_and_index().await;
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
        assert_eq!(body["provider_configured"], true);
        assert_eq!(body["index_loaded"], true);
        assert_eq!(body["search_available"], true);
        assert_eq!(body["provider_name"], "mock-ollama");
        assert_eq!(body["index_size"], 0);
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

    #[test]
    fn index_status_response_serialization() {
        let resp = IndexStatusResponse {
            total_chunks: 100,
            embedded_chunks: 80,
            dirty_chunks: 20,
            freshness_pct: 80.0,
            last_indexed_at: Some("2026-03-23T12:00:00Z".to_string()),
            model_id: Some("nomic-embed-text".to_string()),
            provider_configured: true,
            index_loaded: true,
            index_size: 80,
            deployment_mode: "desktop".to_string(),
            search_available: true,
            provider_name: Some("ollama".to_string()),
        };
        let json = serde_json::to_value(&resp).expect("serialize");
        assert_eq!(json["total_chunks"], 100);
        assert_eq!(json["embedded_chunks"], 80);
        assert_eq!(json["dirty_chunks"], 20);
        assert_eq!(json["freshness_pct"], 80.0);
        assert_eq!(json["last_indexed_at"], "2026-03-23T12:00:00Z");
        assert_eq!(json["model_id"], "nomic-embed-text");
        assert_eq!(json["provider_configured"], true);
        assert_eq!(json["index_loaded"], true);
        assert_eq!(json["index_size"], 80);
        assert_eq!(json["deployment_mode"], "desktop");
        assert_eq!(json["search_available"], true);
        assert_eq!(json["provider_name"], "ollama");
    }

    #[test]
    fn index_status_response_serialization_with_nulls() {
        let resp = IndexStatusResponse {
            total_chunks: 0,
            embedded_chunks: 0,
            dirty_chunks: 0,
            freshness_pct: 100.0,
            last_indexed_at: None,
            model_id: None,
            provider_configured: false,
            index_loaded: false,
            index_size: 0,
            deployment_mode: "cloud".to_string(),
            search_available: false,
            provider_name: None,
        };
        let json = serde_json::to_value(&resp).expect("serialize");
        assert!(json["last_indexed_at"].is_null());
        assert!(json["model_id"].is_null());
        assert!(json["provider_name"].is_null());
        assert_eq!(json["deployment_mode"], "cloud");
    }
}
