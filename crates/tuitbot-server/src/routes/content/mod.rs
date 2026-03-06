//! Content endpoints (tweets, threads, calendar, compose, scheduled content, drafts).

mod calendar;
mod compose;
mod drafts;
mod list;
mod scheduled;

use tuitbot_core::config::Config;

use crate::error::ApiError;
use crate::state::AppState;

// Re-export all handlers so route registration in lib.rs stays unchanged.
pub use calendar::{calendar, schedule};
pub use compose::{compose, compose_thread, compose_tweet};
pub use drafts::{
    create_draft, delete_draft, edit_draft, list_drafts, publish_draft, schedule_draft,
};
pub use list::{list_threads, list_tweets};
pub use scheduled::{cancel_scheduled, edit_scheduled};

// Re-export types used by route registration (if any).
pub use calendar::{CalendarItem, CalendarQuery};
pub use compose::{ComposeRequest, ComposeThreadRequest, ComposeTweetRequest, ThreadBlockRequest};
pub use drafts::{CreateDraftRequest, EditDraftRequest, ScheduleDraftRequest};
pub use list::{ThreadsQuery, TweetsQuery};
pub use scheduled::EditScheduledRequest;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Read the explicit `approval_mode` setting from the config file.
///
/// This uses the raw `approval_mode` field (not `effective_approval_mode`)
/// because compose endpoints handle user-initiated manual posts — the
/// Composer-mode override that forces approval for autonomous loops
/// should not apply here.
fn read_approval_mode(state: &AppState) -> Result<bool, ApiError> {
    let config = read_config(state)?;
    Ok(config.approval_mode)
}

/// Return an error if the provider backend cannot post.
///
/// Posting is possible with the official X API, or with the scraper
/// backend when a cookie session file is present.
fn require_post_capable(state: &AppState) -> Result<(), ApiError> {
    let config = read_config(state)?;
    let can_post = match config.x_api.provider_backend.as_str() {
        "x_api" => true,
        "scraper" => state.data_dir.join("scraper_session.json").exists(),
        _ => false,
    };
    if !can_post {
        return Err(ApiError::BadRequest(
            "Direct posting requires X API credentials or an imported browser session. \
             Configure in Settings → X API."
                .to_string(),
        ));
    }
    Ok(())
}

/// Read the full config from the config file.
fn read_config(state: &AppState) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: Config = toml::from_str(&contents).unwrap_or_default();
    Ok(config)
}
