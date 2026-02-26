//! Write (mutation) X API tools: post, reply, quote, delete, thread.

use std::time::Instant;

use serde::Serialize;

use crate::state::SharedState;

use super::validate::check_tweet_length;
use super::{error_response, not_configured_response};
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

/// Post a new tweet, optionally with media.
pub async fn post_tweet(state: &SharedState, text: &str, media_ids: Option<&[String]>) -> String {
    let start = Instant::now();
    if let Some(err) = check_tweet_length(text, start) {
        return err;
    }
    let params = serde_json::json!({"text": text}).to_string();
    if let Some(err) = state.idempotency.check_and_record("post_tweet", &params) {
        return err;
    }
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    match super::super::policy_gate::check_policy(state, "post_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
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
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
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
    let params = serde_json::json!({"text": text, "in_reply_to_id": in_reply_to_id}).to_string();
    if let Some(err) = state
        .idempotency
        .check_and_record("reply_to_tweet", &params)
    {
        return err;
    }
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    match super::super::policy_gate::check_policy(state, "reply_to_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
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
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
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
    let params = serde_json::json!({"text": text, "quoted_tweet_id": quoted_tweet_id}).to_string();
    if let Some(err) = state.idempotency.check_and_record("quote_tweet", &params) {
        return err;
    }
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    match super::super::policy_gate::check_policy(state, "quote_tweet", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
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
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
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

    match client.delete_tweet(tweet_id).await {
        Ok(deleted) => {
            let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                &state.pool,
                "delete_tweet",
                &state.config.mcp_policy.rate_limits,
            )
            .await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct DeleteResult {
                deleted: bool,
                tweet_id: String,
            }
            ToolResponse::success(DeleteResult {
                deleted,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
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

    // Single policy gate for the whole thread.
    let params =
        serde_json::json!({"tweet_count": tweets.len(), "first_tweet": tweets[0]}).to_string();
    if let Some(err) = state.idempotency.check_and_record("post_thread", &params) {
        return err;
    }
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    match super::super::policy_gate::check_policy(state, "post_thread", &params, start).await {
        super::super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::super::policy_gate::GateResult::Proceed => {}
    }

    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
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
                // Partial failure: include posted IDs so agent can resume.
                let elapsed = start.elapsed().as_millis() as u64;
                let mut resp = ToolResponse::error(
                    ErrorCode::ThreadPartialFailure,
                    format!(
                        "Thread failed at tweet {i}: {e}. Successfully posted {}/{} tweets.",
                        posted_ids.len(),
                        tweets.len()
                    ),
                )
                .with_meta(ToolMeta::new(elapsed));
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

    let elapsed = start.elapsed().as_millis() as u64;
    #[derive(Serialize)]
    struct ThreadResult {
        thread_tweet_ids: Vec<String>,
        tweet_count: usize,
        root_tweet_id: String,
    }
    let root_id = posted_ids[0].clone();
    ToolResponse::success(ThreadResult {
        tweet_count: posted_ids.len(),
        thread_tweet_ids: posted_ids,
        root_tweet_id: root_id,
    })
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
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
