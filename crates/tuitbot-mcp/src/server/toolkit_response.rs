//! Response helpers for utility profile servers.
//!
//! Maps toolkit results to MCP `CallToolResult` using the standard
//! response envelope contract. Simpler than the workflow x_actions
//! error path — no audit guard, no policy, no meta.

use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

use crate::contract::envelope::ToolResponse;
use crate::contract::error_code::ErrorCode;
use tuitbot_core::error::XApiError;
use tuitbot_core::toolkit::ToolkitError;

/// Convert a toolkit result to a `CallToolResult` for read operations.
///
/// Serializes the value as pretty JSON on success; maps `ToolkitError`
/// to the standard error envelope on failure.
pub fn toolkit_read_result<T: Serialize>(
    result: Result<T, ToolkitError>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    match result {
        Ok(val) => {
            let json = serde_json::to_string_pretty(&val)
                .unwrap_or_else(|e| format!(r#"{{"error": "Serialization failed: {e}"}}"#));
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
        Err(e) => Ok(toolkit_error_to_envelope(e)),
    }
}

/// Convert a toolkit result (already serializable value) to a `CallToolResult`.
pub fn toolkit_error_to_result<T: Serialize>(
    result: Result<T, ToolkitError>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    match result {
        Ok(val) => {
            let json = serde_json::to_string_pretty(&val)
                .unwrap_or_else(|e| format!(r#"{{"error": "Serialization failed: {e}"}}"#));
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
        Err(e) => Ok(toolkit_error_to_envelope(e)),
    }
}

/// Map a `ToolkitError` to a `CallToolResult` with the appropriate error envelope.
fn toolkit_error_to_envelope(err: ToolkitError) -> CallToolResult {
    let response = match &err {
        ToolkitError::XApi(x_err) => match x_err {
            XApiError::RateLimited { retry_after } => {
                let mut r = ToolResponse::error(ErrorCode::XRateLimited, err.to_string());
                if let Some(secs) = retry_after {
                    r = r.with_retry_after_ms(*secs * 1000);
                }
                r
            }
            XApiError::AuthExpired => ToolResponse::error(ErrorCode::XAuthExpired, err.to_string()),
            XApiError::Forbidden { .. } => {
                ToolResponse::error(ErrorCode::XForbidden, err.to_string())
            }
            XApiError::Network { .. } => {
                ToolResponse::error(ErrorCode::XNetworkError, err.to_string())
            }
            _ => ToolResponse::error(ErrorCode::XApiError, err.to_string()),
        },
        ToolkitError::InvalidInput { .. } => {
            ToolResponse::error(ErrorCode::InvalidInput, err.to_string())
        }
        ToolkitError::TweetTooLong { .. } => {
            ToolResponse::error(ErrorCode::TweetTooLong, err.to_string())
        }
        ToolkitError::UnsupportedMediaType { .. } => {
            ToolResponse::error(ErrorCode::UnsupportedMediaType, err.to_string())
        }
        ToolkitError::MediaTooLarge { .. } => {
            ToolResponse::error(ErrorCode::MediaUploadError, err.to_string())
        }
        ToolkitError::ThreadPartialFailure { .. } => {
            ToolResponse::error(ErrorCode::ThreadPartialFailure, err.to_string())
        }
    };

    CallToolResult::success(vec![Content::text(response.to_json())])
}
