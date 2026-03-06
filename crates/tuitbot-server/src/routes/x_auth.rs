//! X API credential-linking endpoints for per-account OAuth PKCE flows.
//!
//! Provides endpoints for initiating, completing, and checking the status
//! of X API OAuth credential linking for individual accounts.
//!
//! - `POST   /api/accounts/{id}/x-auth/start`    — start OAuth PKCE flow
//! - `POST   /api/accounts/{id}/x-auth/callback`  — exchange code for tokens
//! - `GET    /api/accounts/{id}/x-auth/status`    — check credential status
//! - `DELETE /api/accounts/{id}/x-auth/tokens`    — unlink OAuth tokens

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::startup::{build_auth_url, build_redirect_uri, generate_pkce};
use tuitbot_core::storage::accounts::{self, account_scraper_session_path, account_token_path};
use tuitbot_core::x_api::auth;

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::{AppState, PendingOAuth};

/// Maximum age for a pending OAuth state entry before it expires.
const OAUTH_STATE_TTL: Duration = Duration::from_secs(600);

/// `POST /api/accounts/{id}/x-auth/start` — start an OAuth PKCE flow.
///
/// Generates PKCE challenge, stores it keyed by state, returns the
/// authorization URL the frontend should open in a new tab/window.
pub async fn start_link(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Validate account exists.
    accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {id}")))?;

    if state.x_client_id.is_empty() {
        return Err(ApiError::BadRequest(
            "X API client_id not configured. Set x_api.client_id in config.toml.".to_string(),
        ));
    }

    // Read auth config for redirect URI.
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: tuitbot_core::config::Config = toml::from_str(&contents).unwrap_or_default();
    let redirect_uri = build_redirect_uri(&config.auth.callback_host, config.auth.callback_port);

    let pkce = generate_pkce();

    let auth_url = build_auth_url(
        &state.x_client_id,
        &redirect_uri,
        &pkce.state,
        &pkce.challenge,
    );

    // Store pending PKCE state.
    {
        let mut pending = state.pending_oauth.lock().await;
        // Clean up expired entries.
        pending.retain(|_, v| v.created_at.elapsed() < OAUTH_STATE_TTL);

        pending.insert(
            pkce.state.clone(),
            PendingOAuth {
                code_verifier: pkce.verifier,
                created_at: std::time::Instant::now(),
                account_id: id.clone(),
            },
        );
    }

    Ok(Json(json!({
        "authorization_url": auth_url,
        "state": pkce.state,
    })))
}

/// Request body for completing OAuth callback.
#[derive(Deserialize)]
pub struct CallbackRequest {
    /// The authorization code from X.
    pub code: String,
    /// The state parameter to validate the flow.
    pub state: String,
}

/// `POST /api/accounts/{id}/x-auth/callback` — exchange code for tokens.
///
/// Validates the state parameter, exchanges the authorization code for
/// tokens, converts to `auth::Tokens` format, and saves to the account's
/// token path.
pub async fn complete_link(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
    Json(body): Json<CallbackRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Look up and consume the pending PKCE state.
    let code_verifier = {
        let mut pending = state.pending_oauth.lock().await;
        match pending.remove(&body.state) {
            Some(p) if p.created_at.elapsed() < OAUTH_STATE_TTL => {
                if p.account_id != id {
                    return Err(ApiError::BadRequest(
                        "state parameter does not match this account".to_string(),
                    ));
                }
                p.code_verifier
            }
            Some(_) => {
                return Err(ApiError::BadRequest("state expired".to_string()));
            }
            None => {
                return Err(ApiError::BadRequest("invalid or expired state".to_string()));
            }
        }
    };

    // Read auth config for redirect URI.
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: tuitbot_core::config::Config = toml::from_str(&contents).unwrap_or_default();
    let redirect_uri = build_redirect_uri(&config.auth.callback_host, config.auth.callback_port);

    // Exchange code for tokens.
    let stored_tokens = tuitbot_core::startup::exchange_auth_code(
        &state.x_client_id,
        &body.code,
        &redirect_uri,
        &code_verifier,
    )
    .await
    .map_err(|e| ApiError::Internal(format!("token exchange failed: {e}")))?;

    // Convert StoredTokens → auth::Tokens.
    let tokens = auth::Tokens {
        access_token: stored_tokens.access_token,
        refresh_token: stored_tokens.refresh_token.unwrap_or_default(),
        expires_at: stored_tokens
            .expires_at
            .unwrap_or_else(|| chrono::Utc::now() + chrono::TimeDelta::hours(2)),
        scopes: stored_tokens.scopes,
    };

    // Save to account-specific token path.
    let token_path = account_token_path(&state.data_dir, &id);
    auth::save_tokens(&tokens, &token_path)
        .map_err(|e| ApiError::Internal(format!("failed to save tokens: {e}")))?;

    // Evict any existing TokenManager for this account (force reload on next use).
    {
        let mut managers = state.token_managers.lock().await;
        managers.remove(&id);
    }

    Ok(Json(json!({
        "status": "linked",
        "token_path": token_path.display().to_string(),
    })))
}

/// `DELETE /api/accounts/{id}/x-auth/tokens` — unlink OAuth tokens.
///
/// Deletes the token file for the specified account and evicts any cached
/// `TokenManager`, effectively disconnecting the OAuth credential.
pub async fn unlink(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Validate account exists.
    accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {id}")))?;

    let token_path = account_token_path(&state.data_dir, &id);

    let deleted = if token_path.exists() {
        std::fs::remove_file(&token_path)
            .map_err(|e| ApiError::Internal(format!("failed to delete tokens: {e}")))?;
        true
    } else {
        false
    };

    // Evict cached TokenManager regardless.
    {
        let mut managers = state.token_managers.lock().await;
        managers.remove(&id);
    }

    tracing::info!(account_id = %id, deleted, "OAuth tokens unlinked");

    Ok(Json(json!({
        "deleted": deleted,
    })))
}

/// `GET /api/accounts/{id}/x-auth/status` — check credential status.
///
/// Returns whether the account has linked OAuth tokens and/or a scraper
/// session, along with expiry information.
pub async fn link_status(
    State(state): State<Arc<AppState>>,
    _ctx: AccountContext,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Validate account exists.
    accounts::get_account(&state.db, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {id}")))?;

    // Check OAuth tokens.
    let token_path = account_token_path(&state.data_dir, &id);
    let (oauth_linked, oauth_expired, oauth_expires_at) = if token_path.exists() {
        match auth::load_tokens(&token_path) {
            Ok(Some(tokens)) => {
                let expired = tokens.expires_at < chrono::Utc::now();
                let expires_at = tokens.expires_at.to_rfc3339();
                (true, expired, Some(expires_at))
            }
            _ => (false, false, None),
        }
    } else {
        (false, false, None)
    };

    // Check scraper session.
    let session_path = account_scraper_session_path(&state.data_dir, &id);
    let scraper_linked = session_path.exists();

    Ok(Json(json!({
        "oauth_linked": oauth_linked,
        "oauth_expired": oauth_expired,
        "oauth_expires_at": oauth_expires_at,
        "scraper_linked": scraper_linked,
        "has_credentials": oauth_linked || scraper_linked,
    })))
}
