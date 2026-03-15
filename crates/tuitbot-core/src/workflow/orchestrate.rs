//! Deterministic orchestrator for autopilot discovery cycles.
//!
//! Composes the atomic workflow steps (discover → draft → queue) into a single
//! entrypoint that autopilot loops and batch operations can call.
//!
//! This replaces the pattern of each consumer reimplementing the composition.

use std::sync::Arc;

use crate::config::Config;
use crate::llm::LlmProvider;
use crate::storage::DbPool;
use crate::x_api::XApiClient;

use super::discover::{self, DiscoverInput};
use super::draft::{self, DraftInput};
use super::queue::{self, QueueInput};
use super::{DraftResult, ProposeResult, QueueItem, ScoredCandidate, WorkflowError};

/// Input for a full discovery cycle.
#[derive(Debug, Clone)]
pub struct CycleInput {
    /// Search query (optional — falls back to product keywords).
    pub query: Option<String>,
    /// Minimum score threshold.
    pub min_score: Option<f64>,
    /// Maximum candidates to discover.
    pub limit: Option<u32>,
    /// Only discover tweets newer than this ID.
    pub since_id: Option<String>,
    /// Whether to mention the product in generated replies.
    pub mention_product: bool,
}

/// Report from a completed discovery cycle.
#[derive(Debug, Clone)]
pub struct CycleReport {
    /// Candidates found during discovery.
    pub discovered: Vec<ScoredCandidate>,
    /// The search query that was used.
    pub query_used: String,
    /// Draft results (one per candidate that was drafted).
    pub drafts: Vec<DraftResult>,
    /// Queue/execution results.
    pub queued: Vec<ProposeResult>,
    /// Summary counts.
    pub summary: CycleSummary,
}

/// Summary statistics for the cycle.
#[derive(Debug, Clone)]
pub struct CycleSummary {
    pub candidates_found: usize,
    pub drafts_generated: usize,
    pub drafts_failed: usize,
    pub replies_queued: usize,
    pub replies_executed: usize,
    pub replies_blocked: usize,
}

/// Run a complete discovery cycle: discover → draft → queue.
///
/// This is the canonical entrypoint for autopilot loops. It composes the
/// three atomic steps in sequence, passing outputs forward.
///
/// The cycle is deterministic: given the same inputs and external state,
/// it produces the same outputs. Side effects (DB writes, X API calls)
/// are explicit and auditable through the step functions.
pub async fn run_discovery_cycle(
    db: &DbPool,
    x_client: &dyn XApiClient,
    llm: &Arc<dyn LlmProvider>,
    config: &Config,
    input: CycleInput,
) -> Result<CycleReport, WorkflowError> {
    // Step 1: Discover — search, score, rank
    let discover_output = discover::execute(
        db,
        x_client,
        config,
        DiscoverInput {
            query: input.query,
            min_score: input.min_score,
            limit: input.limit,
            since_id: input.since_id,
        },
    )
    .await?;

    if discover_output.candidates.is_empty() {
        return Ok(CycleReport {
            query_used: discover_output.query_used,
            discovered: vec![],
            drafts: vec![],
            queued: vec![],
            summary: CycleSummary {
                candidates_found: 0,
                drafts_generated: 0,
                drafts_failed: 0,
                replies_queued: 0,
                replies_executed: 0,
                replies_blocked: 0,
            },
        });
    }

    // Filter to actionable candidates (skip already-replied and low-action)
    let actionable_ids: Vec<String> = discover_output
        .candidates
        .iter()
        .filter(|c| !c.already_replied && c.recommended_action != "skip")
        .map(|c| c.tweet_id.clone())
        .collect();

    let candidates_found = discover_output.candidates.len();

    if actionable_ids.is_empty() {
        return Ok(CycleReport {
            query_used: discover_output.query_used,
            discovered: discover_output.candidates,
            drafts: vec![],
            queued: vec![],
            summary: CycleSummary {
                candidates_found,
                drafts_generated: 0,
                drafts_failed: 0,
                replies_queued: 0,
                replies_executed: 0,
                replies_blocked: 0,
            },
        });
    }

    // Step 2: Draft — generate LLM replies for actionable candidates
    let drafts = draft::execute(
        db,
        llm,
        config,
        DraftInput {
            candidate_ids: actionable_ids,
            archetype: None,
            mention_product: input.mention_product,
            account_id: None,
        },
    )
    .await?;

    // Collect successful drafts as queue items
    let queue_items: Vec<QueueItem> = drafts
        .iter()
        .filter_map(|d| match d {
            DraftResult::Success {
                candidate_id,
                draft_text,
                ..
            } => Some(QueueItem {
                candidate_id: candidate_id.clone(),
                pre_drafted_text: Some(draft_text.clone()),
            }),
            DraftResult::Error { .. } => None,
        })
        .collect();

    let drafts_generated = queue_items.len();
    let drafts_failed = drafts.len() - drafts_generated;

    // Step 3: Queue — safety check + route to approval or execute
    let queued = if queue_items.is_empty() {
        vec![]
    } else {
        queue::execute(
            db,
            Some(x_client),
            Some(llm),
            config,
            QueueInput {
                items: queue_items,
                mention_product: input.mention_product,
            },
        )
        .await?
    };

    // Count results
    let replies_queued = queued
        .iter()
        .filter(|r| matches!(r, ProposeResult::Queued { .. }))
        .count();
    let replies_executed = queued
        .iter()
        .filter(|r| matches!(r, ProposeResult::Executed { .. }))
        .count();
    let replies_blocked = queued
        .iter()
        .filter(|r| matches!(r, ProposeResult::Blocked { .. }))
        .count();

    Ok(CycleReport {
        query_used: discover_output.query_used,
        discovered: discover_output.candidates,
        drafts,
        queued,
        summary: CycleSummary {
            candidates_found,
            drafts_generated,
            drafts_failed,
            replies_queued,
            replies_executed,
            replies_blocked,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::ScoreBreakdown;

    fn make_candidate(tweet_id: &str, already_replied: bool, action: &str) -> ScoredCandidate {
        ScoredCandidate {
            tweet_id: tweet_id.to_string(),
            author_username: "user".to_string(),
            author_followers: 100,
            text: "test".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            score_total: 50.0,
            score_breakdown: ScoreBreakdown {
                keyword_relevance: 10.0,
                follower: 10.0,
                recency: 10.0,
                engagement: 10.0,
                reply_count: 5.0,
                content_type: 5.0,
            },
            matched_keywords: vec!["test".to_string()],
            recommended_action: action.to_string(),
            already_replied,
        }
    }

    // ── Actionable candidate filtering ──────────────────────────────

    #[test]
    fn filter_actionable_excludes_already_replied() {
        let candidates = vec![
            make_candidate("t1", false, "reply"),
            make_candidate("t2", true, "reply"),
            make_candidate("t3", false, "reply"),
        ];

        let actionable: Vec<String> = candidates
            .iter()
            .filter(|c| !c.already_replied && c.recommended_action != "skip")
            .map(|c| c.tweet_id.clone())
            .collect();

        assert_eq!(actionable, vec!["t1", "t3"]);
    }

    #[test]
    fn filter_actionable_excludes_skip_action() {
        let candidates = vec![
            make_candidate("t1", false, "reply"),
            make_candidate("t2", false, "skip"),
            make_candidate("t3", false, "like"),
        ];

        let actionable: Vec<String> = candidates
            .iter()
            .filter(|c| !c.already_replied && c.recommended_action != "skip")
            .map(|c| c.tweet_id.clone())
            .collect();

        assert_eq!(actionable, vec!["t1", "t3"]);
    }

    #[test]
    fn filter_actionable_all_replied_returns_empty() {
        let candidates = vec![
            make_candidate("t1", true, "reply"),
            make_candidate("t2", true, "reply"),
        ];

        let actionable: Vec<String> = candidates
            .iter()
            .filter(|c| !c.already_replied && c.recommended_action != "skip")
            .map(|c| c.tweet_id.clone())
            .collect();

        assert!(actionable.is_empty());
    }

    #[test]
    fn filter_actionable_all_skip_returns_empty() {
        let candidates = vec![
            make_candidate("t1", false, "skip"),
            make_candidate("t2", false, "skip"),
        ];

        let actionable: Vec<String> = candidates
            .iter()
            .filter(|c| !c.already_replied && c.recommended_action != "skip")
            .map(|c| c.tweet_id.clone())
            .collect();

        assert!(actionable.is_empty());
    }

    // ── Queue item collection from drafts ───────────────────────────

    #[test]
    fn collect_queue_items_from_drafts() {
        let drafts = vec![
            DraftResult::Success {
                candidate_id: "t1".to_string(),
                draft_text: "Great point!".to_string(),
                archetype: "agree_and_expand".to_string(),
                char_count: 12,
                confidence: "high".to_string(),
                risks: vec![],
                vault_citations: vec![],
            },
            DraftResult::Error {
                candidate_id: "t2".to_string(),
                error_code: "llm_error".to_string(),
                error_message: "timeout".to_string(),
            },
            DraftResult::Success {
                candidate_id: "t3".to_string(),
                draft_text: "Interesting!".to_string(),
                archetype: "ask_question".to_string(),
                char_count: 12,
                confidence: "medium".to_string(),
                risks: vec![],
                vault_citations: vec![],
            },
        ];

        let queue_items: Vec<QueueItem> = drafts
            .iter()
            .filter_map(|d| match d {
                DraftResult::Success {
                    candidate_id,
                    draft_text,
                    ..
                } => Some(QueueItem {
                    candidate_id: candidate_id.clone(),
                    pre_drafted_text: Some(draft_text.clone()),
                }),
                DraftResult::Error { .. } => None,
            })
            .collect();

        assert_eq!(queue_items.len(), 2);
        assert_eq!(queue_items[0].candidate_id, "t1");
        assert_eq!(
            queue_items[0].pre_drafted_text.as_deref(),
            Some("Great point!")
        );
        assert_eq!(queue_items[1].candidate_id, "t3");

        let drafts_generated = queue_items.len();
        let drafts_failed = drafts.len() - drafts_generated;
        assert_eq!(drafts_generated, 2);
        assert_eq!(drafts_failed, 1);
    }

    // ── Summary counting from ProposeResult ─────────────────────────

    #[test]
    fn count_propose_results() {
        let results = vec![
            ProposeResult::Queued {
                candidate_id: "t1".to_string(),
                approval_queue_id: 1,
            },
            ProposeResult::Executed {
                candidate_id: "t2".to_string(),
                reply_tweet_id: "r1".to_string(),
            },
            ProposeResult::Blocked {
                candidate_id: "t3".to_string(),
                reason: "rate limit".to_string(),
            },
            ProposeResult::Queued {
                candidate_id: "t4".to_string(),
                approval_queue_id: 2,
            },
        ];

        let queued = results
            .iter()
            .filter(|r| matches!(r, ProposeResult::Queued { .. }))
            .count();
        let executed = results
            .iter()
            .filter(|r| matches!(r, ProposeResult::Executed { .. }))
            .count();
        let blocked = results
            .iter()
            .filter(|r| matches!(r, ProposeResult::Blocked { .. }))
            .count();

        assert_eq!(queued, 2);
        assert_eq!(executed, 1);
        assert_eq!(blocked, 1);
    }

    // ── CycleInput construction ─────────────────────────────────────

    #[test]
    fn cycle_input_defaults() {
        let input = CycleInput {
            query: None,
            min_score: None,
            limit: None,
            since_id: None,
            mention_product: false,
        };
        assert!(input.query.is_none());
        assert!(!input.mention_product);
    }

    #[test]
    fn cycle_input_with_all_fields() {
        let input = CycleInput {
            query: Some("rust async".to_string()),
            min_score: Some(50.0),
            limit: Some(20),
            since_id: Some("12345".to_string()),
            mention_product: true,
        };
        assert_eq!(input.query.as_deref(), Some("rust async"));
        assert_eq!(input.min_score, Some(50.0));
        assert_eq!(input.limit, Some(20));
        assert!(input.mention_product);
    }

    // ── CycleSummary zeroed report ──────────────────────────────────

    #[test]
    fn cycle_summary_empty() {
        let summary = CycleSummary {
            candidates_found: 0,
            drafts_generated: 0,
            drafts_failed: 0,
            replies_queued: 0,
            replies_executed: 0,
            replies_blocked: 0,
        };
        assert_eq!(summary.candidates_found, 0);
        assert_eq!(summary.drafts_generated, 0);
        assert_eq!(summary.replies_queued, 0);
    }

    // ── CycleReport with no candidates ──────────────────────────────

    #[test]
    fn cycle_report_empty_candidates() {
        let report = CycleReport {
            query_used: "rust".to_string(),
            discovered: vec![],
            drafts: vec![],
            queued: vec![],
            summary: CycleSummary {
                candidates_found: 0,
                drafts_generated: 0,
                drafts_failed: 0,
                replies_queued: 0,
                replies_executed: 0,
                replies_blocked: 0,
            },
        };
        assert_eq!(report.query_used, "rust");
        assert!(report.discovered.is_empty());
        assert!(report.drafts.is_empty());
        assert!(report.queued.is_empty());
    }
}
