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

/// Looks up a user by username and optionally follows them.
#[async_trait::async_trait]
pub trait TargetUserManager: Send + Sync {
    /// Look up a user by username. Returns (user_id, username).
    async fn lookup_user(&self, username: &str) -> Result<(String, String), LoopError>;

    /// Follow a user.
    async fn follow_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Result<(), LoopError>;
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

    /// Get the followed_at timestamp for a target account.
    async fn get_followed_at(&self, account_id: &str) -> Result<Option<String>, LoopError>;

    /// Record that we followed a target account.
    async fn record_follow(&self, account_id: &str) -> Result<(), LoopError>;

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
    /// Whether to auto-follow target accounts.
    pub auto_follow: bool,
    /// Days to wait after following before engaging.
    pub follow_warmup_days: u32,
    /// Our own user ID (to pass for follow_user).
    pub own_user_id: String,
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
    pub async fn run(&self, cancel: CancellationToken, interval: Duration) {
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
                _ = tokio::time::sleep(interval) => {},
            }
        }

        tracing::info!("Target monitoring loop stopped");
    }

    /// Run a single iteration across all target accounts.
    async fn run_iteration(&self) -> Result<Vec<TargetResult>, LoopError> {
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

    /// Process a single target account: resolve, optionally follow, fetch tweets, reply.
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

        // Handle auto-follow
        if self.config.auto_follow {
            let followed_at = self.storage.get_followed_at(&user_id).await?;
            if followed_at.is_none() {
                tracing::info!(username = %resolved_username, "Auto-following target account");
                if !self.config.dry_run {
                    match self
                        .user_mgr
                        .follow_user(&self.config.own_user_id, &user_id)
                        .await
                    {
                        Ok(()) => {
                            self.storage.record_follow(&user_id).await?;

                            let _ = self
                                .storage
                                .log_action(
                                    "target_follow",
                                    "success",
                                    &format!("Followed @{resolved_username}"),
                                )
                                .await;

                            // Don't engage yet — warmup period starts now
                            return Ok(Vec::new());
                        }
                        Err(e) => {
                            // Follow failed (e.g. 403 on Basic tier). Log warning but
                            // continue to engagement — following is best-effort.
                            tracing::warn!(
                                username = %resolved_username,
                                error = %e,
                                "Auto-follow failed (API tier may not support follows), skipping follow"
                            );

                            let _ = self
                                .storage
                                .log_action(
                                    "target_follow",
                                    "skipped",
                                    &format!("Follow @{resolved_username} failed: {e}"),
                                )
                                .await;

                            // Record as "followed" to avoid retrying every iteration
                            let _ = self.storage.record_follow(&user_id).await;
                        }
                    }
                } else {
                    let _ = self
                        .storage
                        .log_action(
                            "target_follow",
                            "dry_run",
                            &format!("Followed @{resolved_username}"),
                        )
                        .await;

                    // Don't engage yet — warmup period starts now
                    return Ok(Vec::new());
                }
            }

            // Check warmup period (skip if follow was recorded due to failure)
            if self.config.follow_warmup_days > 0 {
                if let Some(ref followed_str) = self.storage.get_followed_at(&user_id).await? {
                    if !warmup_elapsed(followed_str, self.config.follow_warmup_days) {
                        tracing::debug!(
                            username = %resolved_username,
                            warmup_days = self.config.follow_warmup_days,
                            "Still in follow warmup period"
                        );
                        return Ok(Vec::new());
                    }
                }
            }
        }

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

        // Generate reply (no product mention for target accounts — be genuine)
        let reply_text = match self
            .generator
            .generate_reply(&tweet.text, username, false)
            .await
        {
            Ok(text) => text,
            Err(e) => {
                return TargetResult::Failed {
                    tweet_id: tweet.id.clone(),
                    error: e.to_string(),
                };
            }
        };

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

/// Check if the follow warmup period has elapsed.
fn warmup_elapsed(followed_at: &str, warmup_days: u32) -> bool {
    // Parse the SQLite datetime format ("YYYY-MM-DD HH:MM:SS")
    let followed = match chrono::NaiveDateTime::parse_from_str(followed_at, "%Y-%m-%d %H:%M:%S") {
        Ok(dt) => dt,
        Err(_) => return true, // If we can't parse, assume warmup is done
    };

    let now = chrono::Utc::now().naive_utc();
    let elapsed = now.signed_duration_since(followed);
    elapsed.num_days() >= warmup_days as i64
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
    use std::sync::Mutex;

    // --- Mock implementations ---

    struct MockFetcher {
        tweets: Vec<LoopTweet>,
    }

    #[async_trait::async_trait]
    impl TargetTweetFetcher for MockFetcher {
        async fn fetch_user_tweets(&self, _user_id: &str) -> Result<Vec<LoopTweet>, LoopError> {
            Ok(self.tweets.clone())
        }
    }

    struct MockUserManager {
        users: Vec<(String, String, String)>, // (username, user_id, resolved_username)
    }

    #[async_trait::async_trait]
    impl TargetUserManager for MockUserManager {
        async fn lookup_user(&self, username: &str) -> Result<(String, String), LoopError> {
            for (uname, uid, resolved) in &self.users {
                if uname == username {
                    return Ok((uid.clone(), resolved.clone()));
                }
            }
            Err(LoopError::Other(format!("user not found: {username}")))
        }

        async fn follow_user(
            &self,
            _source_user_id: &str,
            _target_user_id: &str,
        ) -> Result<(), LoopError> {
            Ok(())
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

    struct MockTargetStorage {
        followed_at: Mutex<Option<String>>,
        existing_tweets: Mutex<Vec<String>>,
        replies_today: Mutex<i64>,
    }

    impl MockTargetStorage {
        fn new() -> Self {
            Self {
                followed_at: Mutex::new(None),
                existing_tweets: Mutex::new(Vec::new()),
                replies_today: Mutex::new(0),
            }
        }

        fn with_followed_at(followed_at: &str) -> Self {
            Self {
                followed_at: Mutex::new(Some(followed_at.to_string())),
                existing_tweets: Mutex::new(Vec::new()),
                replies_today: Mutex::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl TargetStorage for MockTargetStorage {
        async fn upsert_target_account(
            &self,
            _account_id: &str,
            _username: &str,
        ) -> Result<(), LoopError> {
            Ok(())
        }
        async fn get_followed_at(&self, _account_id: &str) -> Result<Option<String>, LoopError> {
            Ok(self.followed_at.lock().expect("lock").clone())
        }
        async fn record_follow(&self, _account_id: &str) -> Result<(), LoopError> {
            *self.followed_at.lock().expect("lock") = Some("2026-01-01 00:00:00".to_string());
            Ok(())
        }
        async fn target_tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError> {
            Ok(self
                .existing_tweets
                .lock()
                .expect("lock")
                .contains(&tweet_id.to_string()))
        }
        async fn store_target_tweet(
            &self,
            _tweet_id: &str,
            _account_id: &str,
            _content: &str,
            _created_at: &str,
            _reply_count: i64,
            _like_count: i64,
            _relevance_score: f64,
        ) -> Result<(), LoopError> {
            Ok(())
        }
        async fn mark_target_tweet_replied(&self, _tweet_id: &str) -> Result<(), LoopError> {
            Ok(())
        }
        async fn record_target_reply(&self, _account_id: &str) -> Result<(), LoopError> {
            *self.replies_today.lock().expect("lock") += 1;
            Ok(())
        }
        async fn count_target_replies_today(&self) -> Result<i64, LoopError> {
            Ok(*self.replies_today.lock().expect("lock"))
        }
        async fn log_action(
            &self,
            _action_type: &str,
            _status: &str,
            _message: &str,
        ) -> Result<(), LoopError> {
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
            text: format!("Interesting thoughts on tech from @{author}"),
            author_id: format!("uid_{author}"),
            author_username: author.to_string(),
            author_followers: 5000,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            likes: 10,
            retweets: 2,
            replies: 1,
        }
    }

    fn default_config() -> TargetLoopConfig {
        TargetLoopConfig {
            accounts: vec!["alice".to_string()],
            max_target_replies_per_day: 3,
            auto_follow: false,
            follow_warmup_days: 3,
            own_user_id: "own_123".to_string(),
            dry_run: false,
        }
    }

    fn build_loop(
        tweets: Vec<LoopTweet>,
        config: TargetLoopConfig,
        storage: Arc<MockTargetStorage>,
    ) -> (TargetLoop, Arc<MockPoster>) {
        let poster = Arc::new(MockPoster::new());
        let user_mgr = Arc::new(MockUserManager {
            users: vec![(
                "alice".to_string(),
                "uid_alice".to_string(),
                "alice".to_string(),
            )],
        });
        let target_loop = TargetLoop::new(
            Arc::new(MockFetcher { tweets }),
            user_mgr,
            Arc::new(MockGenerator {
                reply: "Great point!".to_string(),
            }),
            Arc::new(MockSafety::new(true)),
            storage,
            poster.clone(),
            config,
        );
        (target_loop, poster)
    }

    // --- Tests ---

    #[tokio::test]
    async fn empty_accounts_does_nothing() {
        let storage = Arc::new(MockTargetStorage::new());
        let mut config = default_config();
        config.accounts = Vec::new();
        let (target_loop, poster) = build_loop(Vec::new(), config, storage);

        let results = target_loop.run_iteration().await.expect("iteration");
        assert!(results.is_empty());
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn replies_to_target_tweet() {
        let tweets = vec![test_tweet("tw1", "alice")];
        let storage = Arc::new(MockTargetStorage::new());
        let (target_loop, poster) = build_loop(tweets, default_config(), storage);

        let results = target_loop.run_iteration().await.expect("iteration");
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], TargetResult::Replied { .. }));
        assert_eq!(poster.sent_count(), 1);
    }

    #[tokio::test]
    async fn skips_existing_target_tweet() {
        let tweets = vec![test_tweet("tw1", "alice")];
        let storage = Arc::new(MockTargetStorage::new());
        storage
            .existing_tweets
            .lock()
            .expect("lock")
            .push("tw1".to_string());
        let (target_loop, poster) = build_loop(tweets, default_config(), storage);

        let results = target_loop.run_iteration().await.expect("iteration");
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], TargetResult::Skipped { .. }));
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn respects_daily_limit() {
        let tweets = vec![test_tweet("tw1", "alice")];
        let storage = Arc::new(MockTargetStorage::new());
        *storage.replies_today.lock().expect("lock") = 3;
        let (target_loop, poster) = build_loop(tweets, default_config(), storage);

        let results = target_loop.run_iteration().await.expect("iteration");
        assert!(results.is_empty());
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn dry_run_does_not_post() {
        let tweets = vec![test_tweet("tw1", "alice")];
        let storage = Arc::new(MockTargetStorage::new());
        let mut config = default_config();
        config.dry_run = true;
        let (target_loop, poster) = build_loop(tweets, config, storage);

        let results = target_loop.run_iteration().await.expect("iteration");
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], TargetResult::Replied { .. }));
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn auto_follow_follows_and_skips_engagement() {
        let tweets = vec![test_tweet("tw1", "alice")];
        let storage = Arc::new(MockTargetStorage::new());
        let mut config = default_config();
        config.auto_follow = true;
        let (target_loop, poster) = build_loop(tweets, config, storage.clone());

        let results = target_loop.run_iteration().await.expect("iteration");
        // Should follow but NOT reply (warmup starts)
        assert!(results.is_empty());
        assert_eq!(poster.sent_count(), 0);
        // Should have recorded the follow
        assert!(storage.followed_at.lock().expect("lock").is_some());
    }

    #[tokio::test]
    async fn auto_follow_warmup_blocks_engagement() {
        let tweets = vec![test_tweet("tw1", "alice")];
        // Followed yesterday — warmup is 3 days
        let now = chrono::Utc::now().naive_utc();
        let yesterday = now - chrono::Duration::days(1);
        let followed_str = yesterday.format("%Y-%m-%d %H:%M:%S").to_string();
        let storage = Arc::new(MockTargetStorage::with_followed_at(&followed_str));
        let mut config = default_config();
        config.auto_follow = true;
        let (target_loop, poster) = build_loop(tweets, config, storage);

        let results = target_loop.run_iteration().await.expect("iteration");
        assert!(results.is_empty());
        assert_eq!(poster.sent_count(), 0);
    }

    #[tokio::test]
    async fn auto_follow_warmup_elapsed_allows_engagement() {
        let tweets = vec![test_tweet("tw1", "alice")];
        // Followed 5 days ago — warmup is 3 days
        let now = chrono::Utc::now().naive_utc();
        let five_days_ago = now - chrono::Duration::days(5);
        let followed_str = five_days_ago.format("%Y-%m-%d %H:%M:%S").to_string();
        let storage = Arc::new(MockTargetStorage::with_followed_at(&followed_str));
        let mut config = default_config();
        config.auto_follow = true;
        let (target_loop, poster) = build_loop(tweets, config, storage);

        let results = target_loop.run_iteration().await.expect("iteration");
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], TargetResult::Replied { .. }));
        assert_eq!(poster.sent_count(), 1);
    }

    #[test]
    fn warmup_elapsed_parses_correctly() {
        assert!(warmup_elapsed("2020-01-01 00:00:00", 3));
        assert!(!warmup_elapsed(
            &chrono::Utc::now()
                .naive_utc()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            3
        ));
    }

    #[test]
    fn warmup_elapsed_invalid_date_returns_true() {
        assert!(warmup_elapsed("not-a-date", 3));
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("hello world", 5), "hello...");
    }
}
