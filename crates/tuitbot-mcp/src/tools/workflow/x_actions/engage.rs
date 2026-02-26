//! Engagement X API tools: like, follow, unfollow, retweet, unretweet.

use std::time::Instant;

use serde::Serialize;

use crate::state::SharedState;

use super::audit;
use super::{no_user_id_response, not_configured_response};
use crate::tools::response::ToolResponse;

/// Like a tweet.
pub async fn like_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
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
    let guard = match audit::begin_audited_mutation(state, "like_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.like_tweet(user_id, tweet_id).await {
        Ok(liked) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "like_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Follow a user.
pub async fn follow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
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
    let guard = match audit::begin_audited_mutation(state, "follow_user", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.follow_user(user_id, target_user_id).await {
        Ok(following) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "follow_user",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Unfollow a user.
pub async fn unfollow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
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
    let guard = match audit::begin_audited_mutation(state, "unfollow_user", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.unfollow_user(user_id, target_user_id).await {
        Ok(following) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "unfollow_user",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Retweet a tweet.
pub async fn retweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
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
    let guard = match audit::begin_audited_mutation(state, "retweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.retweet(user_id, tweet_id).await {
        Ok(retweeted) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "retweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Unlike a tweet.
pub async fn unlike_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "unlike_tweet", &params, start).await {
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
    let guard = match audit::begin_audited_mutation(state, "unlike_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.unlike_tweet(user_id, tweet_id).await {
        Ok(liked) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "unlike_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Bookmark a tweet.
pub async fn bookmark_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "bookmark_tweet", &params, start).await {
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
    let guard = match audit::begin_audited_mutation(state, "bookmark_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.bookmark_tweet(user_id, tweet_id).await {
        Ok(bookmarked) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "bookmark_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Remove a bookmark.
pub async fn unbookmark_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "unbookmark_tweet", &params, start).await {
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
    let guard = match audit::begin_audited_mutation(state, "unbookmark_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.unbookmark_tweet(user_id, tweet_id).await {
        Ok(bookmarked) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "unbookmark_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Undo a retweet.
pub async fn unretweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
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
    let guard = match audit::begin_audited_mutation(state, "unretweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.unretweet(user_id, tweet_id).await {
        Ok(retweeted) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "unretweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
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
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}
