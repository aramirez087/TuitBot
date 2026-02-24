//! Content loop for posting original educational tweets.
//!
//! Generates and posts original educational tweets on a configurable
//! schedule, keeping the user's X account active with thought-leadership
//! content. Rotates through configured topics to avoid repetition.

use super::loop_helpers::{ContentSafety, ContentStorage, TopicScorer, TweetGenerator};
use super::schedule::{apply_slot_jitter, schedule_gate, ActiveSchedule};
use super::scheduler::LoopScheduler;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Fraction of the time to exploit top-performing topics (vs. explore random ones).
const EXPLOIT_RATIO: f64 = 0.8;

/// Content loop that generates and posts original educational tweets.
pub struct ContentLoop {
    generator: Arc<dyn TweetGenerator>,
    safety: Arc<dyn ContentSafety>,
    storage: Arc<dyn ContentStorage>,
    topic_scorer: Option<Arc<dyn TopicScorer>>,
    topics: Vec<String>,
    post_window_secs: u64,
    dry_run: bool,
}

/// Result of a content generation attempt.
#[derive(Debug)]
pub enum ContentResult {
    /// Tweet was posted (or would be in dry-run).
    Posted { topic: String, content: String },
    /// Skipped because not enough time has elapsed since last tweet.
    TooSoon { elapsed_secs: u64, window_secs: u64 },
    /// Skipped due to daily tweet rate limit.
    RateLimited,
    /// No topics configured.
    NoTopics,
    /// Generation failed.
    Failed { error: String },
}

impl ContentLoop {
    /// Create a new content loop.
    pub fn new(
        generator: Arc<dyn TweetGenerator>,
        safety: Arc<dyn ContentSafety>,
        storage: Arc<dyn ContentStorage>,
        topics: Vec<String>,
        post_window_secs: u64,
        dry_run: bool,
    ) -> Self {
        Self {
            generator,
            safety,
            storage,
            topic_scorer: None,
            topics,
            post_window_secs,
            dry_run,
        }
    }

    /// Set a topic scorer for epsilon-greedy topic selection.
    ///
    /// When set, 80% of the time the loop picks from top-performing topics
    /// (exploit), and 20% of the time it picks a random topic (explore).
    pub fn with_topic_scorer(mut self, scorer: Arc<dyn TopicScorer>) -> Self {
        self.topic_scorer = Some(scorer);
        self
    }

    /// Run the continuous content loop until cancellation.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        scheduler: LoopScheduler,
        schedule: Option<Arc<ActiveSchedule>>,
    ) {
        let slot_mode = schedule.as_ref().is_some_and(|s| s.has_preferred_times());

        tracing::info!(
            dry_run = self.dry_run,
            topics = self.topics.len(),
            window_secs = self.post_window_secs,
            slot_mode = slot_mode,
            "Content loop started"
        );

        if self.topics.is_empty() {
            tracing::warn!("No topics configured, content loop has nothing to post");
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
                // Slot-based scheduling: post at preferred times
                let sched = schedule.as_ref().expect("slot_mode requires schedule");

                // Query today's post times from storage
                let today_posts = match self.storage.todays_tweet_times().await {
                    Ok(times) => times,
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to query today's tweet times");
                        Vec::new()
                    }
                };

                match sched.next_unused_slot(&today_posts) {
                    Some((wait, slot)) => {
                        let jittered_wait = apply_slot_jitter(wait);
                        tracing::info!(
                            slot = %slot.format(),
                            wait_secs = jittered_wait.as_secs(),
                            "Slot mode: sleeping until next posting slot"
                        );

                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(jittered_wait) => {},
                        }

                        if cancel.is_cancelled() {
                            break;
                        }

                        // In slot mode, skip the elapsed-time check — post directly
                        let result = self
                            .run_slot_iteration(&mut recent_topics, max_recent, &mut rng)
                            .await;
                        self.log_content_result(&result);
                    }
                    None => {
                        // All slots used today — sleep until next active day
                        tracing::info!(
                            "Slot mode: all slots used today, sleeping until next active period"
                        );
                        if let Some(sched) = &schedule {
                            let wait = sched.time_until_active();
                            if wait.is_zero() {
                                // Currently active but all slots used — sleep 1 hour and recheck
                                tokio::select! {
                                    _ = cancel.cancelled() => break,
                                    _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {},
                                }
                            } else {
                                tokio::select! {
                                    _ = cancel.cancelled() => break,
                                    _ = tokio::time::sleep(wait) => {},
                                }
                            }
                        } else {
                            tokio::select! {
                                _ = cancel.cancelled() => break,
                                _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {},
                            }
                        }
                    }
                }
            } else {
                // Interval-based scheduling (existing behavior)
                let result = self
                    .run_iteration(&mut recent_topics, max_recent, &mut rng)
                    .await;
                self.log_content_result(&result);

                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = scheduler.tick() => {},
                }
            }
        }

        tracing::info!("Content loop stopped");
    }

    /// Log the result of a content iteration.
    fn log_content_result(&self, result: &ContentResult) {
        match result {
            ContentResult::Posted { topic, content } => {
                tracing::info!(
                    topic = %topic,
                    chars = content.len(),
                    dry_run = self.dry_run,
                    "Content iteration: tweet posted"
                );
            }
            ContentResult::TooSoon {
                elapsed_secs,
                window_secs,
            } => {
                tracing::debug!(
                    elapsed = elapsed_secs,
                    window = window_secs,
                    "Content iteration: too soon since last tweet"
                );
            }
            ContentResult::RateLimited => {
                tracing::info!("Content iteration: daily tweet limit reached");
            }
            ContentResult::NoTopics => {
                tracing::warn!("Content iteration: no topics available");
            }
            ContentResult::Failed { error } => {
                tracing::warn!(error = %error, "Content iteration: failed");
            }
        }
    }

    /// Run a single iteration in slot mode (skips elapsed-time check).
    async fn run_slot_iteration(
        &self,
        recent_topics: &mut Vec<String>,
        max_recent: usize,
        rng: &mut impl rand::Rng,
    ) -> ContentResult {
        // Check for manually scheduled content due for posting
        if let Some(result) = self.try_post_scheduled().await {
            return result;
        }

        // Check safety (daily tweet limit)
        if !self.safety.can_post_tweet().await {
            return ContentResult::RateLimited;
        }

        // Pick a topic using epsilon-greedy if scorer is available
        let topic = self.pick_topic_epsilon_greedy(recent_topics, rng).await;

        let result = self.generate_and_post(&topic).await;

        // Update recent_topics on success
        if matches!(result, ContentResult::Posted { .. }) {
            if recent_topics.len() >= max_recent {
                recent_topics.remove(0);
            }
            recent_topics.push(topic);
        }

        result
    }

    /// Run a single content generation (for CLI `tuitbot post` command).
    ///
    /// If `topic` is provided, uses that topic. Otherwise picks a random
    /// topic from the configured list.
    pub async fn run_once(&self, topic: Option<&str>) -> ContentResult {
        let chosen_topic = match topic {
            Some(t) => t.to_string(),
            None => {
                if self.topics.is_empty() {
                    return ContentResult::NoTopics;
                }
                let mut rng = rand::thread_rng();
                self.topics
                    .choose(&mut rng)
                    .expect("topics is non-empty")
                    .clone()
            }
        };

        // Skip window check for single-shot mode, but still check safety
        if !self.safety.can_post_tweet().await {
            return ContentResult::RateLimited;
        }

        self.generate_and_post(&chosen_topic).await
    }

    /// Run a single iteration of the continuous loop.
    async fn run_iteration(
        &self,
        recent_topics: &mut Vec<String>,
        max_recent: usize,
        rng: &mut impl rand::Rng,
    ) -> ContentResult {
        // Check for manually scheduled content due for posting
        if let Some(result) = self.try_post_scheduled().await {
            return result;
        }

        // Check elapsed time since last tweet
        match self.storage.last_tweet_time().await {
            Ok(Some(last_time)) => {
                let elapsed = chrono::Utc::now()
                    .signed_duration_since(last_time)
                    .num_seconds()
                    .max(0) as u64;

                if elapsed < self.post_window_secs {
                    return ContentResult::TooSoon {
                        elapsed_secs: elapsed,
                        window_secs: self.post_window_secs,
                    };
                }
            }
            Ok(None) => {
                // No previous tweets -- proceed
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query last tweet time, proceeding anyway");
            }
        }

        // Check safety (daily tweet limit)
        if !self.safety.can_post_tweet().await {
            return ContentResult::RateLimited;
        }

        // Pick a topic using epsilon-greedy if scorer is available
        let topic = self.pick_topic_epsilon_greedy(recent_topics, rng).await;

        let result = self.generate_and_post(&topic).await;

        // Update recent_topics on success
        if matches!(result, ContentResult::Posted { .. }) {
            if recent_topics.len() >= max_recent {
                recent_topics.remove(0);
            }
            recent_topics.push(topic);
        }

        result
    }

    /// Pick a topic using epsilon-greedy selection.
    ///
    /// If a topic scorer is available:
    /// - 80% of the time: pick from top-performing topics (exploit)
    /// - 20% of the time: pick a random topic (explore)
    ///
    /// Falls back to uniform random selection if no scorer is set or
    /// if the scorer returns no data.
    async fn pick_topic_epsilon_greedy(
        &self,
        recent_topics: &mut Vec<String>,
        rng: &mut impl rand::Rng,
    ) -> String {
        if let Some(scorer) = &self.topic_scorer {
            let roll: f64 = rng.gen();
            if roll < EXPLOIT_RATIO {
                // Exploit: try to pick from top-performing topics
                if let Ok(top_topics) = scorer.get_top_topics(10).await {
                    // Filter to topics that are in our configured list and not recent
                    let candidates: Vec<&String> = top_topics
                        .iter()
                        .filter(|t| self.topics.contains(t) && !recent_topics.contains(t))
                        .collect();

                    if !candidates.is_empty() {
                        let topic = candidates[0].clone();
                        tracing::debug!(topic = %topic, "Epsilon-greedy: exploiting top topic");
                        return topic;
                    }
                }
                // Fall through to random if no top topics match
                tracing::debug!("Epsilon-greedy: no top topics available, falling back to random");
            } else {
                tracing::debug!("Epsilon-greedy: exploring random topic");
            }
        }

        pick_topic(&self.topics, recent_topics, rng)
    }

    /// Check for scheduled content due for posting and post it if found.
    ///
    /// Returns `Some(ContentResult)` if a scheduled item was handled,
    /// `None` if no scheduled items are due.
    async fn try_post_scheduled(&self) -> Option<ContentResult> {
        match self.storage.next_scheduled_item().await {
            Ok(Some((id, content_type, content))) => {
                tracing::info!(
                    id = id,
                    content_type = %content_type,
                    "Posting scheduled content"
                );

                if self.dry_run {
                    tracing::info!(
                        "DRY RUN: Would post scheduled {} (id={}): \"{}\"",
                        content_type,
                        id,
                        &content[..content.len().min(80)]
                    );
                    let _ = self
                        .storage
                        .log_action(
                            &content_type,
                            "dry_run",
                            &format!("Scheduled id={id}: {}", &content[..content.len().min(80)]),
                        )
                        .await;
                } else if let Err(e) = self.storage.post_tweet("scheduled", &content).await {
                    tracing::error!(error = %e, "Failed to post scheduled content");
                    return Some(ContentResult::Failed {
                        error: format!("Scheduled post failed: {e}"),
                    });
                } else {
                    let _ = self.storage.mark_scheduled_posted(id, None).await;
                    let _ = self
                        .storage
                        .log_action(
                            &content_type,
                            "success",
                            &format!("Scheduled id={id}: {}", &content[..content.len().min(80)]),
                        )
                        .await;
                }

                Some(ContentResult::Posted {
                    topic: format!("scheduled:{id}"),
                    content,
                })
            }
            Ok(None) => None,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to check scheduled content");
                None
            }
        }
    }

    /// Generate a tweet and post it (or print in dry-run mode).
    async fn generate_and_post(&self, topic: &str) -> ContentResult {
        tracing::info!(topic = %topic, "Generating tweet on topic");

        // Generate tweet
        let content = match self.generator.generate_tweet(topic).await {
            Ok(text) => text,
            Err(e) => {
                return ContentResult::Failed {
                    error: format!("Generation failed: {e}"),
                }
            }
        };

        // Validate length (280 char limit)
        let content = if content.len() > 280 {
            // Retry once with explicit shorter instruction
            tracing::debug!(
                chars = content.len(),
                "Generated tweet too long, retrying with shorter instruction"
            );

            let shorter_topic = format!("{topic} (IMPORTANT: keep under 280 characters)");
            match self.generator.generate_tweet(&shorter_topic).await {
                Ok(text) if text.len() <= 280 => text,
                Ok(text) => {
                    // Truncate at word boundary
                    tracing::warn!(
                        chars = text.len(),
                        "Retry still too long, truncating at word boundary"
                    );
                    truncate_at_word_boundary(&text, 280)
                }
                Err(e) => {
                    // Use original but truncated
                    tracing::warn!(error = %e, "Retry generation failed, truncating original");
                    truncate_at_word_boundary(&content, 280)
                }
            }
        } else {
            content
        };

        if self.dry_run {
            tracing::info!(
                "DRY RUN: Would post tweet on topic '{}': \"{}\" ({} chars)",
                topic,
                content,
                content.len()
            );

            let _ = self
                .storage
                .log_action(
                    "tweet",
                    "dry_run",
                    &format!("Topic '{}': {}", topic, truncate_display(&content, 80)),
                )
                .await;
        } else {
            if let Err(e) = self.storage.post_tweet(topic, &content).await {
                tracing::error!(error = %e, "Failed to post tweet");
                let _ = self
                    .storage
                    .log_action("tweet", "failure", &format!("Post failed: {e}"))
                    .await;
                return ContentResult::Failed {
                    error: e.to_string(),
                };
            }

            let _ = self
                .storage
                .log_action(
                    "tweet",
                    "success",
                    &format!("Topic '{}': {}", topic, truncate_display(&content, 80)),
                )
                .await;
        }

        ContentResult::Posted {
            topic: topic.to_string(),
            content,
        }
    }
}

/// Pick a topic that is not in the recent list.
/// If all topics are recent, clear the list and pick any.
fn pick_topic(topics: &[String], recent: &mut Vec<String>, rng: &mut impl rand::Rng) -> String {
    let available: Vec<&String> = topics.iter().filter(|t| !recent.contains(t)).collect();

    if available.is_empty() {
        // All topics recently used -- clear and pick any
        recent.clear();
        topics.choose(rng).expect("topics is non-empty").clone()
    } else {
        available
            .choose(rng)
            .expect("available is non-empty")
            .to_string()
    }
}

/// Truncate content at a word boundary, fitting within max_len characters.
fn truncate_at_word_boundary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    // Find last space before max_len - 3 (for "...")
    let cutoff = max_len.saturating_sub(3);
    match s[..cutoff].rfind(' ') {
        Some(pos) => format!("{}...", &s[..pos]),
        None => format!("{}...", &s[..cutoff]),
    }
}

/// Truncate a string for display purposes.
fn truncate_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::ContentLoopError;
    use std::sync::Mutex;

    // --- Mock implementations ---

    struct MockGenerator {
        response: String,
    }

    #[async_trait::async_trait]
    impl TweetGenerator for MockGenerator {
        async fn generate_tweet(&self, _topic: &str) -> Result<String, ContentLoopError> {
            Ok(self.response.clone())
        }
    }

    struct OverlongGenerator {
        first_response: String,
        retry_response: String,
        call_count: Mutex<usize>,
    }

    #[async_trait::async_trait]
    impl TweetGenerator for OverlongGenerator {
        async fn generate_tweet(&self, _topic: &str) -> Result<String, ContentLoopError> {
            let mut count = self.call_count.lock().expect("lock");
            *count += 1;
            if *count == 1 {
                Ok(self.first_response.clone())
            } else {
                Ok(self.retry_response.clone())
            }
        }
    }

    struct FailingGenerator;

    #[async_trait::async_trait]
    impl TweetGenerator for FailingGenerator {
        async fn generate_tweet(&self, _topic: &str) -> Result<String, ContentLoopError> {
            Err(ContentLoopError::LlmFailure(
                "model unavailable".to_string(),
            ))
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
        last_tweet: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
        posted_tweets: Mutex<Vec<(String, String)>>,
        actions: Mutex<Vec<(String, String, String)>>,
    }

    impl MockStorage {
        fn new(last_tweet: Option<chrono::DateTime<chrono::Utc>>) -> Self {
            Self {
                last_tweet: Mutex::new(last_tweet),
                posted_tweets: Mutex::new(Vec::new()),
                actions: Mutex::new(Vec::new()),
            }
        }

        fn posted_count(&self) -> usize {
            self.posted_tweets.lock().expect("lock").len()
        }

        fn action_count(&self) -> usize {
            self.actions.lock().expect("lock").len()
        }
    }

    #[async_trait::async_trait]
    impl ContentStorage for MockStorage {
        async fn last_tweet_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(*self.last_tweet.lock().expect("lock"))
        }

        async fn last_thread_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }

        async fn todays_tweet_times(
            &self,
        ) -> Result<Vec<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(Vec::new())
        }

        async fn post_tweet(&self, topic: &str, content: &str) -> Result<(), ContentLoopError> {
            self.posted_tweets
                .lock()
                .expect("lock")
                .push((topic.to_string(), content.to_string()));
            Ok(())
        }

        async fn create_thread(
            &self,
            _topic: &str,
            _tweet_count: usize,
        ) -> Result<String, ContentLoopError> {
            Ok("thread-1".to_string())
        }

        async fn update_thread_status(
            &self,
            _thread_id: &str,
            _status: &str,
            _tweet_count: usize,
            _root_tweet_id: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }

        async fn store_thread_tweet(
            &self,
            _thread_id: &str,
            _position: usize,
            _tweet_id: &str,
            _content: &str,
        ) -> Result<(), ContentLoopError> {
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

    fn make_topics() -> Vec<String> {
        vec![
            "Rust".to_string(),
            "CLI tools".to_string(),
            "Open source".to_string(),
            "Developer productivity".to_string(),
        ]
    }

    // --- Tests ---

    #[tokio::test]
    async fn run_once_posts_tweet() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Great tweet about Rust!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1);
    }

    #[tokio::test]
    async fn run_once_dry_run_does_not_post() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Great tweet about Rust!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            true,
        );

        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 0); // Not posted in dry-run
        assert_eq!(storage.action_count(), 1); // Action logged
    }

    #[tokio::test]
    async fn run_once_rate_limited() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: false,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(None).await;
        assert!(matches!(result, ContentResult::RateLimited));
    }

    #[tokio::test]
    async fn run_once_no_topics_returns_no_topics() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Vec::new(),
            14400,
            false,
        );

        let result = content.run_once(None).await;
        assert!(matches!(result, ContentResult::NoTopics));
    }

    #[tokio::test]
    async fn run_once_generation_failure() {
        let content = ContentLoop::new(
            Arc::new(FailingGenerator),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Failed { .. }));
    }

    #[tokio::test]
    async fn run_iteration_skips_when_too_soon() {
        let now = chrono::Utc::now();
        // Last tweet was 1 hour ago, window is 4 hours
        let last_tweet = now - chrono::Duration::hours(1);
        let storage = Arc::new(MockStorage::new(Some(last_tweet)));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400, // 4 hours
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::TooSoon { .. }));
    }

    #[tokio::test]
    async fn run_iteration_posts_when_window_elapsed() {
        let now = chrono::Utc::now();
        // Last tweet was 5 hours ago, window is 4 hours
        let last_tweet = now - chrono::Duration::hours(5);
        let storage = Arc::new(MockStorage::new(Some(last_tweet)));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Fresh tweet!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1);
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn overlong_tweet_gets_truncated() {
        let long_text = "a ".repeat(200); // 400 chars
        let content = ContentLoop::new(
            Arc::new(OverlongGenerator {
                first_response: long_text.clone(),
                retry_response: long_text, // retry also too long
                call_count: Mutex::new(0),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            14400,
            true,
        );

        let result = content.run_once(Some("Rust")).await;
        if let ContentResult::Posted { content, .. } = result {
            assert!(content.len() <= 280);
        } else {
            panic!("Expected Posted result");
        }
    }

    #[test]
    fn truncate_at_word_boundary_short() {
        let result = truncate_at_word_boundary("Hello world", 280);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn truncate_at_word_boundary_long() {
        let text = "The quick brown fox jumps over the lazy dog and more words here";
        let result = truncate_at_word_boundary(text, 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn pick_topic_avoids_recent() {
        let topics = make_topics();
        let mut recent = vec!["Rust".to_string(), "CLI tools".to_string()];
        let mut rng = rand::thread_rng();

        for _ in 0..20 {
            let topic = pick_topic(&topics, &mut recent, &mut rng);
            // Should not pick Rust or CLI tools
            assert_ne!(topic, "Rust");
            assert_ne!(topic, "CLI tools");
        }
    }

    #[test]
    fn pick_topic_clears_when_all_recent() {
        let topics = make_topics();
        let mut recent = topics.clone();
        let mut rng = rand::thread_rng();

        // Should clear recent and pick any topic
        let topic = pick_topic(&topics, &mut recent, &mut rng);
        assert!(topics.contains(&topic));
        assert!(recent.is_empty()); // Cleared
    }

    #[test]
    fn truncate_display_short() {
        assert_eq!(truncate_display("hello", 10), "hello");
    }

    #[test]
    fn truncate_display_long() {
        let result = truncate_display("hello world this is long", 10);
        assert_eq!(result, "hello worl...");
    }

    // --- Epsilon-greedy tests ---

    struct MockTopicScorer {
        top_topics: Vec<String>,
    }

    #[async_trait::async_trait]
    impl TopicScorer for MockTopicScorer {
        async fn get_top_topics(&self, _limit: u32) -> Result<Vec<String>, ContentLoopError> {
            Ok(self.top_topics.clone())
        }
    }

    struct FailingTopicScorer;

    #[async_trait::async_trait]
    impl TopicScorer for FailingTopicScorer {
        async fn get_top_topics(&self, _limit: u32) -> Result<Vec<String>, ContentLoopError> {
            Err(ContentLoopError::StorageError("db error".to_string()))
        }
    }

    #[tokio::test]
    async fn epsilon_greedy_exploits_top_topic() {
        let storage = Arc::new(MockStorage::new(None));
        let scorer = Arc::new(MockTopicScorer {
            top_topics: vec!["Rust".to_string()],
        });

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        )
        .with_topic_scorer(scorer);

        let mut recent = Vec::new();
        // Low roll => exploit. Use thread_rng for the pick_topic fallback path.
        let mut rng = FirstCallRng::low_roll();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert_eq!(topic, "Rust");
    }

    #[tokio::test]
    async fn epsilon_greedy_explores_when_roll_high() {
        let storage = Arc::new(MockStorage::new(None));
        let scorer = Arc::new(MockTopicScorer {
            top_topics: vec!["Rust".to_string()],
        });

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        )
        .with_topic_scorer(scorer);

        let mut recent = Vec::new();
        // High roll => explore, delegates to pick_topic with real rng
        let mut rng = FirstCallRng::high_roll();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert!(make_topics().contains(&topic));
    }

    #[tokio::test]
    async fn epsilon_greedy_falls_back_on_scorer_error() {
        let storage = Arc::new(MockStorage::new(None));
        let scorer = Arc::new(FailingTopicScorer);

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        )
        .with_topic_scorer(scorer);

        let mut recent = Vec::new();
        // Low roll => exploit, but scorer fails => falls back to random
        let mut rng = FirstCallRng::low_roll();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert!(make_topics().contains(&topic));
    }

    #[tokio::test]
    async fn epsilon_greedy_without_scorer_picks_random() {
        let storage = Arc::new(MockStorage::new(None));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert!(make_topics().contains(&topic));
    }

    /// RNG wrapper that overrides only the first `next_u64()` call,
    /// then delegates everything to a real ThreadRng. This lets us
    /// control the initial `gen::<f64>()` roll without breaking
    /// subsequent `choose()` / `gen_range()` calls.
    struct FirstCallRng {
        first_u64: Option<u64>,
        inner: rand::rngs::ThreadRng,
    }

    impl FirstCallRng {
        /// Create an RNG whose first `gen::<f64>()` returns ~0.0 (exploit).
        fn low_roll() -> Self {
            Self {
                first_u64: Some(0),
                inner: rand::thread_rng(),
            }
        }

        /// Create an RNG whose first `gen::<f64>()` returns ~1.0 (explore).
        fn high_roll() -> Self {
            Self {
                first_u64: Some(u64::MAX),
                inner: rand::thread_rng(),
            }
        }
    }

    impl rand::RngCore for FirstCallRng {
        fn next_u32(&mut self) -> u32 {
            self.inner.next_u32()
        }
        fn next_u64(&mut self) -> u64 {
            if let Some(val) = self.first_u64.take() {
                val
            } else {
                self.inner.next_u64()
            }
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.inner.fill_bytes(dest);
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.inner.try_fill_bytes(dest)
        }
    }
}
