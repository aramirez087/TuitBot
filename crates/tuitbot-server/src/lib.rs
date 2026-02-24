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
use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the complete axum router with all API routes and middleware.
pub fn build_router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/health", get(routes::health::health))
        // Analytics
        .route("/analytics/followers", get(routes::analytics::followers))
        .route(
            "/analytics/performance",
            get(routes::analytics::performance),
        )
        .route("/analytics/topics", get(routes::analytics::topics))
        // Approval
        .route("/approval", get(routes::approval::list_pending))
        .route(
            "/approval/{id}/approve",
            post(routes::approval::approve_item),
        )
        .route("/approval/{id}/reject", post(routes::approval::reject_item))
        .route("/approval/approve-all", post(routes::approval::approve_all))
        // Activity
        .route("/activity", get(routes::activity::list_activity))
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
        // Targets
        .route(
            "/targets",
            get(routes::targets::list_targets).post(routes::targets::add_target),
        )
        .route(
            "/targets/{username}",
            delete(routes::targets::remove_target),
        )
        // Settings
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
