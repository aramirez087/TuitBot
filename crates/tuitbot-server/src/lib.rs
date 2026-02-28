//! Tuitbot HTTP API server.
//!
//! Exposes `tuitbot-core`'s storage layer as a REST API with read + write
//! endpoints, multi-strategy auth (bearer token + session cookie), and a
//! WebSocket for real-time events.

pub mod account;
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
        .route("/health/detailed", get(routes::health::health_detailed))
        // Auth
        .route("/auth/login", post(auth::routes::login))
        .route("/auth/logout", post(auth::routes::logout))
        .route("/auth/status", get(auth::routes::status))
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
        .route("/approval/export", get(routes::approval::export_items))
        .route("/approval", get(routes::approval::list_items))
        .route("/approval/stats", get(routes::approval::stats))
        .route("/approval/approve-all", post(routes::approval::approve_all))
        .route(
            "/approval/{id}/history",
            get(routes::approval::get_edit_history),
        )
        .route("/approval/{id}", patch(routes::approval::edit_item))
        .route(
            "/approval/{id}/approve",
            post(routes::approval::approve_item),
        )
        .route("/approval/{id}/reject", post(routes::approval::reject_item))
        // Activity
        .route("/activity/export", get(routes::activity::export_activity))
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
        // Drafts
        .route(
            "/content/drafts",
            get(routes::content::list_drafts).post(routes::content::create_draft),
        )
        .route(
            "/content/drafts/{id}",
            patch(routes::content::edit_draft).delete(routes::content::delete_draft),
        )
        .route(
            "/content/drafts/{id}/schedule",
            post(routes::content::schedule_draft),
        )
        .route(
            "/content/drafts/{id}/publish",
            post(routes::content::publish_draft),
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
        // Costs — LLM
        .route("/costs/summary", get(routes::costs::summary))
        .route("/costs/daily", get(routes::costs::daily))
        .route("/costs/by-model", get(routes::costs::by_model))
        .route("/costs/by-type", get(routes::costs::by_type))
        // Costs — X API
        .route("/costs/x-api/summary", get(routes::costs::x_api_summary))
        .route("/costs/x-api/daily", get(routes::costs::x_api_daily))
        .route(
            "/costs/x-api/by-endpoint",
            get(routes::costs::x_api_by_endpoint),
        )
        // AI Assist
        .route("/assist/tweet", post(routes::assist::assist_tweet))
        .route("/assist/reply", post(routes::assist::assist_reply))
        .route("/assist/thread", post(routes::assist::assist_thread))
        .route("/assist/improve", post(routes::assist::assist_improve))
        .route("/assist/topics", get(routes::assist::assist_topics))
        .route(
            "/assist/optimal-times",
            get(routes::assist::assist_optimal_times),
        )
        .route("/assist/mode", get(routes::assist::get_mode))
        // Discovery feed
        .route("/discovery/feed", get(routes::discovery::feed))
        .route("/discovery/keywords", get(routes::discovery::keywords))
        .route(
            "/discovery/{tweet_id}/compose-reply",
            post(routes::discovery::compose_reply),
        )
        .route(
            "/discovery/{tweet_id}/queue-reply",
            post(routes::discovery::queue_reply),
        )
        // Media
        .route("/media/upload", post(routes::media::upload))
        .route("/media/file", get(routes::media::serve_file))
        // LAN settings
        .route(
            "/settings/lan",
            get(routes::lan::get_status).patch(routes::lan::toggle_lan),
        )
        .route(
            "/settings/lan/reset-passphrase",
            post(routes::lan::reset_passphrase),
        )
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
        // MCP governance
        .route(
            "/mcp/policy",
            get(routes::mcp::get_policy).patch(routes::mcp::patch_policy),
        )
        .route("/mcp/policy/templates", get(routes::mcp::list_templates))
        .route(
            "/mcp/policy/templates/{name}",
            post(routes::mcp::apply_template),
        )
        .route(
            "/mcp/telemetry/summary",
            get(routes::mcp::telemetry_summary),
        )
        .route(
            "/mcp/telemetry/metrics",
            get(routes::mcp::telemetry_metrics),
        )
        .route("/mcp/telemetry/errors", get(routes::mcp::telemetry_errors))
        .route("/mcp/telemetry/recent", get(routes::mcp::telemetry_recent))
        // Runtime
        .route("/runtime/status", get(routes::runtime::status))
        .route("/runtime/start", post(routes::runtime::start))
        .route("/runtime/stop", post(routes::runtime::stop))
        // Accounts
        .route(
            "/accounts",
            get(routes::accounts::list_accounts).post(routes::accounts::create_account),
        )
        .route(
            "/accounts/{id}/roles",
            get(routes::accounts::list_roles)
                .post(routes::accounts::set_role)
                .delete(routes::accounts::remove_role),
        )
        .route(
            "/accounts/{id}",
            get(routes::accounts::get_account)
                .patch(routes::accounts::update_account)
                .delete(routes::accounts::delete_account),
        )
        // WebSocket
        .route("/ws", get(ws::ws_handler))
        // Auth middleware — applied to all routes; exempt paths handled internally.
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
