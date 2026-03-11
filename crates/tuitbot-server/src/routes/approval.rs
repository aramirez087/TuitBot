//! Approval queue endpoints.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::config::Config;
use tuitbot_core::storage::{action_log, approval_queue, scheduled_content};

use crate::account::{require_approve, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::{AccountWsEvent, WsEvent};

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
}

fn default_status() -> String {
    "pending".to_string()
}

/// `GET /api/approval` — list approval items with optional status/type/reviewer/date filters.
pub async fn list_items(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<ApprovalQuery>,
) -> Result<Json<Value>, ApiError> {
    let statuses: Vec<&str> = params.status.split(',').map(|s| s.trim()).collect();
    let action_type = params.action_type.as_deref();
    let reviewed_by = params.reviewed_by.as_deref();
    let since = params.since.as_deref();

    let items = approval_queue::get_filtered_for(
        &state.db,
        &ctx.account_id,
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

/// Request body for editing approval item content.
#[derive(Deserialize)]
pub struct EditContentRequest {
    pub content: String,
    /// Optional updated media paths.
    #[serde(default)]
    pub media_paths: Option<Vec<String>>,
    /// Who made the edit (default: "dashboard").
    #[serde(default = "default_editor")]
    pub editor: String,
}

fn default_editor() -> String {
    "dashboard".to_string()
}

/// `PATCH /api/approval/:id` — edit content before approving.
pub async fn edit_item(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    Json(body): Json<EditContentRequest>,
) -> Result<Json<Value>, ApiError> {
    require_approve(&ctx)?;

    let item = approval_queue::get_by_id_for(&state.db, &ctx.account_id, id).await?;
    let item = item.ok_or_else(|| ApiError::NotFound(format!("approval item {id} not found")))?;

    let content = body.content.trim();
    if content.is_empty() {
        return Err(ApiError::BadRequest("content cannot be empty".to_string()));
    }

    // Record edit history before updating (queries by PK, implicitly scoped).
    if content != item.generated_content {
        let _ = approval_queue::record_edit(
            &state.db,
            id,
            &body.editor,
            "generated_content",
            &item.generated_content,
            content,
        )
        .await;
    }

    approval_queue::update_content_for(&state.db, &ctx.account_id, id, content).await?;

    if let Some(media_paths) = &body.media_paths {
        let media_json = serde_json::to_string(media_paths).unwrap_or_else(|_| "[]".to_string());

        // Record media_paths edit if changed.
        if media_json != item.media_paths {
            let _ = approval_queue::record_edit(
                &state.db,
                id,
                &body.editor,
                "media_paths",
                &item.media_paths,
                &media_json,
            )
            .await;
        }

        approval_queue::update_media_paths_for(&state.db, &ctx.account_id, id, &media_json).await?;
    }

    // Log to action log.
    let metadata = json!({
        "approval_id": id,
        "editor": body.editor,
        "field": "generated_content",
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "approval_edited",
        "success",
        Some(&format!("Edited approval item {id}")),
        Some(&metadata.to_string()),
    )
    .await;

    let updated = approval_queue::get_by_id_for(&state.db, &ctx.account_id, id)
        .await?
        .expect("item was just verified to exist");
    Ok(Json(json!(updated)))
}

/// `POST /api/approval/:id/approve` — approve a queued item.
pub async fn approve_item(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    body: Option<Json<approval_queue::ReviewAction>>,
) -> Result<Json<Value>, ApiError> {
    require_approve(&ctx)?;

    let item = approval_queue::get_by_id_for(&state.db, &ctx.account_id, id).await?;
    let item = item.ok_or_else(|| ApiError::NotFound(format!("approval item {id} not found")))?;

    // Verify X auth tokens exist before allowing approval.
    let token_path =
        tuitbot_core::storage::accounts::account_token_path(&state.data_dir, &ctx.account_id);
    if !token_path.exists() {
        return Err(ApiError::BadRequest(
            "Cannot approve: X API not authenticated. Complete X auth setup first.".to_string(),
        ));
    }

    let review = body.map(|b| b.0).unwrap_or_default();

    // Check if this item has a future scheduling intent.
    let schedule_bridge = item.scheduled_for.as_deref().and_then(|sched| {
        chrono::NaiveDateTime::parse_from_str(sched, "%Y-%m-%dT%H:%M:%SZ")
            .ok()
            .filter(|dt| *dt > chrono::Utc::now().naive_utc())
            .map(|_| sched.to_string())
    });

    if let Some(ref sched) = schedule_bridge {
        // Approve and mark as "scheduled" — the posting engine only picks up "approved" items,
        // so "scheduled" prevents double-posting.
        approval_queue::update_status_with_review_for(
            &state.db,
            &ctx.account_id,
            id,
            "scheduled",
            &review,
        )
        .await?;

        // Bridge to scheduled_content so the scheduler posts at the intended time.
        let sc_id = scheduled_content::insert_for(
            &state.db,
            &ctx.account_id,
            &item.action_type,
            &item.generated_content,
            Some(sched),
        )
        .await?;

        let metadata = json!({
            "approval_id": id,
            "scheduled_content_id": sc_id,
            "scheduled_for": sched,
            "actor": review.actor,
            "notes": review.notes,
            "action_type": item.action_type,
        });
        let _ = action_log::log_action_for(
            &state.db,
            &ctx.account_id,
            "approval_approved_scheduled",
            "success",
            Some(&format!("Approved item {id} → scheduled for {sched}")),
            Some(&metadata.to_string()),
        )
        .await;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ApprovalUpdated {
                id,
                status: "scheduled".to_string(),
                action_type: item.action_type,
                actor: review.actor,
            },
        });

        return Ok(Json(json!({
            "status": "scheduled",
            "id": id,
            "scheduled_content_id": sc_id,
            "scheduled_for": sched,
        })));
    }

    // No scheduling intent (or scheduled_for is in the past) — approve for immediate posting.
    approval_queue::update_status_with_review_for(
        &state.db,
        &ctx.account_id,
        id,
        "approved",
        &review,
    )
    .await?;

    // Log to action log.
    let metadata = json!({
        "approval_id": id,
        "actor": review.actor,
        "notes": review.notes,
        "action_type": item.action_type,
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "approval_approved",
        "success",
        Some(&format!("Approved item {id}")),
        Some(&metadata.to_string()),
    )
    .await;

    let _ = state.event_tx.send(AccountWsEvent {
        account_id: ctx.account_id.clone(),
        event: WsEvent::ApprovalUpdated {
            id,
            status: "approved".to_string(),
            action_type: item.action_type,
            actor: review.actor,
        },
    });

    Ok(Json(json!({"status": "approved", "id": id})))
}

/// `POST /api/approval/:id/reject` — reject a queued item.
pub async fn reject_item(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(id): Path<i64>,
    body: Option<Json<approval_queue::ReviewAction>>,
) -> Result<Json<Value>, ApiError> {
    require_approve(&ctx)?;

    let item = approval_queue::get_by_id_for(&state.db, &ctx.account_id, id).await?;
    let item = item.ok_or_else(|| ApiError::NotFound(format!("approval item {id} not found")))?;

    let review = body.map(|b| b.0).unwrap_or_default();
    approval_queue::update_status_with_review_for(
        &state.db,
        &ctx.account_id,
        id,
        "rejected",
        &review,
    )
    .await?;

    // Log to action log.
    let metadata = json!({
        "approval_id": id,
        "actor": review.actor,
        "notes": review.notes,
        "action_type": item.action_type,
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "approval_rejected",
        "success",
        Some(&format!("Rejected item {id}")),
        Some(&metadata.to_string()),
    )
    .await;

    let _ = state.event_tx.send(AccountWsEvent {
        account_id: ctx.account_id.clone(),
        event: WsEvent::ApprovalUpdated {
            id,
            status: "rejected".to_string(),
            action_type: item.action_type,
            actor: review.actor,
        },
    });

    Ok(Json(json!({"status": "rejected", "id": id})))
}

/// Request body for batch approve.
#[derive(Deserialize)]
pub struct BatchApproveRequest {
    /// Maximum number of items to approve (clamped to server config).
    #[serde(default)]
    pub max: Option<usize>,
    /// Specific IDs to approve (if provided, `max` is ignored).
    #[serde(default)]
    pub ids: Option<Vec<i64>>,
    /// Review metadata.
    #[serde(default)]
    pub review: approval_queue::ReviewAction,
}

/// `POST /api/approval/approve-all` — batch-approve pending items.
pub async fn approve_all(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    body: Option<Json<BatchApproveRequest>>,
) -> Result<Json<Value>, ApiError> {
    require_approve(&ctx)?;

    // Verify X auth tokens exist before allowing approval.
    let token_path =
        tuitbot_core::storage::accounts::account_token_path(&state.data_dir, &ctx.account_id);
    if !token_path.exists() {
        return Err(ApiError::BadRequest(
            "Cannot approve: X API not authenticated. Complete X auth setup first.".to_string(),
        ));
    }

    let config = read_config(&state);
    let max_batch = config.max_batch_approve;

    let body = body.map(|b| b.0);
    let review = body.as_ref().map(|b| b.review.clone()).unwrap_or_default();

    let approved_ids = if let Some(ids) = body.as_ref().and_then(|b| b.ids.as_ref()) {
        // Approve specific IDs (still clamped to max_batch).
        let clamped: Vec<&i64> = ids.iter().take(max_batch).collect();
        let mut approved = Vec::with_capacity(clamped.len());
        for &id in &clamped {
            if let Ok(Some(item)) =
                approval_queue::get_by_id_for(&state.db, &ctx.account_id, *id).await
            {
                let result = approve_single_item(&state, &ctx.account_id, &item, &review).await;
                if result.is_ok() {
                    approved.push(*id);
                }
            }
        }
        approved
    } else {
        // Approve oldest N pending items, handling scheduling intent per-item.
        let effective_max = body
            .as_ref()
            .and_then(|b| b.max)
            .map(|m| m.min(max_batch))
            .unwrap_or(max_batch);

        let pending = approval_queue::get_pending_for(&state.db, &ctx.account_id).await?;
        let mut approved = Vec::with_capacity(effective_max);
        for item in pending.iter().take(effective_max) {
            if approve_single_item(&state, &ctx.account_id, item, &review)
                .await
                .is_ok()
            {
                approved.push(item.id);
            }
        }
        approved
    };

    let count = approved_ids.len();

    // Log to action log.
    let metadata = json!({
        "count": count,
        "ids": approved_ids,
        "actor": review.actor,
        "max_configured": max_batch,
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "approval_batch_approved",
        "success",
        Some(&format!("Batch approved {count} items")),
        Some(&metadata.to_string()),
    )
    .await;

    let _ = state.event_tx.send(AccountWsEvent {
        account_id: ctx.account_id.clone(),
        event: WsEvent::ApprovalUpdated {
            id: 0,
            status: "approved_all".to_string(),
            action_type: String::new(),
            actor: review.actor,
        },
    });

    Ok(Json(
        json!({"status": "approved", "count": count, "ids": approved_ids, "max_batch": max_batch}),
    ))
}

/// Query parameters for the approval export endpoint.
#[derive(Deserialize)]
pub struct ExportQuery {
    /// Export format: "csv" or "json" (default: "csv").
    #[serde(default = "default_csv")]
    pub format: String,
    /// Comma-separated status values (default: all).
    #[serde(default = "default_export_status")]
    pub status: String,
    /// Filter by action type.
    #[serde(rename = "type")]
    pub action_type: Option<String>,
}

fn default_csv() -> String {
    "csv".to_string()
}

fn default_export_status() -> String {
    "pending,approved,rejected,posted".to_string()
}

/// `GET /api/approval/export` — export approval items as CSV or JSON.
pub async fn export_items(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<ExportQuery>,
) -> Result<axum::response::Response, ApiError> {
    use axum::response::IntoResponse;

    let statuses: Vec<&str> = params.status.split(',').map(|s| s.trim()).collect();
    let action_type = params.action_type.as_deref();

    let items =
        approval_queue::get_by_statuses_for(&state.db, &ctx.account_id, &statuses, action_type)
            .await?;

    if params.format == "json" {
        let body = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
        Ok((
            [
                (
                    axum::http::header::CONTENT_TYPE,
                    "application/json; charset=utf-8",
                ),
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    "attachment; filename=\"approval_export.json\"",
                ),
            ],
            body,
        )
            .into_response())
    } else {
        let mut csv = String::from(
            "id,action_type,target_author,generated_content,topic,score,status,reviewed_by,review_notes,created_at\n",
        );
        for item in &items {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                item.id,
                escape_csv(&item.action_type),
                escape_csv(&item.target_author),
                escape_csv(&item.generated_content),
                escape_csv(&item.topic),
                item.score,
                escape_csv(&item.status),
                escape_csv(item.reviewed_by.as_deref().unwrap_or("")),
                escape_csv(item.review_notes.as_deref().unwrap_or("")),
                escape_csv(&item.created_at),
            ));
        }
        Ok((
            [
                (axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    "attachment; filename=\"approval_export.csv\"",
                ),
            ],
            csv,
        )
            .into_response())
    }
}

/// Escape a value for CSV output.
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// `GET /api/approval/:id/history` — get edit history for an item.
pub async fn get_edit_history(
    State(state): State<Arc<AppState>>,
    _ctx: AccountContext,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    // Query by approval_id PK is already implicitly scoped.
    let history = approval_queue::get_edit_history(&state.db, id).await?;
    Ok(Json(json!(history)))
}

/// Approve a single item, bridging to scheduled_content if it has a future `scheduled_for`.
async fn approve_single_item(
    state: &AppState,
    account_id: &str,
    item: &approval_queue::ApprovalItem,
    review: &approval_queue::ReviewAction,
) -> Result<(), ApiError> {
    let schedule_bridge = item.scheduled_for.as_deref().and_then(|sched| {
        chrono::NaiveDateTime::parse_from_str(sched, "%Y-%m-%dT%H:%M:%SZ")
            .ok()
            .filter(|dt| *dt > chrono::Utc::now().naive_utc())
            .map(|_| sched.to_string())
    });

    if let Some(ref sched) = schedule_bridge {
        approval_queue::update_status_with_review_for(
            &state.db,
            account_id,
            item.id,
            "scheduled",
            review,
        )
        .await?;

        scheduled_content::insert_for(
            &state.db,
            account_id,
            &item.action_type,
            &item.generated_content,
            Some(sched),
        )
        .await?;
    } else {
        approval_queue::update_status_with_review_for(
            &state.db, account_id, item.id, "approved", review,
        )
        .await?;
    }

    Ok(())
}

/// Read the config from disk (best-effort, returns defaults on failure).
fn read_config(state: &AppState) -> Config {
    std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}
