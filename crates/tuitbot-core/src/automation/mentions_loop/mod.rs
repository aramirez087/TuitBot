//! Mentions monitoring loop.
//!
//! Fetches new @-mentions from X API, generates contextual replies
//! via LLM, and posts them through the posting queue. Persists
//! `since_id` to survive restarts and avoid reprocessing.

mod loop_impl;
mod responder;
#[cfg(test)]
mod tests;

use super::loop_helpers::{MentionsFetcher, PostSender, ReplyGenerator, SafetyChecker};
use std::sync::Arc;

/// Mentions loop that monitors and replies to @-mentions.
pub struct MentionsLoop {
    pub(crate) fetcher: Arc<dyn MentionsFetcher>,
    pub(crate) generator: Arc<dyn ReplyGenerator>,
    pub(crate) safety: Arc<dyn SafetyChecker>,
    pub(crate) poster: Arc<dyn PostSender>,
    pub(crate) dry_run: bool,
}

/// Result of processing a single mention.
#[derive(Debug)]
pub enum MentionResult {
    /// Reply was sent (or would be sent in dry-run).
    Replied {
        tweet_id: String,
        author: String,
        reply_text: String,
    },
    /// Mention was skipped (safety check, already replied).
    Skipped { tweet_id: String, reason: String },
    /// Processing failed for this mention.
    Failed { tweet_id: String, error: String },
}

impl MentionsLoop {
    /// Create a new mentions loop.
    pub fn new(
        fetcher: Arc<dyn MentionsFetcher>,
        generator: Arc<dyn ReplyGenerator>,
        safety: Arc<dyn SafetyChecker>,
        poster: Arc<dyn PostSender>,
        dry_run: bool,
    ) -> Self {
        Self {
            fetcher,
            generator,
            safety,
            poster,
            dry_run,
        }
    }
}

/// Update max_id tracking. Tweet IDs are numeric strings; higher = newer.
///
/// Compares by length first (longer numeric string = larger number),
/// then lexicographically for equal-length strings.
pub(crate) fn update_max_id(current: &mut Option<String>, candidate: &str) {
    let is_greater = match current {
        Some(ref existing) => {
            if candidate.len() != existing.len() {
                candidate.len() > existing.len()
            } else {
                candidate > existing.as_str()
            }
        }
        None => true,
    };

    if is_greater {
        *current = Some(candidate.to_string());
    }
}

/// Truncate a string for display.
pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
