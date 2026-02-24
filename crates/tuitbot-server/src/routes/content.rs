//! Content endpoints (tweets and threads).

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::config::Config;
use tuitbot_core::storage::{approval_queue, threads};

use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::WsEvent;

/// Query parameters for the tweets endpoint.
#[derive(Deserialize)]
pub struct TweetsQuery {
    /// Maximum number of tweets to return (default: 50).
    #[serde(default = "default_tweet_limit")]
    pub limit: u32,
}

fn default_tweet_limit() -> u32 {
    50
}

/// Query parameters for the threads endpoint.
#[derive(Deserialize)]
pub struct ThreadsQuery {
    /// Maximum number of threads to return (default: 20).
    #[serde(default = "default_thread_limit")]
    pub limit: u32,
}

fn default_thread_limit() -> u32 {
    20
}

/// `GET /api/content/tweets` — recent original tweets posted.
pub async fn list_tweets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TweetsQuery>,
) -> Result<Json<Value>, ApiError> {
    let tweets = threads::get_recent_original_tweets(&state.db, params.limit).await?;
    Ok(Json(json!(tweets)))
}

/// `GET /api/content/threads` — recent threads posted.
pub async fn list_threads(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ThreadsQuery>,
) -> Result<Json<Value>, ApiError> {
    let threads = threads::get_recent_threads(&state.db, params.limit).await?;
    Ok(Json(json!(threads)))
}

/// Request body for composing a manual tweet.
#[derive(Deserialize)]
pub struct ComposeTweetRequest {
    /// The tweet text.
    pub text: String,
    /// Optional ISO 8601 timestamp to schedule the tweet.
    pub scheduled_for: Option<String>,
}

/// `POST /api/content/tweets` — compose and queue a manual tweet.
pub async fn compose_tweet(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ComposeTweetRequest>,
) -> Result<Json<Value>, ApiError> {
    let text = body.text.trim();
    if text.is_empty() {
        return Err(ApiError::BadRequest("text is required".to_string()));
    }

    // Check if approval mode is enabled.
    let approval_mode = read_approval_mode(&state)?;

    if approval_mode {
        let id = approval_queue::enqueue(
            &state.db, "tweet", "", // no target tweet
            "", // no target author
            text, "", // no topic
            "", // no archetype
            0.0,
        )
        .await?;

        let _ = state.event_tx.send(WsEvent::ApprovalQueued {
            id,
            action_type: "tweet".to_string(),
            content: text.to_string(),
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else {
        // Without X API client in AppState, we can only acknowledge the intent.
        Ok(Json(json!({
            "status": "accepted",
            "text": text,
            "scheduled_for": body.scheduled_for,
        })))
    }
}

/// Request body for composing a manual thread.
#[derive(Deserialize)]
pub struct ComposeThreadRequest {
    /// The tweets forming the thread.
    pub tweets: Vec<String>,
    /// Optional ISO 8601 timestamp to schedule the thread.
    pub scheduled_for: Option<String>,
}

/// `POST /api/content/threads` — compose and queue a manual thread.
pub async fn compose_thread(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ComposeThreadRequest>,
) -> Result<Json<Value>, ApiError> {
    if body.tweets.is_empty() {
        return Err(ApiError::BadRequest(
            "tweets array must not be empty".to_string(),
        ));
    }

    let approval_mode = read_approval_mode(&state)?;
    let combined = body.tweets.join("\n---\n");

    if approval_mode {
        let id =
            approval_queue::enqueue(&state.db, "thread", "", "", &combined, "", "", 0.0).await?;

        let _ = state.event_tx.send(WsEvent::ApprovalQueued {
            id,
            action_type: "thread".to_string(),
            content: combined,
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else {
        Ok(Json(json!({
            "status": "accepted",
            "tweet_count": body.tweets.len(),
            "scheduled_for": body.scheduled_for,
        })))
    }
}

/// Read `approval_mode` from the config file.
fn read_approval_mode(state: &AppState) -> Result<bool, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: Config = toml::from_str(&contents).unwrap_or_default();
    Ok(config.approval_mode)
}
