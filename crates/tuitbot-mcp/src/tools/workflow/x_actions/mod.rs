//! Direct X API tool implementations.
//!
//! Split into submodules by concern: read, write, engage, media, validate.

pub(crate) mod audit;
mod engage;
mod media;
mod read;
mod validate;
mod write;
pub mod x_request;

#[cfg(test)]
mod tests;

use std::time::Instant;

use crate::contract::error::provider_error_to_response;
use crate::provider::x_api::map_x_error;
use tuitbot_core::error::XApiError;

use crate::tools::response::{ToolMeta, ToolResponse};

// Re-export all public tool functions.
pub use engage::{
    bookmark_tweet, follow_user, like_tweet, retweet, unbookmark_tweet, unfollow_user,
    unlike_tweet, unretweet,
};
pub use media::upload_media;
pub use read::{
    get_bookmarks, get_followers, get_following, get_home_timeline, get_liked_tweets,
    get_tweet_by_id, get_tweet_liking_users, get_user_by_id, get_user_by_username,
    get_user_mentions, get_user_tweets, get_users_by_ids, get_x_usage, search_tweets,
};
pub use write::{
    delete_tweet, post_thread, post_thread_dry_run, post_tweet, post_tweet_dry_run, quote_tweet,
    reply_to_tweet,
};

/// Return an error response when the X client is not configured.
fn not_configured_response(start: Instant) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::x_not_configured()
        .with_meta(ToolMeta::new(elapsed))
        .to_json()
}

/// Helper: build an error response from an XApiError via the ProviderError path.
#[allow(dead_code)]
fn error_response(e: &XApiError, start: Instant) -> String {
    provider_error_to_response(&map_x_error(e), start)
}

/// Check if the scraper backend is active and mutations are blocked.
///
/// Returns `Some(error_json)` if the mutation should be rejected,
/// `None` if the operation may proceed.
fn scraper_mutation_guard(state: &crate::state::SharedState, start: Instant) -> Option<String> {
    if crate::provider::parse_backend(&state.config.x_api.provider_backend)
        == crate::provider::ProviderBackend::Scraper
        && !state.config.x_api.scraper_allow_mutations
    {
        let elapsed = start.elapsed().as_millis() as u64;
        Some(
            ToolResponse::scraper_mutation_blocked()
                .with_meta(ToolMeta::new(elapsed))
                .to_json(),
        )
    } else {
        None
    }
}

/// Return an error response when the authenticated user ID is missing.
fn no_user_id_response(start: Instant) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(
        crate::tools::response::ErrorCode::XNotConfigured,
        "Authenticated user ID not available. X client may not be fully initialized.",
    )
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}
