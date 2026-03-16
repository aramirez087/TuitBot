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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    /// Helper to extract status code and body JSON from an ApiError response.
    async fn error_response(err: ApiError) -> (StatusCode, serde_json::Value) {
        let resp = err.into_response();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let (status, body) = error_response(ApiError::NotFound("missing item".into())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"], "missing item");
    }

    #[tokio::test]
    async fn bad_request_returns_400() {
        let (status, body) = error_response(ApiError::BadRequest("invalid field".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "invalid field");
    }

    #[tokio::test]
    async fn conflict_returns_409() {
        let (status, body) = error_response(ApiError::Conflict("already exists".into())).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"], "already exists");
    }

    #[tokio::test]
    async fn internal_returns_500() {
        let (status, body) = error_response(ApiError::Internal("crash".into())).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"], "crash");
    }

    #[tokio::test]
    async fn forbidden_returns_403() {
        let (status, body) = error_response(ApiError::Forbidden("no access".into())).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"], "no access");
    }

    #[tokio::test]
    async fn storage_error_returns_500() {
        let storage_err = tuitbot_core::error::StorageError::Query {
            source: sqlx::Error::RowNotFound,
        };
        let (status, body) = error_response(ApiError::Storage(storage_err)).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body["error"].as_str().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn already_reviewed_storage_converts_to_conflict() {
        let storage_err = tuitbot_core::error::StorageError::AlreadyReviewed {
            id: 42,
            current_status: "approved".into(),
        };
        let api_err: ApiError = storage_err.into();
        let (status, body) = error_response(api_err).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert!(body["error"]
            .as_str()
            .unwrap()
            .contains("already been reviewed"));
    }

    #[test]
    fn account_error_forbidden_converts() {
        let account_err = crate::account::AccountError {
            status: StatusCode::FORBIDDEN,
            message: "no perms".into(),
        };
        let api_err: ApiError = account_err.into();
        match api_err {
            ApiError::Forbidden(msg) => assert_eq!(msg, "no perms"),
            _ => panic!("expected Forbidden"),
        }
    }

    #[test]
    fn account_error_not_found_converts() {
        let account_err = crate::account::AccountError {
            status: StatusCode::NOT_FOUND,
            message: "gone".into(),
        };
        let api_err: ApiError = account_err.into();
        match api_err {
            ApiError::NotFound(msg) => assert_eq!(msg, "gone"),
            _ => panic!("expected NotFound"),
        }
    }

    #[test]
    fn account_error_other_converts_to_internal() {
        let account_err = crate::account::AccountError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "db down".into(),
        };
        let api_err: ApiError = account_err.into();
        match api_err {
            ApiError::Internal(msg) => assert_eq!(msg, "db down"),
            _ => panic!("expected Internal"),
        }
    }
}
