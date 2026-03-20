//! Compose endpoints for tweets, threads, and unified compose.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::content::ThreadBlock;
use tuitbot_core::storage::approval_queue;
use tuitbot_core::storage::provenance::ProvenanceRef;

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::{AccountWsEvent, WsEvent};

use super::read_approval_mode;

/// A single thread block in an API request payload.
#[derive(Debug, Deserialize)]
pub struct ThreadBlockRequest {
    /// Client-generated stable UUID.
    pub id: String,
    /// Tweet text content.
    pub text: String,
    /// Per-block media file paths.
    #[serde(default)]
    pub media_paths: Vec<String>,
    /// Zero-based ordering index.
    pub order: u32,
}

impl ThreadBlockRequest {
    /// Convert to the core domain type.
    pub(crate) fn into_core(self) -> ThreadBlock {
        ThreadBlock {
            id: self.id,
            text: self.text,
            media_paths: self.media_paths,
            order: self.order,
        }
    }
}

/// Request body for composing a manual tweet.
#[derive(Deserialize)]
pub struct ComposeTweetRequest {
    /// The tweet text.
    pub text: String,
    /// Optional ISO 8601 timestamp to schedule the tweet.
    pub scheduled_for: Option<String>,
    /// Optional provenance refs linking this content to vault source material.
    #[serde(default)]
    pub provenance: Option<Vec<ProvenanceRef>>,
}

/// `POST /api/content/tweets` — compose and queue a manual tweet.
pub async fn compose_tweet(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<ComposeTweetRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let text = body.text.trim();
    if text.is_empty() {
        return Err(ApiError::BadRequest("text is required".to_string()));
    }

    // Check if approval mode is enabled.
    let approval_mode = read_approval_mode(&state, &ctx.account_id).await?;

    if approval_mode {
        let prov_input = build_provenance_input(body.provenance.as_deref());

        let id = approval_queue::enqueue_with_provenance_for(
            &state.db,
            &ctx.account_id,
            "tweet",
            "", // no target tweet
            "", // no target author
            text,
            "", // no topic
            "", // no archetype
            0.0,
            "[]",
            None,
            None,
            prov_input.as_ref(),
            body.scheduled_for.as_deref(),
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ApprovalQueued {
                id,
                action_type: "tweet".to_string(),
                content: text.to_string(),
                media_paths: vec![],
            },
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
            "scheduled_for": body.scheduled_for,
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
    ctx: AccountContext,
    Json(body): Json<ComposeThreadRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    if body.tweets.is_empty() {
        return Err(ApiError::BadRequest(
            "tweets array must not be empty".to_string(),
        ));
    }

    let approval_mode = read_approval_mode(&state, &ctx.account_id).await?;
    let combined = body.tweets.join("\n---\n");

    if approval_mode {
        let id = approval_queue::enqueue_with_context_for(
            &state.db,
            &ctx.account_id,
            "thread",
            "",
            "",
            &combined,
            "",
            "",
            0.0,
            "[]",
            None,
            None,
            body.scheduled_for.as_deref(),
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ApprovalQueued {
                id,
                action_type: "thread".to_string(),
                content: combined,
                media_paths: vec![],
            },
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
            "scheduled_for": body.scheduled_for,
        })))
    } else {
        Ok(Json(json!({
            "status": "accepted",
            "tweet_count": body.tweets.len(),
            "scheduled_for": body.scheduled_for,
        })))
    }
}

/// Request body for the unified compose endpoint.
#[derive(Deserialize)]
pub struct ComposeRequest {
    /// Content type: "tweet" or "thread".
    pub content_type: String,
    /// Content text (string for tweet, JSON array string for thread).
    pub content: String,
    /// Optional ISO 8601 timestamp to schedule the content.
    pub scheduled_for: Option<String>,
    /// Optional local media file paths to attach (top-level, used for tweets).
    #[serde(default)]
    pub media_paths: Option<Vec<String>>,
    /// Optional structured thread blocks. Takes precedence over `content` for threads.
    #[serde(default)]
    pub blocks: Option<Vec<ThreadBlockRequest>>,
    /// Optional provenance refs linking this content to vault source material.
    #[serde(default)]
    pub provenance: Option<Vec<ProvenanceRef>>,
}

/// `POST /api/content/compose` — compose manual content (tweet or thread).
pub async fn compose(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(mut body): Json<ComposeRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let blocks = body.blocks.take();

    match body.content_type.as_str() {
        "tweet" => transforms::compose_tweet_flow(&state, &ctx, &body).await,
        "thread" => {
            if let Some(blocks) = blocks {
                transforms::compose_thread_blocks_flow(&state, &ctx, &body, blocks).await
            } else {
                transforms::compose_thread_legacy_flow(&state, &ctx, &body).await
            }
        }
        _ => Err(ApiError::BadRequest(
            "content_type must be 'tweet' or 'thread'".to_string(),
        )),
    }
}

// Handle tweet compose via the unified endpoint.

// ---------------------------------------------------------------------------
// Helpers used by the handlers above (kept here to avoid cross-module imports)
// ---------------------------------------------------------------------------

fn build_provenance_input(
    provenance: Option<&[ProvenanceRef]>,
) -> Option<approval_queue::ProvenanceInput> {
    let refs = provenance?;
    if refs.is_empty() {
        return None;
    }

    let source_node_id = refs.iter().find_map(|r| r.node_id);
    let source_seed_id = refs.iter().find_map(|r| r.seed_id);
    let source_chunks_json = serde_json::to_string(refs).unwrap_or_else(|_| "[]".to_string());

    Some(approval_queue::ProvenanceInput {
        source_node_id,
        source_seed_id,
        source_chunks_json,
        refs: refs.to_vec(),
    })
}

pub(crate) mod transforms;

#[cfg(test)]
mod tests;
