//! Provider-agnostic read tools.
//!
//! Each function takes a [`SocialReadProvider`] reference and returns a
//! JSON-encoded [`ToolResponse`] envelope. No `AppState` or DB access.

use std::time::Instant;

use crate::contract::envelope::{ToolMeta, ToolResponse};
use crate::contract::error::provider_error_to_response;
use crate::provider::SocialReadProvider;

/// Get a single tweet by ID via the provider.
pub async fn get_tweet(provider: &dyn SocialReadProvider, tweet_id: &str) -> String {
    let start = Instant::now();
    match provider.get_tweet(tweet_id).await {
        Ok(tweet) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => provider_error_to_response(&e, start),
    }
}

/// Look up a user by username via the provider.
pub async fn get_user_by_username(provider: &dyn SocialReadProvider, username: &str) -> String {
    let start = Instant::now();
    match provider.get_user_by_username(username).await {
        Ok(user) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&user)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => provider_error_to_response(&e, start),
    }
}

/// Search recent tweets via the provider.
pub async fn search_tweets(
    provider: &dyn SocialReadProvider,
    query: &str,
    max_results: u32,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .search_tweets(query, max_results, since_id, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => provider_error_to_response(&e, start),
    }
}
