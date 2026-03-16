//! Scheduled content management endpoints.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::scheduled_content;

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;

/// Request body for editing a scheduled content item.
#[derive(Deserialize)]
pub struct EditScheduledRequest {
    /// Updated content text.
    pub content: Option<String>,
    /// Updated scheduled time.
    pub scheduled_for: Option<String>,
}

/// `PATCH /api/content/scheduled/{id}` — edit a scheduled content item.
pub async fn edit_scheduled(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<EditScheduledRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("scheduled content {id} not found")))?;

    if item.status != "scheduled" {
        return Err(ApiError::BadRequest(
            "can only edit items with status 'scheduled'".to_string(),
        ));
    }

    let new_content = body.content.as_deref().unwrap_or(&item.content);
    let validated_time = match &body.scheduled_for {
        Some(raw) => Some(
            tuitbot_core::scheduling::validate_and_normalize(
                raw,
                tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
            )
            .map_err(|e| ApiError::BadRequest(e.to_string()))?,
        ),
        None => None,
    };
    let new_scheduled_for = match &validated_time {
        Some(t) => Some(t.as_str()),
        None => item.scheduled_for.as_deref(),
    };

    scheduled_content::update_content_for(
        &state.db,
        &ctx.account_id,
        id,
        new_content,
        new_scheduled_for,
    )
    .await?;

    let updated = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("scheduled content {id} not found")))?;

    Ok(Json(json!(updated)))
}

/// `DELETE /api/content/scheduled/{id}` — cancel a scheduled content item.
pub async fn cancel_scheduled(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let item = scheduled_content::get_by_id_for(&state.db, &ctx.account_id, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("scheduled content {id} not found")))?;

    if item.status != "scheduled" {
        return Err(ApiError::BadRequest(
            "can only cancel items with status 'scheduled'".to_string(),
        ));
    }

    scheduled_content::cancel_for(&state.db, &ctx.account_id, id).await?;

    Ok(Json(json!({
        "status": "cancelled",
        "id": id,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_scheduled_request_all_none() {
        let json = "{}";
        let req: EditScheduledRequest = serde_json::from_str(json).expect("deser");
        assert!(req.content.is_none());
        assert!(req.scheduled_for.is_none());
    }

    #[test]
    fn edit_scheduled_request_with_content() {
        let json = r#"{"content": "updated text"}"#;
        let req: EditScheduledRequest = serde_json::from_str(json).expect("deser");
        assert_eq!(req.content.as_deref(), Some("updated text"));
        assert!(req.scheduled_for.is_none());
    }

    #[test]
    fn edit_scheduled_request_with_both() {
        let json = r#"{"content": "new", "scheduled_for": "2026-03-16T12:00:00Z"}"#;
        let req: EditScheduledRequest = serde_json::from_str(json).expect("deser");
        assert_eq!(req.content.as_deref(), Some("new"));
        assert_eq!(req.scheduled_for.as_deref(), Some("2026-03-16T12:00:00Z"));
    }
}
