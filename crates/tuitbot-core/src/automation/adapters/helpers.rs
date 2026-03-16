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

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Timelike;

    // --- parse_datetime ---

    #[test]
    fn parse_datetime_rfc3339() {
        let dt = parse_datetime("2026-03-15T10:30:00+00:00");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().hour(), 10);
    }

    #[test]
    fn parse_datetime_rfc3339_with_offset() {
        let dt = parse_datetime("2026-03-15T10:30:00-05:00");
        assert!(dt.is_some());
        // -05:00 → UTC is 15:30
        assert_eq!(dt.unwrap().hour(), 15);
    }

    #[test]
    fn parse_datetime_sqlite_format() {
        let dt = parse_datetime("2026-03-15 10:30:00");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().hour(), 10);
    }

    #[test]
    fn parse_datetime_iso_z_format() {
        let dt = parse_datetime("2026-03-15T10:30:00Z");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().hour(), 10);
    }

    #[test]
    fn parse_datetime_invalid_returns_none() {
        assert!(parse_datetime("not-a-date").is_none());
        assert!(parse_datetime("").is_none());
        assert!(parse_datetime("2026/03/15").is_none());
    }

    // --- search_response_to_loop_tweets ---

    #[test]
    fn search_response_to_loop_tweets_empty() {
        let response = SearchResponse {
            data: vec![],
            includes: None,
            meta: crate::x_api::types::SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 0,
                next_token: None,
            },
        };
        let tweets = search_response_to_loop_tweets(response);
        assert!(tweets.is_empty());
    }

    #[test]
    fn search_response_to_loop_tweets_with_includes() {
        use crate::x_api::types::*;

        let response = SearchResponse {
            data: vec![Tweet {
                id: "t1".into(),
                text: "hello world".into(),
                author_id: "u1".into(),
                created_at: "2026-03-15T10:00:00Z".into(),
                public_metrics: PublicMetrics {
                    like_count: 5,
                    retweet_count: 2,
                    reply_count: 1,
                    impression_count: 100,
                    ..Default::default()
                },
                conversation_id: None,
            }],
            includes: Some(Includes {
                users: vec![User {
                    id: "u1".into(),
                    username: "alice".into(),
                    name: "Alice".into(),
                    profile_image_url: None,
                    description: None,
                    location: None,
                    url: None,
                    public_metrics: UserMetrics {
                        followers_count: 1000,
                        following_count: 500,
                        tweet_count: 200,
                    },
                }],
            }),
            meta: SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 1,
                next_token: None,
            },
        };
        let tweets = search_response_to_loop_tweets(response);
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0].author_username, "alice");
        assert_eq!(tweets[0].author_followers, 1000);
        assert_eq!(tweets[0].likes, 5);
        assert_eq!(tweets[0].retweets, 2);
    }

    #[test]
    fn search_response_to_loop_tweets_without_includes() {
        use crate::x_api::types::*;

        let response = SearchResponse {
            data: vec![Tweet {
                id: "t1".into(),
                text: "hello".into(),
                author_id: "u1".into(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 1,
                next_token: None,
            },
        };
        let tweets = search_response_to_loop_tweets(response);
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0].author_username, "");
        assert_eq!(tweets[0].author_followers, 0);
    }

    // --- error mapping functions ---

    #[test]
    fn toolkit_to_loop_error_rate_limited() {
        let err = toolkit_to_loop_error(ToolkitError::XApi(crate::error::XApiError::RateLimited {
            retry_after: Some(30),
        }));
        assert!(matches!(
            err,
            LoopError::RateLimited {
                retry_after: Some(30)
            }
        ));
    }

    #[test]
    fn toolkit_to_loop_error_auth_expired() {
        let err = toolkit_to_loop_error(ToolkitError::XApi(crate::error::XApiError::AuthExpired));
        assert!(matches!(err, LoopError::AuthExpired));
    }

    #[test]
    fn toolkit_to_loop_error_other() {
        let err = toolkit_to_loop_error(ToolkitError::XApi(crate::error::XApiError::ApiError {
            status: 500,
            message: "internal".into(),
        }));
        assert!(matches!(err, LoopError::Other(_)));
    }

    #[test]
    fn toolkit_to_content_error_rate_limited() {
        let err =
            toolkit_to_content_error(ToolkitError::XApi(crate::error::XApiError::RateLimited {
                retry_after: Some(60),
            }));
        match err {
            ContentLoopError::PostFailed(msg) => {
                assert!(msg.contains("rate limited"));
                assert!(msg.contains("60"));
            }
            other => panic!("expected PostFailed, got: {other:?}"),
        }
    }

    #[test]
    fn toolkit_to_analytics_error_wraps_message() {
        let err =
            toolkit_to_analytics_error(ToolkitError::XApi(crate::error::XApiError::ApiError {
                status: 404,
                message: "not found".into(),
            }));
        match err {
            AnalyticsError::ApiError(msg) => assert!(msg.contains("not found")),
            other => panic!("expected ApiError, got: {other:?}"),
        }
    }

    #[test]
    fn llm_to_loop_error_wraps() {
        let err = llm_to_loop_error(crate::error::LlmError::GenerationFailed("bad".into()));
        assert!(matches!(err, LoopError::LlmFailure(_)));
    }

    #[test]
    fn llm_to_content_error_wraps() {
        let err = llm_to_content_error(crate::error::LlmError::GenerationFailed("bad".into()));
        assert!(matches!(err, ContentLoopError::LlmFailure(_)));
    }

    #[test]
    fn sqlx_to_content_error_wraps() {
        let err = sqlx_to_content_error(sqlx::Error::RowNotFound);
        assert!(matches!(err, ContentLoopError::StorageError(_)));
    }

    #[test]
    fn storage_to_loop_error_wraps() {
        let err = storage_to_loop_error(crate::error::StorageError::Query {
            source: sqlx::Error::RowNotFound,
        });
        assert!(matches!(err, LoopError::StorageError(_)));
    }

    use chrono::TimeZone;

    #[test]
    fn parse_datetime_returns_utc() {
        let dt = parse_datetime("2026-01-01T00:00:00Z").unwrap();
        assert_eq!(dt, Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap());
    }
}
