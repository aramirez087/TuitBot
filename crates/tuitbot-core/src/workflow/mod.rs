//! Workflow layer: stateful composite operations over toolkit primitives.
//!
//! Each workflow step defines explicit typed IO contracts and composes
//! toolkit functions with DB and LLM state. Steps are the building blocks
//! for both MCP composite tools and autopilot cycles.
//!
//! **Layer rules (charter §5):**
//! - Workflow MAY access DB (`SqlitePool`) and LLM (`LlmProvider`).
//! - Workflow MUST call X API operations through `toolkit::*`, never `XApiClient` directly.
//! - Workflow MUST NOT import from `automation::`.

pub mod discover;
pub mod draft;
pub mod orchestrate;
pub mod publish;
pub mod queue;
pub mod thread_plan;

#[cfg(test)]
mod e2e_tests;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use serde::Serialize;

use crate::content::frameworks::ReplyArchetype;
use crate::error::XApiError;
use crate::llm::{GenerationParams, LlmProvider, LlmResponse};
use crate::toolkit::ToolkitError;
use crate::LlmError;

// ── WorkflowError ────────────────────────────────────────────────────

/// Errors from workflow operations.
///
/// Maps to existing `ErrorCode` variants in MCP responses (AD-10).
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Toolkit-level error (X API, validation).
    #[error(transparent)]
    Toolkit(#[from] ToolkitError),

    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Storage-layer error.
    #[error("storage error: {0}")]
    Storage(#[from] crate::error::StorageError),

    /// LLM provider not configured.
    #[error("LLM provider not configured")]
    LlmNotConfigured,

    /// LLM generation error.
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    /// X API client not configured.
    #[error("X API client not configured")]
    XNotConfigured,

    /// Invalid input parameter.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// ── SharedProvider ───────────────────────────────────────────────────

/// Bridge an `Arc<dyn LlmProvider>` into a `Box<dyn LlmProvider>` for
/// `ContentGenerator`, which requires owned provider instances.
///
/// This adapter clones the Arc to construct ContentGenerator while allowing
/// multiple workflow steps to share the same underlying LLM provider.
pub(crate) struct SharedProvider(pub Arc<dyn LlmProvider>);

#[async_trait::async_trait]
impl LlmProvider for SharedProvider {
    fn name(&self) -> &str {
        self.0.name()
    }

    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        params: &GenerationParams,
    ) -> Result<LlmResponse, LlmError> {
        self.0.complete(system, user_message, params).await
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        self.0.health_check().await
    }
}

// ── Shared IO types ─────────────────────────────────────────────────

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

/// Input item for the queue step.
#[derive(Debug, Clone)]
pub struct QueueItem {
    /// The tweet ID to reply to.
    pub candidate_id: String,
    /// Pre-drafted reply text. If omitted, generates one via LLM.
    pub pre_drafted_text: Option<String>,
}

// ── Helper: parse archetype string ──────────────────────────────────

/// Parse an archetype string into a `ReplyArchetype`.
pub fn parse_archetype(s: &str) -> Option<ReplyArchetype> {
    match s.to_lowercase().replace(' ', "_").as_str() {
        "agree_and_expand" | "agreeandexpand" => Some(ReplyArchetype::AgreeAndExpand),
        "respectful_disagree" | "respectfuldisagree" => Some(ReplyArchetype::RespectfulDisagree),
        "add_data" | "adddata" => Some(ReplyArchetype::AddData),
        "ask_question" | "askquestion" => Some(ReplyArchetype::AskQuestion),
        "share_experience" | "shareexperience" => Some(ReplyArchetype::ShareExperience),
        _ => None,
    }
}

// ── Helper: build content generator ─────────────────────────────────

/// Build a `ContentGenerator` from a shared LLM provider.
pub(crate) fn make_content_gen(
    llm: &Arc<dyn LlmProvider>,
    business: &crate::config::BusinessProfile,
) -> crate::content::ContentGenerator {
    let provider = Box::new(SharedProvider(Arc::clone(llm)));
    crate::content::ContentGenerator::new(provider, business.clone())
}

// ── Helper: convert XApiError fields ────────────────────────────────

impl WorkflowError {
    /// Convenience: create from a toolkit error.
    pub fn from_x_api(e: XApiError) -> Self {
        Self::Toolkit(ToolkitError::XApi(e))
    }
}

// ── Re-exports for convenience ──────────────────────────────────────

pub use discover::{DiscoverInput, DiscoverOutput};
pub use draft::DraftInput;
pub use orchestrate::{CycleInput, CycleReport};
pub use publish::PublishOutput;
pub use queue::QueueInput;
pub use thread_plan::{ThreadPlanInput, ThreadPlanOutput};
