//! Automation runtime control endpoints.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use tuitbot_core::automation::Runtime;

use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::WsEvent;

/// `GET /api/runtime/status` — check if the automation runtime is running.
pub async fn status(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let runtime = state.runtime.lock().await;
    let running = runtime.is_some();
    let task_count = runtime.as_ref().map_or(0, |r| r.task_count());

    Ok(Json(json!({
        "running": running,
        "task_count": task_count,
    })))
}

/// `POST /api/runtime/start` — start the automation runtime.
///
/// Creates an empty `Runtime` (no loops spawned yet — full loop setup requires
/// X API client and LLM provider which are not available in the server context).
pub async fn start(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let mut runtime = state.runtime.lock().await;

    if runtime.is_some() {
        return Err(ApiError::Conflict("runtime is already running".to_string()));
    }

    *runtime = Some(Runtime::new());

    // Publish runtime status event.
    let _ = state.event_tx.send(WsEvent::RuntimeStatus {
        running: true,
        active_loops: vec![],
    });

    Ok(Json(json!({"status": "started"})))
}

/// `POST /api/runtime/stop` — gracefully stop the automation runtime.
pub async fn stop(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let mut runtime_guard = state.runtime.lock().await;

    match runtime_guard.take() {
        Some(mut rt) => {
            rt.shutdown().await;

            // Publish runtime status event.
            let _ = state.event_tx.send(WsEvent::RuntimeStatus {
                running: false,
                active_loops: vec![],
            });

            Ok(Json(json!({"status": "stopped"})))
        }
        None => Err(ApiError::Conflict("runtime is not running".to_string())),
    }
}
