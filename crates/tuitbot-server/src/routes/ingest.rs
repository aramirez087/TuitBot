//! Ingest endpoint for the Watchtower pipeline.
//!
//! Accepts inline content nodes for direct ingestion (e.g. from iOS Shortcuts
//! or Telegram) and file hints for future filesystem scanning.

use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tuitbot_core::storage::watchtower;

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct IngestRequest {
    /// Specific files to re-scan (relative paths within source).
    #[serde(default)]
    pub file_hints: Vec<String>,
    /// Re-ingest even if content hash is unchanged.
    #[serde(default)]
    pub force: bool,
    /// Inline content nodes for direct ingestion (Shortcuts/Telegram).
    #[serde(default)]
    pub inline_nodes: Vec<InlineNode>,
}

#[derive(Deserialize)]
pub struct InlineNode {
    pub relative_path: String,
    pub body_text: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub ingested: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// `POST /api/ingest` — ingest content into the Watchtower pipeline.
pub async fn ingest(
    State(state): State<Arc<AppState>>,
    Json(body): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, ApiError> {
    let start = Instant::now();
    let mut ingested: u32 = 0;
    let mut skipped: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    // Process inline nodes.
    if !body.inline_nodes.is_empty() {
        let source_id = watchtower::ensure_manual_source(&state.db)
            .await
            .map_err(ApiError::Storage)?;

        for node in &body.inline_nodes {
            if node.body_text.is_empty() {
                errors.push(format!("{}: empty body_text", node.relative_path));
                continue;
            }

            let hash = if body.force {
                // Force mode: use timestamp to ensure unique hash.
                let mut hasher = Sha256::new();
                hasher.update(node.body_text.as_bytes());
                hasher.update(start.elapsed().as_nanos().to_le_bytes());
                format!("{:x}", hasher.finalize())
            } else {
                let mut hasher = Sha256::new();
                hasher.update(node.body_text.as_bytes());
                format!("{:x}", hasher.finalize())
            };

            match watchtower::upsert_content_node(
                &state.db,
                source_id,
                &node.relative_path,
                &hash,
                node.title.as_deref(),
                &node.body_text,
                None,
                node.tags.as_deref(),
            )
            .await
            {
                Ok(watchtower::UpsertResult::Inserted | watchtower::UpsertResult::Updated) => {
                    ingested += 1;
                }
                Ok(watchtower::UpsertResult::Skipped) => {
                    skipped += 1;
                }
                Err(e) => {
                    errors.push(format!("{}: {e}", node.relative_path));
                }
            }
        }
    }

    // File hints are acknowledged but not processed until Session 03
    // when the ContentSource trait and filesystem scanning are implemented.
    if !body.file_hints.is_empty() && body.inline_nodes.is_empty() {
        // No-op for now — hints will be routed through ContentSource in S03.
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(Json(IngestResponse {
        ingested,
        skipped,
        errors,
        duration_ms,
    }))
}
