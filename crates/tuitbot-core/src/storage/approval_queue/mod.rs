//! Storage operations for the approval queue.
//!
//! Provides CRUD operations for queuing posts for human review
//! when `approval_mode` is enabled.

mod edit_history;
mod queries;
#[cfg(test)]
mod tests;

pub use edit_history::{get_edit_history, record_edit, EditHistoryEntry};
pub use queries::*;

/// Row type for approval queue queries (expanded with review and QA metadata).
#[derive(Debug, Clone, sqlx::FromRow)]
struct ApprovalRow {
    id: i64,
    action_type: String,
    target_tweet_id: String,
    target_author: String,
    generated_content: String,
    topic: String,
    archetype: String,
    score: f64,
    status: String,
    created_at: String,
    media_paths: String,
    reviewed_by: Option<String>,
    review_notes: Option<String>,
    reason: Option<String>,
    detected_risks: String,
    qa_report: String,
    qa_hard_flags: String,
    qa_soft_flags: String,
    qa_recommendations: String,
    qa_score: f64,
    qa_requires_override: i64,
    qa_override_by: Option<String>,
    qa_override_note: Option<String>,
    qa_override_at: Option<String>,
}

/// A pending item in the approval queue.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ApprovalItem {
    pub id: i64,
    pub action_type: String,
    pub target_tweet_id: String,
    pub target_author: String,
    pub generated_content: String,
    pub topic: String,
    pub archetype: String,
    pub score: f64,
    pub status: String,
    pub created_at: String,
    /// JSON-encoded list of local media file paths.
    #[serde(serialize_with = "serialize_json_string")]
    pub media_paths: String,
    pub reviewed_by: Option<String>,
    pub review_notes: Option<String>,
    pub reason: Option<String>,
    /// JSON-encoded list of detected risks.
    #[serde(serialize_with = "serialize_json_string")]
    pub detected_risks: String,
    /// Full QA report payload as JSON.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_report: String,
    /// JSON-encoded hard QA flags.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_hard_flags: String,
    /// JSON-encoded soft QA flags.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_soft_flags: String,
    /// JSON-encoded QA recommendations.
    #[serde(serialize_with = "serialize_json_string")]
    pub qa_recommendations: String,
    /// QA score summary (0-100).
    pub qa_score: f64,
    /// Whether approval requires explicit hard-flag override.
    pub qa_requires_override: bool,
    /// Actor who performed override.
    pub qa_override_by: Option<String>,
    /// Required override note.
    pub qa_override_note: Option<String>,
    /// Timestamp of override action.
    pub qa_override_at: Option<String>,
}

/// Serialize a JSON-encoded string as a raw JSON value.
///
/// The database stores `media_paths` and `detected_risks` as JSON strings.
/// This serializer emits them as actual JSON arrays in the API response.
fn serialize_json_string<S: serde::Serializer>(
    value: &str,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    let parsed: serde_json::Value =
        serde_json::from_str(value).unwrap_or(serde_json::Value::Array(vec![]));
    parsed.serialize(serializer)
}

impl From<ApprovalRow> for ApprovalItem {
    fn from(r: ApprovalRow) -> Self {
        Self {
            id: r.id,
            action_type: r.action_type,
            target_tweet_id: r.target_tweet_id,
            target_author: r.target_author,
            generated_content: r.generated_content,
            topic: r.topic,
            archetype: r.archetype,
            score: r.score,
            status: r.status,
            created_at: r.created_at,
            media_paths: r.media_paths,
            reviewed_by: r.reviewed_by,
            review_notes: r.review_notes,
            reason: r.reason,
            detected_risks: r.detected_risks,
            qa_report: r.qa_report,
            qa_hard_flags: r.qa_hard_flags,
            qa_soft_flags: r.qa_soft_flags,
            qa_recommendations: r.qa_recommendations,
            qa_score: r.qa_score,
            qa_requires_override: r.qa_requires_override != 0,
            qa_override_by: r.qa_override_by,
            qa_override_note: r.qa_override_note,
            qa_override_at: r.qa_override_at,
        }
    }
}

/// Counts of approval items grouped by status.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ApprovalStats {
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
}

/// Optional review metadata for approve/reject actions.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ReviewAction {
    pub actor: Option<String>,
    pub notes: Option<String>,
}
