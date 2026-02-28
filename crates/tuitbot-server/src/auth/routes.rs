//! Authentication HTTP endpoints.
//!
//! - `POST /api/auth/login` — passphrase login → session cookie
//! - `POST /api/auth/logout` — clear session
//! - `GET  /api/auth/status` — check if current session is valid

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tuitbot_core::auth::{passphrase, session};

use crate::state::AppState;

/// Maximum login attempts per IP before rate limiting.
const MAX_ATTEMPTS_PER_MINUTE: u32 = 5;
/// Rate limit window in seconds.
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

#[derive(Deserialize)]
pub struct LoginRequest {
    passphrase: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    csrf_token: String,
    expires_at: String,
}

#[derive(Serialize)]
pub struct AuthStatusResponse {
    authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    csrf_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
}

/// Extract client IP from X-Forwarded-For or fall back to a default.
fn client_ip(headers: &HeaderMap) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|ip| ip.trim().parse().ok())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]))
}

/// Extract the session cookie value from headers.
fn extract_session_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix("tuitbot_session=").map(|v| v.to_string())
            })
        })
}

/// Check rate limit for the given IP. Returns true if allowed.
async fn check_rate_limit(state: &AppState, ip: IpAddr) -> bool {
    let attempts = state.login_attempts.lock().await;
    let now = Instant::now();

    if let Some((count, window_start)) = attempts.get(&ip) {
        if now.duration_since(*window_start).as_secs() < RATE_LIMIT_WINDOW_SECS
            && *count >= MAX_ATTEMPTS_PER_MINUTE
        {
            return false;
        }
    }
    true
}

/// Record a login attempt for rate limiting.
async fn record_attempt(state: &AppState, ip: IpAddr) {
    let mut attempts = state.login_attempts.lock().await;
    let now = Instant::now();

    let entry = attempts.entry(ip).or_insert((0, now));
    if now.duration_since(entry.1).as_secs() >= RATE_LIMIT_WINDOW_SECS {
        // Reset window
        *entry = (1, now);
    } else {
        entry.0 += 1;
    }
}

/// `POST /api/auth/login` — verify passphrase and create a session cookie.
pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::Json(body): axum::Json<LoginRequest>,
) -> impl IntoResponse {
    let ip = client_ip(&headers);

    // Rate limit check
    if !check_rate_limit(&state, ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({"error": "too many login attempts, try again later"})),
        )
            .into_response();
    }

    record_attempt(&state, ip).await;

    // Check if passphrase auth is configured
    let hash = state.passphrase_hash.read().await;
    let Some(ref hash) = *hash else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(json!({"error": "passphrase authentication not configured"})),
        )
            .into_response();
    };

    // Verify passphrase
    match passphrase::verify_passphrase(&body.passphrase, hash) {
        Ok(true) => { /* valid */ }
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({"error": "invalid passphrase"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "Passphrase verification failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({"error": "authentication error"})),
            )
                .into_response();
        }
    }

    // Create session
    match session::create_session(&state.db).await {
        Ok(new_session) => {
            let cookie = format!(
                "tuitbot_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=604800",
                new_session.raw_token,
            );

            let response = LoginResponse {
                csrf_token: new_session.csrf_token,
                expires_at: new_session.expires_at,
            };

            (
                StatusCode::OK,
                [(axum::http::header::SET_COOKIE, cookie)],
                axum::Json(serde_json::to_value(response).unwrap()),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create session");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({"error": "failed to create session"})),
            )
                .into_response()
        }
    }
}

/// `POST /api/auth/logout` — delete the session and clear the cookie.
pub async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if let Some(token) = extract_session_cookie(&headers) {
        if let Err(e) = session::delete_session(&state.db, &token).await {
            tracing::error!(error = %e, "Failed to delete session");
        }
    }

    let clear_cookie = "tuitbot_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0".to_string();

    (
        StatusCode::OK,
        [(axum::http::header::SET_COOKIE, clear_cookie)],
        axum::Json(json!({"ok": true})),
    )
        .into_response()
}

/// `GET /api/auth/status` — check if the current request has a valid session.
pub async fn status(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    // Check bearer token first
    let bearer_ok = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|token| token == state.api_token);

    if bearer_ok {
        return axum::Json(
            serde_json::to_value(AuthStatusResponse {
                authenticated: true,
                csrf_token: None,
                expires_at: None,
            })
            .unwrap(),
        )
        .into_response();
    }

    // Check session cookie
    if let Some(token) = extract_session_cookie(&headers) {
        if let Ok(Some(sess)) = session::validate_session(&state.db, &token).await {
            return axum::Json(
                serde_json::to_value(AuthStatusResponse {
                    authenticated: true,
                    csrf_token: Some(sess.csrf_token),
                    expires_at: Some(sess.expires_at),
                })
                .unwrap(),
            )
            .into_response();
        }
    }

    axum::Json(
        serde_json::to_value(AuthStatusResponse {
            authenticated: false,
            csrf_token: None,
            expires_at: None,
        })
        .unwrap(),
    )
    .into_response()
}
