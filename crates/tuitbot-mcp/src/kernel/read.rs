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

/// Get mentions for a user via the provider.
pub async fn get_user_mentions(
    provider: &dyn SocialReadProvider,
    user_id: &str,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_user_mentions(user_id, since_id, pagination_token)
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

/// Get recent tweets from a specific user via the provider.
pub async fn get_user_tweets(
    provider: &dyn SocialReadProvider,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_user_tweets(user_id, max_results, pagination_token)
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

/// Get the authenticated user's home timeline via the provider.
pub async fn get_home_timeline(
    provider: &dyn SocialReadProvider,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_home_timeline(user_id, max_results, pagination_token)
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

/// Get followers of a user via the provider.
pub async fn get_followers(
    provider: &dyn SocialReadProvider,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_followers(user_id, max_results, pagination_token)
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

/// Get accounts a user is following via the provider.
pub async fn get_following(
    provider: &dyn SocialReadProvider,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_following(user_id, max_results, pagination_token)
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

/// Get a user by their ID via the provider.
pub async fn get_user_by_id(provider: &dyn SocialReadProvider, user_id: &str) -> String {
    let start = Instant::now();
    match provider.get_user_by_id(user_id).await {
        Ok(user) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&user)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => provider_error_to_response(&e, start),
    }
}

/// Get tweets liked by a user via the provider.
pub async fn get_liked_tweets(
    provider: &dyn SocialReadProvider,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_liked_tweets(user_id, max_results, pagination_token)
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

/// Get the authenticated user's bookmarks via the provider.
pub async fn get_bookmarks(
    provider: &dyn SocialReadProvider,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_bookmarks(user_id, max_results, pagination_token)
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

/// Get multiple users by their IDs via the provider.
pub async fn get_users_by_ids(provider: &dyn SocialReadProvider, user_ids: &[&str]) -> String {
    let start = Instant::now();
    if user_ids.is_empty() || user_ids.len() > 100 {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::error(
            "invalid_input",
            format!("user_ids must contain 1-100 IDs, got {}", user_ids.len()),
            false,
        )
        .with_meta(ToolMeta::new(elapsed))
        .to_json();
    }
    match provider.get_users_by_ids(user_ids).await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => provider_error_to_response(&e, start),
    }
}

/// Get users who liked a specific tweet via the provider.
pub async fn get_tweet_liking_users(
    provider: &dyn SocialReadProvider,
    tweet_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    match provider
        .get_tweet_liking_users(tweet_id, max_results, pagination_token)
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
