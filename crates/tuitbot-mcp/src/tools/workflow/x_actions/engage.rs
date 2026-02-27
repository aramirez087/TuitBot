//! Engagement X API tools: like, follow, unfollow, retweet, unretweet.
//!
//! All raw X API calls go through `tuitbot_core::toolkit::engage`.
//! All mutation governance (policy, idempotency, audit, rate recording) goes
//! through the unified gateway in `policy_gate::run_gateway`.

use std::time::Instant;

use serde::Serialize;

use crate::state::SharedState;

use super::{no_user_id_response, not_configured_response};
use crate::tools::response::ToolResponse;
use crate::tools::workflow::policy_gate::{
    complete_gateway_failure, complete_gateway_success, run_gateway, GatewayResult,
};

/// Like a tweet.
pub async fn like_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    let ticket = match run_gateway(state, "like_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::like_tweet(client.as_ref(), user_id, tweet_id).await {
        Ok(liked) => {
            #[derive(Serialize)]
            struct LikeResult {
                liked: bool,
                tweet_id: String,
            }
            let result = LikeResult {
                liked,
                tweet_id: tweet_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Follow a user.
pub async fn follow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"target_user_id": target_user_id}).to_string();
    let ticket = match run_gateway(state, "follow_user", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::follow_user(client.as_ref(), user_id, target_user_id).await
    {
        Ok(following) => {
            #[derive(Serialize)]
            struct FollowResult {
                following: bool,
                target_user_id: String,
            }
            let result = FollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Unfollow a user.
pub async fn unfollow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"target_user_id": target_user_id}).to_string();
    let ticket = match run_gateway(state, "unfollow_user", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::unfollow_user(client.as_ref(), user_id, target_user_id)
        .await
    {
        Ok(following) => {
            #[derive(Serialize)]
            struct UnfollowResult {
                following: bool,
                target_user_id: String,
            }
            let result = UnfollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Retweet a tweet.
pub async fn retweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    let ticket = match run_gateway(state, "retweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::retweet(client.as_ref(), user_id, tweet_id).await {
        Ok(retweeted) => {
            #[derive(Serialize)]
            struct RetweetResult {
                retweeted: bool,
                tweet_id: String,
            }
            let result = RetweetResult {
                retweeted,
                tweet_id: tweet_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Unlike a tweet.
pub async fn unlike_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    let ticket = match run_gateway(state, "unlike_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::unlike_tweet(client.as_ref(), user_id, tweet_id).await {
        Ok(liked) => {
            #[derive(Serialize)]
            struct UnlikeResult {
                liked: bool,
                tweet_id: String,
            }
            let result = UnlikeResult {
                liked,
                tweet_id: tweet_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Bookmark a tweet.
pub async fn bookmark_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    let ticket = match run_gateway(state, "bookmark_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::bookmark_tweet(client.as_ref(), user_id, tweet_id).await {
        Ok(bookmarked) => {
            #[derive(Serialize)]
            struct BookmarkResult {
                bookmarked: bool,
                tweet_id: String,
            }
            let result = BookmarkResult {
                bookmarked,
                tweet_id: tweet_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Remove a bookmark.
pub async fn unbookmark_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    let ticket = match run_gateway(state, "unbookmark_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::unbookmark_tweet(client.as_ref(), user_id, tweet_id).await
    {
        Ok(bookmarked) => {
            #[derive(Serialize)]
            struct UnbookmarkResult {
                bookmarked: bool,
                tweet_id: String,
            }
            let result = UnbookmarkResult {
                bookmarked,
                tweet_id: tweet_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Undo a retweet.
pub async fn unretweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    let ticket = match run_gateway(state, "unretweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match tuitbot_core::toolkit::engage::unretweet(client.as_ref(), user_id, tweet_id).await {
        Ok(retweeted) => {
            #[derive(Serialize)]
            struct UnretweetResult {
                retweeted: bool,
                tweet_id: String,
            }
            let result = UnretweetResult {
                retweeted,
                tweet_id: tweet_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}
