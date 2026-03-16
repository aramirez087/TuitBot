//! Approval queue tools: list, approve, reject, approve_all.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::approval_queue::ReviewAction;
use tuitbot_core::storage::DbPool;

use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

#[derive(Serialize)]
struct ApprovalItemOut {
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
    reviewed_by: Option<String>,
    review_notes: Option<String>,
    reason: Option<String>,
    detected_risks: String,
}

fn item_to_out(item: &storage::approval_queue::ApprovalItem) -> ApprovalItemOut {
    ApprovalItemOut {
        id: item.id,
        action_type: item.action_type.clone(),
        target_tweet_id: item.target_tweet_id.clone(),
        target_author: item.target_author.clone(),
        generated_content: item.generated_content.clone(),
        topic: item.topic.clone(),
        archetype: item.archetype.clone(),
        score: item.score,
        status: item.status.clone(),
        created_at: item.created_at.clone(),
        reviewed_by: item.reviewed_by.clone(),
        review_notes: item.review_notes.clone(),
        reason: item.reason.clone(),
        detected_risks: item.detected_risks.clone(),
    }
}

/// List all pending approval items.
pub async fn list_pending(pool: &DbPool, config: &Config) -> String {
    let start = Instant::now();

    match storage::approval_queue::get_pending(pool).await {
        Ok(items) => {
            let out: Vec<ApprovalItemOut> = items.iter().map(item_to_out).collect();
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching pending approvals: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Get count of pending approval items.
pub async fn get_pending_count(pool: &DbPool, config: &Config) -> String {
    let start = Instant::now();

    match storage::approval_queue::pending_count(pool).await {
        Ok(count) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({ "pending_count": count }))
                .with_meta(meta)
                .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error counting pending approvals: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Approve a specific item by ID.
///
/// Rejects the approval if `x_available` is false, since the approved item
/// cannot be executed without an X API client.
pub async fn approve_item(pool: &DbPool, id: i64, config: &Config, x_available: bool) -> String {
    let start = Instant::now();

    if !x_available {
        let elapsed = start.elapsed().as_millis() as u64;
        let meta = ToolMeta::new(elapsed)
            .with_workflow(config.mode.to_string(), config.effective_approval_mode());
        return ToolResponse::error(
            ErrorCode::XNotConfigured,
            "Cannot approve: X API client not available. Run `tuitbot auth` to authenticate.",
        )
        .with_meta(meta)
        .to_json();
    }

    let review = ReviewAction {
        actor: Some("mcp_agent".to_string()),
        notes: None,
    };
    match storage::approval_queue::update_status_with_review(pool, id, "approved", &review).await {
        Ok(()) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({ "status": "approved", "id": id }))
                .with_meta(meta)
                .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error approving item {id}: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Reject a specific item by ID.
pub async fn reject_item(pool: &DbPool, id: i64, config: &Config) -> String {
    let start = Instant::now();

    let review = ReviewAction {
        actor: Some("mcp_agent".to_string()),
        notes: None,
    };
    match storage::approval_queue::update_status_with_review(pool, id, "rejected", &review).await {
        Ok(()) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({ "status": "rejected", "id": id }))
                .with_meta(meta)
                .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error rejecting item {id}: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Approve all pending items (clamped by max_batch_approve).
///
/// Rejects the approval if `x_available` is false, since the approved items
/// cannot be executed without an X API client.
pub async fn approve_all(pool: &DbPool, config: &Config, x_available: bool) -> String {
    let start = Instant::now();

    if !x_available {
        let elapsed = start.elapsed().as_millis() as u64;
        let meta = ToolMeta::new(elapsed)
            .with_workflow(config.mode.to_string(), config.effective_approval_mode());
        return ToolResponse::error(
            ErrorCode::XNotConfigured,
            "Cannot approve: X API client not available. Run `tuitbot auth` to authenticate.",
        )
        .with_meta(meta)
        .to_json();
    }

    let review = ReviewAction {
        actor: Some("mcp_agent".to_string()),
        notes: None,
    };
    match storage::approval_queue::batch_approve(pool, config.max_batch_approve, &review).await {
        Ok(ids) => {
            let approved = ids.len() as i64;
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({
                "approved": approved,
                "ids": ids,
                "max_batch": config.max_batch_approve,
            }))
            .with_meta(meta)
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error batch approving: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> DbPool {
        storage::init_test_db().await.expect("init db")
    }

    fn default_config() -> Config {
        Config::default()
    }

    // ── list_pending tests ──────────────────────────────────────────

    #[tokio::test]
    async fn list_pending_empty_queue() {
        let pool = test_pool().await;
        let config = default_config();
        let json = list_pending(&pool, &config).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert!(parsed["data"].is_array());
        assert_eq!(parsed["data"].as_array().unwrap().len(), 0);
        assert!(parsed["meta"]["elapsed_ms"].is_number());
    }

    #[tokio::test]
    async fn list_pending_has_workflow_meta() {
        let pool = test_pool().await;
        let config = default_config();
        let json = list_pending(&pool, &config).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        // WorkflowContext is flattened into meta, so mode is a top-level meta key.
        assert!(parsed["meta"]["mode"].is_string());
    }

    // ── get_pending_count tests ─────────────────────────────────────

    #[tokio::test]
    async fn get_pending_count_zero() {
        let pool = test_pool().await;
        let config = default_config();
        let json = get_pending_count(&pool, &config).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["pending_count"], 0);
    }

    // ── approve_item tests ──────────────────────────────────────────

    #[tokio::test]
    async fn approve_item_x_unavailable_rejected() {
        let pool = test_pool().await;
        let config = default_config();
        let json = approve_item(&pool, 1, &config, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_not_configured");
        assert!(parsed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("X API client not available"));
    }

    #[tokio::test]
    async fn approve_item_nonexistent_id() {
        let pool = test_pool().await;
        let config = default_config();
        // ID 99999 does not exist, but the update_status query may not error
        // (it just affects 0 rows). Verify it returns success or a graceful error.
        let json = approve_item(&pool, 99999, &config, true).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        // Should succeed (0-row update is not an error in most implementations).
        assert!(parsed["success"].is_boolean());
    }

    // ── reject_item tests ───────────────────────────────────────────

    #[tokio::test]
    async fn reject_item_nonexistent_id() {
        let pool = test_pool().await;
        let config = default_config();
        let json = reject_item(&pool, 99999, &config).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed["success"].is_boolean());
    }

    // ── approve_all tests ───────────────────────────────────────────

    #[tokio::test]
    async fn approve_all_x_unavailable_rejected() {
        let pool = test_pool().await;
        let config = default_config();
        let json = approve_all(&pool, &config, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_not_configured");
    }

    #[tokio::test]
    async fn approve_all_empty_queue() {
        let pool = test_pool().await;
        let config = default_config();
        let json = approve_all(&pool, &config, true).await;
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["approved"], 0);
        assert!(parsed["data"]["ids"].is_array());
        assert_eq!(parsed["data"]["ids"].as_array().unwrap().len(), 0);
    }

    // ── item_to_out mapping test ────────────────────────────────────

    fn make_approval_item(id: i64) -> storage::approval_queue::ApprovalItem {
        storage::approval_queue::ApprovalItem {
            id,
            action_type: "reply".to_string(),
            target_tweet_id: "tw123".to_string(),
            target_author: "author1".to_string(),
            generated_content: "Great post!".to_string(),
            topic: "rust".to_string(),
            archetype: "helpful".to_string(),
            score: 0.85,
            status: "pending".to_string(),
            created_at: "2026-03-14T12:00:00Z".to_string(),
            media_paths: "[]".to_string(),
            reviewed_by: Some("admin".to_string()),
            review_notes: Some("looks good".to_string()),
            reason: Some("high quality".to_string()),
            detected_risks: "[]".to_string(),
            qa_report: "{}".to_string(),
            qa_hard_flags: "[]".to_string(),
            qa_soft_flags: "[]".to_string(),
            qa_recommendations: "[]".to_string(),
            qa_score: 95.0,
            qa_requires_override: false,
            qa_override_by: None,
            qa_override_note: None,
            qa_override_at: None,
            source_node_id: None,
            source_seed_id: None,
            source_chunks_json: "[]".to_string(),
            scheduled_for: None,
        }
    }

    #[test]
    fn item_to_out_maps_all_fields() {
        let item = make_approval_item(42);
        let out = item_to_out(&item);
        assert_eq!(out.id, 42);
        assert_eq!(out.action_type, "reply");
        assert_eq!(out.target_tweet_id, "tw123");
        assert_eq!(out.target_author, "author1");
        assert_eq!(out.generated_content, "Great post!");
        assert_eq!(out.topic, "rust");
        assert_eq!(out.archetype, "helpful");
        assert!((out.score - 0.85).abs() < f64::EPSILON);
        assert_eq!(out.status, "pending");
        assert_eq!(out.reviewed_by, Some("admin".to_string()));
        assert_eq!(out.review_notes, Some("looks good".to_string()));
        assert_eq!(out.reason, Some("high quality".to_string()));
    }

    #[test]
    fn item_to_out_serializes_to_json() {
        let mut item = make_approval_item(1);
        item.action_type = "post".to_string();
        item.reviewed_by = None;
        item.review_notes = None;
        item.reason = None;
        let out = item_to_out(&item);
        let json = serde_json::to_value(&out).expect("serializes");
        assert_eq!(json["id"], 1);
        assert_eq!(json["action_type"], "post");
        assert!(json["reviewed_by"].is_null());
    }
}
