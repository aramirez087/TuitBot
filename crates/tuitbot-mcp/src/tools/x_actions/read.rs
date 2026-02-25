//! Read-only X API tools.

use std::time::Instant;

use crate::state::SharedState;

use super::super::response::{ToolMeta, ToolResponse};
use super::{error_response, no_user_id_response, not_configured_response};

/// Get a single tweet by ID.
pub async fn get_tweet_by_id(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.get_tweet(tweet_id).await {
        Ok(tweet) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Look up a user by username.
pub async fn get_user_by_username(state: &SharedState, username: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.get_user_by_username(username).await {
        Ok(user) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&user)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Search recent tweets.
pub async fn search_tweets(
    state: &SharedState,
    query: &str,
    max_results: u32,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client
        .search_tweets(query, max_results, since_id, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Get mentions for the authenticated user.
pub async fn get_user_mentions(
    state: &SharedState,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match client
        .get_mentions(user_id, since_id, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Get recent tweets from a specific user.
pub async fn get_user_tweets(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client
        .get_user_tweets(user_id, max_results, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Get the authenticated user's home timeline.
pub async fn get_home_timeline(
    state: &SharedState,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match client
        .get_home_timeline(user_id, max_results, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
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
