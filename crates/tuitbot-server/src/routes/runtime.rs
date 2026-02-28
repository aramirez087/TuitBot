//! Automation runtime control endpoints.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use tuitbot_core::automation::Runtime;

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::WsEvent;

/// `GET /api/runtime/status` — check if the automation runtime is running.
///
/// Also returns `deployment_mode` and `capabilities` so the frontend can
/// adapt its source-type UI without platform guessing.
pub async fn status(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let runtimes = state.runtimes.lock().await;
    let runtime = runtimes.get(&ctx.account_id);
    let running = runtime.is_some();
    let task_count = runtime.map_or(0, |r| r.task_count());
    let capabilities = state.deployment_mode.capabilities();

    Ok(Json(json!({
        "running": running,
        "task_count": task_count,
        "deployment_mode": state.deployment_mode,
        "capabilities": capabilities,
    })))
}

/// `POST /api/runtime/start` — start the automation runtime.
///
/// Creates an empty `Runtime` (no loops spawned yet — full loop setup requires
/// X API client and LLM provider which are not available in the server context).
pub async fn start(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let mut runtimes = state.runtimes.lock().await;

    if runtimes.contains_key(&ctx.account_id) {
        return Err(ApiError::Conflict("runtime is already running".to_string()));
    }

    runtimes.insert(ctx.account_id.clone(), Runtime::new());

    // Publish runtime status event.
    let _ = state.event_tx.send(WsEvent::RuntimeStatus {
        running: true,
        active_loops: vec![],
    });

    Ok(Json(json!({"status": "started"})))
}

/// `POST /api/runtime/stop` — gracefully stop the automation runtime.
pub async fn stop(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let mut runtimes = state.runtimes.lock().await;

    match runtimes.remove(&ctx.account_id) {
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
