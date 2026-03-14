//! Mock implementation of [`XApiClient`] for unit tests.
//!
//! Returns configurable canned responses without making any network requests.
//! Thread-safe via `Arc<Mutex<_>>` internals — safe to use in async tests.
//!
//! # Example
//! ```rust
//! use tuitbot_core::testing::{MockXClient, TweetFactory};
//!
//! let client = MockXClient::new();
//! client.set_search_tweets(TweetFactory::build_many(5));
//!
//! // The mock can also be set up to fail:
//! let failing = MockXClient::new().with_search_error();
//! ```

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::error::XApiError;
use crate::x_api::types::{
    MediaId, MediaType, MentionResponse, PostedTweet, SearchMeta, SearchResponse, Tweet, User,
    UserMetrics, UsersMeta, UsersResponse,
};
use crate::x_api::XApiClient;

// ---------------------------------------------------------------------------
// Internal call record
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PostedTweetRecord {
    pub text: String,
    pub in_reply_to: Option<String>,
}

// ---------------------------------------------------------------------------
// MockXClient
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Inner {
    search_tweets: Option<Vec<Tweet>>,
    search_error: bool,
    me: Option<User>,
    post_error: bool,
    posted_tweets: Vec<PostedTweetRecord>,
}

/// Mock [`XApiClient`] that returns canned responses.
#[derive(Clone, Default)]
pub struct MockXClient {
    inner: Arc<Mutex<Inner>>,
}

impl MockXClient {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Setup helpers -------------------------------------------------------

    /// Pre-load tweets to return from `search_tweets` and `get_mentions`.
    pub fn set_search_tweets(&self, tweets: Vec<Tweet>) -> &Self {
        self.inner.lock().unwrap().search_tweets = Some(tweets);
        self
    }

    /// Cause `search_tweets` to return a rate-limit error.
    pub fn with_search_error(self) -> Self {
        self.inner.lock().unwrap().search_error = true;
        self
    }

    /// Pre-load the authenticated user to return from `get_me`.
    pub fn set_me(&self, user: User) -> &Self {
        self.inner.lock().unwrap().me = Some(user);
        self
    }

    /// Cause `post_tweet` and `reply_to_tweet` to return an error.
    pub fn with_post_error(self) -> Self {
        self.inner.lock().unwrap().post_error = true;
        self
    }

    // --- Assertion helpers ---------------------------------------------------

    /// Returns all tweets that were posted via `post_tweet` or `reply_to_tweet`.
    pub fn posted_tweets(&self) -> Vec<PostedTweetRecord> {
        self.inner.lock().unwrap().posted_tweets.clone()
    }

    /// Returns the number of tweets posted so far.
    pub fn post_count(&self) -> usize {
        self.inner.lock().unwrap().posted_tweets.len()
    }
}

// ---------------------------------------------------------------------------
// XApiClient implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl XApiClient for MockXClient {
    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let inner = self.inner.lock().unwrap();
        if inner.search_error {
            return Err(XApiError::RateLimited {
                retry_after: Some(60),
            });
        }
        let tweets = inner.search_tweets.clone().unwrap_or_default();
        Ok(SearchResponse {
            data: tweets,
            includes: None,
            meta: SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 0,
                next_token: None,
            },
        })
    }

    async fn get_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        self.search_tweets("", 10, None, None).await
    }

    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        let mut inner = self.inner.lock().unwrap();
        if inner.post_error {
            return Err(XApiError::Forbidden {
                message: "post_tweet: mock error".to_string(),
            });
        }
        inner.posted_tweets.push(PostedTweetRecord {
            text: text.to_string(),
            in_reply_to: None,
        });
        Ok(PostedTweet {
            id: format!("mock-tweet-{}", inner.posted_tweets.len()),
            text: text.to_string(),
        })
    }

    async fn reply_to_tweet(
        &self,
        text: &str,
        in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        let mut inner = self.inner.lock().unwrap();
        if inner.post_error {
            return Err(XApiError::Forbidden {
                message: "reply_to_tweet: mock error".to_string(),
            });
        }
        inner.posted_tweets.push(PostedTweetRecord {
            text: text.to_string(),
            in_reply_to: Some(in_reply_to_id.to_string()),
        });
        Ok(PostedTweet {
            id: format!("mock-reply-{}", inner.posted_tweets.len()),
            text: text.to_string(),
        })
    }

    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, XApiError> {
        use crate::testing::TweetFactory;
        Ok(TweetFactory::new().with_id(tweet_id).build())
    }

    async fn get_me(&self) -> Result<User, XApiError> {
        let inner = self.inner.lock().unwrap();
        Ok(inner.me.clone().unwrap_or_else(default_user))
    }

    async fn get_user_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        self.search_tweets("", 10, None, None).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, XApiError> {
        Ok(User {
            id: "mock-user-id".to_string(),
            username: username.to_string(),
            name: format!("Mock {username}"),
            profile_image_url: None,
            description: None,
            location: None,
            url: None,
            public_metrics: UserMetrics {
                followers_count: 1000,
                following_count: 200,
                tweet_count: 500,
            },
        })
    }

    async fn upload_media(
        &self,
        _data: &[u8],
        _media_type: MediaType,
    ) -> Result<MediaId, XApiError> {
        Ok(MediaId("mock-media-id-1234".to_string()))
    }

    async fn get_followers(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        Ok(UsersResponse {
            data: vec![],
            meta: UsersMeta {
                result_count: 0,
                next_token: None,
            },
        })
    }

    async fn get_following(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        Ok(UsersResponse {
            data: vec![],
            meta: UsersMeta {
                result_count: 0,
                next_token: None,
            },
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_user() -> User {
    User {
        id: "mock-me-id".to_string(),
        username: "mock_me".to_string(),
        name: "Mock Me".to_string(),
        profile_image_url: None,
        description: None,
        location: None,
        url: None,
        public_metrics: UserMetrics {
            followers_count: 500,
            following_count: 100,
            tweet_count: 200,
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TweetFactory;

    #[tokio::test]
    async fn search_returns_preloaded_tweets() {
        let client = MockXClient::new();
        client.set_search_tweets(TweetFactory::build_many(3));

        let resp = client.search_tweets("test", 10, None, None).await.unwrap();
        assert_eq!(resp.data.len(), 3);
    }

    #[tokio::test]
    async fn search_error_returns_rate_limited() {
        let client = MockXClient::new().with_search_error();
        let err = client
            .search_tweets("test", 10, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, XApiError::RateLimited { .. }));
    }

    #[tokio::test]
    async fn post_tweet_records_call() {
        let client = MockXClient::new();
        client.post_tweet("Hello world").await.unwrap();
        client.post_tweet("Second tweet").await.unwrap();
        assert_eq!(client.post_count(), 2);
        assert_eq!(client.posted_tweets()[0].text, "Hello world");
    }

    #[tokio::test]
    async fn reply_records_in_reply_to() {
        let client = MockXClient::new();
        client
            .reply_to_tweet("Nice thread!", "root-id-123")
            .await
            .unwrap();
        let records = client.posted_tweets();
        assert_eq!(records[0].in_reply_to, Some("root-id-123".to_string()));
    }

    #[tokio::test]
    async fn post_error_returns_forbidden() {
        let client = MockXClient::new().with_post_error();
        let err = client.post_tweet("Blocked").await.unwrap_err();
        assert!(matches!(err, XApiError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn get_me_returns_custom_user() {
        let client = MockXClient::new();
        client.set_me(User {
            id: "custom-id".to_string(),
            username: "custom_user".to_string(),
            name: "Custom".to_string(),
            profile_image_url: None,
            description: None,
            location: None,
            url: None,
            public_metrics: UserMetrics {
                followers_count: 9999,
                following_count: 1,
                tweet_count: 1,
            },
        });
        let me = client.get_me().await.unwrap();
        assert_eq!(me.username, "custom_user");
        assert_eq!(me.public_metrics.followers_count, 9999);
    }
}
