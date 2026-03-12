//! AI assist endpoints for on-demand content generation.
//!
//! These are stateless: they generate content and return it without posting.
//! The user decides what to do with the results.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use tuitbot_core::content::ContentGenerator;
use tuitbot_core::context::retrieval::VaultCitation;
use tuitbot_core::storage;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::routes::rag_helpers::resolve_composer_rag_context;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn get_generator(
    state: &AppState,
    account_id: &str,
) -> Result<Arc<ContentGenerator>, ApiError> {
    state
        .get_or_create_content_generator(account_id)
        .await
        .map_err(ApiError::BadRequest)
}

// ---------------------------------------------------------------------------
// POST /api/assist/tweet
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistTweetRequest {
    pub topic: String,
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
}

#[derive(Serialize)]
pub struct AssistTweetResponse {
    pub content: String,
    pub topic: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

pub async fn assist_tweet(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<AssistTweetRequest>,
) -> Result<Json<AssistTweetResponse>, ApiError> {
    let gen = get_generator(&state, &ctx.account_id).await?;
    let node_ids = body.selected_node_ids.as_deref();
    let rag_context = resolve_composer_rag_context(&state, &ctx.account_id, node_ids).await;

    let prompt_block = rag_context.as_ref().map(|c| c.prompt_block.as_str());
    let citations = rag_context
        .as_ref()
        .map(|c| c.vault_citations.clone())
        .unwrap_or_default();

    let output = gen
        .generate_tweet_with_context(&body.topic, None, prompt_block)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistTweetResponse {
        content: output.text,
        topic: body.topic,
        vault_citations: citations,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/assist/reply
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistReplyRequest {
    pub tweet_text: String,
    pub tweet_author: String,
    #[serde(default)]
    pub mention_product: bool,
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
}

#[derive(Serialize)]
pub struct AssistReplyResponse {
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

pub async fn assist_reply(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<AssistReplyRequest>,
) -> Result<Json<AssistReplyResponse>, ApiError> {
    let gen = get_generator(&state, &ctx.account_id).await?;
    let node_ids = body.selected_node_ids.as_deref();
    let rag_context = resolve_composer_rag_context(&state, &ctx.account_id, node_ids).await;

    let prompt_block = rag_context.as_ref().map(|c| c.prompt_block.as_str());
    let citations = rag_context
        .as_ref()
        .map(|c| c.vault_citations.clone())
        .unwrap_or_default();

    let output = gen
        .generate_reply_with_context(
            &body.tweet_text,
            &body.tweet_author,
            body.mention_product,
            None,
            prompt_block,
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistReplyResponse {
        content: output.text,
        vault_citations: citations,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/assist/thread
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistThreadRequest {
    pub topic: String,
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
}

#[derive(Serialize)]
pub struct AssistThreadResponse {
    pub tweets: Vec<String>,
    pub topic: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

pub async fn assist_thread(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<AssistThreadRequest>,
) -> Result<Json<AssistThreadResponse>, ApiError> {
    let gen = get_generator(&state, &ctx.account_id).await?;
    let node_ids = body.selected_node_ids.as_deref();
    let rag_context = resolve_composer_rag_context(&state, &ctx.account_id, node_ids).await;

    let prompt_block = rag_context.as_ref().map(|c| c.prompt_block.as_str());
    let citations = rag_context
        .as_ref()
        .map(|c| c.vault_citations.clone())
        .unwrap_or_default();

    let output = gen
        .generate_thread_with_context(&body.topic, None, prompt_block)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistThreadResponse {
        tweets: output.tweets,
        topic: body.topic,
        vault_citations: citations,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/assist/improve
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistImproveRequest {
    pub draft: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
}

#[derive(Serialize)]
pub struct AssistImproveResponse {
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

pub async fn assist_improve(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<AssistImproveRequest>,
) -> Result<Json<AssistImproveResponse>, ApiError> {
    let gen = get_generator(&state, &ctx.account_id).await?;
    let node_ids = body.selected_node_ids.as_deref();
    let rag_context = resolve_composer_rag_context(&state, &ctx.account_id, node_ids).await;

    let prompt_block = rag_context.as_ref().map(|c| c.prompt_block.as_str());
    let citations = rag_context
        .as_ref()
        .map(|c| c.vault_citations.clone())
        .unwrap_or_default();

    let output = gen
        .improve_draft_with_context(&body.draft, body.context.as_deref(), prompt_block)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistImproveResponse {
        content: output.text,
        vault_citations: citations,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/assist/topics
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AssistTopicsResponse {
    pub topics: Vec<TopicRecommendation>,
}

#[derive(Serialize)]
pub struct TopicRecommendation {
    pub topic: String,
    pub score: f64,
}

pub async fn assist_topics(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<AssistTopicsResponse>, ApiError> {
    let top = storage::analytics::get_top_topics_for(&state.db, &ctx.account_id, 10).await?;

    let topics = top
        .into_iter()
        .map(|cs| TopicRecommendation {
            topic: cs.topic,
            score: cs.avg_performance,
        })
        .collect();

    Ok(Json(AssistTopicsResponse { topics }))
}

// ---------------------------------------------------------------------------
// GET /api/assist/optimal-times
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct OptimalTimesResponse {
    pub times: Vec<OptimalTime>,
}

#[derive(Serialize)]
pub struct OptimalTime {
    pub hour: u32,
    pub avg_engagement: f64,
    pub post_count: i64,
}

pub async fn assist_optimal_times(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<OptimalTimesResponse>, ApiError> {
    let rows =
        storage::analytics::get_optimal_posting_times_for(&state.db, &ctx.account_id).await?;

    let times = rows
        .into_iter()
        .map(|r| OptimalTime {
            hour: r.hour as u32,
            avg_engagement: r.avg_engagement,
            post_count: r.post_count,
        })
        .collect();

    Ok(Json(OptimalTimesResponse { times }))
}

// ---------------------------------------------------------------------------
// GET /api/assist/mode
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ModeResponse {
    pub mode: String,
    pub approval_mode: bool,
}

pub async fn get_mode(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<(StatusCode, Json<ModeResponse>), ApiError> {
    let config = crate::routes::content::read_effective_config(&state, &ctx.account_id).await?;

    Ok((
        StatusCode::OK,
        Json(ModeResponse {
            mode: config.mode.to_string(),
            // Return the raw `approval_mode` setting — not the effective one.
            // The Composer-mode override that forces approval for autonomous
            // loops should not affect user-initiated manual compose actions
            // in the dashboard (the Publish button).
            approval_mode: config.approval_mode,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::path::PathBuf;

    use tokio::sync::{broadcast, Mutex, RwLock};

    use crate::ws::AccountWsEvent;

    /// Build a minimal `AppState` for testing the RAG resolver.
    async fn test_state(config_path: PathBuf) -> AppState {
        let db = tuitbot_core::storage::init_test_db()
            .await
            .expect("init test db");
        let (event_tx, _) = broadcast::channel::<AccountWsEvent>(16);
        AppState {
            db,
            config_path: config_path.clone(),
            data_dir: config_path.parent().unwrap_or(&config_path).to_path_buf(),
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
            watchtower_cancel: RwLock::new(None),
            content_sources: RwLock::new(Default::default()),
            connector_config: Default::default(),
            deployment_mode: Default::default(),

            pending_oauth: Mutex::new(HashMap::new()),
            token_managers: Mutex::new(HashMap::new()),
            x_client_id: String::new(),
        }
    }

    #[tokio::test]
    async fn resolve_rag_returns_none_when_config_missing() {
        let state = test_state(PathBuf::from("/nonexistent/config.toml")).await;
        let result = resolve_composer_rag_context(&state, "test-account", None).await;
        assert!(
            result.is_none(),
            "should return None when config is missing"
        );
    }

    #[tokio::test]
    async fn resolve_rag_returns_none_when_db_empty() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            "[business]\nproduct_name = \"TestProduct\"\nproduct_keywords = [\"rust\", \"testing\"]\n",
        )
        .expect("write config");

        let state = test_state(config_path).await;
        let result = resolve_composer_rag_context(&state, "test-account", None).await;
        assert!(
            result.is_none(),
            "should return None when DB has no ancestor data"
        );
    }

    #[tokio::test]
    async fn resolve_rag_returns_none_when_no_keywords() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config_path = dir.path().join("config.toml");
        // Empty business profile → no keywords → early return None.
        std::fs::write(&config_path, "[business]\nproduct_name = \"Empty\"\n")
            .expect("write config");

        let state = test_state(config_path).await;
        let result = resolve_composer_rag_context(&state, "test-account", None).await;
        assert!(
            result.is_none(),
            "should return None when keywords are empty"
        );
    }

    #[test]
    fn selected_node_ids_is_optional() {
        // Verify existing request shapes still deserialize without selected_node_ids.
        let json = r#"{"topic": "Rust async"}"#;
        let req: AssistTweetRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.topic, "Rust async");
        assert!(req.selected_node_ids.is_none());

        let json = r#"{"topic": "Rust async", "selected_node_ids": [1, 2, 3]}"#;
        let req: AssistTweetRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.selected_node_ids.unwrap(), vec![1, 2, 3]);

        let json = r#"{"topic": "threads"}"#;
        let req: AssistThreadRequest = serde_json::from_str(json).expect("deserialize");
        assert!(req.selected_node_ids.is_none());

        let json = r#"{"draft": "hello"}"#;
        let req: AssistImproveRequest = serde_json::from_str(json).expect("deserialize");
        assert!(req.selected_node_ids.is_none());
    }

    #[test]
    fn reply_request_selected_node_ids_is_optional() {
        let json = r#"{"tweet_text": "hello", "tweet_author": "user"}"#;
        let req: AssistReplyRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.tweet_text, "hello");
        assert!(!req.mention_product);
        assert!(req.selected_node_ids.is_none());

        let json = r#"{"tweet_text": "hi", "tweet_author": "u", "selected_node_ids": [10, 20]}"#;
        let req: AssistReplyRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.selected_node_ids.unwrap(), vec![10, 20]);
    }

    #[test]
    fn reply_response_omits_empty_citations() {
        let resp = AssistReplyResponse {
            content: "Great point!".to_string(),
            vault_citations: vec![],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(!json.contains("vault_citations"));
    }
}
