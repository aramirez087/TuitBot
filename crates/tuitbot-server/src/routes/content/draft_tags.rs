//! Tag management API endpoints for Draft Studio.
//!
//! Provides routes for listing, creating, assigning, and unassigning
//! content tags on drafts.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::scheduled_content;

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateTagBody {
    pub name: String,
    pub color: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/tags` — list all tags for the current account.
pub async fn list_account_tags(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Vec<scheduled_content::ContentTag>>, ApiError> {
    let tags = scheduled_content::list_tags_for(&state.db, &ctx.account_id)
        .await
        .map_err(ApiError::Storage)?;
    Ok(Json(tags))
}

/// `POST /api/tags` — create a new tag for the current account.
pub async fn create_account_tag(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<CreateTagBody>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let id = scheduled_content::create_tag_for(
        &state.db,
        &ctx.account_id,
        &body.name,
        body.color.as_deref(),
    )
    .await
    .map_err(ApiError::Storage)?;

    Ok(Json(json!({ "id": id })))
}

/// `GET /api/drafts/:id/tags` — list tags assigned to a draft.
pub async fn list_draft_tags(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Vec<scheduled_content::ContentTag>>, ApiError> {
    let tags = scheduled_content::list_draft_tags_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?;
    Ok(Json(tags))
}

/// `POST /api/drafts/:id/tags/:tag_id` — assign a tag to a draft.
pub async fn assign_draft_tag(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path((id, tag_id)): Path<(i64, i64)>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Verify the draft exists and belongs to this account
    scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("Draft {id} not found")))?;

    scheduled_content::assign_tag_for(&state.db, id, tag_id)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(json!({ "status": "assigned" })))
}

/// `DELETE /api/drafts/:id/tags/:tag_id` — unassign a tag from a draft.
pub async fn unassign_draft_tag(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path((id, tag_id)): Path<(i64, i64)>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let removed = scheduled_content::unassign_tag_for(&state.db, id, tag_id)
        .await
        .map_err(ApiError::Storage)?;

    Ok(Json(
        json!({ "status": if removed { "removed" } else { "not_found" } }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tag_body_deserializes_with_color() {
        let json = r##"{"name": "urgent", "color": "#ff0000"}"##;
        let body: CreateTagBody = serde_json::from_str(json).expect("deser");
        assert_eq!(body.name, "urgent");
        assert_eq!(body.color.as_deref(), Some("#ff0000"));
    }

    #[test]
    fn create_tag_body_deserializes_without_color() {
        let json = r#"{"name": "backlog"}"#;
        let body: CreateTagBody = serde_json::from_str(json).expect("deser");
        assert_eq!(body.name, "backlog");
        assert!(body.color.is_none());
    }
}
