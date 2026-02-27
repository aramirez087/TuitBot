//! Queue step: validate replies, safety-check, then route to approval queue or execute.
//!
//! This is the third step in the reply pipeline: given draft text and candidates,
//! either queue for human approval or execute directly via toolkit.
//! All X API writes route through `toolkit::write::reply_to_tweet`.

use std::sync::Arc;

use crate::config::Config;
use crate::llm::LlmProvider;
use crate::safety::{contains_banned_phrase, DedupChecker};
use crate::storage;
use crate::storage::DbPool;
use crate::toolkit;
use crate::x_api::XApiClient;

use super::{make_content_gen, ProposeResult, QueueItem, WorkflowError};

/// Input for the queue step.
#[derive(Debug, Clone)]
pub struct QueueInput {
    /// Items to process (each is a candidate + optional pre-drafted text).
    pub items: Vec<QueueItem>,
    /// Whether to mention the product in generated replies.
    pub mention_product: bool,
}

/// Execute the queue step: validate, safety-check, route or execute.
///
/// When `approval_mode` is true, replies are queued for human review.
/// When false, replies are executed immediately via toolkit.
///
/// All X API writes go through `toolkit::write::reply_to_tweet`.
pub async fn execute(
    db: &DbPool,
    x_client: Option<&dyn XApiClient>,
    llm: Option<&Arc<dyn LlmProvider>>,
    config: &Config,
    input: QueueInput,
) -> Result<Vec<ProposeResult>, WorkflowError> {
    if input.items.is_empty() {
        return Err(WorkflowError::InvalidInput(
            "items must not be empty.".to_string(),
        ));
    }

    let approval_mode = config.effective_approval_mode();
    let dedup = DedupChecker::new(db.clone());
    let banned = &config.limits.banned_phrases;

    // Build content generator if LLM is available (needed for auto-generation)
    let gen = llm.map(|l| make_content_gen(l, &config.business));

    let mut results = Vec::with_capacity(input.items.len());

    for item in &input.items {
        // Fetch tweet from DB
        let tweet = match storage::tweets::get_tweet_by_id(db, &item.candidate_id).await {
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
            let content_gen = match &gen {
                Some(g) => g,
                None => {
                    results.push(ProposeResult::Blocked {
                        candidate_id: item.candidate_id.clone(),
                        reason: "LLM not configured for auto-generation.".to_string(),
                    });
                    continue;
                }
            };
            match content_gen
                .generate_reply(
                    &tweet.content,
                    &tweet.author_username,
                    input.mention_product,
                )
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
                db,
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
            let client = match x_client {
                Some(c) => c,
                None => {
                    results.push(ProposeResult::Blocked {
                        candidate_id: item.candidate_id.clone(),
                        reason: "X API client not available.".to_string(),
                    });
                    continue;
                }
            };

            // Route through toolkit, not direct XApiClient
            match toolkit::write::reply_to_tweet(client, &reply_text, &item.candidate_id, None)
                .await
            {
                Ok(posted) => {
                    let _ = storage::tweets::mark_tweet_replied(db, &item.candidate_id).await;
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

    Ok(results)
}
