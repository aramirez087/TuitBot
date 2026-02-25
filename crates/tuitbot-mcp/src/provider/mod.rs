//! Provider layer: backend-agnostic trait for social platform operations.
//!
//! [`SocialReadProvider`] defines the read surface that kernel tools depend on.
//! Concrete implementations live in submodules (e.g. [`x_api::XApiProvider`]).

pub mod x_api;

use crate::contract::ProviderError;
use tuitbot_core::x_api::types::{SearchResponse, Tweet, User};

/// Read-only social platform operations.
///
/// Kernel tools program against this trait, allowing the backend to be
/// swapped (official X API, scraper, mock) without changing tool logic.
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
}
