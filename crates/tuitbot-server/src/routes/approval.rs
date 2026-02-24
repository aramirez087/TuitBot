//! Approval queue endpoints.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};
use tuitbot_core::storage::approval_queue;

use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::WsEvent;

/// `GET /api/approval` — list pending approval items.
pub async fn list_pending(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let items = approval_queue::get_pending(&state.db).await?;
    Ok(Json(json!(items)))
}

/// `POST /api/approval/:id/approve` — approve a queued item.
pub async fn approve_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let item = approval_queue::get_by_id(&state.db, id).await?;
    if item.is_none() {
        return Err(ApiError::NotFound(format!("approval item {id} not found")));
    }

    approval_queue::update_status(&state.db, id, "approved").await?;

    // Publish event to WebSocket clients.
    let _ = state.event_tx.send(WsEvent::ApprovalQueued {
        id,
        action_type: "approved".to_string(),
        content: "Item approved".to_string(),
    });

    Ok(Json(json!({"status": "approved", "id": id})))
}

/// `POST /api/approval/:id/reject` — reject a queued item.
pub async fn reject_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let item = approval_queue::get_by_id(&state.db, id).await?;
    if item.is_none() {
        return Err(ApiError::NotFound(format!("approval item {id} not found")));
    }

    approval_queue::update_status(&state.db, id, "rejected").await?;

    Ok(Json(json!({"status": "rejected", "id": id})))
}

/// `POST /api/approval/approve-all` — batch-approve all pending items.
pub async fn approve_all(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let items = approval_queue::get_pending(&state.db).await?;
    let count = items.len();

    for item in &items {
        approval_queue::update_status(&state.db, item.id, "approved").await?;
    }

    Ok(Json(json!({"status": "approved", "count": count})))
}
