//! Composite goal-oriented MCP tools.
//!
//! Each tool orchestrates multiple core primitives into a single high-value
//! call, reducing agent round-trips while enforcing safety at every mutation.

pub mod draft_replies;
pub mod find_opportunities;
pub mod propose_queue;
pub mod thread_plan;

#[cfg(test)]
mod tests;

use serde::Serialize;

/// A tweet candidate scored for reply-worthiness.
#[derive(Debug, Clone, Serialize)]
pub struct ScoredCandidate {
    pub tweet_id: String,
    pub author_username: String,
    pub author_followers: u64,
    pub text: String,
    pub created_at: String,
    pub score_total: f32,
    pub score_breakdown: ScoreBreakdown,
    pub matched_keywords: Vec<String>,
    pub recommended_action: String,
    pub already_replied: bool,
}

/// Per-signal score breakdown.
#[derive(Debug, Clone, Serialize)]
pub struct ScoreBreakdown {
    pub keyword_relevance: f32,
    pub follower: f32,
    pub recency: f32,
    pub engagement: f32,
    pub reply_count: f32,
    pub content_type: f32,
}

/// Result of drafting a reply for a single candidate.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum DraftResult {
    #[serde(rename = "success")]
    Success {
        candidate_id: String,
        draft_text: String,
        archetype: String,
        char_count: usize,
        confidence: String,
        risks: Vec<String>,
    },
    #[serde(rename = "error")]
    Error {
        candidate_id: String,
        error_code: String,
        error_message: String,
    },
}

/// Result of proposing/queueing a single reply.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum ProposeResult {
    #[serde(rename = "queued")]
    Queued {
        candidate_id: String,
        approval_queue_id: i64,
    },
    #[serde(rename = "executed")]
    Executed {
        candidate_id: String,
        reply_tweet_id: String,
    },
    #[serde(rename = "blocked")]
    Blocked {
        candidate_id: String,
        reason: String,
    },
}
