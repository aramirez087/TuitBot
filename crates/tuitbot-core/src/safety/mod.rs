//! Safety module for rate limiting and duplicate prevention.
//!
//! Provides the `SafetyGuard` as the primary pre-flight check interface
//! for all automation loops. Combines rate limiting with deduplication
//! to prevent API abuse and duplicate content.

pub mod dedup;
pub mod redact;

use crate::error::StorageError;
use crate::storage::rate_limits;
use crate::storage::{author_interactions, DbPool};

pub use dedup::DedupChecker;

/// Wraps rate limit database operations with a clean API.
pub struct RateLimiter {
    pool: DbPool,
}

impl RateLimiter {
    /// Create a new rate limiter backed by the given database pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Check if a reply action is allowed under the current rate limit.
    pub async fn can_reply(&self) -> Result<bool, StorageError> {
        rate_limits::check_rate_limit(&self.pool, "reply").await
    }

    /// Check if a tweet action is allowed under the current rate limit.
    pub async fn can_tweet(&self) -> Result<bool, StorageError> {
        rate_limits::check_rate_limit(&self.pool, "tweet").await
    }

    /// Check if a thread action is allowed under the current rate limit.
    pub async fn can_thread(&self) -> Result<bool, StorageError> {
        rate_limits::check_rate_limit(&self.pool, "thread").await
    }

    /// Check if a search action is allowed under the current rate limit.
    pub async fn can_search(&self) -> Result<bool, StorageError> {
        rate_limits::check_rate_limit(&self.pool, "search").await
    }

    /// Record a successful reply action (increments counter).
    pub async fn record_reply(&self) -> Result<(), StorageError> {
        rate_limits::increment_rate_limit(&self.pool, "reply").await
    }

    /// Record a successful tweet action (increments counter).
    pub async fn record_tweet(&self) -> Result<(), StorageError> {
        rate_limits::increment_rate_limit(&self.pool, "tweet").await
    }

    /// Record a successful thread action (increments counter).
    pub async fn record_thread(&self) -> Result<(), StorageError> {
        rate_limits::increment_rate_limit(&self.pool, "thread").await
    }

    /// Record a successful search action (increments counter).
    pub async fn record_search(&self) -> Result<(), StorageError> {
        rate_limits::increment_rate_limit(&self.pool, "search").await
    }

    /// Atomically check and claim a rate limit slot.
    ///
    /// Returns `Ok(true)` if permitted (counter incremented),
    /// `Ok(false)` if the rate limit is reached.
    /// Preferred over separate check + record for posting actions.
    pub async fn acquire_posting_permit(&self, action_type: &str) -> Result<bool, StorageError> {
        rate_limits::check_and_increment_rate_limit(&self.pool, action_type).await
    }
}

/// Reason an action was denied by the safety guard.
#[derive(Debug, Clone, PartialEq)]
pub enum DenialReason {
    /// Action blocked by rate limiting.
    RateLimited {
        /// Which action type hit the limit.
        action_type: String,
        /// Current request count.
        current: i64,
        /// Maximum allowed requests.
        max: i64,
    },
    /// Already replied to this tweet.
    AlreadyReplied {
        /// The tweet ID that was already replied to.
        tweet_id: String,
    },
    /// Proposed reply is too similar to a recent reply.
    SimilarPhrasing,
    /// Reply contains a banned phrase.
    BannedPhrase {
        /// The banned phrase that was found.
        phrase: String,
    },
    /// Already reached the per-author daily reply limit.
    AuthorLimitReached,
    /// Replying to own tweet.
    SelfReply,
}

impl std::fmt::Display for DenialReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited {
                action_type,
                current,
                max,
            } => write!(f, "Rate limited: {action_type} ({current}/{max})"),
            Self::AlreadyReplied { tweet_id } => {
                write!(f, "Already replied to tweet {tweet_id}")
            }
            Self::SimilarPhrasing => {
                write!(f, "Reply phrasing too similar to recent replies")
            }
            Self::BannedPhrase { phrase } => {
                write!(f, "Reply contains banned phrase: \"{phrase}\"")
            }
            Self::AuthorLimitReached => {
                write!(f, "Already reached daily reply limit for this author")
            }
            Self::SelfReply => {
                write!(f, "Cannot reply to own tweets")
            }
        }
    }
}

/// Check if any banned phrase appears in the text (case-insensitive).
///
/// Returns the first matching banned phrase, or `None` if clean.
pub fn contains_banned_phrase(text: &str, banned: &[String]) -> Option<String> {
    let text_lower = text.to_lowercase();
    for phrase in banned {
        if text_lower.contains(&phrase.to_lowercase()) {
            return Some(phrase.clone());
        }
    }
    None
}

/// Check if the tweet author is the bot's own user ID.
pub fn is_self_reply(tweet_author_id: &str, own_user_id: &str) -> bool {
    !tweet_author_id.is_empty() && !own_user_id.is_empty() && tweet_author_id == own_user_id
}

/// Combined safety guard for all automation loops.
///
/// Provides pre-flight checks that combine rate limiting with deduplication.
/// All automation loops should call `SafetyGuard` methods before taking actions.
pub struct SafetyGuard {
    rate_limiter: RateLimiter,
    dedup_checker: DedupChecker,
    pool: DbPool,
}

impl SafetyGuard {
    /// Create a new safety guard backed by the given database pool.
    pub fn new(pool: DbPool) -> Self {
        Self {
            rate_limiter: RateLimiter::new(pool.clone()),
            dedup_checker: DedupChecker::new(pool.clone()),
            pool,
        }
    }

    /// Check whether replying to a tweet is permitted.
    ///
    /// Checks rate limits, exact dedup, and optionally phrasing similarity.
    /// Returns `Ok(Ok(()))` if allowed, `Ok(Err(DenialReason))` if blocked,
    /// or `Err(StorageError)` on infrastructure failure.
    pub async fn can_reply_to(
        &self,
        tweet_id: &str,
        proposed_reply: Option<&str>,
    ) -> Result<Result<(), DenialReason>, StorageError> {
        // Check rate limit
        if !self.rate_limiter.can_reply().await? {
            let limits = rate_limits::get_all_rate_limits(&self.rate_limiter.pool).await?;
            let reply_limit = limits.iter().find(|l| l.action_type == "reply");
            let (current, max) = reply_limit
                .map(|l| (l.request_count, l.max_requests))
                .unwrap_or((0, 0));

            tracing::debug!(
                action = "reply",
                current,
                max,
                "Action denied: rate limited"
            );

            return Ok(Err(DenialReason::RateLimited {
                action_type: "reply".to_string(),
                current,
                max,
            }));
        }

        // Check exact dedup
        if self.dedup_checker.has_replied_to(tweet_id).await? {
            tracing::debug!(tweet_id, "Action denied: already replied");
            return Ok(Err(DenialReason::AlreadyReplied {
                tweet_id: tweet_id.to_string(),
            }));
        }

        // Check phrasing similarity
        if let Some(reply_text) = proposed_reply {
            if self
                .dedup_checker
                .is_phrasing_similar(reply_text, 20)
                .await?
            {
                tracing::debug!("Action denied: similar phrasing");
                return Ok(Err(DenialReason::SimilarPhrasing));
            }
        }

        Ok(Ok(()))
    }

    /// Check whether posting an original tweet is permitted.
    ///
    /// Only checks rate limits (no dedup for original tweets).
    pub async fn can_post_tweet(&self) -> Result<Result<(), DenialReason>, StorageError> {
        if !self.rate_limiter.can_tweet().await? {
            let limits = rate_limits::get_all_rate_limits(&self.rate_limiter.pool).await?;
            let tweet_limit = limits.iter().find(|l| l.action_type == "tweet");
            let (current, max) = tweet_limit
                .map(|l| (l.request_count, l.max_requests))
                .unwrap_or((0, 0));

            tracing::debug!(
                action = "tweet",
                current,
                max,
                "Action denied: rate limited"
            );

            return Ok(Err(DenialReason::RateLimited {
                action_type: "tweet".to_string(),
                current,
                max,
            }));
        }

        Ok(Ok(()))
    }

    /// Check whether posting a thread is permitted.
    ///
    /// Only checks rate limits (no dedup for threads).
    pub async fn can_post_thread(&self) -> Result<Result<(), DenialReason>, StorageError> {
        if !self.rate_limiter.can_thread().await? {
            let limits = rate_limits::get_all_rate_limits(&self.rate_limiter.pool).await?;
            let thread_limit = limits.iter().find(|l| l.action_type == "thread");
            let (current, max) = thread_limit
                .map(|l| (l.request_count, l.max_requests))
                .unwrap_or((0, 0));

            tracing::debug!(
                action = "thread",
                current,
                max,
                "Action denied: rate limited"
            );

            return Ok(Err(DenialReason::RateLimited {
                action_type: "thread".to_string(),
                current,
                max,
            }));
        }

        Ok(Ok(()))
    }

    /// Check if replying to this author is permitted (per-author daily limit).
    pub async fn check_author_limit(
        &self,
        author_id: &str,
        max_per_day: u32,
    ) -> Result<Result<(), DenialReason>, StorageError> {
        let count =
            author_interactions::get_author_reply_count_today(&self.pool, author_id).await?;
        if count >= max_per_day as i64 {
            tracing::debug!(
                author_id,
                count,
                max = max_per_day,
                "Action denied: author daily limit reached"
            );
            return Ok(Err(DenialReason::AuthorLimitReached));
        }
        Ok(Ok(()))
    }

    /// Check if a generated reply contains a banned phrase.
    pub fn check_banned_phrases(reply_text: &str, banned: &[String]) -> Result<(), DenialReason> {
        if let Some(phrase) = contains_banned_phrase(reply_text, banned) {
            tracing::debug!(phrase = %phrase, "Action denied: banned phrase");
            return Err(DenialReason::BannedPhrase { phrase });
        }
        Ok(())
    }

    /// Record a reply for an author interaction.
    pub async fn record_author_interaction(
        &self,
        author_id: &str,
        author_username: &str,
    ) -> Result<(), StorageError> {
        author_interactions::increment_author_interaction(&self.pool, author_id, author_username)
            .await
    }

    /// Record a successful reply action.
    pub async fn record_reply(&self) -> Result<(), StorageError> {
        self.rate_limiter.record_reply().await
    }

    /// Record a successful tweet action.
    pub async fn record_tweet(&self) -> Result<(), StorageError> {
        self.rate_limiter.record_tweet().await
    }

    /// Record a successful thread action.
    pub async fn record_thread(&self) -> Result<(), StorageError> {
        self.rate_limiter.record_thread().await
    }

    /// Get a reference to the underlying rate limiter.
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    /// Get a reference to the underlying dedup checker.
    pub fn dedup_checker(&self) -> &DedupChecker {
        &self.dedup_checker
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{IntervalsConfig, LimitsConfig};
    use crate::storage::init_test_db;
    use crate::storage::replies::{insert_reply, ReplySent};

    fn test_limits() -> LimitsConfig {
        LimitsConfig {
            max_replies_per_day: 3,
            max_tweets_per_day: 2,
            max_threads_per_week: 1,
            min_action_delay_seconds: 30,
            max_action_delay_seconds: 120,
            max_replies_per_author_per_day: 1,
            banned_phrases: vec!["check out".to_string(), "you should try".to_string()],
            product_mention_ratio: 0.2,
        }
    }

    fn test_intervals() -> IntervalsConfig {
        IntervalsConfig {
            mentions_check_seconds: 300,
            discovery_search_seconds: 600,
            content_post_window_seconds: 14400,
            thread_interval_seconds: 604800,
        }
    }

    async fn setup_guard() -> (DbPool, SafetyGuard) {
        let pool = init_test_db().await.expect("init db");
        rate_limits::init_rate_limits(&pool, &test_limits(), &test_intervals())
            .await
            .expect("init rate limits");
        let guard = SafetyGuard::new(pool.clone());
        (pool, guard)
    }

    fn sample_reply(target_id: &str, content: &str) -> ReplySent {
        ReplySent {
            id: 0,
            target_tweet_id: target_id.to_string(),
            reply_tweet_id: Some("r_123".to_string()),
            reply_content: content.to_string(),
            llm_provider: None,
            llm_model: None,
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            status: "sent".to_string(),
            error_message: None,
        }
    }

    #[tokio::test]
    async fn rate_limiter_can_reply_and_record() {
        let pool = init_test_db().await.expect("init db");
        rate_limits::init_rate_limits(&pool, &test_limits(), &test_intervals())
            .await
            .expect("init");

        let limiter = RateLimiter::new(pool);

        assert!(limiter.can_reply().await.expect("check"));
        limiter.record_reply().await.expect("record");
        limiter.record_reply().await.expect("record");
        limiter.record_reply().await.expect("record");
        assert!(!limiter.can_reply().await.expect("check"));
    }

    #[tokio::test]
    async fn rate_limiter_acquire_posting_permit() {
        let pool = init_test_db().await.expect("init db");
        rate_limits::init_rate_limits(&pool, &test_limits(), &test_intervals())
            .await
            .expect("init");

        let limiter = RateLimiter::new(pool);

        assert!(limiter.acquire_posting_permit("tweet").await.expect("1"));
        assert!(limiter.acquire_posting_permit("tweet").await.expect("2"));
        assert!(!limiter.acquire_posting_permit("tweet").await.expect("3"));
    }

    #[tokio::test]
    async fn safety_guard_allows_new_reply() {
        let (_pool, guard) = setup_guard().await;

        let result = guard.can_reply_to("tweet_1", None).await.expect("check");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn safety_guard_blocks_already_replied() {
        let (pool, guard) = setup_guard().await;

        let reply = sample_reply("tweet_1", "Some reply content");
        insert_reply(&pool, &reply).await.expect("insert");

        let result = guard.can_reply_to("tweet_1", None).await.expect("check");
        assert_eq!(
            result,
            Err(DenialReason::AlreadyReplied {
                tweet_id: "tweet_1".to_string()
            })
        );
    }

    #[tokio::test]
    async fn safety_guard_blocks_rate_limited() {
        let (_pool, guard) = setup_guard().await;

        // Exhaust the reply limit (max = 3)
        for _ in 0..3 {
            guard.record_reply().await.expect("record");
        }

        let result = guard.can_reply_to("tweet_new", None).await.expect("check");
        match result {
            Err(DenialReason::RateLimited {
                action_type,
                current,
                max,
            }) => {
                assert_eq!(action_type, "reply");
                assert_eq!(current, 3);
                assert_eq!(max, 3);
            }
            other => panic!("expected RateLimited, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn safety_guard_blocks_similar_phrasing() {
        let (pool, guard) = setup_guard().await;

        let reply = sample_reply(
            "tweet_1",
            "This is a great tool for developers and engineers to use daily",
        );
        insert_reply(&pool, &reply).await.expect("insert");

        let result = guard
            .can_reply_to(
                "tweet_2",
                Some("This is a great tool for developers and engineers to use often"),
            )
            .await
            .expect("check");

        assert_eq!(result, Err(DenialReason::SimilarPhrasing));
    }

    #[tokio::test]
    async fn safety_guard_allows_different_phrasing() {
        let (pool, guard) = setup_guard().await;

        let reply = sample_reply(
            "tweet_1",
            "This is a great tool for developers and engineers to use daily",
        );
        insert_reply(&pool, &reply).await.expect("insert");

        let result = guard
            .can_reply_to(
                "tweet_2",
                Some("I love cooking pasta with fresh basil and tomatoes every day"),
            )
            .await
            .expect("check");

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn safety_guard_can_post_tweet_allowed() {
        let (_pool, guard) = setup_guard().await;

        let result = guard.can_post_tweet().await.expect("check");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn safety_guard_can_post_tweet_blocked() {
        let (_pool, guard) = setup_guard().await;

        // Exhaust tweet limit (max = 2)
        guard.record_tweet().await.expect("record");
        guard.record_tweet().await.expect("record");

        let result = guard.can_post_tweet().await.expect("check");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn safety_guard_can_post_thread_allowed() {
        let (_pool, guard) = setup_guard().await;

        let result = guard.can_post_thread().await.expect("check");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn safety_guard_can_post_thread_blocked() {
        let (_pool, guard) = setup_guard().await;

        // Exhaust thread limit (max = 1)
        guard.record_thread().await.expect("record");

        let result = guard.can_post_thread().await.expect("check");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn denial_reason_display() {
        let rate = DenialReason::RateLimited {
            action_type: "reply".to_string(),
            current: 20,
            max: 20,
        };
        assert_eq!(rate.to_string(), "Rate limited: reply (20/20)");

        let replied = DenialReason::AlreadyReplied {
            tweet_id: "abc123".to_string(),
        };
        assert_eq!(replied.to_string(), "Already replied to tweet abc123");

        let similar = DenialReason::SimilarPhrasing;
        assert_eq!(
            similar.to_string(),
            "Reply phrasing too similar to recent replies"
        );

        let banned = DenialReason::BannedPhrase {
            phrase: "check out".to_string(),
        };
        assert_eq!(
            banned.to_string(),
            "Reply contains banned phrase: \"check out\""
        );

        let author = DenialReason::AuthorLimitReached;
        assert_eq!(
            author.to_string(),
            "Already reached daily reply limit for this author"
        );

        let self_reply = DenialReason::SelfReply;
        assert_eq!(self_reply.to_string(), "Cannot reply to own tweets");
    }

    #[test]
    fn contains_banned_phrase_detects_match() {
        let banned = vec!["check out".to_string(), "link in bio".to_string()];
        assert_eq!(
            contains_banned_phrase("You should check out this tool!", &banned),
            Some("check out".to_string())
        );
    }

    #[test]
    fn contains_banned_phrase_case_insensitive() {
        let banned = vec!["Check Out".to_string()];
        assert_eq!(
            contains_banned_phrase("check out this thing", &banned),
            Some("Check Out".to_string())
        );
    }

    #[test]
    fn contains_banned_phrase_no_match() {
        let banned = vec!["check out".to_string()];
        assert_eq!(
            contains_banned_phrase("This is a helpful reply", &banned),
            None
        );
    }

    #[test]
    fn is_self_reply_detects_self() {
        assert!(is_self_reply("user_123", "user_123"));
    }

    #[test]
    fn is_self_reply_different_users() {
        assert!(!is_self_reply("user_123", "user_456"));
    }

    #[test]
    fn is_self_reply_empty_ids() {
        assert!(!is_self_reply("", "user_123"));
        assert!(!is_self_reply("user_123", ""));
        assert!(!is_self_reply("", ""));
    }

    #[tokio::test]
    async fn safety_guard_check_author_limit_allows_first() {
        let (_pool, guard) = setup_guard().await;
        let result = guard
            .check_author_limit("author_1", 1)
            .await
            .expect("check");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn safety_guard_check_author_limit_blocks_over_limit() {
        let (_pool, guard) = setup_guard().await;
        guard
            .record_author_interaction("author_1", "alice")
            .await
            .expect("record");

        let result = guard
            .check_author_limit("author_1", 1)
            .await
            .expect("check");
        assert_eq!(result, Err(DenialReason::AuthorLimitReached));
    }

    #[test]
    fn check_banned_phrases_blocks_banned() {
        let banned = vec!["check out".to_string(), "I recommend".to_string()];
        let result = SafetyGuard::check_banned_phrases("You should check out this tool!", &banned);
        assert_eq!(
            result,
            Err(DenialReason::BannedPhrase {
                phrase: "check out".to_string()
            })
        );
    }

    #[test]
    fn check_banned_phrases_allows_clean() {
        let banned = vec!["check out".to_string()];
        let result = SafetyGuard::check_banned_phrases("Great insight on testing!", &banned);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn safety_guard_exposes_rate_limiter_and_dedup() {
        let (_pool, guard) = setup_guard().await;

        // Verify accessors work without panicking
        assert!(guard.rate_limiter().can_search().await.expect("search"));
        let phrases = guard
            .dedup_checker()
            .get_recent_reply_phrases(5)
            .await
            .expect("phrases");
        assert!(phrases.is_empty());
    }
}
