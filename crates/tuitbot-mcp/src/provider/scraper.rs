//! Scraper-backed [`SocialReadProvider`] stub.
//!
//! All methods return placeholder errors — actual scraper integration is
//! deferred to a future session. Auth-gated methods (mentions, timeline,
//! me, bookmarks) return [`ProviderError::NotConfigured`]; public-data
//! methods return [`ProviderError::Other`] with a "not yet implemented" message.

use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;
use tuitbot_core::x_api::types::{MentionResponse, SearchResponse, Tweet, User, UsersResponse};

/// Scraper-based read provider (stub).
#[allow(dead_code)]
pub struct ScraperReadProvider;

impl ScraperReadProvider {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ScraperReadProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Message for methods that require authentication and cannot work with a scraper.
#[allow(dead_code)]
const AUTH_GATED_MSG: &str =
    "This method requires authentication and is not available via the scraper backend. \
     Switch to provider_backend = \"x_api\" in config.toml to use this feature.";

/// Message for public-data methods not yet implemented.
#[allow(dead_code)]
const STUB_MSG: &str = "Scraper backend: method not yet implemented.";

#[async_trait::async_trait]
impl SocialReadProvider for ScraperReadProvider {
    // ── Stub (future impl) ──────────────────────────────────────────

    async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<User, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_user_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_user_by_id(&self, _user_id: &str) -> Result<User, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_followers(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_following(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_liked_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_users_by_ids(&self, _user_ids: &[&str]) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    async fn get_tweet_liking_users(
        &self,
        _tweet_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Other {
            message: STUB_MSG.to_string(),
        })
    }

    // ── Auth-gated (rejected) ───────────────────────────────────────

    async fn get_user_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, ProviderError> {
        Err(ProviderError::NotConfigured {
            message: AUTH_GATED_MSG.to_string(),
        })
    }

    async fn get_home_timeline(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::NotConfigured {
            message: AUTH_GATED_MSG.to_string(),
        })
    }

    async fn get_me(&self) -> Result<User, ProviderError> {
        Err(ProviderError::NotConfigured {
            message: AUTH_GATED_MSG.to_string(),
        })
    }

    async fn get_bookmarks(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::NotConfigured {
            message: AUTH_GATED_MSG.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stub_methods_return_other_error() {
        let provider = ScraperReadProvider::new();
        let err = provider.get_tweet("123").await.unwrap_err();
        assert!(
            matches!(err, ProviderError::Other { .. }),
            "expected Other, got {err:?}"
        );

        let err = provider.get_user_by_username("u").await.unwrap_err();
        assert!(matches!(err, ProviderError::Other { .. }));

        let err = provider
            .search_tweets("q", 10, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, ProviderError::Other { .. }));
    }

    #[tokio::test]
    async fn auth_gated_methods_return_not_configured() {
        let provider = ScraperReadProvider::new();

        let err = provider
            .get_user_mentions("u1", None, None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, ProviderError::NotConfigured { .. }),
            "expected NotConfigured, got {err:?}"
        );

        let err = provider
            .get_home_timeline("u1", 10, None)
            .await
            .unwrap_err();
        assert!(matches!(err, ProviderError::NotConfigured { .. }));

        let err = provider.get_me().await.unwrap_err();
        assert!(matches!(err, ProviderError::NotConfigured { .. }));

        let err = provider.get_bookmarks("u1", 10, None).await.unwrap_err();
        assert!(matches!(err, ProviderError::NotConfigured { .. }));
    }
}
