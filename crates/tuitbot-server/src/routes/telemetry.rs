//! Lightweight telemetry ingestion endpoint.
//!
//! Receives batched frontend funnel events and logs them via `tracing`.
//! No database writes — structured logging only, for future aggregation.

use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

/// Maximum events per batch to prevent abuse.
const MAX_BATCH_SIZE: usize = 50;

/// Allowed event name prefixes (namespace isolation).
const ALLOWED_PREFIXES: &[&str] = &["backlink.", "hook_miner.", "forge."];

#[derive(Deserialize)]
pub struct TelemetryBatch {
    events: Vec<TelemetryEvent>,
}

#[derive(Deserialize)]
pub struct TelemetryEvent {
    event: String,
    #[allow(dead_code)]
    properties: Option<serde_json::Map<String, serde_json::Value>>,
    #[allow(dead_code)]
    timestamp: String,
}

/// `POST /api/telemetry/events` — ingest a batch of frontend funnel events.
///
/// Validates batch size and event name prefix, then logs each event via
/// `tracing::info!`. Returns 204 No Content on success.
pub async fn ingest_events(
    Json(batch): Json<TelemetryBatch>,
) -> Result<StatusCode, (StatusCode, String)> {
    if batch.events.len() > MAX_BATCH_SIZE {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Batch too large: {} events (max {})",
                batch.events.len(),
                MAX_BATCH_SIZE
            ),
        ));
    }

    for ev in &batch.events {
        if !ALLOWED_PREFIXES.iter().any(|p| ev.event.starts_with(p)) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "Invalid event name \"{}\": must start with one of {:?}",
                    ev.event, ALLOWED_PREFIXES
                ),
            ));
        }
    }

    for ev in &batch.events {
        tracing::info!(
            event = %ev.event,
            timestamp = %ev.timestamp,
            properties = ?ev.properties,
            "telemetry_event"
        );
    }

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;

    fn make_event(name: &str) -> TelemetryEvent {
        TelemetryEvent {
            event: name.to_string(),
            properties: Some(serde_json::Map::new()),
            timestamp: "2026-03-21T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn valid_batch_returns_204() {
        let batch = TelemetryBatch {
            events: vec![
                make_event("backlink.suggestions_shown"),
                make_event("backlink.suggestion_accepted"),
            ],
        };
        let result = ingest_events(Json(batch)).await;
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn oversized_batch_returns_400() {
        let events: Vec<TelemetryEvent> = (0..51)
            .map(|i| make_event(&format!("backlink.event_{i}")))
            .collect();
        let batch = TelemetryBatch { events };
        let result = ingest_events(Json(batch)).await;
        let err = result.unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("too large"));
    }

    #[tokio::test]
    async fn invalid_prefix_returns_400() {
        let batch = TelemetryBatch {
            events: vec![make_event("malicious.event")],
        };
        let result = ingest_events(Json(batch)).await;
        let err = result.unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("must start with"));
    }

    #[tokio::test]
    async fn empty_batch_returns_204() {
        let batch = TelemetryBatch { events: vec![] };
        let result = ingest_events(Json(batch)).await;
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn hook_miner_prefix_accepted() {
        let batch = TelemetryBatch {
            events: vec![make_event("hook_miner.angles_shown")],
        };
        let result = ingest_events(Json(batch)).await;
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn forge_prefix_accepted() {
        let batch = TelemetryBatch {
            events: vec![make_event("forge.sync_succeeded")],
        };
        let result = ingest_events(Json(batch)).await;
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn mixed_namespace_batch_accepted() {
        let batch = TelemetryBatch {
            events: vec![
                make_event("backlink.suggestions_shown"),
                make_event("hook_miner.angle_selected"),
                make_event("forge.enabled"),
            ],
        };
        let result = ingest_events(Json(batch)).await;
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn unknown_prefix_rejected() {
        let batch = TelemetryBatch {
            events: vec![make_event("other.event")],
        };
        let result = ingest_events(Json(batch)).await;
        let err = result.unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("must start with one of"));
    }
}
