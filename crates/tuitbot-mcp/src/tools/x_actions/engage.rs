//! Engagement X API tools: like, follow, unfollow, retweet, unretweet.

use std::time::Instant;

use serde::Serialize;

use crate::state::SharedState;

use super::super::response::{ToolMeta, ToolResponse};
use super::{error_response, no_user_id_response, not_configured_response};

/// Like a tweet.
pub async fn like_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "like_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match client.like_tweet(user_id, tweet_id).await {
        Ok(liked) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "like_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct LikeResult {
                liked: bool,
                tweet_id: String,
            }
            ToolResponse::success(LikeResult {
                liked,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Follow a user.
pub async fn follow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"target_user_id": target_user_id}).to_string();
    match super::super::policy_gate::check_policy(state, "follow_user", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match client.follow_user(user_id, target_user_id).await {
        Ok(following) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "follow_user",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct FollowResult {
                following: bool,
                target_user_id: String,
            }
            ToolResponse::success(FollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Unfollow a user.
pub async fn unfollow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"target_user_id": target_user_id}).to_string();
    match super::super::policy_gate::check_policy(state, "unfollow_user", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match client.unfollow_user(user_id, target_user_id).await {
        Ok(following) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "unfollow_user",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UnfollowResult {
                following: bool,
                target_user_id: String,
            }
            ToolResponse::success(UnfollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Retweet a tweet.
pub async fn retweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "retweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match client.retweet(user_id, tweet_id).await {
        Ok(retweeted) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "retweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct RetweetResult {
                retweeted: bool,
                tweet_id: String,
            }
            ToolResponse::success(RetweetResult {
                retweeted,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Undo a retweet.
pub async fn unretweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "unretweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => return no_user_id_response(start),
    };

    match client.unretweet(user_id, tweet_id).await {
        Ok(retweeted) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "unretweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UnretweetResult {
                retweeted: bool,
                tweet_id: String,
            }
            ToolResponse::success(UnretweetResult {
                retweeted,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}
