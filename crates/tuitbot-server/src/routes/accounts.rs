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
use tuitbot_core::config::{effective_config, validate_override_keys, Config};
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
///
/// Sets `token_path` to `accounts/{id}/tokens.json` so each account
/// has an isolated credential file.
pub async fn create_account(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<CreateAccountRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let id = uuid::Uuid::new_v4().to_string();
    accounts::create_account(&state.db, &id, &body.label).await?;

    // Set token_path for credential isolation.
    let token_path = format!("accounts/{}/tokens.json", id);
    accounts::update_account(
        &state.db,
        &id,
        UpdateAccountParams {
            token_path: Some(&token_path),
            ..Default::default()
        },
    )
    .await?;

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
///
/// When `config_overrides` is provided, validates that:
/// 1. The JSON only contains account-scoped keys.
/// 2. Merging with the base config produces a valid effective config.
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

    // Validate config_overrides if provided.
    if let Some(ref overrides_str) = body.config_overrides {
        let trimmed = overrides_str.trim();
        if !trimmed.is_empty() && trimmed != "{}" {
            let overrides: serde_json::Value = serde_json::from_str(trimmed)
                .map_err(|e| ApiError::BadRequest(format!("invalid config_overrides JSON: {e}")))?;

            validate_override_keys(&overrides).map_err(|e| ApiError::BadRequest(e.to_string()))?;

            // Validate the effective config by merging with base.
            let base_config = load_base_config(&state.config_path)?;
            effective_config(&base_config, trimmed)
                .map_err(|e| ApiError::BadRequest(format!("invalid effective config: {e}")))?;
        }
    }

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

/// Load and parse the base config from the TOML file.
fn load_base_config(config_path: &std::path::Path) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            config_path.display()
        ))
    })?;

    toml::from_str(&contents)
        .map_err(|e| ApiError::BadRequest(format!("failed to parse config: {e}")))
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
