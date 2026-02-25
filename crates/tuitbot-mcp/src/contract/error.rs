//! Provider-agnostic error taxonomy.
//!
//! [`ProviderError`] is the canonical error type returned by provider
//! implementations. The kernel maps these to [`ToolResponse`] envelopes
//! via [`provider_error_to_response`].

use std::fmt;
use std::time::Instant;

use super::envelope::{ToolMeta, ToolResponse};

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
            Self::Other { message } => write!(f, "provider error: {message}"),
        }
    }
}

impl std::error::Error for ProviderError {}

impl ProviderError {
    /// Map this error to an `(error_code, message, retryable)` triple.
    fn to_triple(&self) -> (&'static str, String, bool) {
        match self {
            Self::RateLimited { retry_after } => (
                "x_rate_limited",
                format!(
                    "X API rate limited{}",
                    match retry_after {
                        Some(s) => format!(", retry after {s}s"),
                        None => String::new(),
                    }
                ),
                true,
            ),
            Self::AuthExpired => (
                "x_auth_expired",
                "X API authentication expired. Run `tuitbot auth` to re-authenticate.".to_string(),
                false,
            ),
            Self::Forbidden { message } => {
                ("x_forbidden", format!("X API forbidden: {message}"), false)
            }
            Self::AccountRestricted { message } => (
                "x_account_restricted",
                format!("X API account restricted: {message}"),
                false,
            ),
            Self::Network { message } => (
                "x_network_error",
                format!("X API network error: {message}"),
                true,
            ),
            Self::NotConfigured { message } => ("x_not_configured", message.clone(), false),
            Self::Other { message } => ("x_api_error", message.clone(), false),
        }
    }
}

/// Convert a [`ProviderError`] into a [`ToolResponse`] JSON string with elapsed metadata.
pub fn provider_error_to_response(e: &ProviderError, start: Instant) -> String {
    let (code, message, retryable) = e.to_triple();
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(code, message, retryable)
        .with_meta(ToolMeta::new(elapsed))
        .to_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limited_maps_correctly() {
        let err = ProviderError::RateLimited {
            retry_after: Some(30),
        };
        let (code, msg, retryable) = err.to_triple();
        assert_eq!(code, "x_rate_limited");
        assert!(msg.contains("30s"));
        assert!(retryable);
    }

    #[test]
    fn auth_expired_maps_correctly() {
        let err = ProviderError::AuthExpired;
        let (code, _, retryable) = err.to_triple();
        assert_eq!(code, "x_auth_expired");
        assert!(!retryable);
    }

    #[test]
    fn not_configured_maps_correctly() {
        let err = ProviderError::NotConfigured {
            message: "X API client not available.".to_string(),
        };
        let (code, _, retryable) = err.to_triple();
        assert_eq!(code, "x_not_configured");
        assert!(!retryable);
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
}
