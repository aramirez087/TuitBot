use std::sync::Arc;

use super::*;
use crate::automation::loop_helpers::{ContentSafety, SafetyChecker};
use crate::config::{IntervalsConfig, LimitsConfig};
use crate::safety::SafetyGuard;
use crate::storage::{init_test_db, rate_limits, DbPool};

fn test_limits() -> LimitsConfig {
    LimitsConfig {
        max_replies_per_day: 3,
        max_tweets_per_day: 2,
        max_threads_per_week: 1,
        min_action_delay_seconds: 30,
        max_action_delay_seconds: 120,
        max_replies_per_author_per_day: 1,
        banned_phrases: vec![],
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

async fn setup() -> (DbPool, Arc<SafetyGuard>) {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_rate_limits(&pool, &test_limits(), &test_intervals())
        .await
        .expect("init rate limits");
    let guard = Arc::new(SafetyGuard::new(pool.clone()));
    (pool, guard)
}

// ============================================================================
// SafetyAdapter (SafetyChecker)
// ============================================================================

#[tokio::test]
async fn safety_can_reply_under_limit() {
    let (pool, guard) = setup().await;
    let adapter = SafetyAdapter::new(guard, pool);

    assert!(adapter.can_reply().await);
}

#[tokio::test]
async fn safety_has_replied_to_false() {
    let (pool, guard) = setup().await;
    let adapter = SafetyAdapter::new(guard, pool);

    assert!(!adapter.has_replied_to("tweet_999").await);
}

#[tokio::test]
async fn safety_has_replied_to_true() {
    let (pool, guard) = setup().await;
    let adapter = SafetyAdapter::new(guard, pool);

    adapter
        .record_reply("tweet_42", "Great insight!")
        .await
        .unwrap();

    assert!(adapter.has_replied_to("tweet_42").await);
}

#[tokio::test]
async fn safety_record_reply_inserts() {
    let (pool, guard) = setup().await;
    let adapter = SafetyAdapter::new(guard, pool);

    let result = adapter.record_reply("tweet_7", "Nice thread!").await;
    assert!(result.is_ok());

    // Verify it was recorded by checking dedup.
    assert!(adapter.has_replied_to("tweet_7").await);
}

// ============================================================================
// ContentSafetyAdapter (ContentSafety)
// ============================================================================

#[tokio::test]
async fn content_safety_can_post_tweet() {
    let (_pool, guard) = setup().await;
    let adapter = ContentSafetyAdapter::new(guard);

    assert!(adapter.can_post_tweet().await);
}

#[tokio::test]
async fn content_safety_can_post_thread() {
    let (_pool, guard) = setup().await;
    let adapter = ContentSafetyAdapter::new(guard);

    assert!(adapter.can_post_thread().await);
}
