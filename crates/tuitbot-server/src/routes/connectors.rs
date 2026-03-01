//! Connector management endpoints for remote source linking.
//!
//! Provides endpoints for starting, completing, inspecting, and
//! disconnecting OAuth-based remote connections (e.g. Google Drive).
//!
//! - `POST /api/connectors/google-drive/link` -- start link flow (auth required)
//! - `GET  /api/connectors/google-drive/callback` -- OAuth callback (auth-exempt)
//! - `GET  /api/connectors/google-drive/status` -- connection status (auth required)
//! - `DELETE /api/connectors/google-drive/{id}` -- disconnect (auth required)

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::state::{AppState, PendingOAuth};

/// Maximum age for a pending OAuth state entry before it expires.
const OAUTH_STATE_TTL: Duration = Duration::from_secs(600); // 10 minutes

// ---------------------------------------------------------------------------
// POST /api/connectors/google-drive/link
// ---------------------------------------------------------------------------

/// Start a Google Drive OAuth link flow.
///
/// Generates PKCE challenge + state, stores them in memory, and returns
/// the authorization URL the client should redirect the user to.
pub async fn link_google_drive(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LinkParams>,
) -> Response {
    // Load connector config from disk.
    let config = match load_connector_config(&state) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Build the connector (validates client_id/secret are set).
    let connector =
        match tuitbot_core::source::connector::google_drive::GoogleDriveConnector::new(&config) {
            Ok(c) => c,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        };

    // Check for existing active connection.
    let existing =
        tuitbot_core::storage::watchtower::get_connections_by_type(&state.db, "google_drive").await;

    if let Ok(ref conns) = existing {
        if !conns.is_empty() && !params.force.unwrap_or(false) {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "an active Google Drive connection already exists",
                    "hint": "disconnect first or pass ?force=true"
                })),
            )
                .into_response();
        }
    }

    // Generate PKCE code_verifier (64 random bytes, hex-encoded = 128 chars).
    let code_verifier = hex::encode(random_bytes(64));

    // Compute code_challenge = BASE64URL(SHA256(code_verifier)).
    let hash = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = base64url_encode(&hash);

    // Generate state (32 random bytes, hex-encoded).
    let oauth_state = hex::encode(random_bytes(32));

    // Build authorization URL.
    let auth_url = match tuitbot_core::source::connector::RemoteConnector::authorization_url(
        &connector,
        &oauth_state,
        &code_challenge,
    ) {
        Ok(url) => url,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Store pending PKCE state.
    {
        let mut pending = state.pending_oauth.lock().await;

        // Clean up expired entries while we have the lock.
        pending.retain(|_, v| v.created_at.elapsed() < OAUTH_STATE_TTL);

        pending.insert(
            oauth_state.clone(),
            PendingOAuth {
                code_verifier,
                created_at: std::time::Instant::now(),
            },
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "authorization_url": auth_url,
            "state": oauth_state
        })),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
pub struct LinkParams {
    force: Option<bool>,
}

// ---------------------------------------------------------------------------
// GET /api/connectors/google-drive/callback
// ---------------------------------------------------------------------------

/// OAuth callback endpoint (auth-exempt, state-validated).
///
/// Receives the authorization code from Google, exchanges it for tokens,
/// encrypts the refresh token, and stores the connection.
pub async fn callback_google_drive(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CallbackParams>,
) -> Response {
    let code = match params.code {
        Some(c) if !c.is_empty() => c,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "missing code parameter"})),
            )
                .into_response();
        }
    };

    let oauth_state = match params.state {
        Some(s) if !s.is_empty() => s,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "missing state parameter"})),
            )
                .into_response();
        }
    };

    // Look up and consume the pending PKCE state.
    let code_verifier = {
        let mut pending = state.pending_oauth.lock().await;
        match pending.remove(&oauth_state) {
            Some(p) if p.created_at.elapsed() < OAUTH_STATE_TTL => p.code_verifier,
            Some(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "state expired"})),
                )
                    .into_response();
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "invalid or expired state"})),
                )
                    .into_response();
            }
        }
    };

    // Load connector config.
    let config = match load_connector_config(&state) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let connector =
        match tuitbot_core::source::connector::google_drive::GoogleDriveConnector::new(&config) {
            Ok(c) => c,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        };

    // Exchange code for tokens.
    let tokens = match tuitbot_core::source::connector::RemoteConnector::exchange_code(
        &connector,
        &code,
        &code_verifier,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "OAuth token exchange failed");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("token exchange failed: {e}")})),
            )
                .into_response();
        }
    };

    // Fetch user info.
    let user_info = match tuitbot_core::source::connector::RemoteConnector::user_info(
        &connector,
        &tokens.access_token,
    )
    .await
    {
        Ok(info) => info,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch user info, proceeding without");
            tuitbot_core::source::connector::UserInfo {
                email: "unknown".to_string(),
                display_name: None,
            }
        }
    };

    // Load connector encryption key.
    let key = match tuitbot_core::source::connector::crypto::ensure_connector_key(&state.data_dir) {
        Ok(k) => k,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load connector key");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "encryption key error"})),
            )
                .into_response();
        }
    };

    // Encrypt refresh token.
    let encrypted = match tuitbot_core::source::connector::google_drive::encrypt_refresh_token(
        &tokens.refresh_token,
        &key,
    ) {
        Ok(enc) => enc,
        Err(e) => {
            tracing::error!(error = %e, "Failed to encrypt refresh token");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "encryption failed"})),
            )
                .into_response();
        }
    };

    // Insert connection row.
    let conn_id = match tuitbot_core::storage::watchtower::insert_connection(
        &state.db,
        "google_drive",
        Some(&user_info.email),
        user_info.display_name.as_deref(),
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "Failed to insert connection");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "database error"})),
            )
                .into_response();
        }
    };

    // Store encrypted credentials.
    if let Err(e) = tuitbot_core::storage::watchtower::store_encrypted_credentials(
        &state.db, conn_id, &encrypted,
    )
    .await
    {
        tracing::error!(error = %e, "Failed to store encrypted credentials");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "credential storage error"})),
        )
            .into_response();
    }

    // Update metadata.
    let metadata = json!({
        "scope": tokens.scope,
        "linked_at": chrono::Utc::now().to_rfc3339(),
    });
    if let Err(e) = tuitbot_core::storage::watchtower::update_connection_metadata(
        &state.db,
        conn_id,
        &metadata.to_string(),
    )
    .await
    {
        tracing::warn!(error = %e, "Failed to update connection metadata");
    }

    // Return an HTML success page.
    Html(format!(
        r#"<!DOCTYPE html>
<html><head><title>Tuitbot - Connected</title></head>
<body style="font-family:system-ui;text-align:center;padding:60px">
<h2>Google Drive Connected</h2>
<p>Account: {email}</p>
<p>You can close this tab and return to the dashboard.</p>
<script>
if (window.opener) {{
    window.opener.postMessage({{ type: "connector_linked", connector: "google_drive", id: {conn_id} }}, "*");
}}
</script>
</body></html>"#,
        email = html_escape(&user_info.email),
    ))
    .into_response()
}

#[derive(serde::Deserialize)]
pub struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/connectors/google-drive/status
// ---------------------------------------------------------------------------

/// Get the status of Google Drive connections.
///
/// Returns all active google_drive connections (without secrets).
pub async fn status_google_drive(State(state): State<Arc<AppState>>) -> Response {
    match tuitbot_core::storage::watchtower::get_connections_by_type(&state.db, "google_drive")
        .await
    {
        Ok(conns) => (StatusCode::OK, Json(json!({ "connections": conns }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// DELETE /api/connectors/google-drive/{id}
// ---------------------------------------------------------------------------

/// Disconnect a Google Drive connection.
///
/// Revokes the token (best-effort) and deletes the connection row.
pub async fn disconnect_google_drive(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Response {
    // Load connection.
    let conn = match tuitbot_core::storage::watchtower::get_connection(&state.db, id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "connection not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    if conn.connector_type != "google_drive" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "not a Google Drive connection"})),
        )
            .into_response();
    }

    // Best-effort revocation.
    let encrypted = tuitbot_core::storage::watchtower::read_encrypted_credentials(&state.db, id)
        .await
        .ok()
        .flatten();

    if let Some(ref enc) = encrypted {
        if let Ok(key) =
            tuitbot_core::source::connector::crypto::ensure_connector_key(&state.data_dir)
        {
            if let Ok(config) = load_connector_config(&state) {
                if let Ok(connector) =
                    tuitbot_core::source::connector::google_drive::GoogleDriveConnector::new(
                        &config,
                    )
                {
                    if let Err(e) = tuitbot_core::source::connector::RemoteConnector::revoke(
                        &connector, enc, &key,
                    )
                    .await
                    {
                        tracing::warn!(
                            connection_id = id,
                            error = %e,
                            "Token revocation failed during disconnect"
                        );
                    }
                }
            }
        }
    }

    // Delete the connection row.
    if let Err(e) = tuitbot_core::storage::watchtower::delete_connection(&state.db, id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({ "disconnected": true, "id": id })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load connector config from the config file at `state.config_path`.
#[allow(clippy::result_large_err)]
fn load_connector_config(
    state: &AppState,
) -> Result<tuitbot_core::config::GoogleDriveConnectorConfig, Response> {
    let config_str = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: tuitbot_core::config::Config = toml::from_str(&config_str).unwrap_or_default();
    Ok(config.connectors.google_drive)
}

/// Generate `n` random bytes.
fn random_bytes(n: usize) -> Vec<u8> {
    (0..n).map(|_| rand::random::<u8>()).collect()
}

/// Base64url-encode without padding (for PKCE code challenge).
fn base64url_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Minimal HTML escaping for user-facing strings.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
