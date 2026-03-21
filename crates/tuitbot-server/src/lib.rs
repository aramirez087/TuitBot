//! Tuitbot HTTP API server.
//!
//! Exposes `tuitbot-core`'s storage layer as a REST API with read + write
//! endpoints, multi-strategy auth (bearer token + session cookie), and a
//! WebSocket for real-time events.

pub mod account;
pub mod auth;
pub mod dashboard;
pub mod error;
pub mod routes;
pub mod state;
pub mod ws;

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
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
        .route(
            "/analytics/engagement-rate",
            get(routes::analytics::engagement_rate),
        )
        .route("/analytics/reach", get(routes::analytics::reach))
        .route(
            "/analytics/follower-growth",
            get(routes::analytics::follower_growth),
        )
        .route("/analytics/best-times", get(routes::analytics::best_times))
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
        // Draft Studio — Tags (literal paths before parameterized)
        .route(
            "/tags",
            get(routes::content::list_account_tags).post(routes::content::create_account_tag),
        )
        // Draft Studio (new canonical paths)
        .route(
            "/drafts",
            get(routes::content::list_studio_drafts).post(routes::content::create_studio_draft),
        )
        .route(
            "/drafts/{id}",
            get(routes::content::get_studio_draft)
                .patch(routes::content::autosave_draft)
                .delete(routes::content::delete_draft),
        )
        .route(
            "/drafts/{id}/meta",
            patch(routes::content::patch_draft_meta),
        )
        .route(
            "/drafts/{id}/schedule",
            post(routes::content::schedule_studio_draft),
        )
        .route(
            "/drafts/{id}/reschedule",
            patch(routes::content::reschedule_studio_draft),
        )
        .route(
            "/drafts/{id}/unschedule",
            post(routes::content::unschedule_studio_draft),
        )
        .route(
            "/drafts/{id}/archive",
            post(routes::content::archive_studio_draft),
        )
        .route(
            "/drafts/{id}/restore",
            post(routes::content::restore_studio_draft),
        )
        .route(
            "/drafts/{id}/duplicate",
            post(routes::content::duplicate_studio_draft),
        )
        .route(
            "/drafts/{id}/revisions",
            get(routes::content::list_draft_revisions).post(routes::content::create_draft_revision),
        )
        .route(
            "/drafts/{id}/revisions/{rev_id}/restore",
            post(routes::content::restore_from_revision),
        )
        .route(
            "/drafts/{id}/activity",
            get(routes::content::list_draft_activity),
        )
        .route("/drafts/{id}/tags", get(routes::content::list_draft_tags))
        .route(
            "/drafts/{id}/tags/{tag_id}",
            post(routes::content::assign_draft_tag).delete(routes::content::unassign_draft_tag),
        )
        // Legacy drafts (backward compat)
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
        // Ingest
        .route("/ingest", post(routes::ingest::ingest))
        // Sources
        .route("/sources/status", get(routes::sources::source_status))
        .route(
            "/sources/{id}/reindex",
            post(routes::sources::reindex_source),
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
        .route(
            "/assist/highlights",
            post(routes::assist::assist_highlights),
        )
        .route("/assist/topics", get(routes::assist::assist_topics))
        .route(
            "/assist/optimal-times",
            get(routes::assist::assist_optimal_times),
        )
        .route("/assist/mode", get(routes::assist::get_mode))
        // Vault
        .route("/vault/sources", get(routes::vault::vault_sources))
        .route("/vault/notes", get(routes::vault::search_notes))
        .route("/vault/notes/{id}", get(routes::vault::note_detail))
        .route("/vault/search", get(routes::vault::search_fragments))
        .route("/vault/resolve-refs", post(routes::vault::resolve_refs))
        .route(
            "/vault/send-selection",
            post(routes::vault::selections::send_selection),
        )
        .route(
            "/vault/selection/{session_id}",
            get(routes::vault::selections::get_selection),
        )
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
        // Media — raise body limit for uploads (default 2MB is too small for images/video).
        .route(
            "/media/upload",
            post(routes::media::upload).layer(DefaultBodyLimit::max(520 * 1024 * 1024)),
        )
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
            "/settings/factory-reset",
            post(routes::settings::factory_reset),
        )
        .route(
            "/settings/scraper-session",
            get(routes::scraper_session::get_scraper_session)
                .post(routes::scraper_session::import_scraper_session)
                .delete(routes::scraper_session::delete_scraper_session),
        )
        .route(
            "/settings",
            get(routes::settings::get_settings).patch(routes::settings::patch_settings),
        )
        // Connectors
        .route(
            "/connectors/google-drive/link",
            post(routes::connectors::link_google_drive),
        )
        .route(
            "/connectors/google-drive/callback",
            get(routes::connectors::callback_google_drive),
        )
        .route(
            "/connectors/google-drive/status",
            get(routes::connectors::status_google_drive),
        )
        .route(
            "/connectors/google-drive/{id}",
            delete(routes::connectors::disconnect_google_drive),
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
        // Onboarding OAuth (pre-account, auth-exempt)
        .route(
            "/onboarding/x-auth/start",
            post(routes::onboarding::start_onboarding_auth),
        )
        .route(
            "/onboarding/x-auth/callback",
            post(routes::onboarding::complete_onboarding_auth),
        )
        .route(
            "/onboarding/x-auth/status",
            get(routes::onboarding::onboarding_auth_status),
        )
        .route(
            "/onboarding/analyze-profile",
            post(routes::onboarding::analyze_profile),
        )
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
            "/accounts/{id}/sync-profile",
            post(routes::accounts::sync_profile),
        )
        // X credential linking (before catch-all /accounts/{id})
        .route(
            "/accounts/{id}/x-auth/start",
            post(routes::x_auth::start_link),
        )
        .route(
            "/accounts/{id}/x-auth/callback",
            post(routes::x_auth::complete_link),
        )
        .route(
            "/accounts/{id}/x-auth/status",
            get(routes::x_auth::link_status),
        )
        .route(
            "/accounts/{id}/x-auth/tokens",
            delete(routes::x_auth::unlink),
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
        .fallback(dashboard::serve_dashboard)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
