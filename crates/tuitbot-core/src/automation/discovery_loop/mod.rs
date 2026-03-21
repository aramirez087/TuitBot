//! Tweet discovery loop.
//!
//! Searches X using configured keywords, scores each tweet with the
//! scoring engine, filters by threshold, generates replies for
//! qualifying tweets, and posts them through the posting queue.
//! Rotates keywords across iterations to distribute API usage.

use super::loop_helpers::{
    ConsecutiveErrorTracker, LoopError, LoopStorage, LoopTweet, PostSender, ReplyGenerator,
    SafetyChecker, TweetScorer, TweetSearcher,
};
use super::schedule::{schedule_gate, ActiveSchedule};
use super::scheduler::LoopScheduler;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Discovery loop that finds and replies to relevant tweets.
pub struct DiscoveryLoop {
    searcher: Arc<dyn TweetSearcher>,
    scorer: Arc<dyn TweetScorer>,
    generator: Arc<dyn ReplyGenerator>,
    safety: Arc<dyn SafetyChecker>,
    storage: Arc<dyn LoopStorage>,
    poster: Arc<dyn PostSender>,
    keywords: Vec<String>,
    threshold: f32,
    dry_run: bool,
}

/// Result of processing a single discovered tweet.
#[derive(Debug)]
pub enum DiscoveryResult {
    /// Reply was sent (or would be sent in dry-run).
    Replied {
        tweet_id: String,
        author: String,
        score: f32,
        reply_text: String,
    },
    /// Tweet scored below threshold.
    BelowThreshold { tweet_id: String, score: f32 },
    /// Tweet was skipped (safety check, already exists).
    Skipped { tweet_id: String, reason: String },
    /// Processing failed for this tweet.
    Failed { tweet_id: String, error: String },
}

/// Summary of a discovery iteration.
#[derive(Debug, Default)]
pub struct DiscoverySummary {
    /// Total tweets found across all keywords searched.
    pub tweets_found: usize,
    /// Tweets that scored above threshold.
    pub qualifying: usize,
    /// Replies sent (or would be sent in dry-run).
    pub replied: usize,
    /// Tweets skipped (safety, dedup, below threshold).
    pub skipped: usize,
    /// Tweets that failed processing.
    pub failed: usize,
}

impl DiscoveryLoop {
    /// Create a new discovery loop.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        searcher: Arc<dyn TweetSearcher>,
        scorer: Arc<dyn TweetScorer>,
        generator: Arc<dyn ReplyGenerator>,
        safety: Arc<dyn SafetyChecker>,
        storage: Arc<dyn LoopStorage>,
        poster: Arc<dyn PostSender>,
        keywords: Vec<String>,
        threshold: f32,
        dry_run: bool,
    ) -> Self {
        Self {
            searcher,
            scorer,
            generator,
            safety,
            storage,
            poster,
            keywords,
            threshold,
            dry_run,
        }
    }

    /// Run the continuous discovery loop until cancellation.
    ///
    /// Rotates through keywords across iterations to distribute API usage.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        scheduler: LoopScheduler,
        schedule: Option<Arc<ActiveSchedule>>,
    ) {
        tracing::info!(
            dry_run = self.dry_run,
            keywords = self.keywords.len(),
            threshold = self.threshold,
            "Discovery loop started"
        );

        if self.keywords.is_empty() {
            tracing::warn!("No keywords configured, discovery loop has nothing to search");
            cancel.cancelled().await;
            return;
        }

        let mut error_tracker = ConsecutiveErrorTracker::new(10, Duration::from_secs(300));
        let mut keyword_index = 0usize;

        loop {
            if cancel.is_cancelled() {
                break;
            }

            if !schedule_gate(&schedule, &cancel).await {
                break;
            }

            // Select next keyword (round-robin)
            let keyword = &self.keywords[keyword_index % self.keywords.len()];
            keyword_index += 1;

            match self.search_and_process(keyword, None).await {
                Ok((_results, summary)) => {
                    error_tracker.record_success();
                    if summary.tweets_found > 0 {
                        tracing::info!(
                            keyword = %keyword,
                            found = summary.tweets_found,
                            qualifying = summary.qualifying,
                            replied = summary.replied,
                            "Discovery iteration complete"
                        );
                    }
                }
                Err(e) => {
                    let should_pause = error_tracker.record_error();
                    tracing::warn!(
                        keyword = %keyword,
                        error = %e,
                        consecutive_errors = error_tracker.count(),
                        "Discovery iteration failed"
                    );

                    if should_pause {
                        tracing::warn!(
                            pause_secs = error_tracker.pause_duration().as_secs(),
                            "Pausing discovery loop due to consecutive errors"
                        );
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(error_tracker.pause_duration()) => {},
                        }
                        error_tracker.reset();
                        continue;
                    }

                    if let LoopError::RateLimited { retry_after } = &e {
                        let backoff = super::loop_helpers::rate_limit_backoff(*retry_after, 0);
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(backoff) => {},
                        }
                        continue;
                    }
                }
            }

            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = scheduler.tick() => {},
            }
        }

        tracing::info!("Discovery loop stopped");
    }

    /// Run a single-shot discovery across all keywords.
    ///
    /// Used by the CLI `tuitbot discover` command. Searches all keywords
    /// (not rotating) and returns all results sorted by score descending.
    pub async fn run_once(
        &self,
        limit: Option<usize>,
    ) -> Result<(Vec<DiscoveryResult>, DiscoverySummary), LoopError> {
        let mut all_results = Vec::new();
        let mut summary = DiscoverySummary::default();
        let mut total_processed = 0usize;
        let mut last_error: Option<LoopError> = None;
        let mut any_success = false;

        for keyword in &self.keywords {
            if let Some(max) = limit {
                if total_processed >= max {
                    break;
                }
            }

            let remaining = limit.map(|max| max.saturating_sub(total_processed));
            match self.search_and_process(keyword, remaining).await {
                Ok((results, iter_summary)) => {
                    any_success = true;
                    summary.tweets_found += iter_summary.tweets_found;
                    summary.qualifying += iter_summary.qualifying;
                    summary.replied += iter_summary.replied;
                    summary.skipped += iter_summary.skipped;
                    summary.failed += iter_summary.failed;
                    total_processed += iter_summary.tweets_found;
                    all_results.extend(results);
                }
                Err(e) => {
                    tracing::warn!(keyword = %keyword, error = %e, "Search failed for keyword");
                    last_error = Some(e);
                }
            }
        }

        // If every keyword failed, surface the last error instead of
        // reporting a misleading empty success.
        if !any_success {
            if let Some(err) = last_error {
                return Err(err);
            }
        }

        Ok((all_results, summary))
    }

    /// Search for a single keyword and process all results.
    pub(crate) async fn search_and_process(
        &self,
        keyword: &str,
        limit: Option<usize>,
    ) -> Result<(Vec<DiscoveryResult>, DiscoverySummary), LoopError> {
        tracing::info!(keyword = %keyword, "Searching keyword");
        let tweets = self.searcher.search_tweets(keyword).await?;

        let mut summary = DiscoverySummary {
            tweets_found: tweets.len(),
            ..Default::default()
        };

        let to_process = match limit {
            Some(n) => &tweets[..tweets.len().min(n)],
            None => &tweets,
        };

        let mut results = Vec::with_capacity(to_process.len());

        for tweet in to_process {
            let result = self.process_tweet(tweet, keyword).await;

            match &result {
                DiscoveryResult::Replied { .. } => {
                    summary.qualifying += 1;
                    summary.replied += 1;
                }
                DiscoveryResult::BelowThreshold { .. } => {
                    summary.skipped += 1;
                }
                DiscoveryResult::Skipped { .. } => {
                    summary.skipped += 1;
                }
                DiscoveryResult::Failed { .. } => {
                    summary.failed += 1;
                }
            }

            results.push(result);
        }

        Ok((results, summary))
    }

    /// Process a single discovered tweet: dedup, score, generate reply, post.
    pub(crate) async fn process_tweet(&self, tweet: &LoopTweet, keyword: &str) -> DiscoveryResult {
        // Check if already discovered (dedup)
        match self.storage.tweet_exists(&tweet.id).await {
            Ok(true) => {
                tracing::debug!(tweet_id = %tweet.id, "Tweet already discovered, skipping");
                return DiscoveryResult::Skipped {
                    tweet_id: tweet.id.clone(),
                    reason: "already discovered".to_string(),
                };
            }
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(tweet_id = %tweet.id, error = %e, "Failed to check tweet existence");
                // Continue anyway -- best effort dedup
            }
        }

        // Score the tweet
        let score_result = self.scorer.score(tweet);

        // Store discovered tweet (even if below threshold, useful for analytics)
        if let Err(e) = self
            .storage
            .store_discovered_tweet(tweet, score_result.total, keyword)
            .await
        {
            tracing::warn!(tweet_id = %tweet.id, error = %e, "Failed to store discovered tweet");
        }

        // Check threshold
        if !score_result.meets_threshold {
            tracing::debug!(
                tweet_id = %tweet.id,
                score = score_result.total,
                threshold = self.threshold,
                "Tweet scored below threshold, skipping"
            );
            return DiscoveryResult::BelowThreshold {
                tweet_id: tweet.id.clone(),
                score: score_result.total,
            };
        }

        // Safety checks
        if self.safety.has_replied_to(&tweet.id).await {
            return DiscoveryResult::Skipped {
                tweet_id: tweet.id.clone(),
                reason: "already replied".to_string(),
            };
        }

        if !self.safety.can_reply().await {
            return DiscoveryResult::Skipped {
                tweet_id: tweet.id.clone(),
                reason: "rate limited".to_string(),
            };
        }

        // Generate reply with vault context (product mention always on for discovery)
        let reply_output = match self
            .generator
            .generate_reply_with_rag(&tweet.text, &tweet.author_username, true)
            .await
        {
            Ok(output) => output,
            Err(e) => {
                tracing::error!(
                    tweet_id = %tweet.id,
                    error = %e,
                    "Failed to generate reply"
                );
                return DiscoveryResult::Failed {
                    tweet_id: tweet.id.clone(),
                    error: e.to_string(),
                };
            }
        };
        let reply_text = reply_output.text;

        tracing::info!(
            author = %tweet.author_username,
            score = format!("{:.0}", score_result.total),
            "Posted reply to @{}",
            tweet.author_username,
        );

        if self.dry_run {
            tracing::info!(
                "DRY RUN: Tweet {} by @{} scored {:.0}/100 -- Would reply: \"{}\"",
                tweet.id,
                tweet.author_username,
                score_result.total,
                reply_text
            );

            let _ = self
                .storage
                .log_action(
                    "discovery_reply",
                    "dry_run",
                    &format!(
                        "Score {:.0}, reply to @{}: {}",
                        score_result.total,
                        tweet.author_username,
                        truncate(&reply_text, 50)
                    ),
                )
                .await;
        } else {
            if let Err(e) = self.poster.send_reply(&tweet.id, &reply_text).await {
                tracing::error!(tweet_id = %tweet.id, error = %e, "Failed to send reply");
                return DiscoveryResult::Failed {
                    tweet_id: tweet.id.clone(),
                    error: e.to_string(),
                };
            }

            if let Err(e) = self.safety.record_reply(&tweet.id, &reply_text).await {
                tracing::warn!(tweet_id = %tweet.id, error = %e, "Failed to record reply");
            }

            let _ = self
                .storage
                .log_action(
                    "discovery_reply",
                    "success",
                    &format!(
                        "Score {:.0}, replied to @{}: {}",
                        score_result.total,
                        tweet.author_username,
                        truncate(&reply_text, 50)
                    ),
                )
                .await;
        }

        DiscoveryResult::Replied {
            tweet_id: tweet.id.clone(),
            author: tweet.author_username.clone(),
            score: score_result.total,
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
