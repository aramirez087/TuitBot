//! Thread loop for posting multi-tweet educational threads.
//!
//! Generates and posts educational threads (5-8 tweets) as reply chains
//! on a configurable schedule. Threads bypass the posting queue since
//! reply chain order must be maintained (each tweet replies to the previous).
//!
//! # Module layout
//!
//! | File            | Responsibility                                         |
//! |-----------------|--------------------------------------------------------|
//! | `mod.rs`        | Public types, constructor, shared test mocks           |
//! | `planner.rs`    | Run loop, scheduling, iteration, topic selection       |
//! | `generator.rs`  | Content generation, validation, reply-chain posting    |

mod generator;
mod planner;

use super::loop_helpers::{ContentLoopError, ContentSafety, ContentStorage, ThreadPoster};
use std::sync::Arc;

/// Thread loop that generates and posts educational threads.
pub struct ThreadLoop {
    pub(super) generator: Arc<dyn ThreadGenerator>,
    pub(super) safety: Arc<dyn ContentSafety>,
    pub(super) storage: Arc<dyn ContentStorage>,
    pub(super) poster: Arc<dyn ThreadPoster>,
    pub(super) topics: Vec<String>,
    pub(super) thread_interval_secs: u64,
    pub(super) dry_run: bool,
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
}

/// Pick a topic that is not in the recent list.
/// If all topics are recent, clear the list and pick any.
pub(super) fn pick_topic(
    topics: &[String],
    recent: &mut Vec<String>,
    rng: &mut impl rand::Rng,
) -> String {
    use rand::seq::SliceRandom;
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

// ---------------------------------------------------------------------------
// Shared test mocks (accessible to all child test modules via super::test_mocks)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(super) mod test_mocks {
    use super::ThreadGenerator;
    use crate::automation::loop_helpers::{
        ContentLoopError, ContentSafety, ContentStorage, ThreadPoster,
    };
    use std::sync::Mutex;

    // --- thread generators ---

    pub struct MockThreadGenerator {
        pub tweets: Vec<String>,
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

    pub struct OverlongThreadGenerator;

    #[async_trait::async_trait]
    impl ThreadGenerator for OverlongThreadGenerator {
        async fn generate_thread(
            &self,
            _topic: &str,
            _count: Option<usize>,
        ) -> Result<Vec<String>, ContentLoopError> {
            Ok(vec!["a".repeat(300), "b".repeat(300)])
        }
    }

    pub struct FailingThreadGenerator;

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
        pub last_thread: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
        pub threads: Mutex<Vec<(String, usize)>>,
        pub thread_statuses: Mutex<Vec<(String, String, usize)>>,
        pub thread_tweets: Mutex<Vec<(String, usize, String, String)>>,
        pub actions: Mutex<Vec<(String, String, String)>>,
    }

    impl MockStorage {
        pub fn new(last_thread: Option<chrono::DateTime<chrono::Utc>>) -> Self {
            Self {
                last_thread: Mutex::new(last_thread),
                threads: Mutex::new(Vec::new()),
                thread_statuses: Mutex::new(Vec::new()),
                thread_tweets: Mutex::new(Vec::new()),
                actions: Mutex::new(Vec::new()),
            }
        }

        pub fn thread_tweet_count(&self) -> usize {
            self.thread_tweets.lock().expect("lock").len()
        }

        pub fn action_statuses(&self) -> Vec<String> {
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

    // --- poster ---

    pub struct MockPoster {
        pub posted: Mutex<Vec<(Option<String>, String)>>,
        pub fail_at_index: Option<usize>,
    }

    impl MockPoster {
        pub fn new() -> Self {
            Self {
                posted: Mutex::new(Vec::new()),
                fail_at_index: None,
            }
        }

        pub fn failing_at(index: usize) -> Self {
            Self {
                posted: Mutex::new(Vec::new()),
                fail_at_index: Some(index),
            }
        }

        pub fn posted_count(&self) -> usize {
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

    // --- fixtures ---

    pub fn make_topics() -> Vec<String> {
        vec![
            "Rust".to_string(),
            "CLI tools".to_string(),
            "Open source".to_string(),
        ]
    }

    pub fn make_thread_tweets() -> Vec<String> {
        vec![
            "Thread on Rust: Let me share what I've learned...".to_string(),
            "First, the ownership model is game-changing.".to_string(),
            "Second, pattern matching makes error handling elegant.".to_string(),
            "Third, the compiler is your best friend.".to_string(),
            "Finally, the community is incredibly welcoming.".to_string(),
        ]
    }
}
