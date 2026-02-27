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
