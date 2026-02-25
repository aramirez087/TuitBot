//! Read-only X API tools.
//!
//! `get_tweet_by_id`, `get_user_by_username`, and `search_tweets` delegate
//! to the kernel layer via a [`SocialReadProvider`]. The remaining read tools
//! still use the X API client directly (to be migrated in future sessions).

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
        Err(e) => super::error_response(&e, start),
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
        Err(e) => super::error_response(&e, start),
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
        Err(e) => super::error_response(&e, start),
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
