//! Approval queue tools: list, approve, reject, approve_all.

use serde::Serialize;

use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

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
pub async fn list_pending(pool: &DbPool) -> String {
    match storage::approval_queue::get_pending(pool).await {
        Ok(items) => {
            let out: Vec<ApprovalItemOut> = items.iter().map(item_to_out).collect();
            serde_json::to_string_pretty(&out)
                .unwrap_or_else(|e| format!("Error serializing approvals: {e}"))
        }
        Err(e) => format!("Error fetching pending approvals: {e}"),
    }
}

/// Get count of pending approval items.
pub async fn get_pending_count(pool: &DbPool) -> String {
    match storage::approval_queue::pending_count(pool).await {
        Ok(count) => serde_json::json!({ "pending_count": count }).to_string(),
        Err(e) => format!("Error counting pending approvals: {e}"),
    }
}

/// Approve a specific item by ID.
pub async fn approve_item(pool: &DbPool, id: i64) -> String {
    match storage::approval_queue::update_status(pool, id, "approved").await {
        Ok(()) => serde_json::json!({ "status": "approved", "id": id }).to_string(),
        Err(e) => format!("Error approving item {id}: {e}"),
    }
}

/// Reject a specific item by ID.
pub async fn reject_item(pool: &DbPool, id: i64) -> String {
    match storage::approval_queue::update_status(pool, id, "rejected").await {
        Ok(()) => serde_json::json!({ "status": "rejected", "id": id }).to_string(),
        Err(e) => format!("Error rejecting item {id}: {e}"),
    }
}

/// Approve all pending items.
pub async fn approve_all(pool: &DbPool) -> String {
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
            serde_json::json!({
                "approved": approved,
                "errors": errors,
                "total": items.len(),
            })
            .to_string()
        }
        Err(e) => format!("Error fetching pending approvals: {e}"),
    }
}
