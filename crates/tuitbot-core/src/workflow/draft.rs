//! Draft step: fetch discovered tweets, generate LLM reply drafts, run safety checks.
//!
//! This is the second step in the reply pipeline: given scored candidates,
//! produce draft reply text for human or automated review.

use std::sync::Arc;

use crate::content::frameworks::ReplyArchetype;
use crate::llm::LlmProvider;
use crate::safety::{contains_banned_phrase, DedupChecker};
use crate::storage;
use crate::storage::DbPool;

use super::{make_content_gen, parse_archetype, DraftResult, WorkflowError};

/// Input for the draft step.
#[derive(Debug, Clone)]
pub struct DraftInput {
    /// Tweet IDs to generate drafts for (must exist in discovery DB).
    pub candidate_ids: Vec<String>,
    /// Override archetype for all drafts (e.g., "ask_question").
    pub archetype: Option<String>,
    /// Whether to mention the product in the reply.
    pub mention_product: bool,
}

/// Execute the draft step: fetch tweets, generate replies, check safety.
///
/// Returns one `DraftResult` per candidate. Individual failures don't
/// abort the batch — they produce `DraftResult::Error` entries.
pub async fn execute(
    db: &DbPool,
    llm: &Arc<dyn LlmProvider>,
    config: &crate::config::Config,
    input: DraftInput,
) -> Result<Vec<DraftResult>, WorkflowError> {
    if input.candidate_ids.is_empty() {
        return Err(WorkflowError::InvalidInput(
            "candidate_ids must not be empty.".to_string(),
        ));
    }

    let archetype_override: Option<ReplyArchetype> =
        input.archetype.as_deref().and_then(parse_archetype);

    let gen = make_content_gen(llm, &config.business);
    let dedup = DedupChecker::new(db.clone());
    let banned = &config.limits.banned_phrases;

    let mut results = Vec::with_capacity(input.candidate_ids.len());

    for candidate_id in &input.candidate_ids {
        // Fetch tweet from DB
        let tweet = match storage::tweets::get_tweet_by_id(db, candidate_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                results.push(DraftResult::Error {
                    candidate_id: candidate_id.clone(),
                    error_code: "not_found".to_string(),
                    error_message: format!("Tweet {candidate_id} not found in discovery DB."),
                });
                continue;
            }
            Err(e) => {
                results.push(DraftResult::Error {
                    candidate_id: candidate_id.clone(),
                    error_code: "db_error".to_string(),
                    error_message: format!("DB error fetching tweet: {e}"),
                });
                continue;
            }
        };

        // Generate reply via ContentGenerator
        let gen_result = gen
            .generate_reply_with_archetype(
                &tweet.content,
                &tweet.author_username,
                input.mention_product,
                archetype_override,
            )
            .await;

        let output = match gen_result {
            Ok(o) => o,
            Err(e) => {
                results.push(DraftResult::Error {
                    candidate_id: candidate_id.clone(),
                    error_code: "llm_error".to_string(),
                    error_message: format!("LLM generation failed: {e}"),
                });
                continue;
            }
        };

        let draft_text = output.text;
        let char_count = draft_text.len();

        // Confidence heuristic
        let confidence = if char_count < 200 {
            "high"
        } else if char_count < 260 {
            "medium"
        } else {
            "low"
        };

        // Risk checks
        let mut risks = Vec::new();
        if let Some(phrase) = contains_banned_phrase(&draft_text, banned) {
            risks.push(format!("contains_banned_phrase: {phrase}"));
        }
        if let Ok(true) = dedup.is_phrasing_similar(&draft_text, 20).await {
            risks.push("similar_to_recent_reply".to_string());
        }

        let archetype_name = archetype_override
            .map(|a| format!("{a:?}"))
            .unwrap_or_else(|| "auto_selected".to_string());

        results.push(DraftResult::Success {
            candidate_id: candidate_id.clone(),
            draft_text,
            archetype: archetype_name,
            char_count,
            confidence: confidence.to_string(),
            risks,
        });
    }

    Ok(results)
}
