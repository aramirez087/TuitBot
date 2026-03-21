//! Target account monitoring loop.
//!
//! Fetches recent tweets from configured target accounts, scores them
//! with adjusted weights (preferring recency and low reply count), and
//! generates relationship-based replies. This loop operates independently
//! from keyword-based discovery to enable genuine engagement with specific
//! people.

use super::loop_helpers::{
    ConsecutiveErrorTracker, LoopError, LoopTweet, PostSender, ReplyGenerator, SafetyChecker,
};
use super::schedule::{schedule_gate, ActiveSchedule};
use super::scheduler::LoopScheduler;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

// ============================================================================
// Port traits specific to target loop
// ============================================================================

/// Fetches tweets from a specific user by user ID.
#[async_trait::async_trait]
pub trait TargetTweetFetcher: Send + Sync {
    /// Fetch recent tweets from the given user.
    async fn fetch_user_tweets(&self, user_id: &str) -> Result<Vec<LoopTweet>, LoopError>;
}

/// Looks up a user by username.
#[async_trait::async_trait]
pub trait TargetUserManager: Send + Sync {
    /// Look up a user by username. Returns (user_id, username).
    async fn lookup_user(&self, username: &str) -> Result<(String, String), LoopError>;
}

/// Storage operations for target account state.
#[allow(clippy::too_many_arguments)]
#[async_trait::async_trait]
pub trait TargetStorage: Send + Sync {
    /// Upsert a target account record.
    async fn upsert_target_account(
        &self,
        account_id: &str,
        username: &str,
    ) -> Result<(), LoopError>;

    /// Check if a target tweet already exists.
    async fn target_tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError>;

    /// Store a discovered target tweet.
    async fn store_target_tweet(
        &self,
        tweet_id: &str,
        account_id: &str,
        content: &str,
        created_at: &str,
        reply_count: i64,
        like_count: i64,
        relevance_score: f64,
    ) -> Result<(), LoopError>;

    /// Mark a target tweet as replied to.
    async fn mark_target_tweet_replied(&self, tweet_id: &str) -> Result<(), LoopError>;

    /// Record a reply to a target account (increments counter).
    async fn record_target_reply(&self, account_id: &str) -> Result<(), LoopError>;

    /// Get count of target replies sent today.
    async fn count_target_replies_today(&self) -> Result<i64, LoopError>;

    /// Log an action.
    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), LoopError>;
}

// ============================================================================
// Target loop config
// ============================================================================

/// Configuration for the target monitoring loop.
#[derive(Debug, Clone)]
pub struct TargetLoopConfig {
    /// Target account usernames (without @).
    pub accounts: Vec<String>,
    /// Maximum target replies per day.
    pub max_target_replies_per_day: u32,
    /// Whether this is a dry run.
    pub dry_run: bool,
}

// ============================================================================
// Target loop result
// ============================================================================

/// Result of processing a single target tweet.
#[derive(Debug)]
pub enum TargetResult {
    /// Reply was sent (or would be in dry-run).
    Replied {
        tweet_id: String,
        account: String,
        reply_text: String,
    },
    /// Tweet was skipped.
    Skipped { tweet_id: String, reason: String },
    /// Processing failed.
    Failed { tweet_id: String, error: String },
}

// ============================================================================
// Target loop
// ============================================================================

/// Monitors target accounts and generates relationship-based replies.
pub struct TargetLoop {
    fetcher: Arc<dyn TargetTweetFetcher>,
    user_mgr: Arc<dyn TargetUserManager>,
    generator: Arc<dyn ReplyGenerator>,
    safety: Arc<dyn SafetyChecker>,
    storage: Arc<dyn TargetStorage>,
    poster: Arc<dyn PostSender>,
    config: TargetLoopConfig,
}

impl TargetLoop {
    /// Create a new target monitoring loop.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fetcher: Arc<dyn TargetTweetFetcher>,
        user_mgr: Arc<dyn TargetUserManager>,
        generator: Arc<dyn ReplyGenerator>,
        safety: Arc<dyn SafetyChecker>,
        storage: Arc<dyn TargetStorage>,
        poster: Arc<dyn PostSender>,
        config: TargetLoopConfig,
    ) -> Self {
        Self {
            fetcher,
            user_mgr,
            generator,
            safety,
            storage,
            poster,
            config,
        }
    }

    /// Run the continuous target monitoring loop until cancellation.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        scheduler: LoopScheduler,
        schedule: Option<Arc<ActiveSchedule>>,
    ) {
        tracing::info!(
            dry_run = self.config.dry_run,
            accounts = self.config.accounts.len(),
            max_replies = self.config.max_target_replies_per_day,
            "Target monitoring loop started"
        );

        if self.config.accounts.is_empty() {
            tracing::info!("No target accounts configured, target loop has nothing to do");
            cancel.cancelled().await;
            return;
        }

        let mut error_tracker = ConsecutiveErrorTracker::new(10, Duration::from_secs(300));

        loop {
            if cancel.is_cancelled() {
                break;
            }

            if !schedule_gate(&schedule, &cancel).await {
                break;
            }

            match self.run_iteration().await {
                Ok(results) => {
                    error_tracker.record_success();
                    let replied = results
                        .iter()
                        .filter(|r| matches!(r, TargetResult::Replied { .. }))
                        .count();
                    let skipped = results
                        .iter()
                        .filter(|r| matches!(r, TargetResult::Skipped { .. }))
                        .count();
                    if !results.is_empty() {
                        tracing::info!(
                            total = results.len(),
                            replied = replied,
                            skipped = skipped,
                            "Target iteration complete"
                        );
                    }
                }
                Err(e) => {
                    let should_pause = error_tracker.record_error();
                    tracing::warn!(
                        error = %e,
                        consecutive_errors = error_tracker.count(),
                        "Target iteration failed"
                    );

                    if should_pause {
                        tracing::warn!(
                            pause_secs = error_tracker.pause_duration().as_secs(),
                            "Pausing target loop due to consecutive errors"
                        );
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(error_tracker.pause_duration()) => {},
                        }
                        error_tracker.reset();
                        continue;
                    }
                }
            }

            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = scheduler.tick() => {},
            }
        }

        tracing::info!("Target monitoring loop stopped");
    }

    /// Run a single iteration across all target accounts.
    pub async fn run_iteration(&self) -> Result<Vec<TargetResult>, LoopError> {
        let mut all_results = Vec::new();

        // Check daily limit
        let replies_today = self.storage.count_target_replies_today().await?;
        if replies_today >= self.config.max_target_replies_per_day as i64 {
            tracing::debug!(
                replies_today = replies_today,
                limit = self.config.max_target_replies_per_day,
                "Target reply daily limit reached"
            );
            return Ok(all_results);
        }

        let mut remaining_replies =
            (self.config.max_target_replies_per_day as i64 - replies_today) as usize;

        for username in &self.config.accounts {
            if remaining_replies == 0 {
                break;
            }

            match self.process_account(username, remaining_replies).await {
                Ok(results) => {
                    let replied_count = results
                        .iter()
                        .filter(|r| matches!(r, TargetResult::Replied { .. }))
                        .count();
                    remaining_replies = remaining_replies.saturating_sub(replied_count);
                    all_results.extend(results);
                }
                Err(e) => {
                    // AuthExpired is global — stop immediately instead of
                    // failing N times with the same 401.
                    if matches!(e, LoopError::AuthExpired) {
                        tracing::error!(
                            username = %username,
                            "X API authentication expired, re-authenticate with `tuitbot init`"
                        );
                        return Err(e);
                    }

                    tracing::warn!(
                        username = %username,
                        error = %e,
                        "Failed to process target account"
                    );
                }
            }
        }

        Ok(all_results)
    }

    /// Process a single target account: resolve, fetch tweets, reply.
    async fn process_account(
        &self,
        username: &str,
        max_replies: usize,
    ) -> Result<Vec<TargetResult>, LoopError> {
        // Look up user
        let (user_id, resolved_username) = self.user_mgr.lookup_user(username).await?;

        // Upsert target account record
        self.storage
            .upsert_target_account(&user_id, &resolved_username)
            .await?;

        // Fetch recent tweets
        let tweets = self.fetcher.fetch_user_tweets(&user_id).await?;
        tracing::info!(
            username = %resolved_username,
            count = tweets.len(),
            "Monitoring @{}, found {} new tweets",
            resolved_username,
            tweets.len(),
        );

        let mut results = Vec::new();

        for tweet in tweets.iter().take(max_replies) {
            let result = self
                .process_target_tweet(tweet, &user_id, &resolved_username)
                .await;
            if matches!(result, TargetResult::Replied { .. }) {
                results.push(result);
                // Only reply to one tweet per account per iteration
                break;
            }
            results.push(result);
        }

        Ok(results)
    }

    /// Process a single target tweet: dedup, safety check, generate reply, post.
    async fn process_target_tweet(
        &self,
        tweet: &LoopTweet,
        account_id: &str,
        username: &str,
    ) -> TargetResult {
        // Check if already seen
        match self.storage.target_tweet_exists(&tweet.id).await {
            Ok(true) => {
                return TargetResult::Skipped {
                    tweet_id: tweet.id.clone(),
                    reason: "already discovered".to_string(),
                };
            }
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(tweet_id = %tweet.id, error = %e, "Failed to check target tweet");
            }
        }

        // Store the discovered tweet
        let _ = self
            .storage
            .store_target_tweet(
                &tweet.id,
                account_id,
                &tweet.text,
                &tweet.created_at,
                tweet.replies as i64,
                tweet.likes as i64,
                0.0,
            )
            .await;

        // Safety checks
        if self.safety.has_replied_to(&tweet.id).await {
            return TargetResult::Skipped {
                tweet_id: tweet.id.clone(),
                reason: "already replied".to_string(),
            };
        }

        if !self.safety.can_reply().await {
            return TargetResult::Skipped {
                tweet_id: tweet.id.clone(),
                reason: "rate limited".to_string(),
            };
        }

        // Generate reply with vault context (no product mention — genuine engagement)
        let reply_output = match self
            .generator
            .generate_reply_with_rag(&tweet.text, username, false)
            .await
        {
            Ok(output) => output,
            Err(e) => {
                return TargetResult::Failed {
                    tweet_id: tweet.id.clone(),
                    error: e.to_string(),
                };
            }
        };
        let reply_text = reply_output.text;

        tracing::info!(
            username = %username,
            "Replied to target @{}",
            username,
        );

        if self.config.dry_run {
            tracing::info!(
                "DRY RUN: Target @{} tweet {} -- Would reply: \"{}\"",
                username,
                tweet.id,
                reply_text
            );

            let _ = self
                .storage
                .log_action(
                    "target_reply",
                    "dry_run",
                    &format!("Reply to @{username}: {}", truncate(&reply_text, 50)),
                )
                .await;
        } else {
            if let Err(e) = self.poster.send_reply(&tweet.id, &reply_text).await {
                return TargetResult::Failed {
                    tweet_id: tweet.id.clone(),
                    error: e.to_string(),
                };
            }

            if let Err(e) = self.safety.record_reply(&tweet.id, &reply_text).await {
                tracing::warn!(tweet_id = %tweet.id, error = %e, "Failed to record reply");
            }

            // Mark tweet as replied and update account stats
            let _ = self.storage.mark_target_tweet_replied(&tweet.id).await;
            let _ = self.storage.record_target_reply(account_id).await;

            let _ = self
                .storage
                .log_action(
                    "target_reply",
                    "success",
                    &format!("Replied to @{username}: {}", truncate(&reply_text, 50)),
                )
                .await;
        }

        TargetResult::Replied {
            tweet_id: tweet.id.clone(),
            account: username.to_string(),
            reply_text,
        }
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

#[cfg(test)]
mod tests;
