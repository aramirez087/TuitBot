//! Approval queue tools: list, approve, reject, approve_all.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

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
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
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
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({ "pending_count": count }))
                .with_meta(meta)
                .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error counting pending approvals: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Approve a specific item by ID.
pub async fn approve_item(pool: &DbPool, id: i64, config: &Config) -> String {
    let start = Instant::now();

    match storage::approval_queue::update_status(pool, id, "approved").await {
        Ok(()) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({ "status": "approved", "id": id }))
                .with_meta(meta)
                .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error approving item {id}: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Reject a specific item by ID.
pub async fn reject_item(pool: &DbPool, id: i64, config: &Config) -> String {
    let start = Instant::now();

    match storage::approval_queue::update_status(pool, id, "rejected").await {
        Ok(()) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({ "status": "rejected", "id": id }))
                .with_meta(meta)
                .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error rejecting item {id}: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Approve all pending items.
pub async fn approve_all(pool: &DbPool, config: &Config) -> String {
    let start = Instant::now();

    match storage::approval_queue::get_pending(pool).await {
        Ok(items) => {
            let mut approved = 0i64;
            let mut errors = 0i64;
            for item in &items {
                match storage::approval_queue::update_status(pool, item.id, "approved").await {
                    Ok(()) => approved += 1,
                    Err(_) => errors += 1,
                }
            }
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({
                "approved": approved,
                "errors": errors,
                "total": items.len(),
            }))
            .with_meta(meta)
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching pending approvals: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
