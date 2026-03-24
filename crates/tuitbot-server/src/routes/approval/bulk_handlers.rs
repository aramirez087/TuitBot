//! Bulk approval/rejection handlers: approve or reject multiple items by ID.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::{action_log, approval_queue};

use crate::account::{require_approve, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::{AccountWsEvent, WsEvent};

use super::handlers::approve_single_item;

/// Request body for bulk approve/reject.
#[derive(Deserialize)]
pub struct BulkActionRequest {
    /// IDs to operate on. Required; empty list is a no-op.
    pub ids: Vec<i64>,
    /// Review metadata (actor, notes, etc.).
    #[serde(default)]
    pub review: approval_queue::ReviewAction,
}

/// Result for a single item in a bulk operation.
#[derive(serde::Serialize)]
pub struct BulkItemResult {
    pub id: i64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// `POST /api/approval/bulk/approve` — approve a list of items by ID.
///
/// Processes each ID independently. Partial failures are reported per-item in
/// the response; the overall HTTP status is 200 as long as the request itself
/// is valid. Callers must inspect `results[].ok` to detect per-item errors.
pub async fn bulk_approve(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<BulkActionRequest>,
) -> Result<Json<Value>, ApiError> {
    require_approve(&ctx)?;

    // Verify X auth tokens exist before allowing bulk approval.
    let token_path =
        tuitbot_core::storage::accounts::account_token_path(&state.data_dir, &ctx.account_id);
    if !token_path.exists() {
        return Err(ApiError::BadRequest(
            "Cannot approve: X API not authenticated. Complete X auth setup first.".to_string(),
        ));
    }

    let mut results: Vec<BulkItemResult> = Vec::with_capacity(body.ids.len());

    for &id in &body.ids {
        let item_result = approval_queue::get_by_id_for(&state.db, &ctx.account_id, id).await;
        match item_result {
            Err(e) => {
                results.push(BulkItemResult {
                    id,
                    ok: false,
                    error: Some(format!("fetch error: {e}")),
                });
            }
            Ok(None) => {
                results.push(BulkItemResult {
                    id,
                    ok: false,
                    error: Some(format!("item {id} not found")),
                });
            }
            Ok(Some(item)) => {
                if item.status != "pending" {
                    results.push(BulkItemResult {
                        id,
                        ok: false,
                        error: Some(format!("status is '{}', expected 'pending'", item.status)),
                    });
                    continue;
                }
                match approve_single_item(&state, &ctx.account_id, &item, &body.review).await {
                    Ok(()) => {
                        let _ = state.event_tx.send(AccountWsEvent {
                            account_id: ctx.account_id.clone(),
                            event: WsEvent::ApprovalUpdated {
                                id,
                                status: "approved".to_string(),
                                action_type: item.action_type,
                                actor: body.review.actor.clone(),
                            },
                        });
                        results.push(BulkItemResult {
                            id,
                            ok: true,
                            error: None,
                        });
                    }
                    Err(_e) => {
                        results.push(BulkItemResult {
                            id,
                            ok: false,
                            error: Some("approval failed".to_string()),
                        });
                    }
                }
            }
        }
    }

    let approved_count = results.iter().filter(|r| r.ok).count();
    let failed_count = results.iter().filter(|r| !r.ok).count();

    // Log aggregate result.
    let approved_ids: Vec<i64> = results.iter().filter(|r| r.ok).map(|r| r.id).collect();
    let metadata = json!({
        "approved_count": approved_count,
        "failed_count": failed_count,
        "approved_ids": approved_ids,
        "actor": body.review.actor,
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "approval_bulk_approved",
        "success",
        Some(&format!(
            "Bulk approved {approved_count}/{} items",
            body.ids.len()
        )),
        Some(&metadata.to_string()),
    )
    .await;

    Ok(Json(json!({
        "approved": approved_count,
        "failed": failed_count,
        "results": results,
    })))
}

/// `POST /api/approval/bulk/reject` — reject a list of items by ID.
///
/// Processes each ID independently. Partial failures are reported per-item in
/// the response body; HTTP status is 200 as long as the request is valid.
pub async fn bulk_reject(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<BulkActionRequest>,
) -> Result<Json<Value>, ApiError> {
    require_approve(&ctx)?;

    let mut results: Vec<BulkItemResult> = Vec::with_capacity(body.ids.len());

    for &id in &body.ids {
        let item_result = approval_queue::get_by_id_for(&state.db, &ctx.account_id, id).await;
        match item_result {
            Err(e) => {
                results.push(BulkItemResult {
                    id,
                    ok: false,
                    error: Some(format!("fetch error: {e}")),
                });
            }
            Ok(None) => {
                results.push(BulkItemResult {
                    id,
                    ok: false,
                    error: Some(format!("item {id} not found")),
                });
            }
            Ok(Some(item)) => {
                if item.status != "pending" {
                    results.push(BulkItemResult {
                        id,
                        ok: false,
                        error: Some(format!("status is '{}', expected 'pending'", item.status)),
                    });
                    continue;
                }
                let reject_result = approval_queue::update_status_with_review_for(
                    &state.db,
                    &ctx.account_id,
                    id,
                    "rejected",
                    &body.review,
                )
                .await;

                match reject_result {
                    Ok(()) => {
                        let _ = state.event_tx.send(AccountWsEvent {
                            account_id: ctx.account_id.clone(),
                            event: WsEvent::ApprovalUpdated {
                                id,
                                status: "rejected".to_string(),
                                action_type: item.action_type,
                                actor: body.review.actor.clone(),
                            },
                        });
                        results.push(BulkItemResult {
                            id,
                            ok: true,
                            error: None,
                        });
                    }
                    Err(_e) => {
                        results.push(BulkItemResult {
                            id,
                            ok: false,
                            error: Some("rejection failed".to_string()),
                        });
                    }
                }
            }
        }
    }

    let rejected_count = results.iter().filter(|r| r.ok).count();
    let failed_count = results.iter().filter(|r| !r.ok).count();

    // Log aggregate result.
    let rejected_ids: Vec<i64> = results.iter().filter(|r| r.ok).map(|r| r.id).collect();
    let metadata = json!({
        "rejected_count": rejected_count,
        "failed_count": failed_count,
        "rejected_ids": rejected_ids,
        "actor": body.review.actor,
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "approval_bulk_rejected",
        "success",
        Some(&format!(
            "Bulk rejected {rejected_count}/{} items",
            body.ids.len()
        )),
        Some(&metadata.to_string()),
    )
    .await;

    Ok(Json(json!({
        "rejected": rejected_count,
        "failed": failed_count,
        "results": results,
    })))
}
