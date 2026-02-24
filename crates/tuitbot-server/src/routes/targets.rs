//! Target accounts endpoints.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::target_accounts;

use crate::error::ApiError;
use crate::state::AppState;

/// `GET /api/targets` — list target accounts and their state.
pub async fn list_targets(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let accounts = target_accounts::get_active_target_accounts(&state.db).await?;
    Ok(Json(json!(accounts)))
}

/// Request body for adding a target account.
#[derive(Deserialize)]
pub struct AddTargetRequest {
    /// Username of the target account (without @).
    pub username: String,
}

/// `POST /api/targets` — add a new target account.
pub async fn add_target(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddTargetRequest>,
) -> Result<Json<Value>, ApiError> {
    let username = body.username.trim().trim_start_matches('@');

    if username.is_empty() {
        return Err(ApiError::BadRequest("username is required".to_string()));
    }

    // Check if already exists and active.
    if let Some(existing) =
        target_accounts::get_target_account_by_username(&state.db, username).await?
    {
        if existing.status == "active" {
            return Err(ApiError::Conflict(format!(
                "target account @{username} already exists"
            )));
        }
    }

    // Use username as a placeholder account_id; the automation runtime will
    // resolve the real X user ID when it runs target monitoring.
    target_accounts::upsert_target_account(&state.db, username, username).await?;

    Ok(Json(
        json!({"status": "added", "username": username.to_string()}),
    ))
}

/// `DELETE /api/targets/:username` — deactivate a target account.
pub async fn remove_target(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let removed = target_accounts::deactivate_target_account(&state.db, &username).await?;

    if !removed {
        return Err(ApiError::NotFound(format!(
            "active target account @{username} not found"
        )));
    }

    Ok(Json(json!({"status": "removed", "username": username})))
}
