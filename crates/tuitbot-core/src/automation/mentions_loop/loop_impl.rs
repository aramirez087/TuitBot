//! Main poll-fetch-filter logic for the mentions loop.

use super::{MentionResult, MentionsLoop};
use crate::automation::loop_helpers::{ConsecutiveErrorTracker, LoopError, LoopStorage};
use crate::automation::schedule::{schedule_gate, ActiveSchedule};
use crate::automation::scheduler::LoopScheduler;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

impl MentionsLoop {
    /// Run the continuous mentions loop until cancellation.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        scheduler: LoopScheduler,
        schedule: Option<Arc<ActiveSchedule>>,
        storage: Arc<dyn LoopStorage>,
    ) {
        tracing::info!(dry_run = self.dry_run, "Mentions loop started");

        let mut error_tracker = ConsecutiveErrorTracker::new(10, Duration::from_secs(300));

        // Load persisted since_id
        let mut since_id = match storage.get_cursor("mentions_since_id").await {
            Ok(id) => {
                if let Some(ref id) = id {
                    tracing::info!(since_id = %id, "Resuming mentions from stored cursor");
                }
                id
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to load mentions since_id, starting fresh");
                None
            }
        };

        loop {
            if cancel.is_cancelled() {
                break;
            }

            if !schedule_gate(&schedule, &cancel).await {
                break;
            }

            match self.run_once(since_id.as_deref(), None, &storage).await {
                Ok((results, new_since_id)) => {
                    error_tracker.record_success();

                    if let Some(ref new_id) = new_since_id {
                        since_id = Some(new_id.clone());
                        if let Err(e) = storage.set_cursor("mentions_since_id", new_id).await {
                            tracing::warn!(error = %e, "Failed to persist mentions since_id");
                        }
                    }

                    let replied = results
                        .iter()
                        .filter(|r| matches!(r, MentionResult::Replied { .. }))
                        .count();
                    if replied > 0 {
                        tracing::info!(
                            total = results.len(),
                            replied = replied,
                            "Mentions iteration complete"
                        );
                    }
                }
                Err(e) => {
                    let should_pause = error_tracker.record_error();
                    tracing::warn!(
                        error = %e,
                        consecutive_errors = error_tracker.count(),
                        "Mentions iteration failed"
                    );

                    if should_pause {
                        tracing::warn!(
                            pause_secs = error_tracker.pause_duration().as_secs(),
                            "Pausing mentions loop due to consecutive errors"
                        );
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(error_tracker.pause_duration()) => {},
                        }
                        error_tracker.reset();
                        continue;
                    }

                    // Rate limit specific backoff
                    if let LoopError::RateLimited { retry_after } = &e {
                        let backoff =
                            crate::automation::loop_helpers::rate_limit_backoff(*retry_after, 0);
                        tracing::info!(
                            backoff_secs = backoff.as_secs(),
                            "Backing off due to rate limit"
                        );
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(backoff) => {},
                        }
                        continue;
                    }
                }
            }

            // Wait for next iteration
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = scheduler.tick() => {},
            }
        }

        tracing::info!("Mentions loop stopped");
    }

    /// Run a single iteration of the mentions loop.
    ///
    /// Returns the results and the new since_id (if any mentions were found).
    /// Used by both the continuous loop and the CLI single-shot command.
    pub async fn run_once(
        &self,
        since_id: Option<&str>,
        limit: Option<usize>,
        storage: &Arc<dyn LoopStorage>,
    ) -> Result<(Vec<MentionResult>, Option<String>), LoopError> {
        let mentions = self.fetcher.get_mentions(since_id).await?;

        if mentions.is_empty() {
            tracing::debug!("No new mentions found");
            return Ok((Vec::new(), None));
        }

        tracing::info!(count = mentions.len(), "Found new mentions");

        let mut results = Vec::new();
        let mut max_id: Option<String> = None;

        let to_process = match limit {
            Some(n) => &mentions[..mentions.len().min(n)],
            None => &mentions,
        };

        for mention in to_process {
            // Track the highest ID for since_id cursor
            super::update_max_id(&mut max_id, &mention.id);

            let result = self.process_mention(mention, storage).await;

            // Log the action
            let (status, message) = match &result {
                MentionResult::Replied {
                    tweet_id,
                    reply_text,
                    ..
                } => (
                    if self.dry_run { "dry_run" } else { "success" },
                    format!(
                        "Replied to mention {tweet_id}: {}",
                        super::truncate(reply_text, 50)
                    ),
                ),
                MentionResult::Skipped { tweet_id, reason } => {
                    ("skipped", format!("Skipped mention {tweet_id}: {reason}"))
                }
                MentionResult::Failed { tweet_id, error } => {
                    ("failure", format!("Failed on mention {tweet_id}: {error}"))
                }
            };

            if let Err(e) = storage.log_action("mention_reply", status, &message).await {
                tracing::warn!(error = %e, "Failed to log action");
            }

            results.push(result);
        }

        Ok((results, max_id))
    }
}
