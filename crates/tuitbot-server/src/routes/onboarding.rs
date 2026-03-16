//! Onboarding-specific endpoints for pre-account X sign-in and profile analysis.
//!
//! These endpoints let users authenticate with X during onboarding,
//! before any account or config exists. Tokens are stored temporarily
//! at `{data_dir}/onboarding_tokens.json` and migrated to the default
//! account's token path when `POST /api/settings/init` completes.
//!
//! - `POST /api/onboarding/x-auth/start`       — start OAuth PKCE flow
//! - `POST /api/onboarding/x-auth/callback`     — exchange code for tokens
//! - `GET  /api/onboarding/x-auth/status`       — check connection status
//! - `POST /api/onboarding/analyze-profile`     — analyze X profile for prefill

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

/// Optional request body for starting onboarding auth.
#[derive(Deserialize, Default)]
pub struct StartAuthRequest {
    /// Optional client_id override. If absent, uses the server's in-memory value.
    #[serde(default)]
    pub client_id: Option<String>,
}

/// `POST /api/onboarding/x-auth/start` — start an OAuth PKCE flow.
///
/// No account exists yet. Generates a PKCE challenge and stores it
/// under the `__onboarding__` sentinel. Returns the authorization URL.
///
/// Accepts an optional `client_id` in the request body. If absent or empty,
/// falls back to the server's in-memory `x_client_id` (useful after factory
/// reset when the client_id is still in memory).
pub async fn start_onboarding_auth(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StartAuthRequest>,
) -> Result<Json<Value>, ApiError> {
    let effective_id = body
        .client_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| state.x_client_id.clone());

    if effective_id.is_empty() {
        return Err(ApiError::BadRequest(
            "X API client_id not configured. Set x_api.client_id in config.toml.".to_string(),
        ));
    }

    // Read auth config for redirect URI.
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: tuitbot_core::config::Config = toml::from_str(&contents).unwrap_or_default();
    let redirect_uri = build_redirect_uri(&config.auth.callback_host, config.auth.callback_port);

    let pkce = generate_pkce();

    let auth_url = build_auth_url(&effective_id, &redirect_uri, &pkce.state, &pkce.challenge);

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
                client_id: effective_id,
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
    let (code_verifier, flow_client_id) = {
        let mut pending = state.pending_oauth.lock().await;
        match pending.remove(&body.state) {
            Some(p) if p.created_at.elapsed() < OAUTH_STATE_TTL => {
                if p.account_id != ONBOARDING_ACCOUNT_ID {
                    return Err(ApiError::BadRequest(
                        "state parameter does not match onboarding flow".to_string(),
                    ));
                }
                (p.code_verifier, p.client_id)
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

    // Exchange code for tokens using the client_id from the start flow.
    let stored_tokens = tuitbot_core::startup::exchange_auth_code(
        &flow_client_id,
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

/// Request body for profile analysis.
#[derive(Deserialize)]
pub struct AnalyzeProfileRequest {
    /// Optional LLM config for enrichment. If absent, heuristic-only.
    pub llm: Option<LlmConfigInput>,
}

/// LLM configuration passed from the frontend during onboarding.
#[derive(Deserialize)]
pub struct LlmConfigInput {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

/// `POST /api/onboarding/analyze-profile` — analyze X profile for onboarding prefill.
///
/// Loads the onboarding tokens, fetches the user's profile and recent tweets,
/// then runs a two-pass inference pipeline (heuristics + optional LLM) to
/// produce normalized `InferredProfile` suggestions.
pub async fn analyze_profile(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AnalyzeProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    use tuitbot_core::config::LlmConfig;
    use tuitbot_core::llm::factory::create_provider;
    use tuitbot_core::toolkit::profile_inference::{
        enrich_with_llm, extract_heuristics, ProfileInput,
    };

    // 1. Load onboarding tokens.
    let token_path = onboarding_token_path(&state.data_dir);
    if !token_path.exists() {
        return Ok(Json(json!({
            "status": "x_api_error",
            "error": "Not connected. Complete X sign-in first."
        })));
    }

    let tokens = match auth::load_tokens(&token_path) {
        Ok(Some(t)) if t.expires_at > chrono::Utc::now() => t,
        Ok(Some(_)) => {
            return Ok(Json(json!({
                "status": "x_api_error",
                "error": "X tokens expired. Please re-authenticate."
            })));
        }
        _ => {
            return Ok(Json(json!({
                "status": "x_api_error",
                "error": "Failed to load onboarding tokens."
            })));
        }
    };

    // 2. Create X API client and fetch profile + tweets.
    let client = XApiHttpClient::new(tokens.access_token.clone());

    let user = match client.get_me().await {
        Ok(u) => u,
        Err(e) => {
            return Ok(Json(json!({
                "status": "x_api_error",
                "error": format!("Failed to fetch profile: {e}")
            })));
        }
    };

    let tweets = match client.get_user_tweets(&user.id, 50, None).await {
        Ok(resp) => resp.data,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch tweets for profile analysis, continuing with profile-only");
            Vec::new()
        }
    };

    let mut warnings: Vec<String> = Vec::new();

    if tweets.is_empty() {
        warnings.push("No recent tweets found. Analysis relies on profile data only.".into());
    }

    // 3. Run heuristic extraction.
    let input = ProfileInput {
        user: user.clone(),
        tweets,
    };
    let mut profile = extract_heuristics(&input);

    // 4. Optionally enrich with LLM.
    let mut status = "partial";

    if let Some(llm_input) = body.llm {
        let llm_config = LlmConfig {
            provider: llm_input.provider,
            api_key: llm_input.api_key,
            model: llm_input.model,
            base_url: llm_input.base_url,
        };

        match create_provider(&llm_config) {
            Ok(provider) => match enrich_with_llm(profile.clone(), &input, provider.as_ref()).await
            {
                Ok(enriched) => {
                    profile = enriched;
                    status = "ok";
                }
                Err(e) => {
                    warnings.push(format!(
                        "LLM enrichment failed: {e}. Using heuristics only."
                    ));
                }
            },
            Err(e) => {
                warnings.push(format!(
                    "LLM provider configuration error: {e}. Using heuristics only."
                ));
            }
        }
    } else {
        warnings
            .push("No LLM configured. Using heuristic analysis only (limited accuracy).".into());
    }

    Ok(Json(json!({
        "status": status,
        "profile": profile,
        "warnings": warnings,
    })))
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

    #[test]
    fn user_to_json_with_none_fields() {
        let user = tuitbot_core::x_api::types::User {
            id: "456".into(),
            username: "minimal".into(),
            name: "Minimal User".into(),
            profile_image_url: None,
            description: None,
            location: None,
            url: None,
            public_metrics: Default::default(),
        };
        let json = user_to_json(&user);
        assert_eq!(json["id"], "456");
        assert_eq!(json["username"], "minimal");
        assert_eq!(json["name"], "Minimal User");
        assert!(json["profile_image_url"].is_null());
        assert!(json["description"].is_null());
        assert!(json["location"].is_null());
        assert!(json["url"].is_null());
    }

    #[test]
    fn onboarding_token_path_nested() {
        let path = onboarding_token_path(Path::new("/home/user/.tuitbot"));
        assert_eq!(
            path,
            PathBuf::from("/home/user/.tuitbot/onboarding_tokens.json")
        );
    }

    #[test]
    fn start_auth_request_default() {
        let req: StartAuthRequest = serde_json::from_str("{}").unwrap();
        assert!(req.client_id.is_none());
    }

    #[test]
    fn start_auth_request_with_client_id() {
        let req: StartAuthRequest = serde_json::from_str(r#"{"client_id": "my-id"}"#).unwrap();
        assert_eq!(req.client_id.as_deref(), Some("my-id"));
    }

    #[test]
    fn onboarding_callback_request_deserialize() {
        let req: OnboardingCallbackRequest =
            serde_json::from_str(r#"{"code": "auth_code_123", "state": "csrf_state_456"}"#)
                .unwrap();
        assert_eq!(req.code, "auth_code_123");
        assert_eq!(req.state, "csrf_state_456");
    }

    #[test]
    fn analyze_profile_request_no_llm() {
        let req: AnalyzeProfileRequest = serde_json::from_str(r#"{}"#).unwrap();
        assert!(req.llm.is_none());
    }

    #[test]
    fn analyze_profile_request_with_llm() {
        let req: AnalyzeProfileRequest = serde_json::from_str(
            r#"{"llm": {"provider": "openai", "api_key": "sk-test", "model": "gpt-4", "base_url": null}}"#,
        )
        .unwrap();
        let llm = req.llm.unwrap();
        assert_eq!(llm.provider, "openai");
        assert_eq!(llm.model, "gpt-4");
        assert_eq!(llm.api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn oauth_state_ttl_is_ten_minutes() {
        assert_eq!(OAUTH_STATE_TTL, Duration::from_secs(600));
    }

    #[test]
    fn onboarding_account_id_sentinel() {
        assert_eq!(ONBOARDING_ACCOUNT_ID, "__onboarding__");
    }
}
