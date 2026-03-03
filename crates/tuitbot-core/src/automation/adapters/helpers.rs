//! Shared helper functions for adapter error mapping and data conversion.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::{LlmError, XApiError};
use crate::toolkit::ToolkitError;
use crate::x_api::SearchResponse;

use super::super::analytics_loop::AnalyticsError;
use super::super::loop_helpers::{ContentLoopError, LoopError, LoopTweet};

/// Convert an X API `SearchResponse` to a `Vec<LoopTweet>`.
///
/// Joins tweet data with user data from the `includes` expansion to populate
/// author username and follower count.
pub(super) fn search_response_to_loop_tweets(response: SearchResponse) -> Vec<LoopTweet> {
    let users: HashMap<&str, _> = response
        .includes
        .as_ref()
        .map(|inc| inc.users.iter().map(|u| (u.id.as_str(), u)).collect())
        .unwrap_or_default();

    response
        .data
        .into_iter()
        .map(|tweet| {
            let user = users.get(tweet.author_id.as_str());
            LoopTweet {
                id: tweet.id,
                text: tweet.text,
                author_id: tweet.author_id,
                author_username: user.map(|u| u.username.clone()).unwrap_or_default(),
                author_followers: user.map(|u| u.public_metrics.followers_count).unwrap_or(0),
                created_at: tweet.created_at,
                likes: tweet.public_metrics.like_count,
                retweets: tweet.public_metrics.retweet_count,
                replies: tweet.public_metrics.reply_count,
            }
        })
        .collect()
}

/// Map `ToolkitError` to `LoopError`.
pub(super) fn toolkit_to_loop_error(e: ToolkitError) -> LoopError {
    match e {
        ToolkitError::XApi(xe) => match xe {
            XApiError::RateLimited { retry_after } => LoopError::RateLimited { retry_after },
            XApiError::AuthExpired => LoopError::AuthExpired,
            XApiError::Network { source } => LoopError::NetworkError(source.to_string()),
            XApiError::ScraperMutationBlocked { .. }
            | XApiError::ScraperTransportUnavailable { .. }
            | XApiError::FeatureRequiresAuth { .. } => LoopError::Other(xe.to_string()),
            other => LoopError::Other(other.to_string()),
        },
        other => LoopError::Other(other.to_string()),
    }
}

/// Map `ToolkitError` to `ContentLoopError`.
pub(super) fn toolkit_to_content_error(e: ToolkitError) -> ContentLoopError {
    match e {
        ToolkitError::XApi(xe) => match xe {
            XApiError::RateLimited { retry_after } => ContentLoopError::PostFailed(format!(
                "rate limited{}",
                retry_after
                    .map(|s| format!(", retry after {s}s"))
                    .unwrap_or_default()
            )),
            XApiError::Network { source } => ContentLoopError::NetworkError(source.to_string()),
            other => ContentLoopError::PostFailed(other.to_string()),
        },
        other => ContentLoopError::PostFailed(other.to_string()),
    }
}

/// Map `ToolkitError` to `AnalyticsError`.
pub(super) fn toolkit_to_analytics_error(e: ToolkitError) -> AnalyticsError {
    AnalyticsError::ApiError(e.to_string())
}

/// Map `LlmError` to `LoopError`.
pub(super) fn llm_to_loop_error(e: LlmError) -> LoopError {
    LoopError::LlmFailure(e.to_string())
}

/// Map `LlmError` to `ContentLoopError`.
pub(super) fn llm_to_content_error(e: LlmError) -> ContentLoopError {
    ContentLoopError::LlmFailure(e.to_string())
}

/// Map `sqlx::Error` to `ContentLoopError`.
pub(super) fn sqlx_to_content_error(e: sqlx::Error) -> ContentLoopError {
    ContentLoopError::StorageError(e.to_string())
}

/// Map `StorageError` to `LoopError`.
pub(super) fn storage_to_loop_error(e: crate::error::StorageError) -> LoopError {
    LoopError::StorageError(e.to_string())
}

/// Parse a datetime string into `DateTime<Utc>`.
///
/// Tries RFC-3339 first, then `%Y-%m-%d %H:%M:%S` (SQLite `datetime()` format),
/// then `%Y-%m-%dT%H:%M:%SZ`.
pub(super) fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(naive.and_utc());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(naive.and_utc());
    }
    None
}
