//! `draft_replies_for_candidates` — generate reply drafts for discovered tweets.

use std::sync::Arc;
use std::time::Instant;

use tuitbot_core::content::frameworks::ReplyArchetype;
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::safety::{contains_banned_phrase, DedupChecker};
use tuitbot_core::storage;

use crate::state::SharedState;
use crate::tools::content::ArcProvider;
use crate::tools::response::{ToolMeta, ToolResponse};

use super::DraftResult;

/// Parse an archetype string into a `ReplyArchetype`.
fn parse_archetype(s: &str) -> Option<ReplyArchetype> {
    match s.to_lowercase().replace(' ', "_").as_str() {
        "agree_and_expand" | "agreeandexpand" => Some(ReplyArchetype::AgreeAndExpand),
        "respectful_disagree" | "respectfuldisagree" => Some(ReplyArchetype::RespectfulDisagree),
        "add_data" | "adddata" => Some(ReplyArchetype::AddData),
        "ask_question" | "askquestion" => Some(ReplyArchetype::AskQuestion),
        "share_experience" | "shareexperience" => Some(ReplyArchetype::ShareExperience),
        _ => None,
    }
}

/// Execute the `draft_replies_for_candidates` composite tool.
pub async fn execute(
    state: &SharedState,
    candidate_ids: &[String],
    archetype_str: Option<&str>,
    mention_product: bool,
) -> String {
    let start = Instant::now();

    if candidate_ids.is_empty() {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::error("invalid_input", "candidate_ids must not be empty.", false)
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
    }

    // Parse archetype override
    let archetype_override = archetype_str.and_then(parse_archetype);

    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, state.config.business.clone());

    let dedup = DedupChecker::new(state.pool.clone());
    let banned = &state.config.limits.banned_phrases;

    let mut results = Vec::with_capacity(candidate_ids.len());

    for candidate_id in candidate_ids {
        // Fetch tweet from DB
        let tweet = match storage::tweets::get_tweet_by_id(&state.pool, candidate_id).await {
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

        // Generate reply
        let gen_result = gen
            .generate_reply_with_archetype(
                &tweet.content,
                &tweet.author_username,
                mention_product,
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

    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::success(&results)
        .with_meta(ToolMeta::new(elapsed).with_mode(
            state.config.mode.to_string(),
            state.config.effective_approval_mode(),
        ))
        .to_json()
}
