//! Stateless X API utility toolkit.
//!
//! Provides pure, stateless functions for all X API operations.
//! Each function takes `&dyn XApiClient` and operation-specific parameters,
//! performs input validation, and delegates to the client trait.
//!
//! No policy enforcement, audit logging, rate limiting, or DB access here.
//! Those concerns belong in the workflow layer (AD-04, AD-12).

pub mod engage;
pub mod media;
pub mod read;
pub mod write;

#[cfg(test)]
mod e2e_tests;

use crate::error::XApiError;

/// Maximum tweet length enforced by the X API.
pub const MAX_TWEET_LENGTH: usize = 280;

/// Errors from toolkit operations.
///
/// Maps to existing `ErrorCode` variants in MCP responses (AD-10).
/// Stateless checks live here; stateful checks in the workflow layer (AD-12).
#[derive(Debug, thiserror::Error)]
pub enum ToolkitError {
    /// Underlying X API error (passthrough).
    #[error(transparent)]
    XApi(#[from] XApiError),

    /// Invalid input parameter.
    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    /// Tweet text exceeds the maximum length.
    #[error("tweet too long: {length} characters (max {max})")]
    TweetTooLong { length: usize, max: usize },

    /// File extension does not map to a supported media type.
    #[error("unsupported media type for file: {path}")]
    UnsupportedMediaType { path: String },

    /// Media data exceeds the size limit for its type.
    #[error("media too large: {size} bytes (max {max} for {media_type})")]
    MediaTooLarge {
        size: u64,
        max: u64,
        media_type: String,
    },

    /// Thread posting failed partway through.
    #[error("thread failed at tweet {failed_index}: posted {posted}/{total} tweets")]
    ThreadPartialFailure {
        /// IDs of tweets successfully posted before the failure.
        posted_ids: Vec<String>,
        /// Zero-based index of the tweet that failed.
        failed_index: usize,
        /// Count of successfully posted tweets.
        posted: usize,
        /// Total tweets in the thread.
        total: usize,
        /// The underlying X API error.
        #[source]
        source: Box<XApiError>,
    },
}

/// Validate tweet text length (stateless check).
pub fn validate_tweet_length(text: &str) -> Result<(), ToolkitError> {
    if text.len() > MAX_TWEET_LENGTH {
        return Err(ToolkitError::TweetTooLong {
            length: text.len(),
            max: MAX_TWEET_LENGTH,
        });
    }
    Ok(())
}

/// Validate that a string ID parameter is non-empty.
fn validate_id(id: &str, name: &str) -> Result<(), ToolkitError> {
    if id.is_empty() {
        return Err(ToolkitError::InvalidInput {
            message: format!("{name} must not be empty"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_tweet_length_ok() {
        assert!(validate_tweet_length("Hello world").is_ok());
    }

    #[test]
    fn validate_tweet_length_exactly_280() {
        let text = "a".repeat(280);
        assert!(validate_tweet_length(&text).is_ok());
    }

    #[test]
    fn validate_tweet_length_too_long() {
        let text = "a".repeat(281);
        let err = validate_tweet_length(&text).unwrap_err();
        assert!(matches!(
            err,
            ToolkitError::TweetTooLong {
                length: 281,
                max: 280
            }
        ));
    }

    #[test]
    fn validate_id_ok() {
        assert!(validate_id("123", "tweet_id").is_ok());
    }

    #[test]
    fn validate_id_empty() {
        let err = validate_id("", "tweet_id").unwrap_err();
        assert!(matches!(err, ToolkitError::InvalidInput { .. }));
    }

    #[test]
    fn toolkit_error_display() {
        let err = ToolkitError::TweetTooLong {
            length: 300,
            max: 280,
        };
        assert_eq!(err.to_string(), "tweet too long: 300 characters (max 280)");
    }

    #[test]
    fn toolkit_error_from_x_api() {
        let xe = XApiError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        let te: ToolkitError = xe.into();
        assert!(matches!(te, ToolkitError::XApi(_)));
    }
}
