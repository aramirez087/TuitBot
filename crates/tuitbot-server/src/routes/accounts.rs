//! Account management endpoints.
//!
//! CRUD for the account registry, role management, and per-account
//! configuration overrides.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::config::{effective_config, validate_override_keys, Config};
use tuitbot_core::storage::accounts::{
    self, account_scraper_session_path, account_token_path, UpdateAccountParams, DEFAULT_ACCOUNT_ID,
};
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

    // Migrate credentials from the default account when this is the first
    // non-default account.  This handles the common onboarding path where the
    // user configures a browser session on the default account and then creates
    // a named account — without this, the session would be orphaned.
    migrate_default_credentials(&state, &id).await;

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

/// Migrate credential files from the default account to a newly created account.
///
/// Only runs when the default account has credential files (scraper session
/// and/or OAuth tokens) and there are no other non-default active accounts —
/// i.e. this is the user's first named account.  Files are *moved* so the
/// default account no longer shows stale "Linked" status.
async fn migrate_default_credentials(state: &AppState, new_account_id: &str) {
    // Only migrate when this is the first non-default account.
    let active = match accounts::list_accounts(&state.db).await {
        Ok(list) => list,
        Err(_) => return,
    };
    let non_default_count = active.iter().filter(|a| a.id != DEFAULT_ACCOUNT_ID).count();
    if non_default_count != 1 {
        return;
    }

    let default_session = account_scraper_session_path(&state.data_dir, DEFAULT_ACCOUNT_ID);
    let default_tokens = account_token_path(&state.data_dir, DEFAULT_ACCOUNT_ID);

    let has_session = default_session.exists();
    let has_tokens = default_tokens.exists();

    if !has_session && !has_tokens {
        return;
    }

    let new_dir = state.data_dir.join("accounts").join(new_account_id);
    if let Err(e) = std::fs::create_dir_all(&new_dir) {
        tracing::warn!("failed to create account dir for migration: {e}");
        return;
    }

    if has_session {
        let dest = account_scraper_session_path(&state.data_dir, new_account_id);
        if let Err(e) = std::fs::rename(&default_session, &dest) {
            tracing::warn!("failed to migrate scraper session: {e}");
        } else {
            tracing::info!(
                account_id = %new_account_id,
                "migrated scraper session from default account"
            );
        }
    }

    if has_tokens {
        let dest = account_token_path(&state.data_dir, new_account_id);
        if let Err(e) = std::fs::rename(&default_tokens, &dest) {
            tracing::warn!("failed to migrate OAuth tokens: {e}");
        } else {
            tracing::info!(
                account_id = %new_account_id,
                "migrated OAuth tokens from default account"
            );
        }
    }
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

    let _account = accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {id}")))?;

    let token_path = account_token_path(&state.data_dir, &id);

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
