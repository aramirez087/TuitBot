//! Provider layer: backend-agnostic trait for social platform operations.
//!
//! [`SocialReadProvider`] defines the read surface that kernel tools depend on.
//! Concrete implementations live in submodules (e.g. [`x_api::XApiProvider`]).

pub mod capabilities;
pub mod retry;
pub mod scraper;
pub mod x_api;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::contract::ProviderError;
use tuitbot_core::x_api::types::{MentionResponse, SearchResponse, Tweet, User, UsersResponse};

/// Backend used for social platform operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderBackend {
    /// Official X API via OAuth 2.0.
    #[default]
    XApi,
    /// Scraper-based backend (elevated risk, read-heavy).
    Scraper,
}

impl fmt::Display for ProviderBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XApi => write!(f, "x_api"),
            Self::Scraper => write!(f, "scraper"),
        }
    }
}

/// Parse a config string into a [`ProviderBackend`].
///
/// Returns `XApi` for empty/unknown strings to preserve backwards compatibility.
pub fn parse_backend(s: &str) -> ProviderBackend {
    match s.to_ascii_lowercase().as_str() {
        "scraper" => ProviderBackend::Scraper,
        _ => ProviderBackend::XApi,
    }
}

/// Post-process a tool response JSON string to inject `provider_backend`
/// into the `meta` object (if present) or add a minimal `meta` object.
#[allow(dead_code)]
pub fn inject_provider_backend(json: &str, backend: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(mut v) => {
            if let Some(meta) = v.get_mut("meta").and_then(|m| m.as_object_mut()) {
                meta.insert(
                    "provider_backend".to_string(),
                    serde_json::Value::String(backend.to_string()),
                );
            } else {
                v["meta"] = serde_json::json!({ "provider_backend": backend });
            }
            serde_json::to_string_pretty(&v).unwrap_or_else(|_| json.to_string())
        }
        Err(_) => json.to_string(),
    }
}

/// Read-only social platform operations.
///
/// Kernel tools program against this trait, allowing the backend to be
/// swapped (official X API, scraper, mock) without changing tool logic.
///
/// New methods have default implementations that return `ProviderError::Other`
/// so existing mock providers (e.g. in kernel tests) don't break.
#[async_trait::async_trait]
pub trait SocialReadProvider: Send + Sync {
    /// Fetch a single post by ID.
    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError>;

    /// Look up a user by username.
    async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError>;

    /// Search recent posts matching a query.
    async fn search_tweets(
        &self,
        query: &str,
        max_results: u32,
        since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError>;

    /// Get mentions for a user.
    async fn get_user_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_user_mentions not implemented by this provider".to_string(),
        })
    }

    /// Get recent tweets from a specific user.
    async fn get_user_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_user_tweets not implemented by this provider".to_string(),
        })
    }

    /// Get the authenticated user's home timeline.
    async fn get_home_timeline(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_home_timeline not implemented by this provider".to_string(),
        })
    }

    /// Get the authenticated user's profile.
    async fn get_me(&self) -> Result<User, ProviderError> {
        Err(ProviderError::Other {
            message: "get_me not implemented by this provider".to_string(),
        })
    }

    /// Get followers of a user.
    async fn get_followers(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_followers not implemented by this provider".to_string(),
        })
    }

    /// Get accounts a user is following.
    async fn get_following(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_following not implemented by this provider".to_string(),
        })
    }

    /// Get a user by their ID.
    async fn get_user_by_id(&self, _user_id: &str) -> Result<User, ProviderError> {
        Err(ProviderError::Other {
            message: "get_user_by_id not implemented by this provider".to_string(),
        })
    }

    /// Get tweets liked by a user.
    async fn get_liked_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_liked_tweets not implemented by this provider".to_string(),
        })
    }

    /// Get the authenticated user's bookmarks.
    async fn get_bookmarks(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_bookmarks not implemented by this provider".to_string(),
        })
    }

    /// Get multiple users by their IDs.
    async fn get_users_by_ids(&self, _user_ids: &[&str]) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_users_by_ids not implemented by this provider".to_string(),
        })
    }

    /// Get users who liked a specific tweet.
    async fn get_tweet_liking_users(
        &self,
        _tweet_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: "get_tweet_liking_users not implemented by this provider".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_backend_default_is_x_api() {
        assert_eq!(ProviderBackend::default(), ProviderBackend::XApi);
    }

    #[test]
    fn provider_backend_display() {
        assert_eq!(ProviderBackend::XApi.to_string(), "x_api");
        assert_eq!(ProviderBackend::Scraper.to_string(), "scraper");
    }

    #[test]
    fn provider_backend_serde_roundtrip() {
        for backend in [ProviderBackend::XApi, ProviderBackend::Scraper] {
            let json = serde_json::to_string(&backend).unwrap();
            let back: ProviderBackend = serde_json::from_str(&json).unwrap();
            assert_eq!(back, backend);
        }
    }

    #[test]
    fn parse_backend_known_values() {
        assert_eq!(parse_backend("x_api"), ProviderBackend::XApi);
        assert_eq!(parse_backend("scraper"), ProviderBackend::Scraper);
        assert_eq!(parse_backend("SCRAPER"), ProviderBackend::Scraper);
        assert_eq!(parse_backend(""), ProviderBackend::XApi);
        assert_eq!(parse_backend("unknown"), ProviderBackend::XApi);
    }

    #[test]
    fn inject_provider_backend_with_meta() {
        let input = r#"{"success":true,"data":{},"meta":{"tool_version":"1.0","elapsed_ms":5}}"#;
        let result = inject_provider_backend(input, "x_api");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["meta"]["provider_backend"], "x_api");
        assert_eq!(parsed["meta"]["tool_version"], "1.0");
    }

    #[test]
    fn inject_provider_backend_without_meta() {
        let input = r#"{"success":true,"data":{}}"#;
        let result = inject_provider_backend(input, "scraper");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["meta"]["provider_backend"], "scraper");
    }

    #[test]
    fn inject_provider_backend_invalid_json() {
        let input = "not json";
        let result = inject_provider_backend(input, "x_api");
        assert_eq!(result, "not json");
    }
}
