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
    pub async fn run(&self, cancel: CancellationToken, interval: Duration) {
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
                _ = tokio::time::sleep(interval) => {},
            }
        }

        tracing::info!("Discovery loop stopped");
    }

    /// Run a single-shot discovery across all keywords.
    ///
    /// Used by the CLI `replyguy discover` command. Searches all keywords
    /// (not rotating) and returns all results sorted by score descending.
    pub async fn run_once(
        &self,
        limit: Option<usize>,
    ) -> Result<(Vec<DiscoveryResult>, DiscoverySummary), LoopError> {
        let mut all_results = Vec::new();
        let mut summary = DiscoverySummary::default();
        let mut total_processed = 0usize;

        for keyword in &self.keywords {
            if let Some(max) = limit {
                if total_processed >= max {
                    break;
                }
            }

            let remaining = limit.map(|max| max.saturating_sub(total_processed));
            match self.search_and_process(keyword, remaining).await {
                Ok((results, iter_summary)) => {
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
                }
            }
        }

        Ok((all_results, summary))
    }

    /// Search for a single keyword and process all results.
    async fn search_and_process(
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
    async fn process_tweet(&self, tweet: &LoopTweet, keyword: &str) -> DiscoveryResult {
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

        // Generate reply (product mention decided by caller or random)
        let reply_text = match self
            .generator
            .generate_reply(&tweet.text, &tweet.author_username, true)
            .await
        {
            Ok(text) => text,
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
    use crate::automation::ScoreResult;
    use std::sync::Mutex;

    // --- Mock implementations ---

    struct MockSearcher {
        results: Vec<LoopTweet>,
    }

    #[async_trait::async_trait]
    impl TweetSearcher for MockSearcher {
        async fn search_tweets(&self, _query: &str) -> Result<Vec<LoopTweet>, LoopError> {
            Ok(self.results.clone())
        }
    }

    struct FailingSearcher;

    #[async_trait::async_trait]
    impl TweetSearcher for FailingSearcher {
        async fn search_tweets(&self, _query: &str) -> Result<Vec<LoopTweet>, LoopError> {
            Err(LoopError::RateLimited {
                retry_after: Some(60),
            })
        }
    }

    struct MockScorer {
        score: f32,
        meets_threshold: bool,
    }

    impl TweetScorer for MockScorer {
        fn score(&self, _tweet: &LoopTweet) -> ScoreResult {
            ScoreResult {
                total: self.score,
                meets_threshold: self.meets_threshold,
                matched_keywords: vec!["test".to_string()],
            }
        }
    }

    struct MockGenerator {
        reply: String,
    }

    #[async_trait::async_trait]
    impl ReplyGenerator for MockGenerator {
        async fn generate_reply(
            &self,
            _tweet_text: &str,
            _author: &str,
            _mention_product: bool,
        ) -> Result<String, LoopError> {
            Ok(self.reply.clone())
        }
    }

    struct MockSafety {
        can_reply: bool,
        replied_ids: Mutex<Vec<String>>,
    }

    impl MockSafety {
        fn new(can_reply: bool) -> Self {
            Self {
                can_reply,
                replied_ids: Mutex::new(Vec::new()),
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

    struct MockStorage {
        existing_ids: Mutex<Vec<String>>,
        discovered: Mutex<Vec<String>>,
        actions: Mutex<Vec<(String, String, String)>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                existing_ids: Mutex::new(Vec::new()),
                discovered: Mutex::new(Vec::new()),
                actions: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl LoopStorage for MockStorage {
        async fn get_cursor(&self, _key: &str) -> Result<Option<String>, LoopError> {
            Ok(None)
        }
        async fn set_cursor(&self, _key: &str, _value: &str) -> Result<(), LoopError> {
            Ok(())
        }
        async fn tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError> {
            Ok(self
                .existing_ids
                .lock()
                .expect("lock")
                .contains(&tweet_id.to_string()))
        }
        async fn store_discovered_tweet(
            &self,
            tweet: &LoopTweet,
            _score: f32,
            _keyword: &str,
        ) -> Result<(), LoopError> {
            self.discovered.lock().expect("lock").push(tweet.id.clone());
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

    fn test_tweet(id: &str, author: &str) -> LoopTweet {
        LoopTweet {
            id: id.to_string(),
            text: format!("Test tweet about rust from @{author}"),
            author_id: format!("uid_{author}"),
            author_username: author.to_string(),
            author_followers: 5000,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            likes: 20,
            retweets: 5,
            replies: 3,
        }
    }

    fn build_loop(
        tweets: Vec<LoopTweet>,
        score: f32,
        meets_threshold: bool,
        dry_run: bool,
    ) -> (DiscoveryLoop, Arc<MockPoster>, Arc<MockStorage>) {
        let poster = Arc::new(MockPoster::new());
        let storage = Arc::new(MockStorage::new());
        let discovery = DiscoveryLoop::new(
            Arc::new(MockSearcher { results: tweets }),
            Arc::new(MockScorer {
                score,
                meets_threshold,
            }),
            Arc::new(MockGenerator {
                reply: "Great insight!".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            storage.clone(),
            poster.clone(),
            vec!["rust".to_string(), "cli".to_string()],
            70.0,
            dry_run,
        );
        (discovery, poster, storage)
    }

    // --- Tests ---

    #[tokio::test]
    async fn search_and_process_no_results() {
        let (discovery, poster, _) = build_loop(Vec::new(), 80.0, true, false);
        let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();
        assert_eq!(summary.tweets_found, 0);
        assert!(results.is_empty());
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn search_and_process_above_threshold() {
        let tweets = vec![test_tweet("100", "alice"), test_tweet("101", "bob")];
        let (discovery, poster, storage) = build_loop(tweets, 85.0, true, false);

        let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();

        assert_eq!(summary.tweets_found, 2);
        assert_eq!(summary.replied, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(poster.sent_count(), 2);

        // Both tweets should be stored as discovered
        let discovered = storage.discovered.lock().expect("lock");
        assert_eq!(discovered.len(), 2);
    }

    #[tokio::test]
    async fn search_and_process_below_threshold() {
        let tweets = vec![test_tweet("100", "alice")];
        let (discovery, poster, storage) = build_loop(tweets, 40.0, false, false);

        let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();

        assert_eq!(summary.tweets_found, 1);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.replied, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(poster.sent_count(), 0);

        // Tweet should still be stored as discovered (for analytics)
        let discovered = storage.discovered.lock().expect("lock");
        assert_eq!(discovered.len(), 1);
    }

    #[tokio::test]
    async fn search_and_process_dry_run() {
        let tweets = vec![test_tweet("100", "alice")];
        let (discovery, poster, _) = build_loop(tweets, 85.0, true, true);

        let (_results, summary) = discovery.search_and_process("rust", None).await.unwrap();

        assert_eq!(summary.replied, 1);
        // Should NOT post in dry-run
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn search_and_process_skips_existing() {
        let tweets = vec![test_tweet("100", "alice")];
        let poster = Arc::new(MockPoster::new());
        let storage = Arc::new(MockStorage::new());
        // Pre-mark tweet as existing
        storage
            .existing_ids
            .lock()
            .expect("lock")
            .push("100".to_string());

        let discovery = DiscoveryLoop::new(
            Arc::new(MockSearcher { results: tweets }),
            Arc::new(MockScorer {
                score: 85.0,
                meets_threshold: true,
            }),
            Arc::new(MockGenerator {
                reply: "Great!".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            storage,
            poster.clone(),
            vec!["rust".to_string()],
            70.0,
            false,
        );

        let (_results, summary) = discovery.search_and_process("rust", None).await.unwrap();
        assert_eq!(summary.skipped, 1);
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn search_and_process_respects_limit() {
        let tweets = vec![
            test_tweet("100", "alice"),
            test_tweet("101", "bob"),
            test_tweet("102", "carol"),
        ];
        let (discovery, poster, _) = build_loop(tweets, 85.0, true, false);

        let (results, summary) = discovery.search_and_process("rust", Some(2)).await.unwrap();

        assert_eq!(summary.tweets_found, 3); // found 3, but...
        assert_eq!(results.len(), 2); // only 2 results returned
        assert_eq!(poster.sent_count(), 2); // only processed 2
    }

    #[tokio::test]
    async fn run_once_searches_all_keywords() {
        let tweets = vec![test_tweet("100", "alice")];
        let (discovery, _, _) = build_loop(tweets, 85.0, true, false);

        let (_, summary) = discovery.run_once(None).await.unwrap();
        // Should search both "rust" and "cli" keywords
        assert_eq!(summary.tweets_found, 2); // 1 tweet per keyword
    }

    #[tokio::test]
    async fn search_error_returns_loop_error() {
        let poster = Arc::new(MockPoster::new());
        let storage = Arc::new(MockStorage::new());
        let discovery = DiscoveryLoop::new(
            Arc::new(FailingSearcher),
            Arc::new(MockScorer {
                score: 85.0,
                meets_threshold: true,
            }),
            Arc::new(MockGenerator {
                reply: "test".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            storage,
            poster,
            vec!["rust".to_string()],
            70.0,
            false,
        );

        let result = discovery.search_and_process("rust", None).await;
        assert!(result.is_err());
    }
}
