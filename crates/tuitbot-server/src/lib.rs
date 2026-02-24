//! Tuitbot HTTP API server.
//!
//! Exposes `tuitbot-core`'s storage layer as a REST API with read + write
//! endpoints, local bearer-token auth, and a WebSocket for real-time events.

pub mod auth;
pub mod error;
pub mod routes;
pub mod state;
pub mod ws;

use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the complete axum router with all API routes and middleware.
pub fn build_router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/health", get(routes::health::health))
        // Analytics
        .route("/analytics/summary", get(routes::analytics::summary))
        .route("/analytics/followers", get(routes::analytics::followers))
        .route(
            "/analytics/performance",
            get(routes::analytics::performance),
        )
        .route("/analytics/topics", get(routes::analytics::topics))
        .route(
            "/analytics/recent-performance",
            get(routes::analytics::recent_performance),
        )
        // Approval
        .route("/approval", get(routes::approval::list_items))
        .route("/approval/stats", get(routes::approval::stats))
        .route("/approval/approve-all", post(routes::approval::approve_all))
        .route("/approval/{id}", patch(routes::approval::edit_item))
        .route(
            "/approval/{id}/approve",
            post(routes::approval::approve_item),
        )
        .route("/approval/{id}/reject", post(routes::approval::reject_item))
        // Activity
        .route("/activity", get(routes::activity::list_activity))
        .route(
            "/activity/rate-limits",
            get(routes::activity::rate_limit_usage),
        )
        // Replies
        .route("/replies", get(routes::replies::list_replies))
        // Content
        .route(
            "/content/tweets",
            get(routes::content::list_tweets).post(routes::content::compose_tweet),
        )
        .route(
            "/content/threads",
            get(routes::content::list_threads).post(routes::content::compose_thread),
        )
        .route("/content/calendar", get(routes::content::calendar))
        .route("/content/schedule", get(routes::content::schedule))
        .route("/content/compose", post(routes::content::compose))
        .route(
            "/content/scheduled/{id}",
            patch(routes::content::edit_scheduled).delete(routes::content::cancel_scheduled),
        )
        // Targets
        .route(
            "/targets",
            get(routes::targets::list_targets).post(routes::targets::add_target),
        )
        .route(
            "/targets/{username}/timeline",
            get(routes::targets::target_timeline),
        )
        .route(
            "/targets/{username}/stats",
            get(routes::targets::target_stats),
        )
        .route(
            "/targets/{username}",
            delete(routes::targets::remove_target),
        )
        // Strategy
        .route("/strategy/current", get(routes::strategy::current))
        .route("/strategy/history", get(routes::strategy::history))
        .route("/strategy/refresh", post(routes::strategy::refresh))
        .route("/strategy/inputs", get(routes::strategy::inputs))
        // Costs
        .route("/costs/summary", get(routes::costs::summary))
        .route("/costs/daily", get(routes::costs::daily))
        .route("/costs/by-model", get(routes::costs::by_model))
        .route("/costs/by-type", get(routes::costs::by_type))
        // Settings
        .route("/settings/status", get(routes::settings::config_status))
        .route("/settings/init", post(routes::settings::init_settings))
        .route(
            "/settings/validate",
            post(routes::settings::validate_settings),
        )
        .route("/settings/defaults", get(routes::settings::get_defaults))
        .route("/settings/test-llm", post(routes::settings::test_llm))
        .route(
            "/settings",
            get(routes::settings::get_settings).patch(routes::settings::patch_settings),
        )
        // Runtime
        .route("/runtime/status", get(routes::runtime::status))
        .route("/runtime/start", post(routes::runtime::start))
        .route("/runtime/stop", post(routes::runtime::stop))
        // WebSocket
        .route("/ws", get(ws::ws_handler))
        // Auth middleware — applied to all routes; health is exempted internally.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    Router::new()
        .nest("/api", api)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
