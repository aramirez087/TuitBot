//! Provider-agnostic error taxonomy.
//!
//! [`ProviderError`] is the canonical error type returned by provider
//! implementations. The kernel maps these to [`ToolResponse`] envelopes
//! via [`provider_error_to_response`].

use std::fmt;
use std::time::Instant;

use super::envelope::{ToolMeta, ToolResponse};
use super::error_code::ErrorCode;

/// Provider-agnostic error returned by [`SocialReadProvider`](crate::provider::SocialReadProvider).
///
/// Maps cleanly to the MCP error envelope without referencing any
/// concrete API client error type.
#[derive(Debug)]
pub enum ProviderError {
    /// The provider's upstream rate limit was exceeded.
    RateLimited { retry_after: Option<u64> },
    /// Authentication credentials are expired or invalid.
    AuthExpired,
    /// The request was understood but refused by the upstream API.
    Forbidden { message: String },
    /// The account is restricted by the upstream platform.
    AccountRestricted { message: String },
    /// Transient network failure.
    Network { message: String },
    /// The provider is not configured or initialized.
    NotConfigured { message: String },
    /// Upstream server error (HTTP 5xx). Transient — may succeed on retry.
    ServerError { status: u16, message: String },
    /// Catch-all for other provider errors.
    Other { message: String },
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited { retry_after } => match retry_after {
                Some(s) => write!(f, "rate limited, retry after {s}s"),
                None => write!(f, "rate limited"),
            },
            Self::AuthExpired => write!(f, "authentication expired"),
            Self::Forbidden { message } => write!(f, "forbidden: {message}"),
            Self::AccountRestricted { message } => write!(f, "account restricted: {message}"),
            Self::Network { message } => write!(f, "network error: {message}"),
            Self::NotConfigured { message } => write!(f, "not configured: {message}"),
            Self::ServerError { status, message } => {
                write!(f, "server error ({status}): {message}")
            }
            Self::Other { message } => write!(f, "provider error: {message}"),
        }
    }
}

impl std::error::Error for ProviderError {}

impl ProviderError {
    /// Map this error to its typed [`ErrorCode`].
    pub fn error_code(&self) -> ErrorCode {
        match self {
            Self::RateLimited { .. } => ErrorCode::XRateLimited,
            Self::AuthExpired => ErrorCode::XAuthExpired,
            Self::Forbidden { .. } => ErrorCode::XForbidden,
            Self::AccountRestricted { .. } => ErrorCode::XAccountRestricted,
            Self::Network { .. } => ErrorCode::XNetworkError,
            Self::NotConfigured { .. } => ErrorCode::XNotConfigured,
            Self::ServerError { .. } => ErrorCode::XApiError,
            Self::Other { .. } => ErrorCode::XApiError,
        }
    }

    /// Build the human-readable error message.
    fn error_message(&self) -> String {
        match self {
            Self::RateLimited { retry_after } => format!(
                "X API rate limited{}",
                match retry_after {
                    Some(s) => format!(", retry after {s}s"),
                    None => String::new(),
                }
            ),
            Self::AuthExpired => {
                "X API authentication expired. Run `tuitbot auth` to re-authenticate.".to_string()
            }
            Self::Forbidden { message } => format!("X API forbidden: {message}"),
            Self::AccountRestricted { message } => {
                format!("X API account restricted: {message}")
            }
            Self::Network { message } => format!("X API network error: {message}"),
            Self::NotConfigured { message } => message.clone(),
            Self::ServerError { status, message } => {
                format!("X API server error ({status}): {message}")
            }
            Self::Other { message } => message.clone(),
        }
    }
}

/// Convert a [`ProviderError`] into a [`ToolResponse`] JSON string with elapsed metadata.
pub fn provider_error_to_response(e: &ProviderError, start: Instant) -> String {
    provider_error_to_audited_response(e, ToolMeta::new(start.elapsed().as_millis() as u64))
}

/// Like [`provider_error_to_response`] but with a pre-built [`ToolMeta`]
/// carrying audit context (correlation ID, etc.).
pub fn provider_error_to_audited_response(e: &ProviderError, meta: ToolMeta) -> String {
    let code = e.error_code();
    let message = e.error_message();
    let mut resp = ToolResponse::error(code, message).with_meta(meta);
    // Populate retry_after_ms for rate-limited errors.
    if let ProviderError::RateLimited {
        retry_after: Some(secs),
    } = e
    {
        resp = resp.with_retry_after_ms(secs * 1000);
    }
    resp.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limited_maps_correctly() {
        let err = ProviderError::RateLimited {
            retry_after: Some(30),
        };
        assert_eq!(err.error_code(), ErrorCode::XRateLimited);
        assert!(err.error_code().is_retryable());
        assert!(err.error_message().contains("30s"));
    }

    #[test]
    fn auth_expired_maps_correctly() {
        let err = ProviderError::AuthExpired;
        assert_eq!(err.error_code(), ErrorCode::XAuthExpired);
        assert!(!err.error_code().is_retryable());
    }

    #[test]
    fn not_configured_maps_correctly() {
        let err = ProviderError::NotConfigured {
            message: "X API client not available.".to_string(),
        };
        assert_eq!(err.error_code(), ErrorCode::XNotConfigured);
        assert!(!err.error_code().is_retryable());
    }

    #[test]
    fn provider_error_to_response_produces_valid_json() {
        let err = ProviderError::Forbidden {
            message: "not allowed".to_string(),
        };
        let json = provider_error_to_response(&err, Instant::now());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_forbidden");
        assert!(parsed["meta"]["elapsed_ms"].is_number());
    }

    #[test]
    fn all_variants_have_error_codes() {
        let errors = [
            ProviderError::RateLimited {
                retry_after: Some(10),
            },
            ProviderError::AuthExpired,
            ProviderError::Forbidden {
                message: "f".into(),
            },
            ProviderError::AccountRestricted {
                message: "a".into(),
            },
            ProviderError::Network {
                message: "n".into(),
            },
            ProviderError::NotConfigured {
                message: "nc".into(),
            },
            ProviderError::ServerError {
                status: 503,
                message: "unavailable".into(),
            },
            ProviderError::Other {
                message: "o".into(),
            },
        ];
        for err in &errors {
            let code = err.error_code();
            let msg = err.error_message();
            assert!(!msg.is_empty(), "empty message for {code:?}");
        }
    }

    #[test]
    fn server_error_maps_correctly() {
        let err = ProviderError::ServerError {
            status: 502,
            message: "bad gateway".into(),
        };
        assert_eq!(err.error_code(), ErrorCode::XApiError);
        assert!(err.error_code().is_retryable());
        assert!(err.error_message().contains("502"));
    }

    #[test]
    fn rate_limited_response_has_retry_after_ms() {
        let err = ProviderError::RateLimited {
            retry_after: Some(30),
        };
        let json = provider_error_to_response(&err, Instant::now());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["error"]["retry_after_ms"], 30000);
        assert!(parsed["error"]["retryable"].as_bool().unwrap());
    }
}
