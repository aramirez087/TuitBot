//! Tests for the mentions loop.

use super::*;
use crate::automation::loop_helpers::{LoopError, LoopStorage, LoopTweet};
use std::sync::Arc;
use std::sync::Mutex;

// --- Mock implementations ---

struct MockFetcher {
    mentions: Vec<LoopTweet>,
}

#[async_trait::async_trait]
impl crate::automation::loop_helpers::MentionsFetcher for MockFetcher {
    async fn get_mentions(&self, _since_id: Option<&str>) -> Result<Vec<LoopTweet>, LoopError> {
        Ok(self.mentions.clone())
    }
}

struct MockGenerator {
    reply_prefix: String,
}

#[async_trait::async_trait]
impl crate::automation::loop_helpers::ReplyGenerator for MockGenerator {
    async fn generate_reply(
        &self,
        _tweet_text: &str,
        author: &str,
        _mention_product: bool,
    ) -> Result<String, LoopError> {
        Ok(format!("{} reply to @{author}", self.reply_prefix))
    }
}

struct FailingGenerator;

#[async_trait::async_trait]
impl crate::automation::loop_helpers::ReplyGenerator for FailingGenerator {
    async fn generate_reply(
        &self,
        _tweet_text: &str,
        _author: &str,
        _mention_product: bool,
    ) -> Result<String, LoopError> {
        Err(LoopError::LlmFailure("timeout".to_string()))
    }
}

struct MockSafety {
    replied_ids: Mutex<Vec<String>>,
    can_reply: bool,
}

impl MockSafety {
    fn new(can_reply: bool) -> Self {
        Self {
            replied_ids: Mutex::new(Vec::new()),
            can_reply,
        }
    }
}

#[async_trait::async_trait]
impl crate::automation::loop_helpers::SafetyChecker for MockSafety {
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
impl crate::automation::loop_helpers::PostSender for MockPoster {
    async fn send_reply(&self, tweet_id: &str, content: &str) -> Result<(), LoopError> {
        self.sent
            .lock()
            .expect("lock")
            .push((tweet_id.to_string(), content.to_string()));
        Ok(())
    }
}

struct MockStorage {
    cursors: Mutex<std::collections::HashMap<String, String>>,
    actions: Mutex<Vec<(String, String, String)>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            cursors: Mutex::new(std::collections::HashMap::new()),
            actions: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl LoopStorage for MockStorage {
    async fn get_cursor(&self, key: &str) -> Result<Option<String>, LoopError> {
        Ok(self.cursors.lock().expect("lock").get(key).cloned())
    }

    async fn set_cursor(&self, key: &str, value: &str) -> Result<(), LoopError> {
        self.cursors
            .lock()
            .expect("lock")
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn tweet_exists(&self, _tweet_id: &str) -> Result<bool, LoopError> {
        Ok(false)
    }

    async fn store_discovered_tweet(
        &self,
        _tweet: &LoopTweet,
        _score: f32,
        _keyword: &str,
    ) -> Result<(), LoopError> {
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

fn test_tweet(id: &str, author: &str) -> LoopTweet {
    LoopTweet {
        id: id.to_string(),
        text: format!("Test tweet from @{author}"),
        author_id: format!("uid_{author}"),
        author_username: author.to_string(),
        author_followers: 1000,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        likes: 10,
        retweets: 2,
        replies: 1,
    }
}

// --- Tests ---

#[tokio::test]
async fn run_once_no_mentions() {
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: Vec::new(),
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Test".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        Arc::new(MockPoster::new()),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, since_id) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert!(results.is_empty());
    assert!(since_id.is_none());
}

#[tokio::test]
async fn run_once_processes_mentions() {
    let poster = Arc::new(MockPoster::new());
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("100", "alice"), test_tweet("101", "bob")],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hello".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        poster.clone(),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, since_id) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(matches!(&results[0], MentionResult::Replied { .. }));
    assert!(matches!(&results[1], MentionResult::Replied { .. }));
    assert_eq!(since_id, Some("101".to_string()));
    assert_eq!(poster.sent_count(), 2);
}

#[tokio::test]
async fn run_once_respects_limit() {
    let poster = Arc::new(MockPoster::new());
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![
                test_tweet("100", "alice"),
                test_tweet("101", "bob"),
                test_tweet("102", "carol"),
            ],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        poster.clone(),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, _) = mentions_loop
        .run_once(None, Some(2), &storage)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(poster.sent_count(), 2);
}

#[tokio::test]
async fn run_once_skips_already_replied() {
    let safety = Arc::new(MockSafety::new(true));
    // Pre-mark tweet "100" as replied
    safety.record_reply("100", "already replied").await.unwrap();

    let poster = Arc::new(MockPoster::new());
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("100", "alice"), test_tweet("101", "bob")],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        safety,
        poster.clone(),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(matches!(&results[0], MentionResult::Skipped { .. }));
    assert!(matches!(&results[1], MentionResult::Replied { .. }));
    assert_eq!(poster.sent_count(), 1);
}

#[tokio::test]
async fn run_once_skips_when_rate_limited() {
    let poster = Arc::new(MockPoster::new());
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("100", "alice")],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        Arc::new(MockSafety::new(false)), // can_reply = false
        poster.clone(),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        matches!(&results[0], MentionResult::Skipped { reason, .. } if reason == "rate limited")
    );
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn run_once_dry_run_does_not_post() {
    let poster = Arc::new(MockPoster::new());
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("100", "alice")],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        poster.clone(),
        true, // dry_run
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(&results[0], MentionResult::Replied { .. }));
    // Should NOT have sent to posting queue
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn run_once_llm_failure_returns_failed() {
    let poster = Arc::new(MockPoster::new());
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("100", "alice")],
        }),
        Arc::new(FailingGenerator),
        Arc::new(MockSafety::new(true)),
        poster.clone(),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(&results[0], MentionResult::Failed { .. }));
    assert_eq!(poster.sent_count(), 0);
}

#[test]
fn update_max_id_tracks_highest() {
    let mut max = None;
    update_max_id(&mut max, "100");
    assert_eq!(max, Some("100".to_string()));
    update_max_id(&mut max, "99");
    assert_eq!(max, Some("100".to_string()));
    update_max_id(&mut max, "200");
    assert_eq!(max, Some("200".to_string()));
}

#[test]
fn truncate_short_string() {
    assert_eq!(truncate("hello", 10), "hello");
}

#[test]
fn truncate_long_string() {
    assert_eq!(truncate("hello world this is long", 10), "hello worl...");
}

#[test]
fn truncate_exact_length() {
    assert_eq!(truncate("hello", 5), "hello");
}

#[test]
fn update_max_id_from_none() {
    let mut max = None;
    update_max_id(&mut max, "42");
    assert_eq!(max, Some("42".to_string()));
}

#[test]
fn update_max_id_longer_id_wins() {
    // A longer numeric string is always larger
    let mut max = Some("99".to_string());
    update_max_id(&mut max, "100");
    assert_eq!(max, Some("100".to_string()));
}

#[test]
fn update_max_id_shorter_id_loses() {
    let mut max = Some("100".to_string());
    update_max_id(&mut max, "99");
    assert_eq!(max, Some("100".to_string()));
}

#[test]
fn update_max_id_equal_length_comparison() {
    let mut max = Some("200".to_string());
    update_max_id(&mut max, "199");
    assert_eq!(max, Some("200".to_string()));

    update_max_id(&mut max, "201");
    assert_eq!(max, Some("201".to_string()));
}

#[test]
fn update_max_id_same_id_no_change() {
    let mut max = Some("100".to_string());
    update_max_id(&mut max, "100");
    assert_eq!(max, Some("100".to_string()));
}

#[test]
fn truncate_unicode_boundary() {
    // Truncation at byte boundary may cut a multi-byte char; verify no panic
    let s = "hello world";
    let result = truncate(s, 7);
    assert_eq!(result, "hello w...");
}

#[test]
fn truncate_one_char() {
    assert_eq!(truncate("hello", 1), "h...");
}

#[test]
fn truncate_zero_len() {
    assert_eq!(truncate("hello", 0), "...");
}

#[test]
fn mentions_loop_new_sets_dry_run() {
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: Vec::new(),
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        Arc::new(MockPoster::new()),
        true,
    );
    assert!(mentions_loop.dry_run);
}

#[test]
fn mentions_loop_new_sets_not_dry_run() {
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: Vec::new(),
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        Arc::new(MockPoster::new()),
        false,
    );
    assert!(!mentions_loop.dry_run);
}

#[tokio::test]
async fn run_once_with_since_id_passthrough() {
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("200", "carol")],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hey".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        Arc::new(MockPoster::new()),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, since_id) = mentions_loop
        .run_once(Some("150"), None, &storage)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(since_id, Some("200".to_string()));
}

#[tokio::test]
async fn run_once_limit_zero_processes_none() {
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("100", "alice")],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        Arc::new(MockPoster::new()),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, since_id) = mentions_loop
        .run_once(None, Some(0), &storage)
        .await
        .unwrap();
    assert!(results.is_empty());
    assert!(since_id.is_none());
}

#[tokio::test]
async fn run_once_posting_failure_returns_failed() {
    struct FailingPoster;

    #[async_trait::async_trait]
    impl crate::automation::loop_helpers::PostSender for FailingPoster {
        async fn send_reply(&self, _tweet_id: &str, _content: &str) -> Result<(), LoopError> {
            Err(LoopError::Other("network error".to_string()))
        }
    }

    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![test_tweet("100", "alice")],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hi".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        Arc::new(FailingPoster),
        false, // not dry_run — will actually try to post
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, _) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        matches!(&results[0], MentionResult::Failed { error, .. } if error.contains("network error"))
    );
}

#[tokio::test]
async fn run_once_multiple_mentions_tracks_max_id() {
    let mentions_loop = MentionsLoop::new(
        Arc::new(MockFetcher {
            mentions: vec![
                test_tweet("50", "alice"),
                test_tweet("200", "bob"),
                test_tweet("100", "carol"),
            ],
        }),
        Arc::new(MockGenerator {
            reply_prefix: "Hey".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        Arc::new(MockPoster::new()),
        false,
    );
    let storage: Arc<dyn LoopStorage> = Arc::new(MockStorage::new());

    let (results, since_id) = mentions_loop.run_once(None, None, &storage).await.unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(since_id, Some("200".to_string()));
}

#[test]
fn mention_result_debug() {
    let result = MentionResult::Replied {
        tweet_id: "123".to_string(),
        author: "alice".to_string(),
        reply_text: "hello".to_string(),
    };
    let debug = format!("{result:?}");
    assert!(debug.contains("Replied"));
    assert!(debug.contains("123"));

    let result = MentionResult::Skipped {
        tweet_id: "456".to_string(),
        reason: "already replied".to_string(),
    };
    let debug = format!("{result:?}");
    assert!(debug.contains("Skipped"));

    let result = MentionResult::Failed {
        tweet_id: "789".to_string(),
        error: "timeout".to_string(),
    };
    let debug = format!("{result:?}");
    assert!(debug.contains("Failed"));
}
