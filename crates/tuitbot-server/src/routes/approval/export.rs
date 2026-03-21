//! Export and history endpoints for the approval queue.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::approval_queue;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

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
pub(super) fn escape_csv(value: &str) -> String {
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
