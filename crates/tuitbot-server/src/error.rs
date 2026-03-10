//! API error types for the tuitbot server.
//!
//! Maps core domain errors to HTTP status codes and JSON error responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// API error type for route handlers.
pub enum ApiError {
    /// Internal storage/database error.
    Storage(tuitbot_core::error::StorageError),
    /// Requested resource not found.
    NotFound(String),
    /// Bad request (invalid query parameters, etc.).
    BadRequest(String),
    /// Conflict (resource already exists, runtime already running, etc.).
    Conflict(String),
    /// Internal server error (non-storage).
    Internal(String),
    /// Forbidden — insufficient role/permissions.
    Forbidden(String),
}

impl From<tuitbot_core::error::StorageError> for ApiError {
    fn from(err: tuitbot_core::error::StorageError) -> Self {
        match err {
            tuitbot_core::error::StorageError::AlreadyReviewed { id, current_status } => {
                Self::Conflict(format!(
                    "item {id} has already been reviewed (current status: {current_status})"
                ))
            }
            other => Self::Storage(other),
        }
    }
}

impl From<crate::account::AccountError> for ApiError {
    fn from(err: crate::account::AccountError) -> Self {
        match err.status {
            StatusCode::FORBIDDEN => Self::Forbidden(err.message),
            StatusCode::NOT_FOUND => Self::NotFound(err.message),
            _ => Self::Internal(err.message),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Storage(e) => {
                tracing::error!("storage error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
            Self::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        };

        let body = axum::Json(json!({ "error": message }));
        (status, body).into_response()
    }
}
