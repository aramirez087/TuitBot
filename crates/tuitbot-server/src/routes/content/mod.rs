//! Content endpoints (tweets, threads, calendar, compose, scheduled content, drafts).

mod calendar;
mod compose;
mod draft_studio;
mod drafts;
mod list;
mod scheduled;

use tuitbot_core::config::Config;
use tuitbot_core::storage::accounts::{account_scraper_session_path, account_token_path};

use crate::error::ApiError;
use crate::state::AppState;

// Re-export all handlers so route registration in lib.rs stays unchanged.
pub use calendar::{calendar, schedule};
pub use compose::{compose, compose_thread, compose_tweet};
pub use draft_studio::{
    archive_studio_draft, autosave_draft, create_draft_revision, create_studio_draft,
    duplicate_studio_draft, get_studio_draft, list_draft_activity, list_draft_revisions,
    list_studio_drafts, patch_draft_meta, restore_studio_draft, schedule_studio_draft,
    unschedule_studio_draft,
};
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

/// Read the explicit `approval_mode` setting from the effective config.
///
/// This uses the raw `approval_mode` field (not `effective_approval_mode`)
/// because compose endpoints handle user-initiated manual posts — the
/// Composer-mode override that forces approval for autonomous loops
/// should not apply here.
async fn read_approval_mode(state: &AppState, account_id: &str) -> Result<bool, ApiError> {
    let config = read_effective_config(state, account_id).await?;
    Ok(config.approval_mode)
}

/// Return an error if the provider backend cannot post for the given account.
///
/// Posting is possible with the official X API (when tokens exist), or with
/// the scraper backend when a cookie session file is present.
pub(crate) async fn require_post_capable(
    state: &AppState,
    account_id: &str,
) -> Result<(), ApiError> {
    let config = read_effective_config(state, account_id).await?;
    let can_post = match config.x_api.provider_backend.as_str() {
        "x_api" => {
            let token_path = account_token_path(&state.data_dir, account_id);
            token_path.exists()
        }
        "scraper" => {
            let session_path = account_scraper_session_path(&state.data_dir, account_id);
            session_path.exists()
        }
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

/// Compute `can_post` for the given account without returning an error.
pub(crate) async fn can_post_for(state: &AppState, account_id: &str) -> bool {
    let config = match read_effective_config(state, account_id).await {
        Ok(c) => c,
        Err(_) => return false,
    };
    match config.x_api.provider_backend.as_str() {
        "x_api" => account_token_path(&state.data_dir, account_id).exists(),
        "scraper" => account_scraper_session_path(&state.data_dir, account_id).exists(),
        _ => false,
    }
}

/// Load the effective config for the given account.
///
/// Delegates to `AppState::load_effective_config` and maps errors to `ApiError`.
pub(crate) async fn read_effective_config(
    state: &AppState,
    account_id: &str,
) -> Result<Config, ApiError> {
    state
        .load_effective_config(account_id)
        .await
        .map_err(|e| ApiError::Internal(format!("failed to load effective config: {e}")))
}
