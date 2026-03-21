//! Target loop tests.

use super::super::*;
use super::truncate;
use std::sync::atomic::{AtomicU32, Ordering};
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
    existing_tweets: Mutex<Vec<String>>,
    replies_today: Mutex<i64>,
}

impl MockTargetStorage {
    fn new() -> Self {
        Self {
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

#[test]
fn truncate_short_string() {
    assert_eq!(truncate("hello", 10), "hello");
}

#[test]
fn truncate_long_string() {
    assert_eq!(truncate("hello world", 5), "hello...");
}

mod core;
