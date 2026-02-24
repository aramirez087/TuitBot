//! Local bearer-token authentication for the tuitbot API server.
//!
//! On first start, generates a random 256-bit token (hex-encoded) and writes it
//! to `~/.tuitbot/api_token`. All `/api/*` routes (except `/api/health`) require
//! `Authorization: Bearer <token>`.

use std::path::Path;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use rand::RngCore;
use serde_json::json;

use std::sync::Arc;

use crate::state::AppState;

/// Ensure the API token file exists, creating one if needed.
///
/// Returns the token string. The file is written with restrictive permissions
/// so only the current user can read it.
pub fn ensure_api_token(config_dir: &Path) -> anyhow::Result<String> {
    let token_path = config_dir.join("api_token");

    if token_path.exists() {
        let token = std::fs::read_to_string(&token_path)?.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // Generate a random 256-bit (32-byte) token and hex-encode it.
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = hex::encode(bytes);

    // Ensure the directory exists.
    std::fs::create_dir_all(config_dir)?;

    std::fs::write(&token_path, &token)?;

    // Set file permissions to 0600 (owner read/write only) on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600))?;
    }

    tracing::info!(path = %token_path.display(), "Generated new API token");

    Ok(token)
}

/// Axum middleware that enforces bearer-token authentication.
///
/// Skips authentication for the `/api/health` endpoint.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Skip auth for health and onboarding endpoints.
    // Inside a nested router (`/api`), the path may appear with or without the `/api` prefix.
    let path = request.uri().path();
    if path == "/health"
        || path == "/api/health"
        || path == "/settings/status"
        || path == "/api/settings/status"
        || path == "/settings/init"
        || path == "/api/settings/init"
        || path == "/ws"
        || path == "/api/ws"
    {
        return next.run(request).await;
    }

    let authorized = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|token| token == state.api_token);

    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({"error": "unauthorized"})),
        )
            .into_response();
    }

    next.run(request).await
}
