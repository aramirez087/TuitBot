//! Write (mutation) X API tools: post, reply, quote, delete, thread.

use std::time::Instant;

use serde::Serialize;

use crate::state::SharedState;

use super::audit;
use super::not_configured_response;
use super::validate::check_tweet_length;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

/// Post a new tweet, optionally with media.
pub async fn post_tweet(state: &SharedState, text: &str, media_ids: Option<&[String]>) -> String {
    let start = Instant::now();
    if let Some(err) = check_tweet_length(text, start) {
        return err;
    }
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"text": text}).to_string();
    match super::super::policy_gate::check_policy(state, "post_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let guard = match audit::begin_audited_mutation(state, "post_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    let result = match media_ids {
        Some(ids) if !ids.is_empty() => client.post_tweet_with_media(text, ids).await,
        _ => client.post_tweet(text).await,
    };

    match result {
        Ok(tweet) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "post_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let result_data = serde_json::to_value(&tweet).unwrap_or_default();
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(&tweet).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Reply to an existing tweet, optionally with media.
pub async fn reply_to_tweet(
    state: &SharedState,
    text: &str,
    in_reply_to_id: &str,
    media_ids: Option<&[String]>,
) -> String {
    let start = Instant::now();
    if let Some(err) = check_tweet_length(text, start) {
        return err;
    }
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"text": text, "in_reply_to_id": in_reply_to_id}).to_string();
    match super::super::policy_gate::check_policy(state, "reply_to_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let guard = match audit::begin_audited_mutation(state, "reply_to_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    let result = match media_ids {
        Some(ids) if !ids.is_empty() => {
            client
                .reply_to_tweet_with_media(text, in_reply_to_id, ids)
                .await
        }
        _ => client.reply_to_tweet(text, in_reply_to_id).await,
    };

    match result {
        Ok(tweet) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "reply_to_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let result_data = serde_json::to_value(&tweet).unwrap_or_default();
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(&tweet).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Post a quote tweet, optionally with media.
pub async fn quote_tweet(
    state: &SharedState,
    text: &str,
    quoted_tweet_id: &str,
    media_ids: Option<&[String]>,
) -> String {
    let start = Instant::now();
    if let Some(err) = check_tweet_length(text, start) {
        return err;
    }
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"text": text, "quoted_tweet_id": quoted_tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "quote_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let guard = match audit::begin_audited_mutation(state, "quote_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    // quote_tweet doesn't have a _with_media variant on the trait,
    // so media_ids are not forwarded for now.
    let _ = media_ids;

    match client.quote_tweet(text, quoted_tweet_id).await {
        Ok(tweet) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "quote_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let result_data = serde_json::to_value(&tweet).unwrap_or_default();
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(&tweet).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Delete a tweet by ID. Always policy-gated.
pub async fn delete_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::super::policy_gate::check_policy(state, "delete_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let guard = match audit::begin_audited_mutation(state, "delete_tweet", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    match client.delete_tweet(tweet_id).await {
        Ok(deleted) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "delete_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            #[derive(Serialize)]
            struct DeleteResult {
                deleted: bool,
                tweet_id: String,
            }
            let result = DeleteResult {
                deleted,
                tweet_id: tweet_id.to_string(),
            };
            let result_data = serde_json::to_value(&result).unwrap_or_default();
            let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(e) => audit::audited_x_error_response(&guard, state, &e, start).await,
    }
}

/// Post a thread (ordered sequence of tweets).
///
/// Validates ALL tweet lengths up front. Uses a single policy gate for the
/// whole thread. On partial failure, returns the IDs of successfully posted
/// tweets so the agent can resume.
pub async fn post_thread(
    state: &SharedState,
    tweets: &[String],
    media_ids: Option<&[Vec<String>]>,
) -> String {
    let start = Instant::now();

    if tweets.is_empty() {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::error(
            ErrorCode::InvalidInput,
            "Thread must contain at least one tweet.",
        )
        .with_meta(ToolMeta::new(elapsed))
        .to_json();
    }

    // Validate all tweet lengths up front.
    for (i, tweet_text) in tweets.iter().enumerate() {
        if let Some(err_json) = check_tweet_length(tweet_text, start) {
            // Augment the error with the tweet index.
            let mut parsed: serde_json::Value = serde_json::from_str(&err_json).unwrap_or_default();
            if let Some(err_obj) = parsed.get_mut("error") {
                err_obj["tweet_index"] = serde_json::json!(i);
            }
            return serde_json::to_string(&parsed).unwrap_or(err_json);
        }
    }

    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }

    // Single policy gate for the whole thread.
    let params =
        serde_json::json!({"tweet_count": tweets.len(), "first_tweet": tweets[0]}).to_string();
    match super::super::policy_gate::check_policy(state, "post_thread", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }

    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    let guard = match audit::begin_audited_mutation(state, "post_thread", &params).await {
        audit::AuditGateResult::Proceed(g) => g,
        audit::AuditGateResult::EarlyReturn(r) => return r,
    };

    let mut posted_ids: Vec<String> = Vec::with_capacity(tweets.len());

    for (i, tweet_text) in tweets.iter().enumerate() {
        let tweet_media = media_ids
            .and_then(|m| m.get(i))
            .cloned()
            .unwrap_or_default();

        let result = if i == 0 {
            // First tweet: standalone post.
            if tweet_media.is_empty() {
                client.post_tweet(tweet_text).await
            } else {
                client.post_tweet_with_media(tweet_text, &tweet_media).await
            }
        } else {
            // Subsequent tweets: reply to the previous one.
            let prev_id = &posted_ids[i - 1];
            if tweet_media.is_empty() {
                client.reply_to_tweet(tweet_text, prev_id).await
            } else {
                client
                    .reply_to_tweet_with_media(tweet_text, prev_id, &tweet_media)
                    .await
            }
        };

        match result {
            Ok(posted) => posted_ids.push(posted.id),
            Err(e) => {
                // Partial failure: record in audit trail and include posted IDs.
                let error_msg = format!(
                    "Thread failed at tweet {i}: {e}. Posted {}/{} tweets. IDs: {:?}",
                    posted_ids.len(),
                    tweets.len(),
                    posted_ids,
                );
                let meta = audit::complete_audited_failure(&guard, state, &error_msg, start).await;
                let mut resp = ToolResponse::error(
                    ErrorCode::ThreadPartialFailure,
                    format!(
                        "Thread failed at tweet {i}: {e}. Successfully posted {}/{} tweets.",
                        posted_ids.len(),
                        tweets.len()
                    ),
                )
                .with_meta(meta);
                resp.data = serde_json::json!({
                    "posted_tweet_ids": posted_ids,
                    "failed_at_index": i,
                });
                return resp.to_json();
            }
        }
    }

    let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
        &state.pool,
        "post_thread",
        &state.config.mcp_policy.rate_limits,
    )
    .await;

    #[derive(Serialize)]
    struct ThreadResult {
        thread_tweet_ids: Vec<String>,
        tweet_count: usize,
        root_tweet_id: String,
    }
    let root_id = posted_ids[0].clone();
    let result = ThreadResult {
        tweet_count: posted_ids.len(),
        thread_tweet_ids: posted_ids,
        root_tweet_id: root_id,
    };
    let result_data = serde_json::to_value(&result).unwrap_or_default();
    let meta = audit::complete_audited_success(&guard, state, &result_data, start).await;
    ToolResponse::success(result).with_meta(meta).to_json()
}

// ── Dry-run validation tools ──────────────────────────────────────────

/// Validate a tweet without posting. Runs all checks (length, policy) but
/// never calls the X API. Returns what *would* be posted.
pub async fn post_tweet_dry_run(
    state: &SharedState,
    text: &str,
    media_ids: Option<&[String]>,
) -> String {
    let start = Instant::now();

    // Length validation.
    if let Some(err) = check_tweet_length(text, start) {
        return err;
    }

    // Policy check (will return DryRun decision if dry_run_mutations is on,
    // but we also want dry-run even when policy is off).
    let params = serde_json::json!({"text": text}).to_string();
    let policy_would_allow =
        match super::super::policy_gate::check_policy(state, "post_tweet", &params, start).await {
            super::super::policy_gate::GateResult::Proceed => true,
            super::super::policy_gate::GateResult::EarlyReturn(_) => false,
        };

    let has_media = media_ids.is_some_and(|ids| !ids.is_empty());
    let media_count = media_ids.map_or(0, |ids| ids.len());
    let x_available = state.x_client.is_some();

    let elapsed = start.elapsed().as_millis() as u64;
    #[derive(Serialize)]
    struct DryRunTweetResult {
        dry_run: bool,
        valid: bool,
        text: String,
        text_length: usize,
        has_media: bool,
        media_count: usize,
        media_ids: Vec<String>,
        policy_would_allow: bool,
        x_client_available: bool,
    }
    ToolResponse::success(DryRunTweetResult {
        dry_run: true,
        valid: true,
        text: text.to_string(),
        text_length: text.len(),
        has_media,
        media_count,
        media_ids: media_ids.map(|ids| ids.to_vec()).unwrap_or_default(),
        policy_would_allow,
        x_client_available: x_available,
    })
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}

/// Validate a thread without posting. Runs all checks (lengths, policy) but
/// never calls the X API. Returns what *would* be posted with reply chain plan.
pub async fn post_thread_dry_run(
    state: &SharedState,
    tweets: &[String],
    media_ids: Option<&[Vec<String>]>,
) -> String {
    let start = Instant::now();

    if tweets.is_empty() {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::error(
            ErrorCode::InvalidInput,
            "Thread must contain at least one tweet.",
        )
        .with_meta(ToolMeta::new(elapsed))
        .to_json();
    }

    // Validate all tweet lengths up front.
    let mut validation_results: Vec<TweetValidation> = Vec::with_capacity(tweets.len());

    for (i, tweet_text) in tweets.iter().enumerate() {
        let tweet_media = media_ids
            .and_then(|m| m.get(i))
            .cloned()
            .unwrap_or_default();

        if let Some(err_json) = check_tweet_length(tweet_text, start) {
            let mut parsed: serde_json::Value = serde_json::from_str(&err_json).unwrap_or_default();
            if let Some(err_obj) = parsed.get_mut("error") {
                err_obj["tweet_index"] = serde_json::json!(i);
            }
            // On first validation failure, return immediately like the real tool.
            return serde_json::to_string(&parsed).unwrap_or(err_json);
        }

        let chain_action = if i == 0 {
            "post_tweet".to_string()
        } else {
            format!("reply_to_tweet(parent=tweet_{})", i - 1)
        };

        validation_results.push(TweetValidation {
            index: i,
            text: tweet_text.clone(),
            text_length: tweet_text.len(),
            valid: true,
            has_media: !tweet_media.is_empty(),
            media_ids: tweet_media,
            chain_action,
        });
    }

    // Policy check.
    let params =
        serde_json::json!({"tweet_count": tweets.len(), "first_tweet": tweets[0]}).to_string();
    let policy_would_allow =
        match super::super::policy_gate::check_policy(state, "post_thread", &params, start).await {
            super::super::policy_gate::GateResult::Proceed => true,
            super::super::policy_gate::GateResult::EarlyReturn(_) => false,
        };

    let x_available = state.x_client.is_some();
    let elapsed = start.elapsed().as_millis() as u64;

    #[derive(Serialize)]
    struct DryRunThreadResult {
        dry_run: bool,
        valid: bool,
        tweet_count: usize,
        tweets: Vec<TweetValidation>,
        policy_would_allow: bool,
        x_client_available: bool,
    }
    ToolResponse::success(DryRunThreadResult {
        dry_run: true,
        valid: true,
        tweet_count: tweets.len(),
        tweets: validation_results,
        policy_would_allow,
        x_client_available: x_available,
    })
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}

#[derive(Serialize)]
struct TweetValidation {
    index: usize,
    text: String,
    text_length: usize,
    valid: bool,
    has_media: bool,
    media_ids: Vec<String>,
    chain_action: String,
}
