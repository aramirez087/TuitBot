//! Source status and reindex endpoints.
//!
//! Exposes runtime status of content sources and a reindex trigger
//! for the Watchtower pipeline.

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use tuitbot_core::automation::WatchtowerLoop;
use tuitbot_core::storage::watchtower as store;

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SourceStatusResponse {
    pub sources: Vec<SourceStatusItem>,
}

#[derive(Serialize)]
pub struct SourceStatusItem {
    pub id: i64,
    pub source_type: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_cursor: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub config_json: String,
}

#[derive(Serialize)]
pub struct ReindexResponse {
    pub status: String,
    pub source_id: i64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/sources/status` — return runtime status of all content sources.
pub async fn source_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SourceStatusResponse>, ApiError> {
    let contexts = store::get_all_source_contexts(&state.db)
        .await
        .map_err(ApiError::Storage)?;

    let sources = contexts
        .into_iter()
        .map(|ctx| SourceStatusItem {
            id: ctx.id,
            source_type: ctx.source_type,
            status: ctx.status,
            error_message: ctx.error_message,
            sync_cursor: ctx.sync_cursor,
            created_at: ctx.created_at,
            updated_at: ctx.updated_at,
            config_json: ctx.config_json,
        })
        .collect();

    Ok(Json(SourceStatusResponse { sources }))
}

/// `POST /api/sources/{id}/reindex` — trigger a full rescan of one source.
///
/// Validates the source exists and is a local_fs source (remote reindex is
/// handled by the normal poll cycle). The reindex runs in a spawned task
/// and returns immediately.
pub async fn reindex_source(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<i64>,
) -> Result<Json<ReindexResponse>, ApiError> {
    // Verify the source exists.
    let ctx = store::get_source_context(&state.db, source_id)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound(format!("source {source_id} not found")))?;

    if ctx.source_type != "local_fs" {
        return Err(ApiError::BadRequest(
            "reindex is only supported for local_fs sources".to_string(),
        ));
    }

    // Extract path and patterns from config_json.
    let config: serde_json::Value = serde_json::from_str(&ctx.config_json)
        .map_err(|e| ApiError::Internal(format!("invalid source config_json: {e}")))?;

    let path_str = config
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::Internal("source config_json missing path".to_string()))?;

    let base_path = PathBuf::from(tuitbot_core::storage::expand_tilde(path_str));

    let patterns: Vec<String> = config
        .get("file_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(|| vec!["*.md".to_string(), "*.txt".to_string()]);

    // Spawn the reindex in a background task.
    let pool = state.db.clone();
    tokio::spawn(async move {
        match WatchtowerLoop::reindex_local_source(&pool, source_id, &base_path, &patterns).await {
            Ok(summary) => {
                tracing::info!(
                    source_id,
                    ingested = summary.ingested,
                    skipped = summary.skipped,
                    errors = summary.errors.len(),
                    "Reindex complete"
                );
            }
            Err(e) => {
                tracing::error!(source_id, error = %e, "Reindex failed");
            }
        }
    });

    Ok(Json(ReindexResponse {
        status: "reindex_started".to_string(),
        source_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_status_response_serializes() {
        let resp = SourceStatusResponse {
            sources: vec![SourceStatusItem {
                id: 1,
                source_type: "local_fs".into(),
                status: "active".into(),
                error_message: None,
                sync_cursor: None,
                created_at: "2026-03-15T10:00:00Z".into(),
                updated_at: "2026-03-15T10:00:00Z".into(),
                config_json: "{}".into(),
            }],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("local_fs"));
        assert!(!json.contains("error_message"));
        assert!(!json.contains("sync_cursor"));
    }

    #[test]
    fn source_status_item_with_error() {
        let item = SourceStatusItem {
            id: 2,
            source_type: "google_drive".into(),
            status: "error".into(),
            error_message: Some("auth failed".into()),
            sync_cursor: Some("cursor_123".into()),
            created_at: "2026-03-15T10:00:00Z".into(),
            updated_at: "2026-03-15T10:00:00Z".into(),
            config_json: r#"{"path":"/vault"}"#.into(),
        };
        let json = serde_json::to_string(&item).expect("serialize");
        assert!(json.contains("error_message"));
        assert!(json.contains("auth failed"));
        assert!(json.contains("sync_cursor"));
    }

    #[test]
    fn reindex_response_serializes() {
        let resp = ReindexResponse {
            status: "reindex_started".into(),
            source_id: 42,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("reindex_started"));
        assert!(json.contains("42"));
    }
}
