//! Factory for building [`ApprovalItem`] instances with realistic test data.
//!
//! # Example
//! ```rust
//! use tuitbot_core::testing::ApprovalItemFactory;
//!
//! let item = ApprovalItemFactory::new().build();
//! let approved = ApprovalItemFactory::new().approved().build();
//! let risky = ApprovalItemFactory::new()
//!     .with_hard_flags(vec!["harassment".to_string()])
//!     .requires_override()
//!     .build();
//! ```

use crate::storage::approval_queue::ApprovalItem;

static COUNTER: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

fn next_id() -> i64 {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Builder for [`ApprovalItem`] test instances.
pub struct ApprovalItemFactory {
    id: Option<i64>,
    action_type: String,
    target_tweet_id: String,
    target_author: String,
    generated_content: String,
    topic: String,
    archetype: String,
    score: f64,
    status: String,
    hard_flags: Vec<String>,
    soft_flags: Vec<String>,
    detected_risks: Vec<String>,
    qa_score: f64,
    requires_override: bool,
    reviewed_by: Option<String>,
    review_notes: Option<String>,
    reason: Option<String>,
}

impl Default for ApprovalItemFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalItemFactory {
    pub fn new() -> Self {
        Self {
            id: None,
            action_type: "reply".to_string(),
            target_tweet_id: "1234567890123456789".to_string(),
            target_author: "test_author".to_string(),
            generated_content: "Great insight! This connects to the core problem you described."
                .to_string(),
            topic: "product-led-growth".to_string(),
            archetype: "insight_builder".to_string(),
            score: 0.87,
            status: "pending".to_string(),
            hard_flags: vec![],
            soft_flags: vec![],
            detected_risks: vec![],
            qa_score: 0.92,
            requires_override: false,
            reviewed_by: None,
            review_notes: None,
            reason: None,
        }
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_action_type(mut self, t: impl Into<String>) -> Self {
        self.action_type = t.into();
        self
    }

    pub fn tweet_action(mut self) -> Self {
        self.action_type = "tweet".to_string();
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.target_author = author.into();
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.generated_content = content.into();
        self
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = topic.into();
        self
    }

    pub fn approved(mut self) -> Self {
        self.status = "approved".to_string();
        self.reviewed_by = Some("human_reviewer".to_string());
        self.review_notes = Some("Looks good.".to_string());
        self
    }

    pub fn rejected(mut self) -> Self {
        self.status = "rejected".to_string();
        self.reason = Some("Off-topic for current campaign.".to_string());
        self.reviewed_by = Some("human_reviewer".to_string());
        self
    }

    pub fn with_hard_flags(mut self, flags: Vec<String>) -> Self {
        self.hard_flags = flags;
        self
    }

    pub fn with_soft_flags(mut self, flags: Vec<String>) -> Self {
        self.soft_flags = flags;
        self
    }

    pub fn with_detected_risks(mut self, risks: Vec<String>) -> Self {
        self.detected_risks = risks;
        self
    }

    pub fn requires_override(mut self) -> Self {
        self.requires_override = true;
        self
    }

    pub fn low_score(mut self) -> Self {
        self.score = 0.35;
        self.qa_score = 0.40;
        self
    }

    pub fn build(self) -> ApprovalItem {
        let now = "2026-03-14T00:00:00.000Z".to_string();
        ApprovalItem {
            id: self.id.unwrap_or_else(next_id),
            action_type: self.action_type,
            target_tweet_id: self.target_tweet_id,
            target_author: self.target_author,
            generated_content: self.generated_content,
            topic: self.topic,
            archetype: self.archetype,
            score: self.score,
            status: self.status,
            created_at: now,
            media_paths: "[]".to_string(),
            reviewed_by: self.reviewed_by,
            review_notes: self.review_notes,
            reason: self.reason,
            detected_risks: serde_json::to_string(&self.detected_risks)
                .unwrap_or_else(|_| "[]".to_string()),
            qa_report: "{}".to_string(),
            qa_hard_flags: serde_json::to_string(&self.hard_flags)
                .unwrap_or_else(|_| "[]".to_string()),
            qa_soft_flags: serde_json::to_string(&self.soft_flags)
                .unwrap_or_else(|_| "[]".to_string()),
            qa_recommendations: "[]".to_string(),
            qa_score: self.qa_score,
            qa_requires_override: self.requires_override,
            qa_override_by: None,
            qa_override_note: None,
            qa_override_at: None,
            source_node_id: None,
            source_seed_id: None,
            source_chunks_json: "[]".to_string(),
            scheduled_for: None,
        }
    }

    pub fn build_many(count: usize) -> Vec<ApprovalItem> {
        (0..count)
            .map(|i| {
                ApprovalItemFactory::new()
                    .with_author(format!("author_{i}"))
                    .with_content(format!("Test reply content #{i}"))
                    .with_score(0.5 + (i as f64) * 0.05)
                    .build()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pending_item_by_default() {
        let item = ApprovalItemFactory::new().build();
        assert_eq!(item.status, "pending");
        assert!(item.qa_hard_flags.contains("[]"));
        assert!(!item.qa_requires_override);
    }

    #[test]
    fn approved_item_has_reviewer() {
        let item = ApprovalItemFactory::new().approved().build();
        assert_eq!(item.status, "approved");
        assert!(item.reviewed_by.is_some());
    }

    #[test]
    fn rejected_item_has_reason() {
        let item = ApprovalItemFactory::new().rejected().build();
        assert_eq!(item.status, "rejected");
        assert!(item.reason.is_some());
    }

    #[test]
    fn hard_flags_serialised_as_json() {
        let item = ApprovalItemFactory::new()
            .with_hard_flags(vec!["harassment".to_string()])
            .requires_override()
            .build();
        assert!(item.qa_hard_flags.contains("harassment"));
        assert!(item.qa_requires_override);
    }

    #[test]
    fn build_many_produces_unique_ids() {
        let items = ApprovalItemFactory::build_many(5);
        assert_eq!(items.len(), 5);
        let ids: std::collections::HashSet<_> = items.iter().map(|i| i.id).collect();
        assert_eq!(ids.len(), 5);
    }
}
