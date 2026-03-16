//! Account context extraction and role-based access control.
//!
//! Resolves the `X-Account-Id` header into an `AccountContext` with the
//! caller's role. Missing header defaults to the backward-compatible
//! default account.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tuitbot_core::storage::accounts::{self, DEFAULT_ACCOUNT_ID};

use crate::state::AppState;

/// Resolved account context available to route handlers.
#[derive(Debug, Clone)]
pub struct AccountContext {
    /// The account ID (UUIDv4 or default sentinel).
    pub account_id: String,
    /// The caller's role on this account.
    pub role: Role,
}

/// Role tiers for account access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Approver,
    Viewer,
}

impl Role {
    /// Whether this role can perform read operations (always true).
    pub fn can_read(self) -> bool {
        true
    }

    /// Whether this role can approve/reject items.
    pub fn can_approve(self) -> bool {
        matches!(self, Role::Admin | Role::Approver)
    }

    /// Whether this role can perform mutations (config, runtime, compose).
    pub fn can_mutate(self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Approver => write!(f, "approver"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Role::Admin),
            "approver" => Ok(Role::Approver),
            "viewer" => Ok(Role::Viewer),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

/// Error returned when account context extraction fails.
pub struct AccountError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for AccountError {
    fn into_response(self) -> Response {
        (self.status, axum::Json(json!({"error": self.message}))).into_response()
    }
}

impl FromRequestParts<Arc<AppState>> for AccountContext {
    type Rejection = AccountError;

    /// Extract account context from the `X-Account-Id` header.
    ///
    /// - Missing header → default account with admin role (backward compat).
    /// - Present header → validates account exists and resolves role.
    fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let account_id = parts
            .headers
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(DEFAULT_ACCOUNT_ID)
            .to_string();

        let db = state.db.clone();

        async move {
            // Default account always grants admin.
            if account_id == DEFAULT_ACCOUNT_ID {
                return Ok(AccountContext {
                    account_id,
                    role: Role::Admin,
                });
            }

            // Validate account exists and is active.
            let exists = accounts::account_exists(&db, &account_id)
                .await
                .map_err(|e| AccountError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("failed to validate account: {e}"),
                })?;

            if !exists {
                return Err(AccountError {
                    status: StatusCode::NOT_FOUND,
                    message: format!("account not found: {account_id}"),
                });
            }

            // Resolve role — default actor is "dashboard" for HTTP requests.
            let role_str = accounts::get_role(&db, &account_id, "dashboard")
                .await
                .map_err(|e| AccountError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("failed to resolve role: {e}"),
                })?;

            let role = role_str
                .as_deref()
                .unwrap_or("viewer")
                .parse::<Role>()
                .unwrap_or(Role::Viewer);

            Ok(AccountContext { account_id, role })
        }
    }
}

/// Helper to reject requests that require approval permissions.
pub fn require_approve(ctx: &AccountContext) -> Result<(), AccountError> {
    if ctx.role.can_approve() {
        Ok(())
    } else {
        Err(AccountError {
            status: StatusCode::FORBIDDEN,
            message: "approver or admin role required".to_string(),
        })
    }
}

/// Helper to reject requests that require mutation permissions.
pub fn require_mutate(ctx: &AccountContext) -> Result<(), AccountError> {
    if ctx.role.can_mutate() {
        Ok(())
    } else {
        Err(AccountError {
            status: StatusCode::FORBIDDEN,
            message: "admin role required".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    // --- Role permissions ---

    #[test]
    fn admin_can_read_approve_mutate() {
        assert!(Role::Admin.can_read());
        assert!(Role::Admin.can_approve());
        assert!(Role::Admin.can_mutate());
    }

    #[test]
    fn approver_can_read_approve_not_mutate() {
        assert!(Role::Approver.can_read());
        assert!(Role::Approver.can_approve());
        assert!(!Role::Approver.can_mutate());
    }

    #[test]
    fn viewer_can_read_only() {
        assert!(Role::Viewer.can_read());
        assert!(!Role::Viewer.can_approve());
        assert!(!Role::Viewer.can_mutate());
    }

    // --- Role Display ---

    #[test]
    fn role_display() {
        assert_eq!(Role::Admin.to_string(), "admin");
        assert_eq!(Role::Approver.to_string(), "approver");
        assert_eq!(Role::Viewer.to_string(), "viewer");
    }

    // --- Role FromStr ---

    #[test]
    fn role_from_str_valid() {
        assert_eq!("admin".parse::<Role>().unwrap(), Role::Admin);
        assert_eq!("approver".parse::<Role>().unwrap(), Role::Approver);
        assert_eq!("viewer".parse::<Role>().unwrap(), Role::Viewer);
    }

    #[test]
    fn role_from_str_invalid() {
        let err = "superuser".parse::<Role>().unwrap_err();
        assert!(err.contains("unknown role"));
    }

    // --- Role serde ---

    #[test]
    fn role_serde_roundtrip() {
        let json = serde_json::to_string(&Role::Admin).unwrap();
        assert_eq!(json, "\"admin\"");
        let parsed: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Role::Admin);

        let json = serde_json::to_string(&Role::Approver).unwrap();
        assert_eq!(json, "\"approver\"");

        let json = serde_json::to_string(&Role::Viewer).unwrap();
        assert_eq!(json, "\"viewer\"");
    }

    // --- require_approve ---

    #[test]
    fn require_approve_admin_ok() {
        let ctx = AccountContext {
            account_id: "test".into(),
            role: Role::Admin,
        };
        assert!(require_approve(&ctx).is_ok());
    }

    #[test]
    fn require_approve_approver_ok() {
        let ctx = AccountContext {
            account_id: "test".into(),
            role: Role::Approver,
        };
        assert!(require_approve(&ctx).is_ok());
    }

    #[test]
    fn require_approve_viewer_rejected() {
        let ctx = AccountContext {
            account_id: "test".into(),
            role: Role::Viewer,
        };
        let err = require_approve(&ctx).unwrap_err();
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert!(err.message.contains("approver"));
    }

    // --- require_mutate ---

    #[test]
    fn require_mutate_admin_ok() {
        let ctx = AccountContext {
            account_id: "test".into(),
            role: Role::Admin,
        };
        assert!(require_mutate(&ctx).is_ok());
    }

    #[test]
    fn require_mutate_approver_rejected() {
        let ctx = AccountContext {
            account_id: "test".into(),
            role: Role::Approver,
        };
        let err = require_mutate(&ctx).unwrap_err();
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert!(err.message.contains("admin"));
    }

    #[test]
    fn require_mutate_viewer_rejected() {
        let ctx = AccountContext {
            account_id: "test".into(),
            role: Role::Viewer,
        };
        let err = require_mutate(&ctx).unwrap_err();
        assert_eq!(err.status, StatusCode::FORBIDDEN);
    }

    // --- AccountError IntoResponse ---

    #[tokio::test]
    async fn account_error_into_response() {
        let err = AccountError {
            status: StatusCode::NOT_FOUND,
            message: "not here".into(),
        };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "not here");
    }

    #[tokio::test]
    async fn account_error_forbidden_response() {
        let err = AccountError {
            status: StatusCode::FORBIDDEN,
            message: "denied".into(),
        };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
