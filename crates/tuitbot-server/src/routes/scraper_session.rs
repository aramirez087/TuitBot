//! Scraper session endpoints for importing/managing browser cookie sessions.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use tuitbot_core::x_api::ScraperSession;

use crate::error::ApiError;
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
) -> Result<Json<Value>, ApiError> {
    let session_path = state.data_dir.join("scraper_session.json");
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

    let session_path = state.data_dir.join("scraper_session.json");
    session
        .save(&session_path)
        .map_err(|e| ApiError::Internal(format!("failed to save session: {e}")))?;

    tracing::info!("Browser session imported successfully");

    Ok(Json(serde_json::json!({
        "status": "imported",
        "username": session.username,
        "created_at": session.created_at,
    })))
}

/// `DELETE /api/settings/scraper-session` — remove the browser session.
pub async fn delete_scraper_session(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let session_path = state.data_dir.join("scraper_session.json");
    let deleted = ScraperSession::delete(&session_path)
        .map_err(|e| ApiError::Internal(format!("failed to delete session: {e}")))?;

    Ok(Json(serde_json::json!({
        "deleted": deleted,
    })))
}
