//! Draft content endpoints.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::content::{
    serialize_blocks_for_storage, tweet_weighted_len, validate_thread_blocks, ThreadBlock,
    MAX_TWEET_CHARS,
};
use tuitbot_core::storage::{approval_queue, scheduled_content};

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;

use super::compose::ThreadBlockRequest;

#[derive(Deserialize)]
pub struct CreateDraftRequest {
    pub content_type: String,
    pub content: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub blocks: Option<Vec<ThreadBlockRequest>>,
}

fn default_source() -> String {
    "manual".to_string()
}

pub async fn list_drafts(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Vec<scheduled_content::ScheduledContent>>, ApiError> {
    let drafts = scheduled_content::list_drafts_for(&state.db, &ctx.account_id)
        .await
        .map_err(ApiError::Storage)?;
    Ok(Json(drafts))
}

pub async fn create_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(mut body): Json<CreateDraftRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let blocks = body.blocks.take();

    // When blocks are provided for a thread, use them.
    let content = if body.content_type == "thread" {
        if let Some(block_requests) = blocks {
            let core_blocks: Vec<ThreadBlock> =
                block_requests.into_iter().map(|b| b.into_core()).collect();
            validate_thread_blocks(&core_blocks)
                .map_err(|e| ApiError::BadRequest(e.api_message()))?;
            serialize_blocks_for_storage(&core_blocks)
        } else {
            validate_draft_content(&body.content_type, &body.content)?;
            body.content.clone()
        }
    } else {
        validate_draft_content(&body.content_type, &body.content)?;
        body.content.clone()
    };

    let id = scheduled_content::insert_draft_for(
        &state.db,
        &ctx.account_id,
        &body.content_type,
        &content,
        &body.source,
    )
    .await
    .map_err(ApiError::Storage)?;

    Ok(Json(json!({ "id": id, "status": "draft" })))
}

/// Validate draft content for legacy (non-blocks) payloads.
fn validate_draft_content(content_type: &str, content: &str) -> Result<(), ApiError> {
    if content.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "content must not be empty".to_string(),
        ));
    }

    if content_type == "tweet"
        && !tuitbot_core::content::validate_tweet_length(content, MAX_TWEET_CHARS)
    {
        return Err(ApiError::BadRequest(format!(
            "Tweet exceeds {} characters (weighted length: {})",
            MAX_TWEET_CHARS,
            tweet_weighted_len(content)
        )));
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct EditDraftRequest {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub blocks: Option<Vec<ThreadBlockRequest>>,
}

pub async fn edit_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<EditDraftRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let content = if let Some(block_requests) = body.blocks {
        let core_blocks: Vec<ThreadBlock> =
            block_requests.into_iter().map(|b| b.into_core()).collect();
        validate_thread_blocks(&core_blocks).map_err(|e| ApiError::BadRequest(e.api_message()))?;
        serialize_blocks_for_storage(&core_blocks)
    } else if let Some(ref text) = body.content {
        if text.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "content must not be empty".to_string(),
            ));
        }
        text.clone()
    } else {
        return Err(ApiError::BadRequest(
            "must provide either 'content' or 'blocks'".to_string(),
        ));
    };

    scheduled_content::update_draft_for(&state.db, &ctx.account_id, id, &content)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(json!({ "id": id, "status": "draft" })))
}

pub async fn delete_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    scheduled_content::delete_draft_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(json!({ "id": id, "status": "cancelled" })))
}

#[derive(Deserialize)]
pub struct ScheduleDraftRequest {
    pub scheduled_for: String,
}

pub async fn schedule_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<ScheduleDraftRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    scheduled_content::schedule_draft_for(&state.db, &ctx.account_id, id, &body.scheduled_for)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(
        json!({ "id": id, "status": "scheduled", "scheduled_for": body.scheduled_for }),
    ))
}

pub async fn publish_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Get the draft.
    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    if item.status != "draft" {
        return Err(ApiError::BadRequest(format!(
            "Item is in '{}' status, not 'draft'",
            item.status
        )));
    }

    // Queue into approval queue for immediate posting.
    let queue_id = approval_queue::enqueue_for(
        &state.db,
        &ctx.account_id,
        &item.content_type,
        "", // no target tweet
        "", // no target author
        &item.content,
        "",  // topic
        "",  // archetype
        0.0, // score
        "[]",
    )
    .await
    .map_err(ApiError::Storage)?;

    // Mark as approved immediately so the approval poster picks it up.
    approval_queue::update_status_for(&state.db, &ctx.account_id, queue_id, "approved")
        .await
        .map_err(ApiError::Storage)?;

    // Mark the draft as posted.
    scheduled_content::update_status_for(&state.db, &ctx.account_id, id, "posted", None)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(
        json!({ "id": id, "approval_queue_id": queue_id, "status": "queued_for_posting" }),
    ))
}
