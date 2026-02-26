//! Provider layer: backend-agnostic trait for social platform operations.
//!
//! [`SocialReadProvider`] defines the read surface that kernel tools depend on.
//! Concrete implementations live in submodules (e.g. [`x_api::XApiProvider`]).

pub mod x_api;

use crate::contract::ProviderError;
use tuitbot_core::x_api::types::{MentionResponse, SearchResponse, Tweet, User, UsersResponse};

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
