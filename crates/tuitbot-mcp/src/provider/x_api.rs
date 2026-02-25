//! X API provider: adapts `dyn XApiClient` to [`SocialReadProvider`].

use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;
use tuitbot_core::error::XApiError;
use tuitbot_core::x_api::types::{SearchResponse, Tweet, User};
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
}

/// Map an [`XApiError`] to a [`ProviderError`].
fn map_x_error(e: &XApiError) -> ProviderError {
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
