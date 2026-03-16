//! Scraper session endpoints for importing/managing browser cookie sessions.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use tuitbot_core::config::merge_overrides;
use tuitbot_core::storage::accounts::{self, account_scraper_session_path, DEFAULT_ACCOUNT_ID};
use tuitbot_core::x_api::ScraperSession;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::routes::settings::merge_patch_and_parse;
use crate::state::AppState;

/// Request body for importing a browser session.
#[derive(Deserialize)]
pub struct ImportSessionRequest {
    /// The `auth_token` cookie value from the browser.
    pub auth_token: String,
    /// The `ct0` cookie value (CSRF token) from the browser.
    pub ct0: String,
    /// Optional X username for display purposes.
    #[serde(default)]
    pub username: Option<String>,
}

/// `GET /api/settings/scraper-session` — check if a browser session exists.
pub async fn get_scraper_session(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let session_path = account_scraper_session_path(&state.data_dir, &ctx.account_id);
    let session = ScraperSession::load(&session_path)
        .map_err(|e| ApiError::Internal(format!("failed to read session: {e}")))?;

    match session {
        Some(s) => Ok(Json(serde_json::json!({
            "exists": true,
            "username": s.username,
            "created_at": s.created_at,
        }))),
        None => Ok(Json(serde_json::json!({
            "exists": false,
        }))),
    }
}

/// `POST /api/settings/scraper-session` — import browser cookies.
pub async fn import_scraper_session(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<ImportSessionRequest>,
) -> Result<Json<Value>, ApiError> {
    if body.auth_token.trim().is_empty() || body.ct0.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "auth_token and ct0 are required".to_string(),
        ));
    }

    let session = ScraperSession {
        auth_token: body.auth_token.trim().to_string(),
        ct0: body.ct0.trim().to_string(),
        username: body.username,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    let session_path = account_scraper_session_path(&state.data_dir, &ctx.account_id);

    // Ensure the parent directory exists for non-default accounts.
    if let Some(parent) = session_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ApiError::Internal(format!("failed to create session directory: {e}")))?;
    }

    session
        .save(&session_path)
        .map_err(|e| ApiError::Internal(format!("failed to save session: {e}")))?;

    // Ensure provider_backend is set to "scraper" so that `can_post_for()`
    // checks the correct credential file. Without this, a user who previously
    // had X API tokens configured would still have provider_backend="" and
    // `can_post_for()` would return false even though a valid session exists.
    let backend_updated = ensure_scraper_backend(&state, &ctx.account_id).await?;

    tracing::info!(
        account_id = %ctx.account_id,
        backend_updated,
        "Browser session imported successfully"
    );

    Ok(Json(serde_json::json!({
        "status": "imported",
        "username": session.username,
        "created_at": session.created_at,
        "backend_updated": backend_updated,
    })))
}

/// Set `provider_backend = "scraper"` in the config if it isn't already.
///
/// Returns `true` if the config was updated, `false` if it was already correct.
async fn ensure_scraper_backend(state: &AppState, account_id: &str) -> Result<bool, ApiError> {
    let config = state
        .load_effective_config(account_id)
        .await
        .map_err(|e| ApiError::Internal(format!("failed to load config: {e}")))?;

    if config.x_api.provider_backend == "scraper" {
        return Ok(false);
    }

    let patch = serde_json::json!({
        "x_api": { "provider_backend": "scraper" }
    });

    if account_id == DEFAULT_ACCOUNT_ID {
        let (merged_str, _config) = merge_patch_and_parse(&state.config_path, &patch)?;
        std::fs::write(&state.config_path, &merged_str).map_err(|e| {
            ApiError::Internal(format!(
                "could not write config file {}: {e}",
                state.config_path.display()
            ))
        })?;
    } else {
        let account = accounts::get_account(&state.db, account_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("account not found: {account_id}")))?;

        let new_overrides = merge_overrides(&account.config_overrides, &patch)
            .map_err(|e| ApiError::Internal(format!("override merge failed: {e}")))?;

        accounts::update_account(
            &state.db,
            account_id,
            accounts::UpdateAccountParams {
                config_overrides: Some(&new_overrides),
                ..Default::default()
            },
        )
        .await?;
    }

    Ok(true)
}

/// `DELETE /api/settings/scraper-session` — remove the browser session.
pub async fn delete_scraper_session(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let session_path = account_scraper_session_path(&state.data_dir, &ctx.account_id);
    let deleted = ScraperSession::delete(&session_path)
        .map_err(|e| ApiError::Internal(format!("failed to delete session: {e}")))?;

    Ok(Json(serde_json::json!({
        "deleted": deleted,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_session_request_deserializes() {
        let json = r#"{"auth_token":"tok123","ct0":"csrf456"}"#;
        let req: ImportSessionRequest = serde_json::from_str(json).expect("deser");
        assert_eq!(req.auth_token, "tok123");
        assert_eq!(req.ct0, "csrf456");
        assert!(req.username.is_none());
    }

    #[test]
    fn import_session_request_with_username() {
        let json = r#"{"auth_token":"tok","ct0":"ct","username":"alice"}"#;
        let req: ImportSessionRequest = serde_json::from_str(json).expect("deser");
        assert_eq!(req.username.as_deref(), Some("alice"));
    }

    #[test]
    fn import_session_request_empty_username() {
        let json = r#"{"auth_token":"tok","ct0":"ct","username":null}"#;
        let req: ImportSessionRequest = serde_json::from_str(json).expect("deser");
        assert!(req.username.is_none());
    }
}
