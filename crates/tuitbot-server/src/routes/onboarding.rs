//! Onboarding-specific OAuth endpoints for pre-account X sign-in.
//!
//! These endpoints let users authenticate with X during onboarding,
//! before any account or config exists. Tokens are stored temporarily
//! at `{data_dir}/onboarding_tokens.json` and migrated to the default
//! account's token path when `POST /api/settings/init` completes.
//!
//! - `POST /api/onboarding/x-auth/start`    — start OAuth PKCE flow
//! - `POST /api/onboarding/x-auth/callback`  — exchange code for tokens
//! - `GET  /api/onboarding/x-auth/status`    — check connection status

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::startup::{build_auth_url, build_redirect_uri, generate_pkce};
use tuitbot_core::x_api::auth;
use tuitbot_core::x_api::client::XApiHttpClient;
use tuitbot_core::x_api::XApiClient;

use crate::error::ApiError;
use crate::state::{AppState, PendingOAuth};

/// Sentinel account ID used for onboarding PKCE entries.
const ONBOARDING_ACCOUNT_ID: &str = "__onboarding__";

/// Maximum age for a pending OAuth state entry before it expires.
const OAUTH_STATE_TTL: Duration = Duration::from_secs(600);

/// Return the path for temporary onboarding tokens.
pub fn onboarding_token_path(data_dir: &Path) -> PathBuf {
    data_dir.join("onboarding_tokens.json")
}

/// Serialize the X user profile fields we care about during onboarding.
fn user_to_json(user: &tuitbot_core::x_api::types::User) -> Value {
    json!({
        "id": user.id,
        "username": user.username,
        "name": user.name,
        "profile_image_url": user.profile_image_url,
        "description": user.description,
        "location": user.location,
        "url": user.url,
    })
}

/// `POST /api/onboarding/x-auth/start` — start an OAuth PKCE flow.
///
/// No account exists yet. Generates a PKCE challenge and stores it
/// under the `__onboarding__` sentinel. Returns the authorization URL.
pub async fn start_onboarding_auth(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
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

    // Store pending PKCE state with onboarding sentinel.
    {
        let mut pending = state.pending_oauth.lock().await;
        // Clean up expired entries.
        pending.retain(|_, v| v.created_at.elapsed() < OAUTH_STATE_TTL);

        pending.insert(
            pkce.state.clone(),
            PendingOAuth {
                code_verifier: pkce.verifier,
                created_at: std::time::Instant::now(),
                account_id: ONBOARDING_ACCOUNT_ID.to_string(),
            },
        );
    }

    Ok(Json(json!({
        "authorization_url": auth_url,
        "state": pkce.state,
    })))
}

/// Request body for the onboarding OAuth callback.
#[derive(Deserialize)]
pub struct OnboardingCallbackRequest {
    /// The authorization code from X.
    pub code: String,
    /// The state parameter to validate the flow.
    pub state: String,
}

/// `POST /api/onboarding/x-auth/callback` — exchange code for tokens.
///
/// Validates the state parameter against the `__onboarding__` sentinel,
/// exchanges the code for tokens, saves them to `onboarding_tokens.json`,
/// then calls `get_me()` to fetch the authenticated user's profile.
pub async fn complete_onboarding_auth(
    State(state): State<Arc<AppState>>,
    Json(body): Json<OnboardingCallbackRequest>,
) -> Result<Json<Value>, ApiError> {
    // Look up and consume the pending PKCE state.
    let code_verifier = {
        let mut pending = state.pending_oauth.lock().await;
        match pending.remove(&body.state) {
            Some(p) if p.created_at.elapsed() < OAUTH_STATE_TTL => {
                if p.account_id != ONBOARDING_ACCOUNT_ID {
                    return Err(ApiError::BadRequest(
                        "state parameter does not match onboarding flow".to_string(),
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

    // Save to temporary onboarding path.
    let token_path = onboarding_token_path(&state.data_dir);
    auth::save_tokens(&tokens, &token_path)
        .map_err(|e| ApiError::Internal(format!("failed to save onboarding tokens: {e}")))?;

    // Fetch user identity using the new access token.
    let client = XApiHttpClient::new(tokens.access_token.clone());
    let user = client
        .get_me()
        .await
        .map_err(|e| ApiError::Internal(format!("failed to fetch user profile after auth: {e}")))?;

    Ok(Json(json!({
        "status": "connected",
        "user": user_to_json(&user),
    })))
}

/// `GET /api/onboarding/x-auth/status` — check onboarding auth status.
///
/// Returns whether `onboarding_tokens.json` exists with valid tokens,
/// and if so, fetches the authenticated user's profile.
pub async fn onboarding_auth_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let token_path = onboarding_token_path(&state.data_dir);

    if !token_path.exists() {
        return Ok(Json(json!({ "connected": false })));
    }

    // Load tokens and check validity.
    let tokens = match auth::load_tokens(&token_path) {
        Ok(Some(t)) if t.expires_at > chrono::Utc::now() => t,
        _ => {
            return Ok(Json(json!({ "connected": false })));
        }
    };

    // Fetch user identity.
    let client = XApiHttpClient::new(tokens.access_token.clone());
    match client.get_me().await {
        Ok(user) => Ok(Json(json!({
            "connected": true,
            "user": user_to_json(&user),
        }))),
        Err(_) => Ok(Json(json!({ "connected": false }))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onboarding_token_path_is_in_data_dir() {
        let path = onboarding_token_path(Path::new("/data"));
        assert_eq!(path, PathBuf::from("/data/onboarding_tokens.json"));
    }

    #[test]
    fn user_to_json_includes_profile_fields() {
        let user = tuitbot_core::x_api::types::User {
            id: "123".into(),
            username: "test".into(),
            name: "Test".into(),
            profile_image_url: Some("https://img.example.com/a.jpg".into()),
            description: Some("A bio".into()),
            location: Some("NYC".into()),
            url: Some("https://example.com".into()),
            public_metrics: Default::default(),
        };
        let json = user_to_json(&user);
        assert_eq!(json["username"], "test");
        assert_eq!(json["description"], "A bio");
        assert_eq!(json["location"], "NYC");
        assert_eq!(json["url"], "https://example.com");
    }
}
