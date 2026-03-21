//! Core discovery loop tests: search_and_process, process_tweet.

use super::*;

#[tokio::test]
async fn search_and_process_no_results() {
    let (discovery, poster, _) = build_loop(Vec::new(), 80.0, true, false);
    let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();
    assert_eq!(summary.tweets_found, 0);
    assert!(results.is_empty());
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn search_and_process_above_threshold() {
    let tweets = vec![test_tweet("100", "alice"), test_tweet("101", "bob")];
    let (discovery, poster, storage) = build_loop(tweets, 85.0, true, false);

    let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();

    assert_eq!(summary.tweets_found, 2);
    assert_eq!(summary.replied, 2);
    assert_eq!(results.len(), 2);
    assert_eq!(poster.sent_count(), 2);

    // Both tweets should be stored as discovered
    let discovered = storage.discovered.lock().expect("lock");
    assert_eq!(discovered.len(), 2);
}

#[tokio::test]
async fn search_and_process_below_threshold() {
    let tweets = vec![test_tweet("100", "alice")];
    let (discovery, poster, storage) = build_loop(tweets, 40.0, false, false);

    let (results, summary) = discovery.search_and_process("rust", None).await.unwrap();

    assert_eq!(summary.tweets_found, 1);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.replied, 0);
    assert_eq!(results.len(), 1);
    assert_eq!(poster.sent_count(), 0);

    // Tweet should still be stored as discovered (for analytics)
    let discovered = storage.discovered.lock().expect("lock");
    assert_eq!(discovered.len(), 1);
}

#[tokio::test]
async fn search_and_process_dry_run() {
    let tweets = vec![test_tweet("100", "alice")];
    let (discovery, poster, _) = build_loop(tweets, 85.0, true, true);

    let (_results, summary) = discovery.search_and_process("rust", None).await.unwrap();

    assert_eq!(summary.replied, 1);
    // Should NOT post in dry-run
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn search_and_process_skips_existing() {
    let tweets = vec![test_tweet("100", "alice")];
    let poster = Arc::new(MockPoster::new());
    let storage = Arc::new(MockStorage::new());
    // Pre-mark tweet as existing
    storage
        .existing_ids
        .lock()
        .expect("lock")
        .push("100".to_string());

    let discovery = DiscoveryLoop::new(
        Arc::new(MockSearcher { results: tweets }),
        Arc::new(MockScorer {
            score: 85.0,
            meets_threshold: true,
        }),
        Arc::new(MockGenerator {
            reply: "Great!".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        storage,
        poster.clone(),
        vec!["rust".to_string()],
        70.0,
        false,
    );

    let (_results, summary) = discovery.search_and_process("rust", None).await.unwrap();
    assert_eq!(summary.skipped, 1);
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn search_and_process_respects_limit() {
    let tweets = vec![
        test_tweet("100", "alice"),
        test_tweet("101", "bob"),
        test_tweet("102", "carol"),
    ];
    let (discovery, poster, _) = build_loop(tweets, 85.0, true, false);

    let (results, summary) = discovery.search_and_process("rust", Some(2)).await.unwrap();

    assert_eq!(summary.tweets_found, 3); // found 3, but...
    assert_eq!(results.len(), 2); // only 2 results returned
    assert_eq!(poster.sent_count(), 2); // only processed 2
}

#[tokio::test]
async fn run_once_searches_all_keywords() {
    let tweets = vec![test_tweet("100", "alice")];
    let (discovery, _, _) = build_loop(tweets, 85.0, true, false);

    let (_, summary) = discovery.run_once(None).await.unwrap();
    // Should search both "rust" and "cli" keywords
    assert_eq!(summary.tweets_found, 2); // 1 tweet per keyword
}

#[test]
fn discovery_summary_default() {
    let s = DiscoverySummary::default();
    assert_eq!(s.tweets_found, 0);
    assert_eq!(s.qualifying, 0);
    assert_eq!(s.replied, 0);
    assert_eq!(s.skipped, 0);
    assert_eq!(s.failed, 0);
}

#[test]
fn discovery_result_debug() {
    let r = DiscoveryResult::Replied {
        tweet_id: "1".to_string(),
        author: "alice".to_string(),
        score: 85.0,
        reply_text: "Great!".to_string(),
    };
    let debug = format!("{r:?}");
    assert!(debug.contains("Replied"));

    let r = DiscoveryResult::BelowThreshold {
        tweet_id: "2".to_string(),
        score: 30.0,
    };
    let debug = format!("{r:?}");
    assert!(debug.contains("BelowThreshold"));

    let r = DiscoveryResult::Skipped {
        tweet_id: "3".to_string(),
        reason: "test".to_string(),
    };
    assert!(format!("{r:?}").contains("Skipped"));

    let r = DiscoveryResult::Failed {
        tweet_id: "4".to_string(),
        error: "boom".to_string(),
    };
    assert!(format!("{r:?}").contains("Failed"));
}

#[test]
fn truncate_exact_length() {
    assert_eq!(truncate("hello", 5), "hello");
}

#[test]
fn truncate_empty_string() {
    assert_eq!(truncate("", 10), "");
}

#[tokio::test]
async fn run_once_all_keywords_fail_returns_error() {
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
        vec!["rust".to_string(), "cli".to_string()],
        70.0,
        false,
    );

    let result = discovery.run_once(None).await;
    assert!(result.is_err());
}
