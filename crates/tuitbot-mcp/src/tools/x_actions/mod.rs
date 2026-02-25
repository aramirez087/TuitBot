//! Direct X API tool implementations.
//!
//! Split into submodules by concern: read, write, engage, media, validate.

mod engage;
mod media;
mod read;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use std::time::Instant;

use tuitbot_core::error::XApiError;

use super::response::{ToolMeta, ToolResponse};

// Re-export all public tool functions.
pub use engage::{follow_user, like_tweet, retweet, unfollow_user, unretweet};
pub use media::upload_media;
pub use read::{
    get_home_timeline, get_tweet_by_id, get_user_by_username, get_user_mentions, get_user_tweets,
    get_x_usage, search_tweets,
};
pub use write::{delete_tweet, post_thread, post_tweet, quote_tweet, reply_to_tweet};

/// Map an `XApiError` to a structured `(code, message, retryable)` triple.
fn x_error_to_response(e: &XApiError) -> (&'static str, String, bool) {
    match e {
        XApiError::RateLimited { retry_after } => (
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
        XApiError::AuthExpired => (
            "x_auth_expired",
            "X API authentication expired. Run `tuitbot auth` to re-authenticate.".to_string(),
            false,
        ),
        XApiError::Forbidden { message } => {
            ("x_forbidden", format!("X API forbidden: {message}"), false)
        }
        XApiError::AccountRestricted { message } => (
            "x_account_restricted",
            format!("X API account restricted: {message}"),
            false,
        ),
        XApiError::Network { source } => (
            "x_network_error",
            format!("X API network error: {source}"),
            true,
        ),
        other => ("x_api_error", other.to_string(), false),
    }
}

/// Return an error response when the X client is not configured.
fn not_configured_response(start: Instant) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(
        "x_not_configured",
        "X API client not available. Run `tuitbot auth` to authenticate.",
        false,
    )
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}

/// Helper: build an error response from an XApiError.
fn error_response(e: &XApiError, start: Instant) -> String {
    let (code, message, retryable) = x_error_to_response(e);
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(code, message, retryable)
        .with_meta(ToolMeta::new(elapsed))
        .to_json()
}

/// Return an error response when the authenticated user ID is missing.
fn no_user_id_response(start: Instant) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(
        "x_not_configured",
        "Authenticated user ID not available. X client may not be fully initialized.",
        false,
    )
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}
