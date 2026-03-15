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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn null_client_search_tweets() {
        let client = NullXApiClient;
        let result = client.search_tweets("query", 10, None, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{err}").contains("not configured"));
    }

    #[tokio::test]
    async fn null_client_get_mentions() {
        let client = NullXApiClient;
        let result = client.get_mentions("u1", None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_post_tweet() {
        let client = NullXApiClient;
        let result = client.post_tweet("text").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_reply_to_tweet() {
        let client = NullXApiClient;
        let result = client.reply_to_tweet("text", "123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_tweet() {
        let client = NullXApiClient;
        let result = client.get_tweet("123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_me() {
        let client = NullXApiClient;
        let result = client.get_me().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_user_tweets() {
        let client = NullXApiClient;
        let result = client.get_user_tweets("u1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_user_by_username() {
        let client = NullXApiClient;
        let result = client.get_user_by_username("testuser").await;
        assert!(result.is_err());
    }

    #[test]
    fn not_configured_error_message() {
        let err = not_configured();
        match err {
            XApiError::ApiError { status, message } => {
                assert_eq!(status, 0);
                assert!(message.contains("not configured"));
                assert!(message.contains("tuitbot auth"));
            }
            _ => panic!("expected ApiError"),
        }
    }

    // ── Default trait method coverage via NullXApiClient ──────────

    #[tokio::test]
    async fn null_client_upload_media() {
        let client = NullXApiClient;
        let result = client.upload_media(b"data", MediaType::Gif).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_post_tweet_with_media() {
        let client = NullXApiClient;
        let result = client
            .post_tweet_with_media("text", &["m1".to_string()])
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_reply_with_media() {
        let client = NullXApiClient;
        let result = client
            .reply_to_tweet_with_media("text", "123", &["m1".to_string()])
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_quote_tweet() {
        let client = NullXApiClient;
        let result = client.quote_tweet("text", "123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_like_tweet() {
        let client = NullXApiClient;
        let result = client.like_tweet("u1", "t1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_follow_user() {
        let client = NullXApiClient;
        let result = client.follow_user("u1", "u2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_unfollow_user() {
        let client = NullXApiClient;
        let result = client.unfollow_user("u1", "u2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_retweet() {
        let client = NullXApiClient;
        let result = client.retweet("u1", "t1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_unretweet() {
        let client = NullXApiClient;
        let result = client.unretweet("u1", "t1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_delete_tweet() {
        let client = NullXApiClient;
        let result = client.delete_tweet("t1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_home_timeline() {
        let client = NullXApiClient;
        let result = client.get_home_timeline("u1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_unlike_tweet() {
        let client = NullXApiClient;
        let result = client.unlike_tweet("u1", "t1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_followers() {
        let client = NullXApiClient;
        let result = client.get_followers("u1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_following() {
        let client = NullXApiClient;
        let result = client.get_following("u1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_user_by_id() {
        let client = NullXApiClient;
        let result = client.get_user_by_id("u1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_liked_tweets() {
        let client = NullXApiClient;
        let result = client.get_liked_tweets("u1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_bookmarks() {
        let client = NullXApiClient;
        let result = client.get_bookmarks("u1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_bookmark_tweet() {
        let client = NullXApiClient;
        let result = client.bookmark_tweet("u1", "t1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_unbookmark_tweet() {
        let client = NullXApiClient;
        let result = client.unbookmark_tweet("u1", "t1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_users_by_ids() {
        let client = NullXApiClient;
        let result = client.get_users_by_ids(&["u1", "u2"]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_get_tweet_liking_users() {
        let client = NullXApiClient;
        let result = client.get_tweet_liking_users("t1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn null_client_raw_request() {
        let client = NullXApiClient;
        let result = client.raw_request("GET", "/test", None, None, None).await;
        assert!(result.is_err());
    }

    // ── Error message consistency ─────────────────────────────────

    #[tokio::test]
    async fn all_null_client_errors_contain_auth_hint() {
        let client = NullXApiClient;
        let err_msg = format!("{}", client.post_tweet("test").await.unwrap_err());
        assert!(
            err_msg.contains("tuitbot auth"),
            "error should contain auth hint"
        );
    }
}
