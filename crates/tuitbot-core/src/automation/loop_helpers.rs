//! Shared types, traits, and helpers for automation loops.
//!
//! Defines port traits that decouple loop logic from concrete
//! implementations (X API, LLM, storage, safety). When all work
//! packages are merged, adapter implementations bridge these traits
//! to the actual types.

use std::fmt;
use std::time::Duration;

// ============================================================================
// WP08 types: Mentions + Discovery loops
// ============================================================================

/// Tweet data as seen by automation loop logic.
///
/// A common representation used by both mentions and discovery loops,
/// decoupled from any specific API response type.
#[derive(Debug, Clone)]
pub struct LoopTweet {
    /// Unique tweet ID.
    pub id: String,
    /// Tweet text content.
    pub text: String,
    /// Author's user ID.
    pub author_id: String,
    /// Author's username (without @).
    pub author_username: String,
    /// Author's follower count.
    pub author_followers: u64,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
    /// Number of likes.
    pub likes: u64,
    /// Number of retweets.
    pub retweets: u64,
    /// Number of replies.
    pub replies: u64,
}

/// Result of scoring a tweet for reply-worthiness.
#[derive(Debug, Clone)]
pub struct ScoreResult {
    /// Total score (0-100).
    pub total: f32,
    /// Whether the score meets the configured threshold.
    pub meets_threshold: bool,
    /// Keywords that matched in the tweet.
    pub matched_keywords: Vec<String>,
}

/// Errors that can occur in mentions/discovery automation loops.
///
/// Wraps specific error categories to enable appropriate handling
/// (e.g., back off on rate limit, skip on LLM failure, refresh on auth expiry).
#[derive(Debug)]
pub enum LoopError {
    /// X API rate limit hit.
    RateLimited {
        /// Seconds to wait before retrying, if known.
        retry_after: Option<u64>,
    },
    /// OAuth token expired and needs refresh.
    AuthExpired,
    /// LLM content generation failed.
    LlmFailure(String),
    /// Network-level error.
    NetworkError(String),
    /// Database/storage error.
    StorageError(String),
    /// Any other error.
    Other(String),
}

impl fmt::Display for LoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoopError::RateLimited { retry_after } => match retry_after {
                Some(secs) => write!(f, "rate limited, retry after {secs}s"),
                None => write!(f, "rate limited"),
            },
            LoopError::AuthExpired => write!(f, "authentication expired"),
            LoopError::LlmFailure(msg) => write!(f, "LLM failure: {msg}"),
            LoopError::NetworkError(msg) => write!(f, "network error: {msg}"),
            LoopError::StorageError(msg) => write!(f, "storage error: {msg}"),
            LoopError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

// ============================================================================
// WP09 types: Content + Thread loops
// ============================================================================

/// Errors that can occur in the content/thread automation loops.
#[derive(Debug)]
pub enum ContentLoopError {
    /// LLM generation failed.
    LlmFailure(String),
    /// Posting to X failed.
    PostFailed(String),
    /// Storage/database error.
    StorageError(String),
    /// Network error.
    NetworkError(String),
    /// Other error.
    Other(String),
}

impl fmt::Display for ContentLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LlmFailure(msg) => write!(f, "LLM failure: {msg}"),
            Self::PostFailed(msg) => write!(f, "Post failed: {msg}"),
            Self::StorageError(msg) => write!(f, "Storage error: {msg}"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ContentLoopError {}

// ============================================================================
// WP08 port traits: Mentions + Discovery loops
// ============================================================================

/// Port for fetching @-mentions from X API.
#[async_trait::async_trait]
pub trait MentionsFetcher: Send + Sync {
    /// Fetch mentions since the given ID. Returns newest first.
    async fn get_mentions(&self, since_id: Option<&str>) -> Result<Vec<LoopTweet>, LoopError>;
}

/// Port for searching tweets by keyword.
#[async_trait::async_trait]
pub trait TweetSearcher: Send + Sync {
    /// Search for tweets matching the query.
    async fn search_tweets(&self, query: &str) -> Result<Vec<LoopTweet>, LoopError>;
}

/// Port for generating reply content via LLM.
#[async_trait::async_trait]
pub trait ReplyGenerator: Send + Sync {
    /// Generate a reply to the given tweet.
    ///
    /// When `mention_product` is true, the reply may reference the product.
    /// When false, the reply must be purely helpful with no product mention.
    async fn generate_reply(
        &self,
        tweet_text: &str,
        author: &str,
        mention_product: bool,
    ) -> Result<String, LoopError>;
}

/// Port for safety checks (rate limits and dedup).
#[async_trait::async_trait]
pub trait SafetyChecker: Send + Sync {
    /// Check if we can reply (under daily rate limit).
    async fn can_reply(&self) -> bool;

    /// Check if we've already replied to this tweet.
    async fn has_replied_to(&self, tweet_id: &str) -> bool;

    /// Record a reply for dedup and rate limit tracking.
    async fn record_reply(&self, tweet_id: &str, reply_content: &str) -> Result<(), LoopError>;
}

/// Port for scoring tweets.
pub trait TweetScorer: Send + Sync {
    /// Score a tweet for reply-worthiness.
    fn score(&self, tweet: &LoopTweet) -> ScoreResult;
}

/// Port for persisting loop state (since_id, discovered tweets, action log).
#[async_trait::async_trait]
pub trait LoopStorage: Send + Sync {
    /// Get a persisted cursor value (e.g., since_id for mentions).
    async fn get_cursor(&self, key: &str) -> Result<Option<String>, LoopError>;

    /// Set a persisted cursor value.
    async fn set_cursor(&self, key: &str, value: &str) -> Result<(), LoopError>;

    /// Check if a discovered tweet already exists (dedup by tweet ID).
    async fn tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError>;

    /// Store a discovered tweet with its score and matched keyword.
    async fn store_discovered_tweet(
        &self,
        tweet: &LoopTweet,
        score: f32,
        keyword: &str,
    ) -> Result<(), LoopError>;

    /// Log an action (for audit trail and status reporting).
    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), LoopError>;
}

/// Port for sending post actions to the posting queue.
#[async_trait::async_trait]
pub trait PostSender: Send + Sync {
    /// Send a reply to a tweet through the posting queue.
    async fn send_reply(&self, tweet_id: &str, content: &str) -> Result<(), LoopError>;
}

// ============================================================================
// WP09 port traits: Content + Thread loops
// ============================================================================

/// Queries top-performing topics for epsilon-greedy topic selection.
#[async_trait::async_trait]
pub trait TopicScorer: Send + Sync {
    /// Get the top-performing topics, ordered by score descending.
    async fn get_top_topics(&self, limit: u32) -> Result<Vec<String>, ContentLoopError>;
}

/// Generates individual tweets on a given topic.
#[async_trait::async_trait]
pub trait TweetGenerator: Send + Sync {
    /// Generate an educational tweet on the given topic.
    async fn generate_tweet(&self, topic: &str) -> Result<String, ContentLoopError>;
}

/// Checks safety limits for content posting.
#[async_trait::async_trait]
pub trait ContentSafety: Send + Sync {
    /// Check if a tweet can be posted (daily limit not reached).
    async fn can_post_tweet(&self) -> bool;
    /// Check if a thread can be posted (weekly limit not reached).
    async fn can_post_thread(&self) -> bool;
}

/// Storage operations for content and thread loops.
#[async_trait::async_trait]
pub trait ContentStorage: Send + Sync {
    /// Get the timestamp of the most recent posted tweet.
    async fn last_tweet_time(
        &self,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError>;

    /// Get the timestamp of the most recent posted thread.
    async fn last_thread_time(
        &self,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError>;

    /// Post a tweet (sends to posting queue and records in DB).
    async fn post_tweet(&self, topic: &str, content: &str) -> Result<(), ContentLoopError>;

    /// Create a thread record in the database. Returns the thread ID.
    async fn create_thread(
        &self,
        topic: &str,
        tweet_count: usize,
    ) -> Result<String, ContentLoopError>;

    /// Update thread status (pending, posting, sent, partial).
    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: &str,
        tweet_count: usize,
        root_tweet_id: Option<&str>,
    ) -> Result<(), ContentLoopError>;

    /// Record a thread tweet (position in reply chain).
    async fn store_thread_tweet(
        &self,
        thread_id: &str,
        position: usize,
        tweet_id: &str,
        content: &str,
    ) -> Result<(), ContentLoopError>;

    /// Log an action to the audit trail.
    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), ContentLoopError>;
}

/// Posts tweets directly to X (for thread reply chains).
///
/// Thread tweets bypass the posting queue because reply chain
/// order must be maintained -- each tweet must reply to the previous.
#[async_trait::async_trait]
pub trait ThreadPoster: Send + Sync {
    /// Post a standalone tweet. Returns the tweet ID.
    async fn post_tweet(&self, content: &str) -> Result<String, ContentLoopError>;

    /// Reply to a tweet. Returns the new tweet ID.
    async fn reply_to_tweet(
        &self,
        in_reply_to: &str,
        content: &str,
    ) -> Result<String, ContentLoopError>;
}

// ============================================================================
// Shared utilities
// ============================================================================

/// Tracks consecutive errors to prevent infinite retry loops.
///
/// If a loop encounters `max_consecutive` errors without a single
/// success, it should pause for `pause_duration` before retrying.
#[derive(Debug)]
pub struct ConsecutiveErrorTracker {
    count: u32,
    max_consecutive: u32,
    pause_duration: Duration,
}

impl ConsecutiveErrorTracker {
    /// Create a new tracker.
    ///
    /// - `max_consecutive`: Number of consecutive errors before pausing.
    /// - `pause_duration`: How long to pause after hitting the limit.
    pub fn new(max_consecutive: u32, pause_duration: Duration) -> Self {
        Self {
            count: 0,
            max_consecutive,
            pause_duration,
        }
    }

    /// Record an error. Returns true if the loop should pause.
    pub fn record_error(&mut self) -> bool {
        self.count += 1;
        self.count >= self.max_consecutive
    }

    /// Record a success, resetting the counter.
    pub fn record_success(&mut self) {
        self.count = 0;
    }

    /// Whether the loop should pause due to too many consecutive errors.
    pub fn should_pause(&self) -> bool {
        self.count >= self.max_consecutive
    }

    /// How long to pause.
    pub fn pause_duration(&self) -> Duration {
        self.pause_duration
    }

    /// Current consecutive error count.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Reset the counter.
    pub fn reset(&mut self) {
        self.count = 0;
    }
}

/// Compute a backoff duration for rate limit errors.
///
/// Uses the `retry_after` hint if available, otherwise exponential
/// backoff starting at 60s, capped at 15 minutes.
pub fn rate_limit_backoff(retry_after: Option<u64>, attempt: u32) -> Duration {
    if let Some(secs) = retry_after {
        Duration::from_secs(secs)
    } else {
        let base = 60u64;
        let exp = base.saturating_mul(2u64.saturating_pow(attempt));
        Duration::from_secs(exp.min(900)) // cap at 15 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_tracker_records_errors() {
        let mut tracker = ConsecutiveErrorTracker::new(3, Duration::from_secs(300));
        assert!(!tracker.should_pause());
        assert_eq!(tracker.count(), 0);

        assert!(!tracker.record_error()); // 1
        assert!(!tracker.record_error()); // 2
        assert!(tracker.record_error()); // 3 -- should pause
        assert!(tracker.should_pause());
        assert_eq!(tracker.count(), 3);
    }

    #[test]
    fn error_tracker_resets_on_success() {
        let mut tracker = ConsecutiveErrorTracker::new(3, Duration::from_secs(300));
        tracker.record_error();
        tracker.record_error();
        tracker.record_success();
        assert_eq!(tracker.count(), 0);
        assert!(!tracker.should_pause());
    }

    #[test]
    fn error_tracker_pause_duration() {
        let tracker = ConsecutiveErrorTracker::new(5, Duration::from_secs(120));
        assert_eq!(tracker.pause_duration(), Duration::from_secs(120));
    }

    #[test]
    fn rate_limit_backoff_with_retry_after() {
        assert_eq!(rate_limit_backoff(Some(30), 0), Duration::from_secs(30));
        assert_eq!(rate_limit_backoff(Some(120), 5), Duration::from_secs(120));
    }

    #[test]
    fn rate_limit_backoff_exponential() {
        assert_eq!(rate_limit_backoff(None, 0), Duration::from_secs(60));
        assert_eq!(rate_limit_backoff(None, 1), Duration::from_secs(120));
        assert_eq!(rate_limit_backoff(None, 2), Duration::from_secs(240));
    }

    #[test]
    fn rate_limit_backoff_capped_at_15_minutes() {
        assert_eq!(rate_limit_backoff(None, 10), Duration::from_secs(900));
    }

    #[test]
    fn loop_error_display() {
        let err = LoopError::RateLimited {
            retry_after: Some(30),
        };
        assert_eq!(err.to_string(), "rate limited, retry after 30s");

        let err = LoopError::AuthExpired;
        assert_eq!(err.to_string(), "authentication expired");

        let err = LoopError::LlmFailure("timeout".to_string());
        assert_eq!(err.to_string(), "LLM failure: timeout");
    }

    #[test]
    fn loop_tweet_debug() {
        let tweet = LoopTweet {
            id: "123".to_string(),
            text: "hello".to_string(),
            author_id: "uid_123".to_string(),
            author_username: "user".to_string(),
            author_followers: 1000,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            likes: 10,
            retweets: 2,
            replies: 1,
        };
        let debug = format!("{tweet:?}");
        assert!(debug.contains("123"));
    }

    #[test]
    fn content_loop_error_display() {
        let err = ContentLoopError::LlmFailure("model down".to_string());
        assert_eq!(err.to_string(), "LLM failure: model down");

        let err = ContentLoopError::PostFailed("429".to_string());
        assert_eq!(err.to_string(), "Post failed: 429");

        let err = ContentLoopError::StorageError("disk full".to_string());
        assert_eq!(err.to_string(), "Storage error: disk full");

        let err = ContentLoopError::NetworkError("timeout".to_string());
        assert_eq!(err.to_string(), "Network error: timeout");

        let err = ContentLoopError::Other("unknown".to_string());
        assert_eq!(err.to_string(), "unknown");
    }
}
