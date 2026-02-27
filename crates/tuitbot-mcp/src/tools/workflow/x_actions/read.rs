//! Read-only X API tools.
//!
//! All read tools delegate to the toolkit layer in `tuitbot-core`,
//! except `get_x_usage` which requires DB access (workflow-only).

use std::time::Instant;

use crate::contract::envelope::{PaginationInfo, ToolMeta, ToolResponse};
use crate::state::SharedState;
use tuitbot_core::x_api::types::{SearchMeta, UsersMeta};

use super::{no_user_id_response, not_configured_response, toolkit_error_response};

fn pagination_from_search(meta: &SearchMeta) -> PaginationInfo {
    PaginationInfo {
        has_more: meta.next_token.is_some(),
        next_token: meta.next_token.clone(),
        result_count: meta.result_count,
    }
}

fn pagination_from_users(meta: &UsersMeta) -> PaginationInfo {
    PaginationInfo {
        has_more: meta.next_token.is_some(),
        next_token: meta.next_token.clone(),
        result_count: meta.result_count,
    }
}

/// Get a single tweet by ID — delegates to toolkit.
pub async fn get_tweet_by_id(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_tweet(client, tweet_id).await {
        Ok(tweet) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Look up a user by username — delegates to toolkit.
pub async fn get_user_by_username(state: &SharedState, username: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_user_by_username(client, username).await {
        Ok(user) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&user)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Search recent tweets — delegates to toolkit.
pub async fn search_tweets(
    state: &SharedState,
    query: &str,
    max_results: u32,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::search_tweets(
        client,
        query,
        max_results,
        since_id,
        pagination_token,
    )
    .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_search(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get mentions for the authenticated user — delegates to toolkit.
pub async fn get_user_mentions(
    state: &SharedState,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };
    match tuitbot_core::toolkit::read::get_mentions(client, user_id, since_id, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_search(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get recent tweets from a specific user — delegates to toolkit.
pub async fn get_user_tweets(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_user_tweets(
        client,
        user_id,
        max_results,
        pagination_token,
    )
    .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_search(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get the authenticated user's home timeline — delegates to toolkit.
pub async fn get_home_timeline(
    state: &SharedState,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };
    match tuitbot_core::toolkit::read::get_home_timeline(
        client,
        user_id,
        max_results,
        pagination_token,
    )
    .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_search(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get followers of a user — delegates to toolkit.
pub async fn get_followers(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_followers(client, user_id, max_results, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_users(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get accounts a user is following — delegates to toolkit.
pub async fn get_following(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_following(client, user_id, max_results, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_users(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get a user by their ID — delegates to toolkit.
pub async fn get_user_by_id(state: &SharedState, user_id: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_user_by_id(client, user_id).await {
        Ok(user) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&user)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get tweets liked by a user — delegates to toolkit.
pub async fn get_liked_tweets(
    state: &SharedState,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_liked_tweets(
        client,
        user_id,
        max_results,
        pagination_token,
    )
    .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_search(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get the authenticated user's bookmarks — delegates to toolkit.
pub async fn get_bookmarks(
    state: &SharedState,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };
    match tuitbot_core::toolkit::read::get_bookmarks(client, user_id, max_results, pagination_token)
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_search(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get multiple users by their IDs — delegates to toolkit.
pub async fn get_users_by_ids(state: &SharedState, user_ids: &[&str]) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_users_by_ids(client, user_ids).await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_users(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get users who liked a specific tweet — delegates to toolkit.
pub async fn get_tweet_liking_users(
    state: &SharedState,
    tweet_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return not_configured_response(start),
    };
    match tuitbot_core::toolkit::read::get_tweet_liking_users(
        client,
        tweet_id,
        max_results,
        pagination_token,
    )
    .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let pagination = pagination_from_users(&resp.meta);
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed).with_pagination(pagination))
                .to_json()
        }
        Err(ref e) => toolkit_error_response(e, start),
    }
}

/// Get X API usage statistics.
///
/// Workflow-only: requires DB access, not a toolkit operation.
pub async fn get_x_usage(state: &SharedState, days: u32) -> String {
    let start = Instant::now();

    let summary = match tuitbot_core::storage::x_api_usage::get_usage_summary(&state.pool).await {
        Ok(s) => s,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                crate::tools::response::ErrorCode::DbError,
                format!("Failed to get usage summary: {e}"),
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
                crate::tools::response::ErrorCode::DbError,
                format!("Failed to get daily usage: {e}"),
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
                    crate::tools::response::ErrorCode::DbError,
                    format!("Failed to get endpoint breakdown: {e}"),
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
