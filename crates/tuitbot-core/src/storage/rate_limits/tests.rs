//! Tests for rate limit tracking and enforcement.

use super::*;
use crate::storage::init_test_db;

fn test_limits_config() -> crate::config::LimitsConfig {
    crate::config::LimitsConfig {
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

fn test_intervals_config() -> crate::config::IntervalsConfig {
    crate::config::IntervalsConfig {
        mentions_check_seconds: 60,
        discovery_search_seconds: 300,
        content_post_window_seconds: 600,
        thread_interval_seconds: 900,
    }
}

#[tokio::test]
async fn init_rate_limits_inserts_rows() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    init_rate_limits(&pool, &config, &intervals)
        .await
        .expect("init");

    let limits = get_all_rate_limits(&pool).await.expect("get");
    assert_eq!(limits.len(), 5);
    assert_eq!(limits[0].action_type, "mention_check");
    assert_eq!(limits[0].max_requests, 180);
    assert_eq!(limits[0].period_seconds, 900);
    assert_eq!(limits[4].action_type, "tweet");
    assert_eq!(limits[4].max_requests, 2);
}

#[tokio::test]
async fn check_rate_limit_under_limit() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    init_rate_limits(&pool, &config, &intervals)
        .await
        .expect("init");

    let under_limit = check_rate_limit(&pool, "reply").await.expect("check");
    assert!(under_limit);
}

#[tokio::test]
async fn check_rate_limit_over_limit() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    init_rate_limits(&pool, &config, &intervals)
        .await
        .expect("init");

    // Set counter to max
    sqlx::query("UPDATE rate_limits SET request_count = 3 WHERE action_type = 'reply'")
        .execute(&pool)
        .await
        .expect("update");

    let over_limit = check_rate_limit(&pool, "reply").await.expect("check");
    assert!(!over_limit);
}

#[tokio::test]
async fn increment_rate_limit_increments() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    init_rate_limits(&pool, &config, &intervals)
        .await
        .expect("init");

    increment_rate_limit(&pool, "reply")
        .await
        .expect("increment");

    let limits = get_all_rate_limits(&pool).await.expect("get");
    let reply = limits
        .iter()
        .find(|l| l.action_type == "reply")
        .expect("reply");
    assert_eq!(reply.request_count, 1);
}

#[tokio::test]
async fn check_and_increment_rate_limit_under_limit() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    init_rate_limits(&pool, &config, &intervals)
        .await
        .expect("init");

    let allowed = check_and_increment_rate_limit(&pool, "reply")
        .await
        .expect("check");
    assert!(allowed);

    let limits = get_all_rate_limits(&pool).await.expect("get");
    let reply = limits
        .iter()
        .find(|l| l.action_type == "reply")
        .expect("reply");
    assert_eq!(reply.request_count, 1);
}

#[tokio::test]
async fn check_and_increment_rate_limit_over_limit() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    init_rate_limits(&pool, &config, &intervals)
        .await
        .expect("init");

    // Set counter to max
    sqlx::query("UPDATE rate_limits SET request_count = 3 WHERE action_type = 'reply'")
        .execute(&pool)
        .await
        .expect("update");

    let allowed = check_and_increment_rate_limit(&pool, "reply")
        .await
        .expect("check");
    assert!(!allowed);

    let limits = get_all_rate_limits(&pool).await.expect("get");
    let reply = limits
        .iter()
        .find(|l| l.action_type == "reply")
        .expect("reply");
    assert_eq!(
        reply.request_count, 3,
        "should not increment when over limit"
    );
}

#[tokio::test]
async fn get_daily_usage_summary() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    init_rate_limits(&pool, &config, &intervals)
        .await
        .expect("init");

    // Increment some counters
    increment_rate_limit(&pool, "reply").await.expect("inc");
    increment_rate_limit(&pool, "reply").await.expect("inc");
    increment_rate_limit(&pool, "tweet").await.expect("inc");

    let usage = get_daily_usage(&pool).await.expect("usage");
    assert_eq!(usage.replies.used, 2);
    assert_eq!(usage.replies.max, 3);
    assert_eq!(usage.tweets.used, 1);
    assert_eq!(usage.tweets.max, 2);
    assert_eq!(usage.threads.used, 0);
    assert_eq!(usage.threads.max, 1);
}

#[tokio::test]
async fn check_and_increment_rate_limit_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let config = test_limits_config();
    let intervals = test_intervals_config();

    let account_a = "account-a";
    let account_b = "account-b";

    crate::storage::accounts::create_account(&pool, account_a, "Account A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, account_b, "Account B")
        .await
        .expect("create b");

    init_rate_limits_for(&pool, account_a, &config, &intervals)
        .await
        .expect("init a");
    init_rate_limits_for(&pool, account_b, &config, &intervals)
        .await
        .expect("init b");

    // Max out account_a
    for _ in 0..3 {
        check_and_increment_rate_limit_for(&pool, account_a, "reply")
            .await
            .expect("check");
    }

    // account_b should still be under limit
    let allowed_b = check_and_increment_rate_limit_for(&pool, account_b, "reply")
        .await
        .expect("check");
    assert!(allowed_b);

    let limits_b = get_all_rate_limits_for(&pool, account_b)
        .await
        .expect("get b");
    let reply_b = limits_b
        .iter()
        .find(|l| l.action_type == "reply")
        .expect("reply");
    assert_eq!(reply_b.request_count, 1);
}

#[tokio::test]
async fn init_mcp_rate_limit() {
    let pool = init_test_db().await.expect("init db");

    super::init_mcp_rate_limit(&pool, 50).await.expect("init");

    let limits = get_all_rate_limits(&pool).await.expect("get");
    let mcp = limits
        .iter()
        .find(|l| l.action_type == "mcp_mutation")
        .expect("mcp");
    assert_eq!(mcp.max_requests, 50);
    assert_eq!(mcp.period_seconds, 3600);
}

#[tokio::test]
async fn policy_rate_limits_basic() {
    let pool = init_test_db().await.expect("init db");
    let limits = vec![
        PolicyRateLimit {
            key: "tool:like:hourly".to_string(),
            dimension: RateLimitDimension::Tool,
            match_value: "like_tweet".to_string(),
            max_count: 10,
            period_seconds: 3600,
        },
        PolicyRateLimit {
            key: "global:hourly".to_string(),
            dimension: RateLimitDimension::Global,
            match_value: String::new(),
            max_count: 20,
            period_seconds: 3600,
        },
    ];

    init_policy_rate_limits(&pool, &limits).await.expect("init");

    // All should pass initially
    let exceeded = check_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
        .await
        .expect("check");
    assert!(exceeded.is_none());

    // Record and re-check
    record_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
        .await
        .expect("record");

    let all = get_all_rate_limits(&pool).await.expect("get");
    let tool = all
        .iter()
        .find(|l| l.action_type == "tool:like:hourly")
        .expect("tool");
    assert_eq!(tool.request_count, 1);
    let global = all
        .iter()
        .find(|l| l.action_type == "global:hourly")
        .expect("global");
    assert_eq!(global.request_count, 1);
}

#[tokio::test]
async fn policy_rate_limits_engagement_type_dimension() {
    let pool = init_test_db().await.expect("init db");
    let limits = vec![PolicyRateLimit {
        key: "engagement_type:like:hourly".to_string(),
        dimension: RateLimitDimension::EngagementType,
        match_value: "like_tweet".to_string(),
        max_count: 1,
        period_seconds: 3600,
    }];

    init_policy_rate_limits(&pool, &limits).await.expect("init");
    increment_rate_limit(&pool, "engagement_type:like:hourly")
        .await
        .expect("inc");

    let exceeded = check_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
        .await
        .expect("check");
    assert_eq!(exceeded, Some("engagement_type:like:hourly".to_string()));
}

#[tokio::test]
async fn policy_rate_limits_record_only_matching() {
    let pool = init_test_db().await.expect("init db");
    let limits = vec![
        PolicyRateLimit {
            key: "tool:follow:hourly".to_string(),
            dimension: RateLimitDimension::Tool,
            match_value: "follow_user".to_string(),
            max_count: 5,
            period_seconds: 3600,
        },
        PolicyRateLimit {
            key: "tool:like:hourly".to_string(),
            dimension: RateLimitDimension::Tool,
            match_value: "like_tweet".to_string(),
            max_count: 5,
            period_seconds: 3600,
        },
    ];

    init_policy_rate_limits(&pool, &limits).await.expect("init");

    // Record only for like_tweet
    record_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
        .await
        .expect("record");

    let all = get_all_rate_limits(&pool).await.expect("get");
    let follow = all
        .iter()
        .find(|l| l.action_type == "tool:follow:hourly")
        .expect("follow");
    assert_eq!(follow.request_count, 0, "follow should not be incremented");
    let like = all
        .iter()
        .find(|l| l.action_type == "tool:like:hourly")
        .expect("like");
    assert_eq!(like.request_count, 1);
}
