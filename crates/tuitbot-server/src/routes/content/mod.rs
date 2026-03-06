//! Content endpoints (tweets, threads, calendar, compose, scheduled content, drafts).

mod calendar;
mod compose;
mod drafts;
mod list;
mod scheduled;

use tuitbot_core::config::{effective_config, Config};
use tuitbot_core::storage::accounts::{
    self, account_scraper_session_path, account_token_path, DEFAULT_ACCOUNT_ID,
};

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
/// Default account: reads config.toml directly (backward compat).
/// Non-default: merges config.toml base with account's config_overrides from DB.
pub(crate) async fn read_effective_config(
    state: &AppState,
    account_id: &str,
) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let base_config: Config = toml::from_str(&contents).unwrap_or_default();

    if account_id == DEFAULT_ACCOUNT_ID {
        return Ok(base_config);
    }

    let account = accounts::get_account(&state.db, account_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("account not found: {account_id}")))?;

    let result = effective_config(&base_config, &account.config_overrides)
        .map_err(|e| ApiError::Internal(format!("failed to compute effective config: {e}")))?;

    Ok(result.config)
}
