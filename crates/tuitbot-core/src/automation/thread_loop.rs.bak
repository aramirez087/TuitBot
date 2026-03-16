//! Thread loop for posting multi-tweet educational threads.
//!
//! Generates and posts educational threads (5-8 tweets) as reply chains
//! on a configurable schedule. Threads bypass the posting queue since
//! reply chain order must be maintained (each tweet replies to the previous).

use super::loop_helpers::{ContentLoopError, ContentSafety, ContentStorage, ThreadPoster};
use super::schedule::{apply_slot_jitter, schedule_gate, ActiveSchedule};
use super::scheduler::LoopScheduler;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Thread loop that generates and posts educational threads.
pub struct ThreadLoop {
    generator: Arc<dyn ThreadGenerator>,
    safety: Arc<dyn ContentSafety>,
    storage: Arc<dyn ContentStorage>,
    poster: Arc<dyn ThreadPoster>,
    topics: Vec<String>,
    thread_interval_secs: u64,
    dry_run: bool,
}

/// Trait for generating multi-tweet threads.
#[async_trait::async_trait]
pub trait ThreadGenerator: Send + Sync {
    /// Generate a thread of tweets on the given topic.
    ///
    /// If `count` is Some, generate exactly that many tweets.
    /// Otherwise, the LLM decides (typically 5-8).
    async fn generate_thread(
        &self,
        topic: &str,
        count: Option<usize>,
    ) -> Result<Vec<String>, ContentLoopError>;
}

/// Result of a thread generation/posting attempt.
#[derive(Debug)]
pub enum ThreadResult {
    /// Thread was posted (or would be in dry-run).
    Posted {
        topic: String,
        tweet_count: usize,
        thread_id: String,
    },
    /// Thread partially posted (some tweets succeeded, one failed).
    PartialFailure {
        topic: String,
        tweets_posted: usize,
        total_tweets: usize,
        error: String,
    },
    /// Skipped because not enough time has elapsed since last thread.
    TooSoon {
        elapsed_secs: u64,
        interval_secs: u64,
    },
    /// Skipped due to weekly thread rate limit.
    RateLimited,
    /// No topics configured.
    NoTopics,
    /// Content validation failed after max retries.
    ValidationFailed { error: String },
    /// Generation failed.
    Failed { error: String },
}

impl ThreadLoop {
    /// Create a new thread loop.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        generator: Arc<dyn ThreadGenerator>,
        safety: Arc<dyn ContentSafety>,
        storage: Arc<dyn ContentStorage>,
        poster: Arc<dyn ThreadPoster>,
        topics: Vec<String>,
        thread_interval_secs: u64,
        dry_run: bool,
    ) -> Self {
        Self {
            generator,
            safety,
            storage,
            poster,
            topics,
            thread_interval_secs,
            dry_run,
        }
    }

    /// Run the continuous thread loop until cancellation.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        scheduler: LoopScheduler,
        schedule: Option<Arc<ActiveSchedule>>,
    ) {
        let slot_mode = schedule
            .as_ref()
            .is_some_and(|s| s.has_thread_preferred_schedule());

        tracing::info!(
            dry_run = self.dry_run,
            topics = self.topics.len(),
            thread_interval_secs = self.thread_interval_secs,
            slot_mode = slot_mode,
            "Thread loop started"
        );

        if self.topics.is_empty() {
            tracing::warn!("No topics configured, thread loop has nothing to post");
            cancel.cancelled().await;
            return;
        }

        let min_recent = 3usize;
        let max_recent = (self.topics.len() / 2)
            .max(min_recent)
            .min(self.topics.len());
        let mut recent_topics: Vec<String> = Vec::with_capacity(max_recent);
        let mut rng = rand::rngs::StdRng::from_entropy();

        loop {
            if cancel.is_cancelled() {
                break;
            }

            if !schedule_gate(&schedule, &cancel).await {
                break;
            }

            if slot_mode {
                let sched = schedule.as_ref().expect("slot_mode requires schedule");

                match sched.next_thread_slot() {
                    Some(wait) => {
                        let jittered_wait = apply_slot_jitter(wait);
                        tracing::info!(
                            wait_secs = jittered_wait.as_secs(),
                            "Thread slot mode: sleeping until preferred thread time"
                        );

                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(jittered_wait) => {},
                        }

                        if cancel.is_cancelled() {
                            break;
                        }

                        // In slot mode, skip the elapsed-time check — post directly
                        if !self.safety.can_post_thread().await {
                            Self::log_thread_result(&ThreadResult::RateLimited, self.dry_run);
                            continue;
                        }

                        let topic = pick_topic(&self.topics, &mut recent_topics, &mut rng);
                        let result = self.generate_and_post(&topic, None).await;

                        if matches!(result, ThreadResult::Posted { .. }) {
                            if recent_topics.len() >= max_recent {
                                recent_topics.remove(0);
                            }
                            recent_topics.push(topic);
                        }

                        Self::log_thread_result(&result, self.dry_run);
                    }
                    None => {
                        // Should not happen since next_thread_slot always returns Some when configured
                        tracing::warn!("Thread slot mode: no next slot found, sleeping 1 hour");
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(Duration::from_secs(3600)) => {},
                        }
                    }
                }
            } else {
                // Interval-based scheduling (existing behavior)
                let result = self
                    .run_iteration(&mut recent_topics, max_recent, &mut rng)
                    .await;
                Self::log_thread_result(&result, self.dry_run);

                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = scheduler.tick() => {},
                }
            }
        }

        tracing::info!("Thread loop stopped");
    }

    /// Log the result of a thread iteration.
    fn log_thread_result(result: &ThreadResult, dry_run: bool) {
        match result {
            ThreadResult::Posted {
                topic, tweet_count, ..
            } => {
                tracing::info!(
                    topic = %topic,
                    tweets = tweet_count,
                    dry_run = dry_run,
                    "Thread iteration: thread posted"
                );
            }
            ThreadResult::PartialFailure {
                tweets_posted,
                total_tweets,
                error,
                ..
            } => {
                tracing::warn!(
                    posted = tweets_posted,
                    total = total_tweets,
                    error = %error,
                    "Thread iteration: partial failure"
                );
            }
            ThreadResult::TooSoon { .. } => {
                tracing::debug!("Thread iteration: too soon since last thread");
            }
            ThreadResult::RateLimited => {
                tracing::info!("Thread iteration: weekly thread limit reached");
            }
            ThreadResult::NoTopics => {
                tracing::warn!("Thread iteration: no topics available");
            }
            ThreadResult::ValidationFailed { error } => {
                tracing::warn!(error = %error, "Thread iteration: validation failed");
            }
            ThreadResult::Failed { error } => {
                tracing::warn!(error = %error, "Thread iteration: failed");
            }
        }
    }

    /// Run a single thread generation (for CLI `tuitbot thread` command).
    ///
    /// If `topic` is provided, uses that topic. Otherwise picks a random one.
    /// If `count` is provided, generates exactly that many tweets (clamped 2-15).
    pub async fn run_once(&self, topic: Option<&str>, count: Option<usize>) -> ThreadResult {
        let chosen_topic = match topic {
            Some(t) => t.to_string(),
            None => {
                if self.topics.is_empty() {
                    return ThreadResult::NoTopics;
                }
                let mut rng = rand::thread_rng();
                self.topics
                    .choose(&mut rng)
                    .expect("topics is non-empty")
                    .clone()
            }
        };

        // Clamp count to reasonable range
        let clamped_count = count.map(|c| c.clamp(2, 15));

        // Skip interval check for single-shot, but check safety
        if !self.safety.can_post_thread().await {
            return ThreadResult::RateLimited;
        }

        self.generate_and_post(&chosen_topic, clamped_count).await
    }

    /// Run a single iteration of the continuous loop.
    async fn run_iteration(
        &self,
        recent_topics: &mut Vec<String>,
        max_recent: usize,
        rng: &mut impl rand::Rng,
    ) -> ThreadResult {
        // Check elapsed time since last thread
        match self.storage.last_thread_time().await {
            Ok(Some(last_time)) => {
                let elapsed = chrono::Utc::now()
                    .signed_duration_since(last_time)
                    .num_seconds()
                    .max(0) as u64;

                if elapsed < self.thread_interval_secs {
                    return ThreadResult::TooSoon {
                        elapsed_secs: elapsed,
                        interval_secs: self.thread_interval_secs,
                    };
                }
            }
            Ok(None) => {
                // No previous threads -- proceed
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query last thread time, proceeding anyway");
            }
        }

        // Check safety (weekly thread limit)
        if !self.safety.can_post_thread().await {
            return ThreadResult::RateLimited;
        }

        // Pick a topic
        let topic = pick_topic(&self.topics, recent_topics, rng);

        let result = self.generate_and_post(&topic, None).await;

        // Update recent_topics on success
        if matches!(result, ThreadResult::Posted { .. }) {
            if recent_topics.len() >= max_recent {
                recent_topics.remove(0);
            }
            recent_topics.push(topic);
        }

        result
    }

    /// Generate a thread and post it (or print in dry-run mode).
    async fn generate_and_post(&self, topic: &str, count: Option<usize>) -> ThreadResult {
        tracing::info!(topic = %topic, "Generating thread on topic");

        // Generate with retries for length validation
        let tweets = match self.generate_with_validation(topic, count).await {
            Ok(tweets) => tweets,
            Err(result) => return result,
        };

        let tweet_count = tweets.len();

        if self.dry_run {
            tracing::info!(
                "DRY RUN: Would post thread on topic '{}' ({} tweets):",
                topic,
                tweet_count
            );

            for (i, tweet) in tweets.iter().enumerate() {
                tracing::info!(
                    "  {}/{}: \"{}\" ({} chars)",
                    i + 1,
                    tweet_count,
                    tweet,
                    tweet.len()
                );
            }

            let _ = self
                .storage
                .log_action(
                    "thread",
                    "dry_run",
                    &format!("Topic '{}': {} tweets", topic, tweet_count),
                )
                .await;

            return ThreadResult::Posted {
                topic: topic.to_string(),
                tweet_count,
                thread_id: "dry-run".to_string(),
            };
        }

        // Create thread record in DB
        let thread_id = match self.storage.create_thread(topic, tweet_count).await {
            Ok(id) => id,
            Err(e) => {
                tracing::error!(error = %e, "Failed to create thread record");
                return ThreadResult::Failed {
                    error: format!("Storage error: {e}"),
                };
            }
        };

        // Post tweets as reply chain
        let result = self.post_reply_chain(&thread_id, &tweets, topic).await;

        // Log action
        let (status, message) = match &result {
            ThreadResult::Posted { tweet_count, .. } => (
                "success",
                format!("Topic '{}': {} tweets posted", topic, tweet_count),
            ),
            ThreadResult::PartialFailure {
                tweets_posted,
                total_tweets,
                error,
                ..
            } => (
                "partial",
                format!(
                    "Topic '{}': {}/{} tweets posted, error: {}",
                    topic, tweets_posted, total_tweets, error
                ),
            ),
            _ => ("failure", format!("Topic '{}': unexpected state", topic)),
        };
        let _ = self.storage.log_action("thread", status, &message).await;

        result
    }

    /// Generate thread with up to 3 retries for length validation.
    async fn generate_with_validation(
        &self,
        topic: &str,
        count: Option<usize>,
    ) -> Result<Vec<String>, ThreadResult> {
        let max_retries = 3;

        for attempt in 0..max_retries {
            let effective_topic = if attempt == 0 {
                topic.to_string()
            } else {
                format!("{topic} (IMPORTANT: each tweet MUST be under 280 characters)")
            };

            let tweets = match self
                .generator
                .generate_thread(&effective_topic, count)
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    return Err(ThreadResult::Failed {
                        error: format!("Generation failed: {e}"),
                    });
                }
            };

            // Validate all tweets are <= 280 chars (URL-aware)
            let all_valid = tweets.iter().all(|t| {
                crate::content::length::tweet_weighted_len(t)
                    <= crate::content::length::MAX_TWEET_CHARS
            });
            if all_valid {
                return Ok(tweets);
            }

            let over_limit: Vec<usize> = tweets
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    crate::content::length::tweet_weighted_len(t)
                        > crate::content::length::MAX_TWEET_CHARS
                })
                .map(|(i, _)| i + 1)
                .collect();

            tracing::debug!(
                attempt = attempt + 1,
                over_limit = ?over_limit,
                "Thread tweets exceed 280 chars, retrying"
            );
        }

        Err(ThreadResult::ValidationFailed {
            error: format!(
                "Thread tweets still exceed 280 characters after {max_retries} attempts"
            ),
        })
    }

    /// Post tweets as a reply chain. First tweet is standalone,
    /// each subsequent tweet replies to the previous one.
    async fn post_reply_chain(
        &self,
        thread_id: &str,
        tweets: &[String],
        topic: &str,
    ) -> ThreadResult {
        let total = tweets.len();
        let mut previous_tweet_id: Option<String> = None;
        let mut root_tweet_id: Option<String> = None;

        for (i, tweet_content) in tweets.iter().enumerate() {
            let post_result = if i == 0 {
                // First tweet: standalone
                self.poster.post_tweet(tweet_content).await
            } else {
                // Reply to previous tweet
                let prev_id = previous_tweet_id
                    .as_ref()
                    .expect("previous_tweet_id set after first tweet");
                self.poster.reply_to_tweet(prev_id, tweet_content).await
            };

            match post_result {
                Ok(new_tweet_id) => {
                    tracing::info!(
                        position = i + 1,
                        total = total,
                        "Posted thread tweet {}/{}",
                        i + 1,
                        total,
                    );
                    if i == 0 {
                        root_tweet_id = Some(new_tweet_id.clone());
                        // Update thread with root tweet ID
                        let _ = self
                            .storage
                            .update_thread_status(thread_id, "posting", i + 1, Some(&new_tweet_id))
                            .await;
                    }

                    // Record thread tweet
                    let _ = self
                        .storage
                        .store_thread_tweet(thread_id, i, &new_tweet_id, tweet_content)
                        .await;

                    previous_tweet_id = Some(new_tweet_id);

                    // Small delay between posts (1-3 seconds)
                    if i < total - 1 {
                        let delay = Duration::from_secs(1)
                            + Duration::from_millis(rand::random::<u64>() % 2000);
                        tokio::time::sleep(delay).await;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        thread_id = %thread_id,
                        tweet_index = i,
                        error = %e,
                        "Failed to post tweet {}/{} in thread",
                        i + 1,
                        total
                    );

                    // Mark thread as partial
                    let _ = self
                        .storage
                        .update_thread_status(thread_id, "partial", i, root_tweet_id.as_deref())
                        .await;

                    return ThreadResult::PartialFailure {
                        topic: topic.to_string(),
                        tweets_posted: i,
                        total_tweets: total,
                        error: e.to_string(),
                    };
                }
            }
        }

        // All tweets posted successfully
        let _ = self
            .storage
            .update_thread_status(thread_id, "sent", total, root_tweet_id.as_deref())
            .await;

        ThreadResult::Posted {
            topic: topic.to_string(),
            tweet_count: total,
            thread_id: thread_id.to_string(),
        }
    }
}

/// Pick a topic that is not in the recent list.
fn pick_topic(topics: &[String], recent: &mut Vec<String>, rng: &mut impl rand::Rng) -> String {
    let available: Vec<&String> = topics.iter().filter(|t| !recent.contains(t)).collect();

    if available.is_empty() {
        recent.clear();
        topics.choose(rng).expect("topics is non-empty").clone()
    } else {
        available
            .choose(rng)
            .expect("available is non-empty")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // --- Mock implementations ---

    struct MockThreadGenerator {
        tweets: Vec<String>,
    }

    #[async_trait::async_trait]
    impl ThreadGenerator for MockThreadGenerator {
        async fn generate_thread(
            &self,
            _topic: &str,
            _count: Option<usize>,
        ) -> Result<Vec<String>, ContentLoopError> {
            Ok(self.tweets.clone())
        }
    }

    struct OverlongThreadGenerator;

    #[async_trait::async_trait]
    impl ThreadGenerator for OverlongThreadGenerator {
        async fn generate_thread(
            &self,
            _topic: &str,
            _count: Option<usize>,
        ) -> Result<Vec<String>, ContentLoopError> {
            // Always returns tweets that are too long
            Ok(vec!["a".repeat(300), "b".repeat(300)])
        }
    }

    struct FailingThreadGenerator;

    #[async_trait::async_trait]
    impl ThreadGenerator for FailingThreadGenerator {
        async fn generate_thread(
            &self,
            _topic: &str,
            _count: Option<usize>,
        ) -> Result<Vec<String>, ContentLoopError> {
            Err(ContentLoopError::LlmFailure("model error".to_string()))
        }
    }

    struct MockSafety {
        can_tweet: bool,
        can_thread: bool,
    }

    #[async_trait::async_trait]
    impl ContentSafety for MockSafety {
        async fn can_post_tweet(&self) -> bool {
            self.can_tweet
        }
        async fn can_post_thread(&self) -> bool {
            self.can_thread
        }
    }

    struct MockStorage {
        last_thread: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
        threads: Mutex<Vec<(String, usize)>>,
        thread_statuses: Mutex<Vec<(String, String, usize)>>,
        thread_tweets: Mutex<Vec<(String, usize, String, String)>>,
        actions: Mutex<Vec<(String, String, String)>>,
    }

    impl MockStorage {
        fn new(last_thread: Option<chrono::DateTime<chrono::Utc>>) -> Self {
            Self {
                last_thread: Mutex::new(last_thread),
                threads: Mutex::new(Vec::new()),
                thread_statuses: Mutex::new(Vec::new()),
                thread_tweets: Mutex::new(Vec::new()),
                actions: Mutex::new(Vec::new()),
            }
        }

        fn thread_tweet_count(&self) -> usize {
            self.thread_tweets.lock().expect("lock").len()
        }

        fn action_statuses(&self) -> Vec<String> {
            self.actions
                .lock()
                .expect("lock")
                .iter()
                .map(|(_, s, _)| s.clone())
                .collect()
        }
    }

    #[async_trait::async_trait]
    impl ContentStorage for MockStorage {
        async fn last_tweet_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }

        async fn last_thread_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(*self.last_thread.lock().expect("lock"))
        }

        async fn todays_tweet_times(
            &self,
        ) -> Result<Vec<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(Vec::new())
        }

        async fn post_tweet(&self, _topic: &str, _content: &str) -> Result<(), ContentLoopError> {
            Ok(())
        }

        async fn create_thread(
            &self,
            topic: &str,
            tweet_count: usize,
        ) -> Result<String, ContentLoopError> {
            let id = format!("thread-{}", self.threads.lock().expect("lock").len() + 1);
            self.threads
                .lock()
                .expect("lock")
                .push((topic.to_string(), tweet_count));
            Ok(id)
        }

        async fn update_thread_status(
            &self,
            thread_id: &str,
            status: &str,
            tweet_count: usize,
            _root_tweet_id: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            self.thread_statuses.lock().expect("lock").push((
                thread_id.to_string(),
                status.to_string(),
                tweet_count,
            ));
            Ok(())
        }

        async fn store_thread_tweet(
            &self,
            thread_id: &str,
            position: usize,
            tweet_id: &str,
            content: &str,
        ) -> Result<(), ContentLoopError> {
            self.thread_tweets.lock().expect("lock").push((
                thread_id.to_string(),
                position,
                tweet_id.to_string(),
                content.to_string(),
            ));
            Ok(())
        }

        async fn log_action(
            &self,
            action_type: &str,
            status: &str,
            message: &str,
        ) -> Result<(), ContentLoopError> {
            self.actions.lock().expect("lock").push((
                action_type.to_string(),
                status.to_string(),
                message.to_string(),
            ));
            Ok(())
        }
    }

    struct MockPoster {
        posted: Mutex<Vec<(Option<String>, String)>>,
        fail_at_index: Option<usize>,
    }

    impl MockPoster {
        fn new() -> Self {
            Self {
                posted: Mutex::new(Vec::new()),
                fail_at_index: None,
            }
        }

        fn failing_at(index: usize) -> Self {
            Self {
                posted: Mutex::new(Vec::new()),
                fail_at_index: Some(index),
            }
        }

        fn posted_count(&self) -> usize {
            self.posted.lock().expect("lock").len()
        }
    }

    #[async_trait::async_trait]
    impl ThreadPoster for MockPoster {
        async fn post_tweet(&self, content: &str) -> Result<String, ContentLoopError> {
            let mut posted = self.posted.lock().expect("lock");
            if self.fail_at_index == Some(posted.len()) {
                return Err(ContentLoopError::PostFailed("API error".to_string()));
            }
            let id = format!("tweet-{}", posted.len() + 1);
            posted.push((None, content.to_string()));
            Ok(id)
        }

        async fn reply_to_tweet(
            &self,
            in_reply_to: &str,
            content: &str,
        ) -> Result<String, ContentLoopError> {
            let mut posted = self.posted.lock().expect("lock");
            if self.fail_at_index == Some(posted.len()) {
                return Err(ContentLoopError::PostFailed("API error".to_string()));
            }
            let id = format!("tweet-{}", posted.len() + 1);
            posted.push((Some(in_reply_to.to_string()), content.to_string()));
            Ok(id)
        }
    }

    fn make_topics() -> Vec<String> {
        vec![
            "Rust".to_string(),
            "CLI tools".to_string(),
            "Open source".to_string(),
        ]
    }

    fn make_thread_tweets() -> Vec<String> {
        vec![
            "Thread on Rust: Let me share what I've learned...".to_string(),
            "First, the ownership model is game-changing.".to_string(),
            "Second, pattern matching makes error handling elegant.".to_string(),
            "Third, the compiler is your best friend.".to_string(),
            "Finally, the community is incredibly welcoming.".to_string(),
        ]
    }

    // --- Tests ---

    #[tokio::test]
    async fn run_once_posts_thread() {
        let storage = Arc::new(MockStorage::new(None));
        let poster = Arc::new(MockPoster::new());

        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            poster.clone(),
            make_topics(),
            604800,
            false,
        );

        let result = thread_loop.run_once(Some("Rust"), None).await;

        assert!(matches!(
            result,
            ThreadResult::Posted { tweet_count: 5, .. }
        ));
        assert_eq!(poster.posted_count(), 5);
        assert_eq!(storage.thread_tweet_count(), 5);
    }

    #[tokio::test]
    async fn run_once_dry_run_does_not_post() {
        let storage = Arc::new(MockStorage::new(None));
        let poster = Arc::new(MockPoster::new());

        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            poster.clone(),
            make_topics(),
            604800,
            true, // dry_run
        );

        let result = thread_loop.run_once(Some("Rust"), None).await;

        assert!(matches!(result, ThreadResult::Posted { .. }));
        assert_eq!(poster.posted_count(), 0); // Not actually posted
        assert_eq!(storage.action_statuses(), vec!["dry_run"]);
    }

    #[tokio::test]
    async fn run_once_rate_limited() {
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: false,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            make_topics(),
            604800,
            false,
        );

        let result = thread_loop.run_once(None, None).await;
        assert!(matches!(result, ThreadResult::RateLimited));
    }

    #[tokio::test]
    async fn run_once_no_topics() {
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            Vec::new(), // No topics
            604800,
            false,
        );

        let result = thread_loop.run_once(None, None).await;
        assert!(matches!(result, ThreadResult::NoTopics));
    }

    #[tokio::test]
    async fn run_once_generation_failure() {
        let thread_loop = ThreadLoop::new(
            Arc::new(FailingThreadGenerator),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            make_topics(),
            604800,
            false,
        );

        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(matches!(result, ThreadResult::Failed { .. }));
    }

    #[tokio::test]
    async fn run_once_validation_failure() {
        let thread_loop = ThreadLoop::new(
            Arc::new(OverlongThreadGenerator),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            make_topics(),
            604800,
            false,
        );

        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(matches!(result, ThreadResult::ValidationFailed { .. }));
    }

    #[tokio::test]
    async fn partial_failure_records_correctly() {
        let storage = Arc::new(MockStorage::new(None));
        // Fail at index 2 (3rd tweet)
        let poster = Arc::new(MockPoster::failing_at(2));

        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            poster.clone(),
            make_topics(),
            604800,
            false,
        );

        let result = thread_loop.run_once(Some("Rust"), None).await;

        match result {
            ThreadResult::PartialFailure {
                tweets_posted,
                total_tweets,
                ..
            } => {
                assert_eq!(tweets_posted, 2); // 0 and 1 succeeded
                assert_eq!(total_tweets, 5);
            }
            other => panic!("Expected PartialFailure, got {other:?}"),
        }

        // Only 2 tweets should be recorded
        assert_eq!(storage.thread_tweet_count(), 2);
        assert_eq!(poster.posted_count(), 2);
    }

    #[tokio::test]
    async fn run_once_clamps_count() {
        let poster = Arc::new(MockPoster::new());
        let storage = Arc::new(MockStorage::new(None));

        // Generator returns whatever count of tweets
        let tweets = vec![
            "Tweet 1".to_string(),
            "Tweet 2".to_string(),
            "Tweet 3".to_string(),
        ];

        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator { tweets }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            poster.clone(),
            make_topics(),
            604800,
            false,
        );

        // count=1 should be clamped to 2
        let result = thread_loop.run_once(Some("Rust"), Some(1)).await;
        // The clamped count is passed to generator but our mock ignores it
        assert!(matches!(result, ThreadResult::Posted { .. }));
    }

    #[tokio::test]
    async fn run_iteration_skips_when_too_soon() {
        let now = chrono::Utc::now();
        let last_thread = now - chrono::Duration::days(3);
        let storage = Arc::new(MockStorage::new(Some(last_thread)));

        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            Arc::new(MockPoster::new()),
            make_topics(),
            604800, // 7 days
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = thread_loop.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ThreadResult::TooSoon { .. }));
    }

    #[tokio::test]
    async fn run_iteration_posts_when_interval_elapsed() {
        let now = chrono::Utc::now();
        let last_thread = now - chrono::Duration::days(8);
        let storage = Arc::new(MockStorage::new(Some(last_thread)));
        let poster = Arc::new(MockPoster::new());

        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            poster.clone(),
            make_topics(),
            604800, // 7 days
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = thread_loop.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ThreadResult::Posted { .. }));
        assert_eq!(poster.posted_count(), 5);
        assert_eq!(recent.len(), 1); // Topic tracked
    }

    #[tokio::test]
    async fn reply_chain_structure_correct() {
        let poster = Arc::new(MockPoster::new());
        let storage = Arc::new(MockStorage::new(None));

        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: vec![
                    "First".to_string(),
                    "Second".to_string(),
                    "Third".to_string(),
                ],
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            poster.clone(),
            make_topics(),
            604800,
            false,
        );

        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(matches!(
            result,
            ThreadResult::Posted { tweet_count: 3, .. }
        ));

        let posted = poster.posted.lock().expect("lock");
        // First tweet: no reply_to
        assert_eq!(posted[0].0, None);
        // Second tweet: replies to first
        assert_eq!(posted[1].0, Some("tweet-1".to_string()));
        // Third tweet: replies to second
        assert_eq!(posted[2].0, Some("tweet-2".to_string()));
    }
}
