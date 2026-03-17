//! Health check endpoints.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::state::AppState;

/// `GET /api/health` — liveness probe (no auth required).
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// `GET /api/health/detailed` — deep health check (requires auth).
pub async fn health_detailed(State(state): State<Arc<AppState>>) -> Json<Value> {
    // Database health
    let db_health = tuitbot_core::storage::health::check_db_health(&state.db).await;
    let db_healthy = db_health.reachable && db_health.wal_mode;

    // Runtime status (aggregate across all accounts)
    let runtimes_guard = state.runtimes.lock().await;
    let runtime_running = !runtimes_guard.is_empty();
    let runtime_tasks: usize = runtimes_guard.values().map(|r| r.task_count()).sum();
    drop(runtimes_guard);

    // Circuit breaker
    let (cb_state, cb_error_count, cb_cooldown) = if let Some(ref cb) = state.circuit_breaker {
        let s = cb.state().await;
        let count = cb.error_count().await;
        let cooldown = cb.cooldown_remaining_seconds().await;
        (s.to_string(), count, cooldown)
    } else {
        ("disabled".to_string(), 0, 0)
    };

    // Scraper health (only present when provider_backend = "scraper")
    let scraper = if let Some(ref sh) = state.scraper_health {
        let snap = sh.lock().await.snapshot();
        Some(serde_json::json!({
            "healthy": snap.state == tuitbot_core::x_api::ScraperState::Healthy
                    || snap.state == tuitbot_core::x_api::ScraperState::Degraded,
            "state": snap.state.to_string(),
            "consecutive_failures": snap.consecutive_failures,
            "last_success_at": snap.last_success_at,
            "last_error": snap.last_error,
            "last_error_at": snap.last_error_at,
        }))
    } else {
        None
    };

    // Overall status — degraded if scraper is down or CB is open.
    let scraper_down = scraper
        .as_ref()
        .and_then(|s| s.get("state"))
        .and_then(|v| v.as_str())
        .map(|s| s == "down")
        .unwrap_or(false);

    let overall = if !db_health.reachable {
        "unhealthy"
    } else if !db_health.wal_mode || cb_state == "open" || scraper_down {
        "degraded"
    } else {
        "healthy"
    };

    let mut checks = serde_json::json!({
        "database": {
            "healthy": db_healthy,
            "reachable": db_health.reachable,
            "latency_ms": db_health.latency_ms,
            "wal_mode": db_health.wal_mode,
        },
        "runtime": {
            "healthy": runtime_running,
            "running": runtime_running,
            "task_count": runtime_tasks,
        },
        "circuit_breaker": {
            "healthy": cb_state != "open",
            "state": cb_state,
            "error_count": cb_error_count,
            "cooldown_remaining_seconds": cb_cooldown,
        },
    });

    if let Some(s) = scraper {
        checks["scraper"] = s;
    }

    Json(json!({
        "status": overall,
        "version": env!("CARGO_PKG_VERSION"),
        "checks": checks,
    }))
}
