//! Multi-strategy authentication middleware.
//!
//! Checks in order:
//! 1. `Authorization: Bearer <token>` header → matches file-based API token
//! 2. `tuitbot_session` cookie → SHA-256 hash lookup in sessions table
//! 3. Neither → 401 Unauthorized
//!
//! For cookie-authenticated requests, mutating methods (POST/PATCH/DELETE/PUT)
//! require a valid `X-CSRF-Token` header matching the session's CSRF token.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tuitbot_core::auth::session;

use crate::state::AppState;

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

/// Routes exempt from authentication.
const AUTH_EXEMPT_PATHS: &[&str] = &[
    "/health",
    "/api/health",
    "/settings/status",
    "/api/settings/status",
    "/settings/init",
    "/api/settings/init",
    "/ws",
    "/api/ws",
    "/auth/login",
    "/api/auth/login",
    "/auth/status",
    "/api/auth/status",
];

/// Axum middleware that enforces multi-strategy authentication.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();

    // Skip auth for exempt endpoints.
    if AUTH_EXEMPT_PATHS.contains(&path) {
        return next.run(request).await;
    }

    // Strategy 1: Bearer token
    let bearer_ok = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|token| token == state.api_token);

    if bearer_ok {
        return next.run(request).await;
    }

    // Strategy 2: Session cookie
    if let Some(session_token) = extract_session_cookie(&headers) {
        match session::validate_session(&state.db, &session_token).await {
            Ok(Some(sess)) => {
                // CSRF check for mutating methods
                let method = request.method().clone();
                if method == Method::POST
                    || method == Method::PATCH
                    || method == Method::DELETE
                    || method == Method::PUT
                {
                    let csrf_ok = headers
                        .get("x-csrf-token")
                        .and_then(|v| v.to_str().ok())
                        .is_some_and(|t| t == sess.csrf_token);

                    if !csrf_ok {
                        return (
                            StatusCode::FORBIDDEN,
                            axum::Json(json!({"error": "missing or invalid CSRF token"})),
                        )
                            .into_response();
                    }
                }
                return next.run(request).await;
            }
            Ok(None) => { /* session not found or expired — fall through to 401 */ }
            Err(e) => {
                tracing::error!(error = %e, "Session validation failed");
            }
        }
    }

    // Neither strategy succeeded.
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({"error": "unauthorized"})),
    )
        .into_response()
}
