//! Read-only X API tools.
//!
//! All read tools delegate to the kernel layer via a [`SocialReadProvider`],
//! except `get_x_usage` which requires DB access (workflow-only).

use std::time::Instant;

use crate::kernel;
use crate::provider::x_api::XApiProvider;
use crate::state::SharedState;

use super::super::response::{ToolMeta, ToolResponse};
use super::{no_user_id_response, not_configured_response};

/// Get a single tweet by ID — delegates to kernel.
pub async fn get_tweet_by_id(state: &SharedState, tweet_id: &str) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_tweet(&provider, tweet_id).await
}

/// Look up a user by username — delegates to kernel.
pub async fn get_user_by_username(state: &SharedState, username: &str) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_user_by_username(&provider, username).await
}

/// Search recent tweets — delegates to kernel.
pub async fn search_tweets(
    state: &SharedState,
    query: &str,
    max_results: u32,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::search_tweets(&provider, query, max_results, since_id, pagination_token).await
}

/// Get mentions for the authenticated user — delegates to kernel.
pub async fn get_user_mentions(
    state: &SharedState,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_user_mentions(&provider, user_id, since_id, pagination_token).await
}

/// Get recent tweets from a specific user — delegates to kernel.
pub async fn get_user_tweets(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_user_tweets(&provider, user_id, max_results, pagination_token).await
}

/// Get the authenticated user's home timeline — delegates to kernel.
pub async fn get_home_timeline(
    state: &SharedState,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_home_timeline(&provider, user_id, max_results, pagination_token).await
}

/// Get followers of a user — delegates to kernel.
pub async fn get_followers(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_followers(&provider, user_id, max_results, pagination_token).await
}

/// Get accounts a user is following — delegates to kernel.
pub async fn get_following(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_following(&provider, user_id, max_results, pagination_token).await
}

/// Get a user by their ID — delegates to kernel.
pub async fn get_user_by_id(state: &SharedState, user_id: &str) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_user_by_id(&provider, user_id).await
}

/// Get tweets liked by a user — delegates to kernel.
pub async fn get_liked_tweets(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_liked_tweets(&provider, user_id, max_results, pagination_token).await
}

/// Get the authenticated user's bookmarks — delegates to kernel.
pub async fn get_bookmarks(
    state: &SharedState,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_bookmarks(&provider, user_id, max_results, pagination_token).await
}

/// Get multiple users by their IDs — delegates to kernel.
pub async fn get_users_by_ids(state: &SharedState, user_ids: &[&str]) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_users_by_ids(&provider, user_ids).await
}

/// Get users who liked a specific tweet — delegates to kernel.
pub async fn get_tweet_liking_users(
    state: &SharedState,
    tweet_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(Instant::now()),
    };
    let provider = XApiProvider::new(client);
    kernel::read::get_tweet_liking_users(&provider, tweet_id, max_results, pagination_token).await
}

/// Get X API usage statistics.
pub async fn get_x_usage(state: &SharedState, days: u32) -> String {
    let start = Instant::now();

    let summary = match tuitbot_core::storage::x_api_usage::get_usage_summary(&state.pool).await {
        Ok(s) => s,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "db_error",
                format!("Failed to get usage summary: {e}"),
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    let daily = match tuitbot_core::storage::x_api_usage::get_daily_usage(&state.pool, days).await {
        Ok(d) => d,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "db_error",
                format!("Failed to get daily usage: {e}"),
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    let endpoints =
        match tuitbot_core::storage::x_api_usage::get_endpoint_breakdown(&state.pool, days).await {
            Ok(b) => b,
            Err(e) => {
                let elapsed = start.elapsed().as_millis() as u64;
                return ToolResponse::error(
                    "db_error",
                    format!("Failed to get endpoint breakdown: {e}"),
                    false,
                )
                .with_meta(ToolMeta::new(elapsed))
                .to_json();
            }
        };

    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::success(serde_json::json!({
        "summary": summary,
        "daily": daily,
        "endpoints": endpoints,
    }))
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}
