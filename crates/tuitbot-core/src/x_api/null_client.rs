//! Null X API client that returns errors for all operations.
//!
//! Used when X API tokens are not available, allowing MCP profiles
//! to start in degraded mode with non-X tools still functional.

use crate::error::XApiError;
use crate::x_api::types::*;
use crate::x_api::XApiClient;

const NOT_CONFIGURED: &str = "X API client not configured. Run `tuitbot auth` to authenticate.";

/// A no-op X API client that returns [`XApiError::ApiError`] for every call.
///
/// Injected when tokens are missing or expired so that the MCP server can
/// still start and serve config/scoring tools.
pub struct NullXApiClient;

#[async_trait::async_trait]
impl XApiClient for NullXApiClient {
    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(not_configured())
    }

    async fn get_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        Err(not_configured())
    }

    async fn post_tweet(&self, _text: &str) -> Result<PostedTweet, XApiError> {
        Err(not_configured())
    }

    async fn reply_to_tweet(
        &self,
        _text: &str,
        _in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        Err(not_configured())
    }

    async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, XApiError> {
        Err(not_configured())
    }

    async fn get_me(&self) -> Result<User, XApiError> {
        Err(not_configured())
    }

    async fn get_user_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(not_configured())
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<User, XApiError> {
        Err(not_configured())
    }
}

fn not_configured() -> XApiError {
    XApiError::ApiError {
        status: 0,
        message: NOT_CONFIGURED.to_string(),
    }
}
