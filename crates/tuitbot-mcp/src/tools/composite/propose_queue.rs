//! `propose_and_queue_replies` — safety-check, then queue or execute replies.

use std::sync::Arc;
use std::time::Instant;

use tuitbot_core::content::ContentGenerator;
use tuitbot_core::mcp_policy::McpPolicyEvaluator;
use tuitbot_core::safety::{contains_banned_phrase, DedupChecker};
use tuitbot_core::storage;

use crate::requests::ProposeItem;
use crate::state::SharedState;
use crate::tools::content::ArcProvider;
use crate::tools::policy_gate::{self, GateResult};
use crate::tools::response::{ToolMeta, ToolResponse};

use super::ProposeResult;

/// Execute the `propose_and_queue_replies` composite tool.
pub async fn execute(state: &SharedState, items: &[ProposeItem], mention_product: bool) -> String {
    let start = Instant::now();

    if items.is_empty() {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::error("invalid_input", "items must not be empty.", false)
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
    }

    // Global policy gate check
    let params = serde_json::json!({
        "item_count": items.len(),
        "mention_product": mention_product,
    })
    .to_string();
    match policy_gate::check_policy(state, "propose_and_queue_replies", &params, start).await {
        GateResult::EarlyReturn(r) => return r,
        GateResult::Proceed => {}
    }

    let approval_mode = state.config.effective_approval_mode();
    let dedup = DedupChecker::new(state.pool.clone());
    let banned = &state.config.limits.banned_phrases;

    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, state.config.business.clone());

    let mut results = Vec::with_capacity(items.len());

    for item in items {
        // Fetch tweet from DB
        let tweet = match storage::tweets::get_tweet_by_id(&state.pool, &item.candidate_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                results.push(ProposeResult::Blocked {
                    candidate_id: item.candidate_id.clone(),
                    reason: format!("Tweet {} not found in discovery DB.", item.candidate_id),
                });
                continue;
            }
            Err(e) => {
                results.push(ProposeResult::Blocked {
                    candidate_id: item.candidate_id.clone(),
                    reason: format!("DB error: {e}"),
                });
                continue;
            }
        };

        // Determine reply text: pre-drafted or auto-generate
        let reply_text = if let Some(text) = &item.pre_drafted_text {
            text.clone()
        } else {
            match gen
                .generate_reply(&tweet.content, &tweet.author_username, mention_product)
                .await
            {
                Ok(output) => output.text,
                Err(e) => {
                    results.push(ProposeResult::Blocked {
                        candidate_id: item.candidate_id.clone(),
                        reason: format!("LLM generation failed: {e}"),
                    });
                    continue;
                }
            }
        };

        // Safety checks
        if let Ok(true) = dedup.has_replied_to(&item.candidate_id).await {
            results.push(ProposeResult::Blocked {
                candidate_id: item.candidate_id.clone(),
                reason: "Already replied to this tweet.".to_string(),
            });
            continue;
        }

        if let Some(phrase) = contains_banned_phrase(&reply_text, banned) {
            results.push(ProposeResult::Blocked {
                candidate_id: item.candidate_id.clone(),
                reason: format!("Contains banned phrase: {phrase}"),
            });
            continue;
        }

        if let Ok(true) = dedup.is_phrasing_similar(&reply_text, 20).await {
            results.push(ProposeResult::Blocked {
                candidate_id: item.candidate_id.clone(),
                reason: "Reply too similar to a recent reply.".to_string(),
            });
            continue;
        }

        // Route: approval queue or direct execution
        if approval_mode {
            match storage::approval_queue::enqueue(
                &state.pool,
                "reply",
                &item.candidate_id,
                &tweet.author_username,
                &reply_text,
                "composite",
                "auto",
                tweet.relevance_score.unwrap_or(0.0),
                "[]",
            )
            .await
            {
                Ok(id) => {
                    results.push(ProposeResult::Queued {
                        candidate_id: item.candidate_id.clone(),
                        approval_queue_id: id,
                    });
                }
                Err(e) => {
                    results.push(ProposeResult::Blocked {
                        candidate_id: item.candidate_id.clone(),
                        reason: format!("Failed to enqueue: {e}"),
                    });
                }
            }
        } else {
            // Direct execution requires X client
            let x_client = match state.x_client.as_ref() {
                Some(c) => c,
                None => {
                    results.push(ProposeResult::Blocked {
                        candidate_id: item.candidate_id.clone(),
                        reason: "X API client not available.".to_string(),
                    });
                    continue;
                }
            };

            match x_client
                .reply_to_tweet(&reply_text, &item.candidate_id)
                .await
            {
                Ok(posted) => {
                    // Mark as replied in DB
                    let _ =
                        storage::tweets::mark_tweet_replied(&state.pool, &item.candidate_id).await;
                    results.push(ProposeResult::Executed {
                        candidate_id: item.candidate_id.clone(),
                        reply_tweet_id: posted.id,
                    });
                }
                Err(e) => {
                    results.push(ProposeResult::Blocked {
                        candidate_id: item.candidate_id.clone(),
                        reason: format!("X API error: {e}"),
                    });
                }
            }
        }
    }

    // Record one mutation for the batch
    let _ = McpPolicyEvaluator::record_mutation(
        &state.pool,
        "propose_and_queue_replies",
        &state.config.mcp_policy.rate_limits,
    )
    .await;

    let elapsed = start.elapsed().as_millis() as u64;
    let has_error = results
        .iter()
        .any(|r| matches!(r, ProposeResult::Blocked { .. }));
    crate::tools::telemetry::record(
        &state.pool,
        "propose_and_queue_replies",
        "composite_mutation",
        elapsed,
        !has_error
            || results
                .iter()
                .any(|r| !matches!(r, ProposeResult::Blocked { .. })),
        None,
        Some("allow"),
    )
    .await;
    ToolResponse::success(&results)
        .with_meta(ToolMeta::new(elapsed).with_mode(
            state.config.mode.to_string(),
            state.config.effective_approval_mode(),
        ))
        .to_json()
}
