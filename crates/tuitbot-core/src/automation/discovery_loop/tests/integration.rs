//! Integration tests: run_once, error handling, edge cases.

use super::*;

#[tokio::test]
async fn search_error_returns_loop_error() {
    let poster = Arc::new(MockPoster::new());
    let storage = Arc::new(MockStorage::new());
    let discovery = DiscoveryLoop::new(
        Arc::new(FailingSearcher),
        Arc::new(MockScorer {
            score: 85.0,
            meets_threshold: true,
        }),
        Arc::new(MockGenerator {
            reply: "test".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        storage,
        poster,
        vec!["rust".to_string()],
        70.0,
        false,
    );

    let result = discovery.search_and_process("rust", None).await;
    assert!(result.is_err());
}

// ── Additional coverage tests ────────────────────────────────────

#[test]
fn truncate_long_string() {
    let result = truncate("hello world, this is a long string", 10);
    assert_eq!(result, "hello worl...");
}

#[test]
fn truncate_one_char() {
    assert_eq!(truncate("x", 1), "x");
}

#[test]
fn truncate_zero_max() {
    assert_eq!(truncate("hello", 0), "...");
}

#[tokio::test]
async fn search_and_process_rate_limited_safety_skips() {
    // Safety checker says can_reply=false, so tweet should be skipped
    let tweets = vec![test_tweet("200", "dave")];
    let poster = Arc::new(MockPoster::new());
    let storage = Arc::new(MockStorage::new());
    let discovery = DiscoveryLoop::new(
        Arc::new(MockSearcher { results: tweets }),
        Arc::new(MockScorer {
            score: 90.0,
            meets_threshold: true,
        }),
        Arc::new(MockGenerator {
            reply: "Great!".to_string(),
        }),
        Arc::new(MockSafety::new(false)), // can_reply = false
        storage,
        poster.clone(),
        vec!["rust".to_string()],
        70.0,
        false,
    );

    let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();
    assert_eq!(summary.tweets_found, 1);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.replied, 0);
    assert_eq!(poster.sent_count(), 0);
    assert!(matches!(results[0], DiscoveryResult::Skipped { .. }));
}

#[tokio::test]
async fn run_once_with_limit() {
    let tweets = vec![
        test_tweet("300", "alice"),
        test_tweet("301", "bob"),
        test_tweet("302", "carol"),
    ];
    let (discovery, poster, _) = build_loop(tweets, 85.0, true, false);

    let (_, summary) = discovery.run_once(Some(2)).await.unwrap();
    // Should stop after processing 2 total across keywords
    assert!(summary.tweets_found <= 3);
    assert!(poster.sent_count() <= 2);
}

#[tokio::test]
async fn run_once_empty_keywords() {
    let poster = Arc::new(MockPoster::new());
    let storage = Arc::new(MockStorage::new());
    let discovery = DiscoveryLoop::new(
        Arc::new(MockSearcher {
            results: Vec::new(),
        }),
        Arc::new(MockScorer {
            score: 85.0,
            meets_threshold: true,
        }),
        Arc::new(MockGenerator {
            reply: "test".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        storage,
        poster,
        Vec::new(), // no keywords
        70.0,
        false,
    );

    let (results, summary) = discovery.run_once(None).await.unwrap();
    assert_eq!(summary.tweets_found, 0);
    assert!(results.is_empty());
}

// ── FailingGenerator ─────────────────────────────────────────────

struct FailingGenerator;

#[async_trait::async_trait]
impl ReplyGenerator for FailingGenerator {
    async fn generate_reply(
        &self,
        _tweet_text: &str,
        _author: &str,
        _mention_product: bool,
    ) -> Result<String, LoopError> {
        Err(LoopError::LlmFailure("LLM error".into()))
    }
}

#[tokio::test]
async fn process_tweet_generation_failure_returns_failed() {
    let tweets = vec![test_tweet("400", "eve")];
    let poster = Arc::new(MockPoster::new());
    let storage = Arc::new(MockStorage::new());
    let discovery = DiscoveryLoop::new(
        Arc::new(MockSearcher { results: tweets }),
        Arc::new(MockScorer {
            score: 90.0,
            meets_threshold: true,
        }),
        Arc::new(FailingGenerator),
        Arc::new(MockSafety::new(true)),
        storage,
        poster.clone(),
        vec!["rust".to_string()],
        70.0,
        false,
    );

    let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();
    assert_eq!(summary.failed, 1);
    assert_eq!(poster.sent_count(), 0);
    assert!(matches!(results[0], DiscoveryResult::Failed { .. }));
}

// ── FailingPoster ────────────────────────────────────────────────

struct FailingPoster;

#[async_trait::async_trait]
impl PostSender for FailingPoster {
    async fn send_reply(&self, _tweet_id: &str, _content: &str) -> Result<(), LoopError> {
        Err(LoopError::NetworkError("API error".into()))
    }
}

#[tokio::test]
async fn process_tweet_post_failure_returns_failed() {
    let tweets = vec![test_tweet("500", "frank")];
    let storage = Arc::new(MockStorage::new());
    let discovery = DiscoveryLoop::new(
        Arc::new(MockSearcher { results: tweets }),
        Arc::new(MockScorer {
            score: 90.0,
            meets_threshold: true,
        }),
        Arc::new(MockGenerator {
            reply: "Great!".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        storage,
        Arc::new(FailingPoster),
        vec!["rust".to_string()],
        70.0,
        false,
    );

    let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();
    assert_eq!(summary.failed, 1);
    assert!(matches!(results[0], DiscoveryResult::Failed { .. }));
}
