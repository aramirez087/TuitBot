//! Capabilities tool: get_capabilities.
//!
//! Reports the detected API tier, boolean capability flags, rate-limit
//! remaining counts, and recommended max actions so agents can plan
//! before taking actions.

use chrono::{DateTime, Utc};
use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

#[derive(Serialize)]
struct Capabilities {
    tier: String,
    tier_detected_at: Option<String>,
    can_post_tweets: bool,
    can_reply: bool,
    can_search: bool,
    can_discover: bool,
    approval_mode: bool,
    llm_available: bool,
    rate_limits: Vec<RateLimitEntry>,
    recommended_max_actions: RecommendedMax,
}

#[derive(Serialize)]
struct RateLimitEntry {
    action_type: String,
    remaining: i64,
    max: i64,
    period_seconds: i64,
}

#[derive(Serialize)]
struct RecommendedMax {
    replies: i64,
    tweets: i64,
    threads: i64,
}

/// Build a capabilities JSON response.
pub async fn get_capabilities(pool: &DbPool, config: &Config, llm_available: bool) -> String {
    // 1. Read persisted tier and its timestamp.
    let (tier_str, tier_detected_at) =
        match storage::cursors::get_cursor_with_timestamp(pool, "api_tier").await {
            Ok(Some((value, ts))) => (value, Some(ts)),
            _ => ("unknown".to_string(), None),
        };

    // 3. Derive capabilities from tier.
    let tier_lower = tier_str.to_lowercase();
    let can_post_tweets = true;
    let can_reply = tier_lower != "free";
    let can_search = tier_lower == "basic" || tier_lower == "pro";
    let can_discover = can_search;

    // 4. Read rate limits and compute remaining (accounting for period expiry).
    let mut rate_entries = Vec::new();
    let mut reply_remaining: i64 = 0;
    let mut tweet_remaining: i64 = 0;
    let mut thread_remaining: i64 = 0;

    if let Ok(limits) = storage::rate_limits::get_all_rate_limits(pool).await {
        let now = Utc::now();
        for limit in limits {
            let period_start = limit.period_start.parse::<DateTime<Utc>>().unwrap_or(now);
            let elapsed = now.signed_duration_since(period_start).num_seconds();
            let remaining = if elapsed >= limit.period_seconds {
                // Period expired — full quota available.
                limit.max_requests
            } else {
                (limit.max_requests - limit.request_count).max(0)
            };

            match limit.action_type.as_str() {
                "reply" => reply_remaining = remaining,
                "tweet" => tweet_remaining = remaining,
                "thread" => thread_remaining = remaining,
                _ => {}
            }

            rate_entries.push(RateLimitEntry {
                action_type: limit.action_type,
                remaining,
                max: limit.max_requests,
                period_seconds: limit.period_seconds,
            });
        }
    }

    let out = Capabilities {
        tier: tier_str,
        tier_detected_at,
        can_post_tweets,
        can_reply,
        can_search,
        can_discover,
        approval_mode: config.approval_mode,
        llm_available,
        rate_limits: rate_entries,
        recommended_max_actions: RecommendedMax {
            replies: reply_remaining,
            tweets: tweet_remaining,
            threads: thread_remaining,
        },
    };

    serde_json::to_string_pretty(&out)
        .unwrap_or_else(|e| format!("Error serializing capabilities: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuitbot_core::config::Config;
    use tuitbot_core::storage;

    async fn setup_db() -> DbPool {
        let pool = storage::init_test_db().await.expect("init db");
        // Seed rate limits with test config.
        let limits = tuitbot_core::config::LimitsConfig {
            max_replies_per_day: 5,
            max_tweets_per_day: 6,
            max_threads_per_week: 1,
            min_action_delay_seconds: 30,
            max_action_delay_seconds: 120,
            max_replies_per_author_per_day: 1,
            banned_phrases: vec![],
            product_mention_ratio: 0.2,
        };
        let intervals = tuitbot_core::config::IntervalsConfig {
            mentions_check_seconds: 300,
            discovery_search_seconds: 600,
            content_post_window_seconds: 14400,
            thread_interval_seconds: 604800,
        };
        storage::rate_limits::init_rate_limits(&pool, &limits, &intervals)
            .await
            .expect("init rate limits");
        pool
    }

    #[tokio::test]
    async fn capabilities_returns_valid_json() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["tier"], "unknown");
        assert_eq!(parsed["can_post_tweets"], true);
        assert_eq!(parsed["llm_available"], true);
    }

    #[tokio::test]
    async fn capabilities_reflects_persisted_tier() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Basic")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["tier"], "Basic");
        assert_eq!(parsed["can_reply"], true);
        assert_eq!(parsed["can_search"], true);
        assert_eq!(parsed["can_discover"], true);
        assert_eq!(parsed["llm_available"], false);
    }

    #[tokio::test]
    async fn capabilities_free_tier_flags() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Free")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["can_reply"], false);
        assert_eq!(parsed["can_search"], false);
        assert_eq!(parsed["can_discover"], false);
    }

    #[tokio::test]
    async fn capabilities_includes_rate_limits() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let rate_limits = parsed["rate_limits"].as_array().expect("rate_limits array");
        assert_eq!(rate_limits.len(), 5);
        assert_eq!(parsed["recommended_max_actions"]["replies"], 5);
        assert_eq!(parsed["recommended_max_actions"]["tweets"], 6);
        assert_eq!(parsed["recommended_max_actions"]["threads"], 1);
    }
}
