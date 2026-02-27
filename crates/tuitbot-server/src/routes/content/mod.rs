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

/// Read `approval_mode` from the config file.
fn read_approval_mode(state: &AppState) -> Result<bool, ApiError> {
    let config = read_config(state)?;
    Ok(config.effective_approval_mode())
}

/// Read the full config from the config file.
fn read_config(state: &AppState) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: Config = toml::from_str(&contents).unwrap_or_default();
    Ok(config)
}
