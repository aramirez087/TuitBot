//! Approval queue route handlers.
//!
//! Split by concern:
//! - mod.rs: shared types + list/stats (read-only endpoints)
//! - handlers.rs: edit/approve/reject/approve_all (write endpoints)
//! - bulk_handlers.rs: bulk approve/reject (batch write endpoints)
//! - export.rs: CSV/JSON export, edit history, internal helpers

pub mod bulk_handlers;
pub mod export;
pub mod handlers;

pub use bulk_handlers::{bulk_approve, bulk_reject};
pub use export::{export_items, get_edit_history};
pub use handlers::{approve_all, approve_item, edit_item, reject_item};

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::approval_queue;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

/// Query parameters for listing approval items.
#[derive(Deserialize)]
pub struct ApprovalQuery {
    /// Comma-separated status values (default: "pending").
    #[serde(default = "default_status")]
    pub status: String,
    /// Filter by action type (reply, tweet, thread_tweet).
    #[serde(rename = "type")]
    pub action_type: Option<String>,
    /// Filter by reviewer name.
    pub reviewed_by: Option<String>,
    /// Filter by items created since this ISO-8601 timestamp.
    pub since: Option<String>,
    /// Override the account to filter by (defaults to X-Account-Id header).
    /// Must match the authenticated account; ignored if it differs.
    pub account_id: Option<String>,
}

fn default_status() -> String {
    "pending".to_string()
}

/// `GET /api/approval` — list approval items with optional status/type/reviewer/date filters.
///
/// Accepts an optional `account_id` query param to scope results. The param is
/// validated against the authenticated account — if it differs, the header
/// account takes precedence (prevents cross-account data leakage).
pub async fn list_items(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<ApprovalQuery>,
) -> Result<Json<Value>, ApiError> {
    let statuses: Vec<&str> = params.status.split(',').map(|s| s.trim()).collect();
    let action_type = params.action_type.as_deref();
    let reviewed_by = params.reviewed_by.as_deref();
    let since = params.since.as_deref();

    // account_id query param is accepted for dashboard convenience but must
    // match the authenticated account (ctx.account_id) to prevent leakage.
    let effective_account_id = match params.account_id.as_deref() {
        Some(qid) if qid == ctx.account_id => qid,
        Some(_) => &ctx.account_id, // silently use header account
        None => &ctx.account_id,
    };

    let items = approval_queue::get_filtered_for(
        &state.db,
        effective_account_id,
        &statuses,
        action_type,
        reviewed_by,
        since,
    )
    .await?;
    Ok(Json(json!(items)))
}

/// `GET /api/approval/stats` — counts by status.
pub async fn stats(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let stats = approval_queue::get_stats_for(&state.db, &ctx.account_id).await?;
    Ok(Json(json!(stats)))
}

#[cfg(test)]
mod tests;
