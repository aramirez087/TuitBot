//! Draft Studio API endpoints.
//!
//! Provides the canonical `/api/drafts` routes for the Draft Studio workspace,
//! including collection queries, CRUD, autosave with conflict detection,
//! workflow transitions, and revision/activity read endpoints.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tuitbot_core::storage::scheduled_content;

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DraftListQuery {
    pub status: Option<String>,
    pub tag: Option<i64>,
    pub search: Option<String>,
    pub archived: Option<bool>,
}

#[derive(Serialize)]
pub struct DraftSummary {
    pub id: i64,
    pub title: Option<String>,
    pub content_type: String,
    pub content_preview: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub archived_at: Option<String>,
    pub updated_at: String,
    pub created_at: String,
    pub source: String,
}

#[derive(Deserialize)]
pub struct CreateStudioDraftBody {
    #[serde(default = "default_tweet")]
    pub content_type: String,
    #[serde(default = "default_blank_content")]
    pub content: String,
    #[serde(default = "default_manual")]
    pub source: String,
    pub title: Option<String>,
}

fn default_tweet() -> String {
    "tweet".to_string()
}

fn default_blank_content() -> String {
    " ".to_string()
}

fn default_manual() -> String {
    "manual".to_string()
}

#[derive(Deserialize)]
pub struct AutosavePatchBody {
    pub content: String,
    pub content_type: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct MetaPatchBody {
    pub title: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ScheduleBody {
    pub scheduled_for: String,
}

#[derive(Deserialize)]
pub struct CreateRevisionBody {
    #[serde(default = "default_manual_trigger")]
    pub trigger_kind: String,
}

fn default_manual_trigger() -> String {
    "manual".to_string()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate content to ~60 chars for list previews.
/// For threads (JSON blocks), extract the first block's text.
fn content_preview(content: &str, content_type: &str) -> String {
    let text = if content_type == "thread" {
        extract_first_block_text(content)
    } else {
        content.to_string()
    };
    let trimmed = text.trim();
    if trimmed.len() <= 60 {
        trimmed.to_string()
    } else {
        let mut preview = trimmed.chars().take(57).collect::<String>();
        preview.push_str("...");
        preview
    }
}

/// Try to extract the first block's text from thread JSON content.
fn extract_first_block_text(content: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(blocks) = parsed.get("blocks").and_then(|b| b.as_array()) {
            if let Some(first) = blocks.first() {
                if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                    return text.to_string();
                }
            }
        }
        // Legacy array format
        if let Some(arr) = parsed.as_array() {
            if let Some(first) = arr.first().and_then(|v| v.as_str()) {
                return first.to_string();
            }
        }
    }
    content.to_string()
}

fn to_summary(item: &scheduled_content::ScheduledContent) -> DraftSummary {
    DraftSummary {
        id: item.id,
        title: item.title.clone(),
        content_type: item.content_type.clone(),
        content_preview: content_preview(&item.content, &item.content_type),
        status: item.status.clone(),
        scheduled_for: item.scheduled_for.clone(),
        archived_at: item.archived_at.clone(),
        updated_at: item.updated_at.clone(),
        created_at: item.created_at.clone(),
        source: item.source.clone(),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/drafts` — list drafts with optional filters.
pub async fn list_studio_drafts(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(query): Query<DraftListQuery>,
) -> Result<Json<Vec<DraftSummary>>, ApiError> {
    let items = if query.archived == Some(true) {
        scheduled_content::list_archived_drafts_for(&state.db, &ctx.account_id)
            .await
            .map_err(ApiError::Storage)?
    } else {
        scheduled_content::list_drafts_for(&state.db, &ctx.account_id)
            .await
            .map_err(ApiError::Storage)?
    };

    let mut summaries: Vec<DraftSummary> = items.iter().map(to_summary).collect();

    // In-application filtering for status
    if let Some(ref status_filter) = query.status {
        if status_filter != "all" {
            summaries.retain(|s| s.status == *status_filter);
        }
    }

    // In-application filtering for search
    if let Some(ref search) = query.search {
        let needle = search.to_lowercase();
        summaries.retain(|s| {
            s.content_preview.to_lowercase().contains(&needle)
                || s.title
                    .as_ref()
                    .is_some_and(|t| t.to_lowercase().contains(&needle))
        });
    }

    Ok(Json(summaries))
}

/// `GET /api/drafts/:id` — get a single draft by ID.
pub async fn get_studio_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<scheduled_content::ScheduledContent>, ApiError> {
    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;
    Ok(Json(item))
}

/// `POST /api/drafts` — create a blank or seeded draft.
pub async fn create_studio_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<CreateStudioDraftBody>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let content = if body.content.trim().is_empty() {
        " ".to_string()
    } else {
        body.content
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

    // Set title if provided
    if let Some(ref title) = body.title {
        let _ = scheduled_content::update_draft_meta_for(
            &state.db,
            &ctx.account_id,
            id,
            Some(title),
            None,
        )
        .await;
    }

    // Log created activity
    let _ = scheduled_content::insert_activity_for(&state.db, &ctx.account_id, id, "created", None)
        .await;

    // Fetch the created item to get updated_at
    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::Internal("Created draft not found".to_string()))?;

    Ok(Json(json!({ "id": id, "updated_at": item.updated_at })))
}

/// `PATCH /api/drafts/:id` — autosave content (side-effect free).
pub async fn autosave_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<AutosavePatchBody>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Verify the draft exists
    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    // Conflict detection: compare updated_at
    if item.updated_at != body.updated_at {
        return Err(ApiError::Conflict(
            json!({
                "error": "stale_write",
                "server_updated_at": item.updated_at
            })
            .to_string(),
        ));
    }

    let new_updated_at = scheduled_content::autosave_draft_for(
        &state.db,
        &ctx.account_id,
        id,
        &body.content,
        &body.content_type,
        &body.updated_at,
    )
    .await
    .map_err(ApiError::Storage)?;

    match new_updated_at {
        Some(updated_at) => Ok(Json(json!({ "id": id, "updated_at": updated_at }))),
        None => {
            // Race condition: updated_at changed between our check and update
            let current = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
                .await
                .map_err(ApiError::Storage)?;
            let server_updated_at = current.map(|c| c.updated_at).unwrap_or_default();
            Err(ApiError::Conflict(
                json!({
                    "error": "stale_write",
                    "server_updated_at": server_updated_at
                })
                .to_string(),
            ))
        }
    }
}

/// `PATCH /api/drafts/:id/meta` — update title and notes.
pub async fn patch_draft_meta(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<MetaPatchBody>,
) -> Result<Json<scheduled_content::ScheduledContent>, ApiError> {
    require_mutate(&ctx)?;

    scheduled_content::update_draft_meta_for(
        &state.db,
        &ctx.account_id,
        id,
        body.title.as_deref(),
        body.notes.as_deref(),
    )
    .await
    .map_err(ApiError::Storage)?;

    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    Ok(Json(item))
}

/// `POST /api/drafts/:id/schedule` — transition draft -> scheduled.
pub async fn schedule_studio_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<ScheduleBody>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

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

    // Create revision snapshot before scheduling
    let _ = scheduled_content::insert_revision_for(
        &state.db,
        &ctx.account_id,
        id,
        &item.content,
        &item.content_type,
        "schedule",
    )
    .await;

    scheduled_content::schedule_draft_for(&state.db, &ctx.account_id, id, &body.scheduled_for)
        .await
        .map_err(ApiError::Storage)?;

    // Log activity
    let _ = scheduled_content::insert_activity_for(
        &state.db,
        &ctx.account_id,
        id,
        "scheduled",
        Some(&json!({ "scheduled_for": body.scheduled_for }).to_string()),
    )
    .await;

    Ok(Json(json!({
        "id": id,
        "status": "scheduled",
        "scheduled_for": body.scheduled_for
    })))
}

/// `POST /api/drafts/:id/unschedule` — transition scheduled -> draft.
pub async fn unschedule_studio_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    if item.status != "scheduled" {
        return Err(ApiError::BadRequest(format!(
            "Item is in '{}' status, not 'scheduled'",
            item.status
        )));
    }

    // Create revision snapshot before unscheduling
    let _ = scheduled_content::insert_revision_for(
        &state.db,
        &ctx.account_id,
        id,
        &item.content,
        &item.content_type,
        "unschedule",
    )
    .await;

    let unscheduled = scheduled_content::unschedule_draft_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;

    if !unscheduled {
        return Err(ApiError::BadRequest(
            "Failed to unschedule — item may have changed status".to_string(),
        ));
    }

    // Log activity
    let _ =
        scheduled_content::insert_activity_for(&state.db, &ctx.account_id, id, "unscheduled", None)
            .await;

    Ok(Json(json!({ "id": id, "status": "draft" })))
}

/// `POST /api/drafts/:id/archive` — soft-delete a draft.
pub async fn archive_studio_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    scheduled_content::archive_draft_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;

    // Log activity
    let _ =
        scheduled_content::insert_activity_for(&state.db, &ctx.account_id, id, "archived", None)
            .await;

    // Fetch to get archived_at
    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;

    let archived_at = item.and_then(|i| i.archived_at).unwrap_or_default();

    Ok(Json(json!({ "id": id, "archived_at": archived_at })))
}

/// `POST /api/drafts/:id/restore` — restore an archived draft.
pub async fn restore_studio_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    scheduled_content::restore_draft_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;

    // Log activity
    let _ =
        scheduled_content::insert_activity_for(&state.db, &ctx.account_id, id, "restored", None)
            .await;

    Ok(Json(json!({ "id": id })))
}

/// `POST /api/drafts/:id/duplicate` — clone a draft.
pub async fn duplicate_studio_draft(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let new_id = scheduled_content::duplicate_draft_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    // Log created activity on the new draft
    let _ = scheduled_content::insert_activity_for(
        &state.db,
        &ctx.account_id,
        new_id,
        "created",
        Some(&json!({ "source": "duplicate", "original_id": id }).to_string()),
    )
    .await;

    Ok(Json(json!({ "id": new_id })))
}

/// `POST /api/drafts/:id/revisions/:rev_id/restore` — restore content from a revision.
///
/// Safety: snapshots the current content as a `pre_restore` revision before
/// overwriting, so restore is always non-lossy.
pub async fn restore_from_revision(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path((id, rev_id)): Path<(i64, i64)>,
) -> Result<Json<scheduled_content::ScheduledContent>, ApiError> {
    require_mutate(&ctx)?;

    // 1. Fetch current draft
    let current = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    // 2. Fetch target revision (verify ownership via account+content scope)
    let target_rev = scheduled_content::get_revision_for(&state.db, &ctx.account_id, id, rev_id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Revision {rev_id} not found")))?;

    // 3. Snapshot current state as pre_restore
    let _ = scheduled_content::insert_revision_for(
        &state.db,
        &ctx.account_id,
        id,
        &current.content,
        &current.content_type,
        "pre_restore",
    )
    .await;

    // 4. Update content to revision's content
    let _ = scheduled_content::autosave_draft_for(
        &state.db,
        &ctx.account_id,
        id,
        &target_rev.content,
        &target_rev.content_type,
        &current.updated_at,
    )
    .await
    .map_err(ApiError::Storage)?;

    // 5. Log activity
    let _ = scheduled_content::insert_activity_for(
        &state.db,
        &ctx.account_id,
        id,
        "revision_restored",
        Some(&json!({"from_revision_id": rev_id}).to_string()),
    )
    .await;

    // 6. Return updated draft
    let updated = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;
    Ok(Json(updated))
}

/// `GET /api/drafts/:id/revisions` — list revision snapshots.
pub async fn list_draft_revisions(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Vec<scheduled_content::ContentRevision>>, ApiError> {
    let revisions = scheduled_content::list_revisions_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;
    Ok(Json(revisions))
}

/// `POST /api/drafts/:id/revisions` — create a manual revision snapshot.
pub async fn create_draft_revision(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<CreateRevisionBody>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    let rev_id = scheduled_content::insert_revision_for(
        &state.db,
        &ctx.account_id,
        id,
        &item.content,
        &item.content_type,
        &body.trigger_kind,
    )
    .await
    .map_err(ApiError::Storage)?;

    Ok(Json(json!({ "id": rev_id })))
}

/// `GET /api/drafts/:id/activity` — list activity log.
pub async fn list_draft_activity(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Vec<scheduled_content::ContentActivity>>, ApiError> {
    let activity = scheduled_content::list_activity_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;
    Ok(Json(activity))
}
