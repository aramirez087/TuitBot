//! Discovery loop tests.

use super::super::*;
use super::truncate;
use crate::automation::loop_helpers::LoopError;
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

mod core_tests;
mod integration;
