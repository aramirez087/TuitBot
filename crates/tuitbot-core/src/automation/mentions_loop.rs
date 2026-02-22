//! Mentions monitoring loop.
//!
//! Fetches new @-mentions from X API, generates contextual replies
//! via LLM, and posts them through the posting queue. Persists
//! `since_id` to survive restarts and avoid reprocessing.

use super::loop_helpers::{
    ConsecutiveErrorTracker, LoopError, LoopTweet, MentionsFetcher, PostSender, ReplyGenerator,
    SafetyChecker,
};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Mentions loop that monitors and replies to @-mentions.
pub struct MentionsLoop {
    fetcher: Arc<dyn MentionsFetcher>,
    generator: Arc<dyn ReplyGenerator>,
    safety: Arc<dyn SafetyChecker>,
    poster: Arc<dyn PostSender>,
    dry_run: bool,
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

    /// Run the continuous mentions loop until cancellation.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        interval: Duration,
        storage: Arc<dyn super::loop_helpers::LoopStorage>,
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
                        let backoff = super::loop_helpers::rate_limit_backoff(*retry_after, 0);
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
                _ = tokio::time::sleep(interval) => {},
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
        storage: &Arc<dyn super::loop_helpers::LoopStorage>,
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
            update_max_id(&mut max_id, &mention.id);

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
                        truncate(reply_text, 50)
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

    /// Process a single mention: safety check, generate reply, post.
    async fn process_mention(
        &self,
        mention: &LoopTweet,
        storage: &Arc<dyn super::loop_helpers::LoopStorage>,
    ) -> MentionResult {
        // Check if already replied
        if self.safety.has_replied_to(&mention.id).await {
            tracing::debug!(tweet_id = %mention.id, "Already replied to mention, skipping");
            return MentionResult::Skipped {
                tweet_id: mention.id.clone(),
                reason: "already replied".to_string(),
            };
        }

        // Check rate limits
        if !self.safety.can_reply().await {
            tracing::warn!(tweet_id = %mention.id, "Reply rate limit reached, skipping");
            return MentionResult::Skipped {
                tweet_id: mention.id.clone(),
                reason: "rate limited".to_string(),
            };
        }

        // Generate reply (always mention product for direct mentions)
        let reply_text = match self
            .generator
            .generate_reply(&mention.text, &mention.author_username, true)
            .await
        {
            Ok(text) => text,
            Err(e) => {
                tracing::error!(
                    tweet_id = %mention.id,
                    error = %e,
                    "Failed to generate reply for mention"
                );
                return MentionResult::Failed {
                    tweet_id: mention.id.clone(),
                    error: e.to_string(),
                };
            }
        };

        tracing::info!(
            author = %mention.author_username,
            "Replied to mention from @{}",
            mention.author_username,
        );

        if self.dry_run {
            tracing::info!(
                "DRY RUN: Would reply to mention {} by @{}: \"{}\"",
                mention.id,
                mention.author_username,
                reply_text
            );
        } else {
            // Send to posting queue
            if let Err(e) = self.poster.send_reply(&mention.id, &reply_text).await {
                tracing::error!(
                    tweet_id = %mention.id,
                    error = %e,
                    "Failed to send reply to posting queue"
                );
                return MentionResult::Failed {
                    tweet_id: mention.id.clone(),
                    error: e.to_string(),
                };
            }

            // Record the reply
            if let Err(e) = self.safety.record_reply(&mention.id, &reply_text).await {
                tracing::warn!(
                    tweet_id = %mention.id,
                    error = %e,
                    "Failed to record reply (post may have been sent)"
                );
            }
        }

        // Log to action log (even dry-run records discovered tweets)
        let _ = storage
            .log_action(
                "mention_reply",
                if self.dry_run { "dry_run" } else { "success" },
                &format!(
                    "Reply to @{}: {}",
                    mention.author_username,
                    truncate(&reply_text, 50)
                ),
            )
            .await;

        MentionResult::Replied {
            tweet_id: mention.id.clone(),
            author: mention.author_username.clone(),
            reply_text,
        }
    }
}

/// Update max_id tracking. Tweet IDs are numeric strings; higher = newer.
///
/// Compares by length first (longer numeric string = larger number),
/// then lexicographically for equal-length strings.
fn update_max_id(current: &mut Option<String>, candidate: &str) {
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
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::loop_helpers::LoopStorage;
    use std::sync::Mutex;

    // --- Mock implementations ---

    struct MockFetcher {
        mentions: Vec<LoopTweet>,
    }

    #[async_trait::async_trait]
    impl MentionsFetcher for MockFetcher {
        async fn get_mentions(&self, _since_id: Option<&str>) -> Result<Vec<LoopTweet>, LoopError> {
            Ok(self.mentions.clone())
        }
    }

    struct MockGenerator {
        reply_prefix: String,
    }

    #[async_trait::async_trait]
    impl ReplyGenerator for MockGenerator {
        async fn generate_reply(
            &self,
            _tweet_text: &str,
            author: &str,
            _mention_product: bool,
        ) -> Result<String, LoopError> {
            Ok(format!("{} reply to @{author}", self.reply_prefix))
        }
    }

    struct FailingGenerator;

    #[async_trait::async_trait]
    impl ReplyGenerator for FailingGenerator {
        async fn generate_reply(
            &self,
            _tweet_text: &str,
            _author: &str,
            _mention_product: bool,
        ) -> Result<String, LoopError> {
            Err(LoopError::LlmFailure("timeout".to_string()))
        }
    }

    struct MockSafety {
        replied_ids: Mutex<Vec<String>>,
        can_reply: bool,
    }

    impl MockSafety {
        fn new(can_reply: bool) -> Self {
            Self {
                replied_ids: Mutex::new(Vec::new()),
                can_reply,
            }
        }
    }

    #[async_trait::async_trait]
    impl SafetyChecker for MockSafety {
        async fn can_reply(&self) -> bool {
            self.can_reply
        }

        async fn has_replied_to(&self, tweet_id: &str) -> bool {
            self.replied_ids
                .lock()
                .expect("lock")
                .contains(&tweet_id.to_string())
        }

        async fn record_reply(&self, tweet_id: &str, _content: &str) -> Result<(), LoopError> {
            self.replied_ids
                .lock()
                .expect("lock")
                .push(tweet_id.to_string());
            Ok(())
        }
    }

    struct MockPoster {
        sent: Mutex<Vec<(String, String)>>,
    }

    impl MockPoster {
        fn new() -> Self {
            Self {
                sent: Mutex::new(Vec::new()),
            }
        }

        fn sent_count(&self) -> usize {
            self.sent.lock().expect("lock").len()
        }
    }

    #[async_trait::async_trait]
    impl PostSender for MockPoster {
        async fn send_reply(&self, tweet_id: &str, content: &str) -> Result<(), LoopError> {
            self.sent
                .lock()
                .expect("lock")
                .push((tweet_id.to_string(), content.to_string()));
            Ok(())
        }
    }

    struct MockStorage {
        cursors: Mutex<std::collections::HashMap<String, String>>,
        actions: Mutex<Vec<(String, String, String)>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                cursors: Mutex::new(std::collections::HashMap::new()),
                actions: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl LoopStorage for MockStorage {
        async fn get_cursor(&self, key: &str) -> Result<Option<String>, LoopError> {
            Ok(self.cursors.lock().expect("lock").get(key).cloned())
        }

        async fn set_cursor(&self, key: &str, value: &str) -> Result<(), LoopError> {
            self.cursors
                .lock()
                .expect("lock")
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        async fn tweet_exists(&self, _tweet_id: &str) -> Result<bool, LoopError> {
            Ok(false)
        }

        async fn store_discovered_tweet(
            &self,
            _tweet: &LoopTweet,
            _score: f32,
            _keyword: &str,
        ) -> Result<(), LoopError> {
            Ok(())
        }

        async fn log_action(
            &self,
            action_type: &str,
            status: &str,
            message: &str,
        ) -> Result<(), LoopError> {
            self.actions.lock().expect("lock").push((
                action_type.to_string(),
                status.to_string(),
                message.to_string(),
            ));
            Ok(())
        }
    }

    fn test_tweet(id: &str, author: &str) -> LoopTweet {
        LoopTweet {
            id: id.to_string(),
            text: format!("Test tweet from @{author}"),
            author_id: format!("uid_{author}"),
            author_username: author.to_string(),
            author_followers: 1000,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            likes: 10,
            retweets: 2,
            replies: 1,
        }
    }

    // --- Tests ---

    #[tokio::test]
    async fn run_once_no_mentions() {
        let mentions_loop = MentionsLoop::new(
            Arc::new(MockFetcher {
                mentions: Vec::new(),
            }),
            Arc::new(MockGenerator {
                reply_prefix: "Test".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            Arc::new(MockPoster::new()),
            false,
        );
        let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

        let (results, since_id) = mentions_loop.run_once(None, None, &storage).await.unwrap();
        assert!(results.is_empty());
        assert!(since_id.is_none());
    }

    #[tokio::test]
    async fn run_once_processes_mentions() {
        let poster = Arc::new(MockPoster::new());
        let mentions_loop = MentionsLoop::new(
            Arc::new(MockFetcher {
                mentions: vec![test_tweet("100", "alice"), test_tweet("101", "bob")],
            }),
            Arc::new(MockGenerator {
                reply_prefix: "Hello".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            poster.clone(),
            false,
        );
        let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

        let (results, since_id) = mentions_loop.run_once(None, None, &storage).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(matches!(&results[0], MentionResult::Replied { .. }));
        assert!(matches!(&results[1], MentionResult::Replied { .. }));
        assert_eq!(since_id, Some("101".to_string()));
        assert_eq!(poster.sent_count(), 2);
    }

    #[tokio::test]
    async fn run_once_respects_limit() {
        let poster = Arc::new(MockPoster::new());
        let mentions_loop = MentionsLoop::new(
            Arc::new(MockFetcher {
                mentions: vec![
                    test_tweet("100", "alice"),
                    test_tweet("101", "bob"),
                    test_tweet("102", "carol"),
                ],
            }),
            Arc::new(MockGenerator {
                reply_prefix: "Hi".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            poster.clone(),
            false,
        );
        let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

        let (results, _) = mentions_loop
            .run_once(None, Some(2), &storage)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(poster.sent_count(), 2);
    }

    #[tokio::test]
    async fn run_once_skips_already_replied() {
        let safety = Arc::new(MockSafety::new(true));
        // Pre-mark tweet "100" as replied
        safety.record_reply("100", "already replied").await.unwrap();

        let poster = Arc::new(MockPoster::new());
        let mentions_loop = MentionsLoop::new(
            Arc::new(MockFetcher {
                mentions: vec![test_tweet("100", "alice"), test_tweet("101", "bob")],
            }),
            Arc::new(MockGenerator {
                reply_prefix: "Hi".to_string(),
            }),
            safety,
            poster.clone(),
            false,
        );
        let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

        let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(matches!(&results[0], MentionResult::Skipped { .. }));
        assert!(matches!(&results[1], MentionResult::Replied { .. }));
        assert_eq!(poster.sent_count(), 1);
    }

    #[tokio::test]
    async fn run_once_skips_when_rate_limited() {
        let poster = Arc::new(MockPoster::new());
        let mentions_loop = MentionsLoop::new(
            Arc::new(MockFetcher {
                mentions: vec![test_tweet("100", "alice")],
            }),
            Arc::new(MockGenerator {
                reply_prefix: "Hi".to_string(),
            }),
            Arc::new(MockSafety::new(false)), // can_reply = false
            poster.clone(),
            false,
        );
        let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

        let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(
            matches!(&results[0], MentionResult::Skipped { reason, .. } if reason == "rate limited")
        );
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn run_once_dry_run_does_not_post() {
        let poster = Arc::new(MockPoster::new());
        let mentions_loop = MentionsLoop::new(
            Arc::new(MockFetcher {
                mentions: vec![test_tweet("100", "alice")],
            }),
            Arc::new(MockGenerator {
                reply_prefix: "Hi".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            poster.clone(),
            true, // dry_run
        );
        let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

        let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], MentionResult::Replied { .. }));
        // Should NOT have sent to posting queue
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn run_once_llm_failure_returns_failed() {
        let poster = Arc::new(MockPoster::new());
        let mentions_loop = MentionsLoop::new(
            Arc::new(MockFetcher {
                mentions: vec![test_tweet("100", "alice")],
            }),
            Arc::new(FailingGenerator),
            Arc::new(MockSafety::new(true)),
            poster.clone(),
            false,
        );
        let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

        let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], MentionResult::Failed { .. }));
        assert_eq!(poster.sent_count(), 0);
    }

    #[test]
    fn update_max_id_tracks_highest() {
        let mut max = None;
        update_max_id(&mut max, "100");
        assert_eq!(max, Some("100".to_string()));
        update_max_id(&mut max, "99");
        assert_eq!(max, Some("100".to_string()));
        update_max_id(&mut max, "200");
        assert_eq!(max, Some("200".to_string()));
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("hello world this is long", 10), "hello worl...");
    }
}
