//! Route handlers: onboarding, config read/write/validate, and factory reset.
//!
//! All handlers delegate config I/O to `validation` helpers; no business logic
//! lives in Axum extractors directly.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;
use tuitbot_core::auth::{passphrase, session};
use tuitbot_core::config::{effective_config, merge_overrides, split_patch_by_scope, Config};
use tuitbot_core::storage::accounts::{self, DEFAULT_ACCOUNT_ID};

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

use super::validation::{
    json_to_toml, load_base_config, merge_patch_and_parse, redact_service_account_keys,
};
use super::{
    config_errors_to_response, ClaimRequest, ValidationErrorItem, ValidationResponse, XProfileData,
};

// ---------------------------------------------------------------------------
// Onboarding endpoints (no auth required)
// ---------------------------------------------------------------------------

/// `GET /api/settings/status` — check if config exists.
///
/// Also returns `deployment_mode` and `capabilities` so unauthenticated
/// pages (e.g. onboarding) can adapt their source-type UI.
pub async fn config_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let configured = state.config_path.exists();
    let claimed = passphrase::is_claimed(&state.data_dir);
    let capabilities = state.deployment_mode.capabilities();

    // Compute capability tier from config if it exists.
    let capability_tier = if configured {
        Config::load(Some(&state.config_path.to_string_lossy()))
            .map(|config| tuitbot_core::config::compute_tier(&config, false))
            .unwrap_or(tuitbot_core::config::CapabilityTier::Unconfigured)
    } else {
        tuitbot_core::config::CapabilityTier::Unconfigured
    };

    Json(serde_json::json!({
        "configured": configured,
        "claimed": claimed,
        "deployment_mode": state.deployment_mode,
        "capabilities": capabilities,
        "capability_tier": capability_tier,
        "has_x_client_id": !state.x_client_id.is_empty(),
    }))
}

/// `POST /api/settings/init` — create initial config from JSON.
///
/// Accepts the full configuration as JSON, validates it, converts to TOML,
/// and writes to `config_path`. Returns 409 if config already exists.
///
/// Optionally accepts a `claim` object containing a passphrase to establish
/// the instance passphrase and return a session cookie in one atomic step.
pub async fn init_settings(
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, ApiError> {
    if state.config_path.exists() {
        return Err(ApiError::Conflict(
            "configuration already exists; use PATCH /api/settings to update".to_string(),
        ));
    }

    if !body.is_object() {
        return Err(ApiError::BadRequest(
            "request body must be a JSON object".to_string(),
        ));
    }

    // Extract and remove `claim` before TOML conversion (it's not a config field).
    let claim: Option<ClaimRequest> = body
        .as_object_mut()
        .and_then(|obj| obj.remove("claim"))
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("invalid claim object: {e}")))?;

    // Extract and remove `x_profile` before TOML conversion (it's not a config field).
    let x_profile: Option<XProfileData> = body
        .as_object_mut()
        .and_then(|obj| obj.remove("x_profile"))
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("invalid x_profile object: {e}")))?;

    // Validate claim early — before any file I/O.
    if let Some(ref claim) = claim {
        if claim.passphrase.len() < 8 {
            return Err(ApiError::BadRequest(
                "passphrase must be at least 8 characters".into(),
            ));
        }
        if passphrase::is_claimed(&state.data_dir) {
            return Err(ApiError::Conflict("instance already claimed".into()));
        }
    }

    // Inject server's in-memory x_client_id if the frontend didn't provide one.
    // This covers the post-reset case where the user logged in via the server's
    // remembered client_id but never typed one into the form.
    if !state.x_client_id.is_empty() {
        if let Some(obj) = body.as_object_mut() {
            let x_api = obj.entry("x_api").or_insert_with(|| serde_json::json!({}));
            if let Some(x_api_obj) = x_api.as_object_mut() {
                let current = x_api_obj
                    .get("client_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if current.is_empty() {
                    x_api_obj.insert(
                        "client_id".to_string(),
                        serde_json::Value::String(state.x_client_id.clone()),
                    );
                }
            }
        }
    }

    // Convert JSON to TOML.
    let toml_value = json_to_toml(&body)
        .map_err(|e| ApiError::BadRequest(format!("invalid config values: {e}")))?;

    let toml_str = toml::to_string_pretty(&toml_value)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;

    // Validate by parsing through Config.
    let config: Config = toml::from_str(&toml_str)
        .map_err(|e| ApiError::BadRequest(format!("invalid config: {e}")))?;

    if let Err(errors) = config.validate_minimum() {
        let items = config_errors_to_response(errors);
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "validation_failed",
                "errors": items
            })),
        )
            .into_response());
    }

    // Ensure parent directory exists and write.
    if let Some(parent) = state.config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ApiError::BadRequest(format!("failed to create config directory: {e}")))?;
    }

    std::fs::write(&state.config_path, &toml_str).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not write config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    // Set file permissions to 0600 on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ =
            std::fs::set_permissions(&state.config_path, std::fs::Permissions::from_mode(0o600));
    }

    // Migrate onboarding tokens to default account token path if they exist.
    let onboarding_path = state.data_dir.join("onboarding_tokens.json");
    if onboarding_path.exists() {
        let target = accounts::account_token_path(&state.data_dir, DEFAULT_ACCOUNT_ID);
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::rename(&onboarding_path, &target) {
            tracing::warn!("Failed to migrate onboarding tokens: {e}");
        } else {
            tracing::info!("Migrated onboarding tokens to default account");
        }
    }

    // Populate the default account's X profile if provided.
    if let Some(ref profile) = x_profile {
        if let Err(e) = accounts::update_account(
            &state.db,
            DEFAULT_ACCOUNT_ID,
            accounts::UpdateAccountParams {
                x_user_id: Some(&profile.x_user_id),
                x_username: Some(&profile.x_username),
                x_display_name: Some(&profile.x_display_name),
                x_avatar_url: profile.x_avatar_url.as_deref(),
                ..Default::default()
            },
        )
        .await
        {
            tracing::warn!("Failed to set X profile on default account during init: {e}");
            // Non-fatal — syncCurrentProfile() will catch this later.
        }
    }

    let json = serde_json::to_value(&config)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;

    // If claim present, create passphrase hash + session.
    if let Some(claim) = claim {
        passphrase::create_passphrase_hash(&state.data_dir, &claim.passphrase).map_err(
            |e| match e {
                tuitbot_core::auth::error::AuthError::AlreadyClaimed => {
                    ApiError::Conflict("instance already claimed".into())
                }
                other => ApiError::Internal(format!("failed to create passphrase: {other}")),
            },
        )?;

        let new_hash = passphrase::load_passphrase_hash(&state.data_dir)
            .map_err(|e| ApiError::Internal(format!("failed to load passphrase hash: {e}")))?;
        {
            let mut hash = state.passphrase_hash.write().await;
            *hash = new_hash;
        }
        {
            let mut mtime = state.passphrase_hash_mtime.write().await;
            *mtime = passphrase::passphrase_hash_mtime(&state.data_dir);
        }

        let new_session = session::create_session(&state.db)
            .await
            .map_err(|e| ApiError::Internal(format!("failed to create session: {e}")))?;

        let cookie = format!(
            "tuitbot_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
            new_session.raw_token,
            session::SESSION_LIFETIME_DAYS * 24 * 60 * 60,
        );

        tracing::info!("instance claimed via /settings/init");

        return Ok((
            StatusCode::OK,
            [(axum::http::header::SET_COOKIE, cookie)],
            Json(serde_json::json!({
                "status": "created",
                "config": json,
                "csrf_token": new_session.csrf_token,
            })),
        )
            .into_response());
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "created",
            "config": json
        })),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Config endpoints
// ---------------------------------------------------------------------------

/// `GET /api/settings` — return the current config as JSON.
///
/// For the default account, returns the raw `config.toml` (existing behavior).
/// For non-default accounts, merges the account's `config_overrides` into
/// the base config and returns the effective result with `_overrides` metadata.
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let base_config = load_base_config(&state.config_path)?;

    if ctx.account_id == DEFAULT_ACCOUNT_ID {
        let mut json = serde_json::to_value(base_config)
            .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;
        redact_service_account_keys(&mut json);
        return Ok(Json(json));
    }

    let account = accounts::get_account(&state.db, &ctx.account_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {}", ctx.account_id)))?;

    let result = effective_config(&base_config, &account.config_overrides)
        .map_err(|e| ApiError::BadRequest(format!("config merge failed: {e}")))?;

    let mut json = serde_json::to_value(&result.config)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;
    redact_service_account_keys(&mut json);

    Ok(Json(serde_json::json!({
        "config": json,
        "_overrides": result.overridden_keys,
    })))
}

/// `PATCH /api/settings` — merge partial JSON into the config and write back.
///
/// For the default account, writes to `config.toml` (existing behavior).
/// For non-default accounts, enforces scope contract: only account-scoped
/// keys are allowed, and changes persist to `accounts.config_overrides`
/// instead of `config.toml`.
pub async fn patch_settings(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(patch): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    if !patch.is_object() {
        return Err(ApiError::BadRequest(
            "request body must be a JSON object".to_string(),
        ));
    }

    if ctx.account_id == DEFAULT_ACCOUNT_ID {
        let (merged_str, config) = merge_patch_and_parse(&state.config_path, &patch)?;

        std::fs::write(&state.config_path, &merged_str).map_err(|e| {
            ApiError::BadRequest(format!(
                "could not write config file {}: {e}",
                state.config_path.display()
            ))
        })?;

        let touches_sources =
            patch.get("content_sources").is_some() || patch.get("deployment_mode").is_some();
        if touches_sources {
            state.restart_watchtower().await;
        }

        let mut json = serde_json::to_value(config)
            .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;
        redact_service_account_keys(&mut json);
        return Ok(Json(json));
    }

    let (_account_patch, rejected) = split_patch_by_scope(&patch);
    if !rejected.is_empty() {
        return Err(ApiError::Forbidden(format!(
            "instance-scoped keys cannot be changed per-account: {}",
            rejected.join(", ")
        )));
    }

    let account = accounts::get_account(&state.db, &ctx.account_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {}", ctx.account_id)))?;

    let new_overrides = merge_overrides(&account.config_overrides, &patch)
        .map_err(|e| ApiError::BadRequest(format!("override merge failed: {e}")))?;

    let base_config = load_base_config(&state.config_path)?;
    let result = effective_config(&base_config, &new_overrides)
        .map_err(|e| ApiError::BadRequest(format!("effective config invalid: {e}")))?;

    accounts::update_account(
        &state.db,
        &ctx.account_id,
        accounts::UpdateAccountParams {
            config_overrides: Some(&new_overrides),
            ..Default::default()
        },
    )
    .await?;

    let mut json = serde_json::to_value(&result.config)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;
    redact_service_account_keys(&mut json);

    Ok(Json(serde_json::json!({
        "config": json,
        "_overrides": result.overridden_keys,
    })))
}

/// `POST /api/settings/validate` — validate a config change without saving.
pub async fn validate_settings(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(patch): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    if !patch.is_object() {
        return Err(ApiError::BadRequest(
            "request body must be a JSON object".to_string(),
        ));
    }

    if ctx.account_id == DEFAULT_ACCOUNT_ID {
        let (_merged_str, config) = merge_patch_and_parse(&state.config_path, &patch)?;

        let response = match config.validate() {
            Ok(()) => ValidationResponse {
                valid: true,
                errors: Vec::new(),
            },
            Err(errors) => ValidationResponse {
                valid: false,
                errors: config_errors_to_response(errors),
            },
        };

        return Ok(Json(serde_json::to_value(response).unwrap()));
    }

    let (_account_patch, rejected) = split_patch_by_scope(&patch);
    if !rejected.is_empty() {
        return Ok(Json(
            serde_json::to_value(ValidationResponse {
                valid: false,
                errors: vec![ValidationErrorItem {
                    field: "config_overrides".to_string(),
                    message: format!(
                        "instance-scoped keys cannot be changed per-account: {}",
                        rejected.join(", ")
                    ),
                }],
            })
            .unwrap(),
        ));
    }

    let account = accounts::get_account(&state.db, &ctx.account_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {}", ctx.account_id)))?;

    let new_overrides = merge_overrides(&account.config_overrides, &patch)
        .map_err(|e| ApiError::BadRequest(format!("override merge failed: {e}")))?;

    let base_config = load_base_config(&state.config_path)?;
    let result = effective_config(&base_config, &new_overrides)
        .map_err(|e| ApiError::BadRequest(format!("effective config invalid: {e}")))?;

    let response = match result.config.validate() {
        Ok(()) => ValidationResponse {
            valid: true,
            errors: Vec::new(),
        },
        Err(errors) => ValidationResponse {
            valid: false,
            errors: config_errors_to_response(errors),
        },
    };

    Ok(Json(serde_json::to_value(response).unwrap()))
}
