//! CRUD handlers: list, get, create, autosave, patch, schedule.

use super::*;

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

    // Validate and normalize the scheduled time
    let normalized = tuitbot_core::scheduling::validate_and_normalize(
        &body.scheduled_for,
        tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
    )
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

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

    scheduled_content::schedule_draft_for(&state.db, &ctx.account_id, id, &normalized)
        .await
        .map_err(ApiError::Storage)?;

    // Log activity
    let _ = scheduled_content::insert_activity_for(
        &state.db,
        &ctx.account_id,
        id,
        "scheduled",
        Some(&json!({ "scheduled_for": normalized }).to_string()),
    )
    .await;

    Ok(Json(json!({
        "id": id,
        "status": "scheduled",
        "scheduled_for": normalized
    })))
}
