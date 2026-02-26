//! X API provider: adapts `dyn XApiClient` to [`SocialReadProvider`].

use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;
use tuitbot_core::error::XApiError;
use tuitbot_core::x_api::types::{MentionResponse, SearchResponse, Tweet, User, UsersResponse};
use tuitbot_core::x_api::XApiClient;

/// Wraps a `dyn XApiClient` reference to implement [`SocialReadProvider`].
pub struct XApiProvider<'a> {
    client: &'a dyn XApiClient,
}

impl<'a> XApiProvider<'a> {
    pub fn new(client: &'a dyn XApiClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl SocialReadProvider for XApiProvider<'_> {
    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError> {
        self.client
            .get_tweet(tweet_id)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError> {
        self.client
            .get_user_by_username(username)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn search_tweets(
        &self,
        query: &str,
        max_results: u32,
        since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        self.client
            .search_tweets(query, max_results, since_id, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_user_mentions(
        &self,
        user_id: &str,
        since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<MentionResponse, ProviderError> {
        self.client
            .get_mentions(user_id, since_id, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_user_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        self.client
            .get_user_tweets(user_id, max_results, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_home_timeline(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        self.client
            .get_home_timeline(user_id, max_results, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_me(&self) -> Result<User, ProviderError> {
        self.client.get_me().await.map_err(|e| map_x_error(&e))
    }

    async fn get_followers(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        self.client
            .get_followers(user_id, max_results, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_following(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        self.client
            .get_following(user_id, max_results, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<User, ProviderError> {
        self.client
            .get_user_by_id(user_id)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_liked_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        self.client
            .get_liked_tweets(user_id, max_results, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_bookmarks(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        self.client
            .get_bookmarks(user_id, max_results, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_users_by_ids(&self, user_ids: &[&str]) -> Result<UsersResponse, ProviderError> {
        self.client
            .get_users_by_ids(user_ids)
            .await
            .map_err(|e| map_x_error(&e))
    }

    async fn get_tweet_liking_users(
        &self,
        tweet_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        self.client
            .get_tweet_liking_users(tweet_id, max_results, pagination_token)
            .await
            .map_err(|e| map_x_error(&e))
    }
}

/// Map an [`XApiError`] to a [`ProviderError`].
///
/// Visible within the crate so kernel write/engage functions can reuse it.
pub(crate) fn map_x_error(e: &XApiError) -> ProviderError {
    match e {
        XApiError::RateLimited { retry_after } => ProviderError::RateLimited {
            retry_after: *retry_after,
        },
        XApiError::AuthExpired => ProviderError::AuthExpired,
        XApiError::Forbidden { message } => ProviderError::Forbidden {
            message: message.clone(),
        },
        XApiError::AccountRestricted { message } => ProviderError::AccountRestricted {
            message: message.clone(),
        },
        XApiError::Network { source } => ProviderError::Network {
            message: source.to_string(),
        },
        other => ProviderError::Other {
            message: other.to_string(),
        },
    }
}
