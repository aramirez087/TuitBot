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
use tuitbot_core::storage::provenance::ProvenanceRef;
use tuitbot_core::storage::{approval_queue, provenance, scheduled_content};

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
    /// Optional provenance refs linking this draft to vault source material.
    #[serde(default)]
    pub provenance: Option<Vec<ProvenanceRef>>,
    /// Optional hook style tag (e.g. "contrarian_take") for source enrichment.
    #[serde(default)]
    pub hook_style: Option<String>,
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

    // Enrich source with hook style when present (e.g. "assist:hook:contrarian_take").
    let effective_source = match body.hook_style.as_deref() {
        Some(style) if !style.is_empty() => format!("assist:hook:{style}"),
        _ => body.source.clone(),
    };

    let id = if let Some(ref refs) = body.provenance {
        scheduled_content::insert_draft_with_provenance_for(
            &state.db,
            &ctx.account_id,
            &body.content_type,
            &content,
            &effective_source,
            refs,
        )
        .await
        .map_err(ApiError::Storage)?
    } else {
        scheduled_content::insert_draft_for(
            &state.db,
            &ctx.account_id,
            &body.content_type,
            &content,
            &effective_source,
        )
        .await
        .map_err(ApiError::Storage)?
    };

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

    let normalized = tuitbot_core::scheduling::validate_and_normalize(
        &body.scheduled_for,
        tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    scheduled_content::schedule_draft_for(&state.db, &ctx.account_id, id, &normalized)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(
        json!({ "id": id, "status": "scheduled", "scheduled_for": normalized }),
    ))
}

pub async fn publish_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    super::require_post_capable(&state, &ctx.account_id).await?;

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

    // Load provenance links from the draft's scheduled_content record.
    let draft_links =
        provenance::get_links_for(&state.db, &ctx.account_id, "scheduled_content", id)
            .await
            .map_err(ApiError::Storage)?;

    let prov_input = if draft_links.is_empty() {
        None
    } else {
        let refs: Vec<ProvenanceRef> = draft_links
            .iter()
            .map(|l| ProvenanceRef {
                node_id: l.node_id,
                chunk_id: l.chunk_id,
                seed_id: l.seed_id,
                source_path: l.source_path.clone(),
                heading_path: l.heading_path.clone(),
                snippet: l.snippet.clone(),
            })
            .collect();
        let source_node_id = refs.iter().find_map(|r| r.node_id);
        let source_seed_id = refs.iter().find_map(|r| r.seed_id);
        Some(approval_queue::ProvenanceInput {
            source_node_id,
            source_seed_id,
            source_chunks_json: serde_json::to_string(&refs).unwrap_or_else(|_| "[]".to_string()),
            refs,
        })
    };

    // Queue into approval queue for immediate posting.
    let queue_id = approval_queue::enqueue_with_provenance_for(
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
        None,
        None,
        prov_input.as_ref(),
        None, // no scheduling intent — direct publish via approval poster
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

/// `GET /api/drafts/:id/provenance` — retrieve provenance links for a draft.
pub async fn get_draft_provenance(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Vec<provenance::ProvenanceLink>>, ApiError> {
    // Verify the draft exists and belongs to this account.
    let _item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    let links = provenance::get_links_for(&state.db, &ctx.account_id, "scheduled_content", id)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(links))
}
