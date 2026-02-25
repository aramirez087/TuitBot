//! X API v2 client, authentication, and tier detection.
//!
//! Provides a trait-based client abstraction for all X API operations,
//! OAuth 2.0 PKCE authentication with token management, and API tier
//! detection for adaptive behavior.

pub mod auth;
pub mod client;
pub mod media;
pub mod scopes;
pub mod tier;
pub mod types;

pub use client::XApiHttpClient;
pub use types::*;

use crate::error::XApiError;

/// Trait abstracting all X API v2 operations.
///
/// Implementations include `XApiHttpClient` for real API calls and
/// mock implementations for testing.
#[async_trait::async_trait]
pub trait XApiClient: Send + Sync {
    /// Search recent tweets matching the given query.
    ///
    /// Returns up to `max_results` tweets. If `since_id` is provided,
    /// only returns tweets newer than that ID.
    async fn search_tweets(
        &self,
        query: &str,
        max_results: u32,
        since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError>;

    /// Get mentions for the authenticated user.
    ///
    /// If `since_id` is provided, only returns mentions newer than that ID.
    async fn get_mentions(
        &self,
        user_id: &str,
        since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<MentionResponse, XApiError>;

    /// Post a new tweet.
    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError>;

    /// Reply to an existing tweet.
    async fn reply_to_tweet(
        &self,
        text: &str,
        in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError>;

    /// Get a single tweet by ID.
    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, XApiError>;

    /// Get the authenticated user's profile.
    async fn get_me(&self) -> Result<User, XApiError>;

    /// Get recent tweets from a specific user.
    async fn get_user_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError>;

    /// Look up a user by their username.
    async fn get_user_by_username(&self, username: &str) -> Result<User, XApiError>;

    /// Upload media to X API for attaching to tweets.
    ///
    /// Default implementation returns an error — override in concrete clients.
    async fn upload_media(
        &self,
        _data: &[u8],
        _media_type: MediaType,
    ) -> Result<MediaId, XApiError> {
        Err(XApiError::MediaUploadError {
            message: "upload_media not implemented".to_string(),
        })
    }

    /// Post a new tweet with media attachments.
    ///
    /// Default delegates to `post_tweet` (ignoring media) for backward compat.
    async fn post_tweet_with_media(
        &self,
        text: &str,
        _media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        self.post_tweet(text).await
    }

    /// Reply to an existing tweet with media attachments.
    ///
    /// Default delegates to `reply_to_tweet` (ignoring media) for backward compat.
    async fn reply_to_tweet_with_media(
        &self,
        text: &str,
        in_reply_to_id: &str,
        _media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        self.reply_to_tweet(text, in_reply_to_id).await
    }

    /// Post a quote tweet referencing another tweet.
    async fn quote_tweet(
        &self,
        _text: &str,
        _quoted_tweet_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }

    /// Like a tweet on behalf of the authenticated user.
    async fn like_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }

    /// Follow a user on behalf of the authenticated user.
    async fn follow_user(&self, _user_id: &str, _target_user_id: &str) -> Result<bool, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }

    /// Unfollow a user on behalf of the authenticated user.
    async fn unfollow_user(
        &self,
        _user_id: &str,
        _target_user_id: &str,
    ) -> Result<bool, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }

    /// Retweet a tweet on behalf of the authenticated user.
    async fn retweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }

    /// Undo a retweet on behalf of the authenticated user.
    async fn unretweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }

    /// Delete a tweet by its ID.
    async fn delete_tweet(&self, _tweet_id: &str) -> Result<bool, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }

    /// Get the authenticated user's home timeline (reverse chronological).
    async fn get_home_timeline(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::ApiError {
            status: 0,
            message: "not implemented".to_string(),
        })
    }
}
