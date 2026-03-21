//! Discovery loop tests.

use super::*;
use crate::automation::ScoreResult;
use std::sync::Mutex;

// --- Mock implementations ---

struct MockSearcher {
    results: Vec<LoopTweet>,
}

impl TweetSearcher for MockSearcher {
    async fn search(&self, _keyword: &str) -> Result<Vec<LoopTweet>, String> {
        Ok(self.results.clone())
    }
}

#[derive(Clone)]
struct MockScorer {
    scores: Vec<f32>,
}

impl TweetScorer for MockScorer {
    fn score(&self, _tweet: &LoopTweet, _threshold: f32) -> Option<f32> {
        self.scores.first().copied()
    }
}

struct MockGenerator {
    reply: String,
}

impl ReplyGenerator for MockGenerator {
    async fn generate(&self, _tweet: &LoopTweet) -> Result<String, String> {
        Ok(self.reply.clone())
    }
}

struct MockSafety;

impl SafetyChecker for MockSafety {
    async fn check(&self, _reply: &str) -> Result<bool, String> {
        Ok(true)
    }

    fn can_generate_reply(&self, _tweet: &LoopTweet) -> bool {
        true
    }
}

struct MockStorage;

#[async_trait::async_trait]
impl LoopStorage for MockStorage {
    async fn get_discovered_id(&self, _id: &str) -> Result<bool, String> {
        Ok(false)
    }

    async fn insert_discovered_id(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn record_loop_run(&self, _result: &DiscoveryResult) -> Result<(), String> {
        Ok(())
    }
}

struct MockPoster {
    sent: Mutex<Vec<String>>,
}

impl MockPoster {
    fn new() -> Self {
        Self {
            sent: Mutex::new(Vec::new()),
        }
    }

    fn sent_count(&self) -> usize {
        self.sent.lock().unwrap().len()
    }
}

#[async_trait::async_trait]
impl PostSender for MockPoster {
    async fn send(&self, reply: &str, _tweet_id: &str, _author: &str) -> Result<String, String> {
        self.sent.lock().unwrap().push(reply.to_string());
        Ok("posted".to_string())
    }
}

// --- Tests ---

#[tokio::test]
async fn discovery_summary_default() {
    let discovery = DiscoveryLoop {
        searcher: Arc::new(MockSearcher {
            results: vec![LoopTweet {
                id: "1".to_string(),
                text: "rust".to_string(),
                author: "user1".to_string(),
                author_followers: 100,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                likes: 10,
                retweets: 5,
                replies: 2,
                has_media: false,
                is_quote_tweet: false,
            }],
        }),
        scorer: Arc::new(MockScorer { scores: vec![90.0] }),
        generator: Arc::new(MockGenerator {
            reply: "great!".to_string(),
        }),
        safety: Arc::new(MockSafety),
        storage: Arc::new(MockStorage),
        poster: Arc::new(MockPoster::new()),
        keywords: vec!["rust".to_string(), "cli".to_string()],
        threshold: 50.0,
        dry_run: false,
    };

    let (_, summary) = discovery.run_once(None).await.unwrap();
    assert_eq!(summary.tweets_found, 1);
}

#[test]
fn discovery_result_debug() {
    let result = DiscoveryResult::Replied {
        tweet_id: "123".to_string(),
        author: "user".to_string(),
        score: 95.0,
        reply_text: "test reply".to_string(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("123"));
}

#[test]
fn truncate_exact_length() {
    let text = "hello world";
    assert_eq!(truncate_tweet_text(text, text.len()), text);
}

#[test]
fn truncate_empty_string() {
    assert_eq!(truncate_tweet_text("", 100), "");
}
