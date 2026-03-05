//! Account management endpoints.
//!
//! CRUD for the account registry, role management, and per-account
//! configuration overrides.

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::accounts::{self, UpdateAccountParams};
use tuitbot_core::x_api::{XApiClient, XApiHttpClient};

use crate::account::{require_mutate, AccountContext, Role};
use crate::error::ApiError;
use crate::state::AppState;

/// `GET /api/accounts` — list all active accounts (admin only).
pub async fn list_accounts(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let accs = accounts::list_accounts(&state.db).await?;
    Ok(Json(json!(accs)))
}

/// `GET /api/accounts/{id}` — get account details.
pub async fn get_account(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let account = accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {id}")))?;
    Ok(Json(json!(account)))
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub label: String,
}

/// `POST /api/accounts` — create a new account (admin only).
pub async fn create_account(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<CreateAccountRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let id = uuid::Uuid::new_v4().to_string();
    accounts::create_account(&state.db, &id, &body.label).await?;

    let account = accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::Internal("account creation failed".to_string()))?;

    Ok(Json(json!(account)))
}

#[derive(Deserialize)]
pub struct UpdateAccountRequest {
    pub label: Option<String>,
    pub config_overrides: Option<String>,
}

/// `PATCH /api/accounts/{id}` — update account config/label (admin only).
pub async fn update_account(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
    Json(body): Json<UpdateAccountRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Verify account exists.
    accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {id}")))?;

    accounts::update_account(
        &state.db,
        &id,
        UpdateAccountParams {
            label: body.label.as_deref(),
            config_overrides: body.config_overrides.as_deref(),
            ..Default::default()
        },
    )
    .await?;

    let updated = accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::Internal("account disappeared".to_string()))?;

    Ok(Json(json!(updated)))
}

/// `DELETE /api/accounts/{id}` — archive an account (admin only).
pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    accounts::delete_account(&state.db, &id)
        .await
        .map_err(|_| ApiError::BadRequest("cannot delete this account".to_string()))?;
    Ok(Json(json!({"status": "archived"})))
}

// ---- Role management ----

/// `GET /api/accounts/{id}/roles` — list roles for an account.
pub async fn list_roles(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let roles = accounts::list_roles(&state.db, &id).await?;
    Ok(Json(json!(roles)))
}

#[derive(Deserialize)]
pub struct SetRoleRequest {
    pub actor: String,
    pub role: String,
}

/// `POST /api/accounts/{id}/roles` — set a role for an actor on an account.
pub async fn set_role(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
    Json(body): Json<SetRoleRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Validate role string.
    let _role: Role = body
        .role
        .parse()
        .map_err(|e: String| ApiError::BadRequest(e))?;

    accounts::set_role(&state.db, &id, &body.actor, &body.role).await?;
    Ok(Json(json!({"status": "ok"})))
}

#[derive(Deserialize)]
pub struct RemoveRoleRequest {
    pub actor: String,
}

/// `DELETE /api/accounts/{id}/roles` — remove a role assignment.
pub async fn remove_role(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
    Json(body): Json<RemoveRoleRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    accounts::remove_role(&state.db, &id, &body.actor).await?;
    Ok(Json(json!({"status": "ok"})))
}

// ---- Profile sync ----

/// `POST /api/accounts/{id}/sync-profile` — fetch X profile and update account.
///
/// Loads the OAuth token for the account, calls `/users/me` on the X API,
/// and stores the display name and avatar URL on the account record.
pub async fn sync_profile(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let account = accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {id}")))?;

    // Resolve token path: account-specific or default.
    let token_path = account
        .token_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| state.data_dir.join("tokens.json"));

    let access_token = state
        .get_x_access_token(&token_path, &id)
        .await
        .map_err(|e| ApiError::Internal(format!("X API error: {e}")))?;

    let client = XApiHttpClient::new(access_token);
    let user = client
        .get_me()
        .await
        .map_err(|e| ApiError::Internal(format!("X API error: {e}")))?;

    accounts::update_account(
        &state.db,
        &id,
        UpdateAccountParams {
            x_user_id: Some(&user.id),
            x_username: Some(&user.username),
            x_display_name: Some(&user.name),
            x_avatar_url: user.profile_image_url.as_deref(),
            ..Default::default()
        },
    )
    .await?;

    let updated = accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::Internal("account disappeared".to_string()))?;

    Ok(Json(json!(updated)))
}
