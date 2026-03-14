//! Content loop for posting original educational tweets.
//!
//! Generates and posts original educational tweets on a configurable
//! schedule, keeping the user's X account active with thought-leadership
//! content. Rotates through configured topics to avoid repetition.
//!
//! # Module layout
//!
//! | File            | Responsibility                                        |
//! |-----------------|-------------------------------------------------------|
//! | `mod.rs`        | Struct definition, constructors, shared test mocks   |
//! | `generator.rs`  | Tweet generation, topic selection, text utilities     |
//! | `scheduler.rs`  | Run loop, iteration logic, slot/interval scheduling   |
//! | `publisher.rs`  | Scheduled-content posting (single tweets & threads)   |

mod generator;
mod publisher;
mod scheduler;
#[cfg(test)]
mod tests_guardrails; // Task 3.5: safety guardrails + publisher tests

use super::loop_helpers::{
    ContentSafety, ContentStorage, ThreadPoster, TopicScorer, TweetGenerator,
};
use std::sync::Arc;

/// Fraction of the time to exploit top-performing topics (vs. explore random ones).
pub(super) const EXPLOIT_RATIO: f64 = 0.8;

/// Content loop that generates and posts original educational tweets.
pub struct ContentLoop {
    pub(super) generator: Arc<dyn TweetGenerator>,
    pub(super) safety: Arc<dyn ContentSafety>,
    pub(super) storage: Arc<dyn ContentStorage>,
    pub(super) topic_scorer: Option<Arc<dyn TopicScorer>>,
    pub(super) thread_poster: Option<Arc<dyn ThreadPoster>>,
    pub(super) topics: Vec<String>,
    pub(super) post_window_secs: u64,
    pub(super) dry_run: bool,
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
            thread_poster: None,
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

    /// Set a thread poster for posting scheduled threads as reply chains.
    pub fn with_thread_poster(mut self, poster: Arc<dyn ThreadPoster>) -> Self {
        self.thread_poster = Some(poster);
        self
    }
}

// ---------------------------------------------------------------------------
// Shared test mocks (accessible to all child test modules).
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(super) mod test_mocks {
    use crate::automation::loop_helpers::{
        ContentSafety, ContentStorage, ThreadPoster, TopicScorer, TweetGenerator,
    };
    use crate::automation::ContentLoopError;
    use std::sync::Mutex;

    // --- generators ---

    pub struct MockGenerator {
        pub response: String,
    }

    #[async_trait::async_trait]
    impl TweetGenerator for MockGenerator {
        async fn generate_tweet(&self, _topic: &str) -> Result<String, ContentLoopError> {
            Ok(self.response.clone())
        }
    }

    pub struct OverlongGenerator {
        pub first_response: String,
        pub retry_response: String,
        pub call_count: Mutex<usize>,
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

    pub struct FailingGenerator;

    #[async_trait::async_trait]
    impl TweetGenerator for FailingGenerator {
        async fn generate_tweet(&self, _topic: &str) -> Result<String, ContentLoopError> {
            Err(ContentLoopError::LlmFailure(
                "model unavailable".to_string(),
            ))
        }
    }

    // --- safety ---

    pub struct MockSafety {
        pub can_tweet: bool,
        pub can_thread: bool,
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

    // --- storage ---

    pub struct MockStorage {
        pub last_tweet: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
        pub posted_tweets: Mutex<Vec<(String, String)>>,
        pub actions: Mutex<Vec<(String, String, String)>>,
    }

    impl MockStorage {
        pub fn new(last_tweet: Option<chrono::DateTime<chrono::Utc>>) -> Self {
            Self {
                last_tweet: Mutex::new(last_tweet),
                posted_tweets: Mutex::new(Vec::new()),
                actions: Mutex::new(Vec::new()),
            }
        }

        pub fn posted_count(&self) -> usize {
            self.posted_tweets.lock().expect("lock").len()
        }

        pub fn action_count(&self) -> usize {
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

    // --- topic scorer ---

    pub struct MockTopicScorer {
        pub top_topics: Vec<String>,
    }

    #[async_trait::async_trait]
    impl TopicScorer for MockTopicScorer {
        async fn get_top_topics(&self, _limit: u32) -> Result<Vec<String>, ContentLoopError> {
            Ok(self.top_topics.clone())
        }
    }

    pub struct FailingTopicScorer;

    #[async_trait::async_trait]
    impl TopicScorer for FailingTopicScorer {
        async fn get_top_topics(&self, _limit: u32) -> Result<Vec<String>, ContentLoopError> {
            Err(ContentLoopError::StorageError("db error".to_string()))
        }
    }

    // --- RNG helper ---

    /// RNG wrapper that overrides only the first `next_u64()` call,
    /// then delegates everything to a real ThreadRng. This lets us
    /// control the initial `gen::<f64>()` roll without breaking
    /// subsequent `choose()` / `gen_range()` calls.
    pub struct FirstCallRng {
        pub first_u64: Option<u64>,
        pub inner: rand::rngs::ThreadRng,
    }

    impl FirstCallRng {
        /// Create an RNG whose first `gen::<f64>()` returns ~0.0 (exploit).
        pub fn low_roll() -> Self {
            Self {
                first_u64: Some(0),
                inner: rand::thread_rng(),
            }
        }

        /// Create an RNG whose first `gen::<f64>()` returns ~1.0 (explore).
        pub fn high_roll() -> Self {
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

    // --- fixtures ---

    pub fn make_topics() -> Vec<String> {
        vec![
            "Rust".to_string(),
            "CLI tools".to_string(),
            "Open source".to_string(),
            "Developer productivity".to_string(),
        ]
    }
}
