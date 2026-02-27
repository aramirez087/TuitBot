//! Write (mutation) X API tools: post, reply, quote, delete, thread.
//!
//! All raw X API calls go through `tuitbot_core::toolkit::write`.
//! All mutation governance (policy, idempotency, audit, rate recording) goes
//! through the unified gateway in `policy_gate::run_gateway`.

use std::time::Instant;

use serde::Serialize;

use crate::state::SharedState;

use super::not_configured_response;
use super::validate::check_tweet_length;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};
use crate::tools::workflow::policy_gate::{
    complete_gateway_failure, complete_gateway_success, run_gateway, GatewayResult,
};

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
    let ticket = match run_gateway(state, "post_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match tuitbot_core::toolkit::write::post_tweet(client.as_ref(), text, media_ids).await {
        Ok(tweet) => {
            let result_data = serde_json::to_value(&tweet).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(&tweet).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
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
    let ticket = match run_gateway(state, "reply_to_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match tuitbot_core::toolkit::write::reply_to_tweet(
        client.as_ref(),
        text,
        in_reply_to_id,
        media_ids,
    )
    .await
    {
        Ok(tweet) => {
            let result_data = serde_json::to_value(&tweet).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(&tweet).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
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
    let ticket = match run_gateway(state, "quote_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    // media_ids not forwarded: quote_tweet trait method has no media variant.
    let _ = media_ids;

    match tuitbot_core::toolkit::write::quote_tweet(client.as_ref(), text, quoted_tweet_id).await {
        Ok(tweet) => {
            let result_data = serde_json::to_value(&tweet).unwrap_or_default();
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(&tweet).with_meta(meta).to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
}

/// Delete a tweet by ID. Always policy-gated.
pub async fn delete_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    let ticket = match run_gateway(state, "delete_tweet", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match tuitbot_core::toolkit::write::delete_tweet(client.as_ref(), tweet_id).await {
        Ok(deleted) => {
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

/// Post a thread (ordered sequence of tweets).
///
/// Validates ALL tweet lengths up front. Uses a single policy gate for the
/// whole thread. Delegates to toolkit for the actual posting loop.
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

    // Validate all tweet lengths up front (fast-fail before gateway).
    for (i, tweet_text) in tweets.iter().enumerate() {
        if let Some(err_json) = check_tweet_length(tweet_text, start) {
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

    let params =
        serde_json::json!({"tweet_count": tweets.len(), "first_tweet": tweets[0]}).to_string();
    let ticket = match run_gateway(state, "post_thread", &params, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };

    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match tuitbot_core::toolkit::write::post_thread(client.as_ref(), tweets, media_ids).await {
        Ok(posted_ids) => {
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
            let meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            ToolResponse::success(result).with_meta(meta).to_json()
        }
        Err(tuitbot_core::toolkit::ToolkitError::ThreadPartialFailure {
            ref posted_ids,
            failed_index,
            ..
        }) => {
            let error_msg = format!(
                "Thread failed at tweet {failed_index}. Posted {}/{} tweets. IDs: {posted_ids:?}",
                posted_ids.len(),
                tweets.len(),
            );
            let meta = complete_gateway_failure(state, &ticket, &error_msg, start).await;
            let mut resp = ToolResponse::error(
                ErrorCode::ThreadPartialFailure,
                format!(
                    "Thread failed at tweet {failed_index}. Successfully posted {}/{} tweets.",
                    posted_ids.len(),
                    tweets.len()
                ),
            )
            .with_meta(meta);
            resp.data = serde_json::json!({
                "posted_tweet_ids": posted_ids,
                "failed_at_index": failed_index,
            });
            resp.to_json()
        }
        Err(ref e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            super::format_toolkit_error_with_meta(e, meta)
        }
    }
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

    if let Some(err) = check_tweet_length(text, start) {
        return err;
    }

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
