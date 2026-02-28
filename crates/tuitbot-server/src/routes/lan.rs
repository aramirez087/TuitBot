//! LAN access settings endpoints.
//!
//! - `GET  /api/settings/lan` — current server/LAN status
//! - `POST /api/settings/lan/reset-passphrase` — reset passphrase at runtime
//! - `PATCH /api/settings/lan` — toggle LAN mode (persisted to config.toml)

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tuitbot_core::auth::passphrase;
use tuitbot_core::config::Config;
use tuitbot_core::net::local_ip;

use crate::state::AppState;

#[derive(Serialize)]
struct LanStatus {
    bind_host: String,
    bind_port: u16,
    lan_enabled: bool,
    local_ip: Option<String>,
    passphrase_configured: bool,
}

/// `GET /api/settings/lan` — return current LAN/server status.
pub async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let hash = state.passphrase_hash.read().await;
    let status = LanStatus {
        bind_host: state.bind_host.clone(),
        bind_port: state.bind_port,
        lan_enabled: state.bind_host == "0.0.0.0",
        local_ip: local_ip(),
        passphrase_configured: hash.is_some(),
    };
    axum::Json(serde_json::to_value(status).unwrap())
}

/// `POST /api/settings/lan/reset-passphrase` — generate a new passphrase.
pub async fn reset_passphrase(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match passphrase::reset_passphrase(&state.data_dir) {
        Ok(new_passphrase) => {
            // Update the in-memory hash.
            match passphrase::load_passphrase_hash(&state.data_dir) {
                Ok(new_hash) => {
                    let mut hash = state.passphrase_hash.write().await;
                    *hash = new_hash;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to reload passphrase hash");
                }
            }
            (
                StatusCode::OK,
                axum::Json(json!({"passphrase": new_passphrase})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"error": format!("failed to reset passphrase: {e}")})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct ToggleLanRequest {
    host: String,
}

/// `PATCH /api/settings/lan` — toggle LAN mode by updating config.toml.
pub async fn toggle_lan(
    State(state): State<Arc<AppState>>,
    axum::Json(body): axum::Json<ToggleLanRequest>,
) -> impl IntoResponse {
    // Validate host value.
    if body.host != "0.0.0.0" && body.host != "127.0.0.1" {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({"error": "host must be \"0.0.0.0\" or \"127.0.0.1\""})),
        )
            .into_response();
    }

    // Load current config, update server.host, write back.
    let config_path_str = state.config_path.to_string_lossy().to_string();
    let mut config = match Config::load(Some(&config_path_str)) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({"error": format!("failed to load config: {e}")})),
            )
                .into_response();
        }
    };

    config.server.host = body.host;

    let toml_str = match toml::to_string_pretty(&config) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({"error": format!("failed to serialize config: {e}")})),
            )
                .into_response();
        }
    };

    if let Err(e) = std::fs::write(&state.config_path, toml_str) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"error": format!("failed to write config: {e}")})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        axum::Json(json!({"restart_required": true})),
    )
        .into_response()
}
