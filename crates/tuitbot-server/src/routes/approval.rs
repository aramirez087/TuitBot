//! Approval queue endpoints.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::approval_queue;

use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::WsEvent;

/// Query parameters for listing approval items.
#[derive(Deserialize)]
pub struct ApprovalQuery {
    /// Comma-separated status values (default: "pending").
    #[serde(default = "default_status")]
    pub status: String,
    /// Filter by action type (reply, tweet, thread_tweet).
    #[serde(rename = "type")]
    pub action_type: Option<String>,
}

fn default_status() -> String {
    "pending".to_string()
}

/// `GET /api/approval` — list approval items with optional status/type filters.
pub async fn list_items(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ApprovalQuery>,
) -> Result<Json<Value>, ApiError> {
    let statuses: Vec<&str> = params.status.split(',').map(|s| s.trim()).collect();
    let action_type = params.action_type.as_deref();

    let items = approval_queue::get_by_statuses(&state.db, &statuses, action_type).await?;
    Ok(Json(json!(items)))
}

/// `GET /api/approval/stats` — counts by status.
pub async fn stats(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let stats = approval_queue::get_stats(&state.db).await?;
    Ok(Json(json!(stats)))
}

/// Request body for editing approval item content.
#[derive(Deserialize)]
pub struct EditContentRequest {
    pub content: String,
}

/// `PATCH /api/approval/:id` — edit content before approving.
pub async fn edit_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<EditContentRequest>,
) -> Result<Json<Value>, ApiError> {
    let item = approval_queue::get_by_id(&state.db, id).await?;
    if item.is_none() {
        return Err(ApiError::NotFound(format!("approval item {id} not found")));
    }

    let content = body.content.trim();
    if content.is_empty() {
        return Err(ApiError::BadRequest("content cannot be empty".to_string()));
    }

    approval_queue::update_content(&state.db, id, content).await?;

    let updated = approval_queue::get_by_id(&state.db, id)
        .await?
        .expect("item was just verified to exist");
    Ok(Json(json!(updated)))
}

/// `POST /api/approval/:id/approve` — approve a queued item.
pub async fn approve_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let item = approval_queue::get_by_id(&state.db, id).await?;
    let item = item.ok_or_else(|| ApiError::NotFound(format!("approval item {id} not found")))?;

    approval_queue::update_status(&state.db, id, "approved").await?;

    let _ = state.event_tx.send(WsEvent::ApprovalUpdated {
        id,
        status: "approved".to_string(),
        action_type: item.action_type,
    });

    Ok(Json(json!({"status": "approved", "id": id})))
}

/// `POST /api/approval/:id/reject` — reject a queued item.
pub async fn reject_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let item = approval_queue::get_by_id(&state.db, id).await?;
    let item = item.ok_or_else(|| ApiError::NotFound(format!("approval item {id} not found")))?;

    approval_queue::update_status(&state.db, id, "rejected").await?;

    let _ = state.event_tx.send(WsEvent::ApprovalUpdated {
        id,
        status: "rejected".to_string(),
        action_type: item.action_type,
    });

    Ok(Json(json!({"status": "rejected", "id": id})))
}

/// `POST /api/approval/approve-all` — batch-approve all pending items.
pub async fn approve_all(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let items = approval_queue::get_pending(&state.db).await?;
    let count = items.len();

    for item in &items {
        approval_queue::update_status(&state.db, item.id, "approved").await?;
    }

    let _ = state.event_tx.send(WsEvent::ApprovalUpdated {
        id: 0,
        status: "approved_all".to_string(),
        action_type: String::new(),
    });

    Ok(Json(json!({"status": "approved", "count": count})))
}
