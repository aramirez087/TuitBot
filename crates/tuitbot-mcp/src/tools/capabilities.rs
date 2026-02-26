//! Capabilities tool: get_capabilities.
//!
//! Reports the detected API tier, boolean capability flags, rate-limit
//! remaining counts, and recommended max actions so agents can plan
//! before taking actions.

use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

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
    direct_tools: DirectToolsMap,
}

#[derive(Serialize)]
struct DirectToolsMap {
    x_client_available: bool,
    authenticated_user_id: Option<String>,
    tools: Vec<DirectToolEntry>,
}

#[derive(Serialize)]
struct DirectToolEntry {
    name: String,
    available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
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
pub async fn get_capabilities(
    pool: &DbPool,
    config: &Config,
    llm_available: bool,
    x_available: bool,
    user_id: Option<&str>,
) -> String {
    let start = Instant::now();

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

    // 5. Build direct tools availability map.
    let has_user_id = user_id.is_some();
    let not_configured = "X API client not configured. Run `tuitbot auth` to authenticate.";
    let no_user_id = "Authenticated user ID not available.";
    let needs_search = "Requires Basic or Pro tier for search.";

    let mut direct_tools_entries = Vec::new();

    // Read tools
    let read_tools = [
        ("get_tweet_by_id", x_available, None),
        ("x_get_user_by_username", x_available, None),
        (
            "x_search_tweets",
            x_available && can_search,
            if !x_available {
                Some(not_configured)
            } else if !can_search {
                Some(needs_search)
            } else {
                None
            },
        ),
        (
            "x_get_user_mentions",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
        ("x_get_user_tweets", x_available, None),
        ("x_get_followers", x_available, None),
        ("x_get_following", x_available, None),
        ("x_get_user_by_id", x_available, None),
        ("x_get_liked_tweets", x_available, None),
        (
            "x_get_bookmarks",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
        ("x_get_users_by_ids", x_available, None),
        ("x_get_tweet_liking_users", x_available, None),
    ];

    // Mutation tools
    let mutation_tools = [
        ("x_post_tweet", x_available, None),
        ("x_reply_to_tweet", x_available, None),
        ("x_quote_tweet", x_available, None),
        (
            "x_like_tweet",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
        (
            "x_unlike_tweet",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
        (
            "x_follow_user",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
        (
            "x_unfollow_user",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
        (
            "x_bookmark_tweet",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
        (
            "x_unbookmark_tweet",
            x_available && has_user_id,
            if !x_available {
                Some(not_configured)
            } else if !has_user_id {
                Some(no_user_id)
            } else {
                None
            },
        ),
    ];

    for (name, available, reason) in read_tools.iter().chain(mutation_tools.iter()) {
        direct_tools_entries.push(DirectToolEntry {
            name: name.to_string(),
            available: *available,
            reason: if *available {
                None
            } else {
                Some(reason.unwrap_or(not_configured).to_string())
            },
        });
    }

    let direct_tools = DirectToolsMap {
        x_client_available: x_available,
        authenticated_user_id: user_id.map(|s| s.to_string()),
        tools: direct_tools_entries,
    };

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
        direct_tools,
    };

    let elapsed = start.elapsed().as_millis() as u64;
    let meta =
        ToolMeta::new(elapsed).with_mode(config.mode.to_string(), config.effective_approval_mode());

    ToolResponse::success(out).with_meta(meta).to_json()
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
        let result = get_capabilities(&pool, &config, true, false, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["tier"], "unknown");
        assert_eq!(parsed["data"]["can_post_tweets"], true);
        assert_eq!(parsed["data"]["llm_available"], true);
    }

    #[tokio::test]
    async fn capabilities_reflects_persisted_tier() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Basic")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, false, true, Some("u1")).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["tier"], "Basic");
        assert_eq!(parsed["data"]["can_reply"], true);
        assert_eq!(parsed["data"]["can_search"], true);
        assert_eq!(parsed["data"]["can_discover"], true);
        assert_eq!(parsed["data"]["llm_available"], false);
    }

    #[tokio::test]
    async fn capabilities_free_tier_flags() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Free")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, false, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["can_reply"], false);
        assert_eq!(parsed["data"]["can_search"], false);
        assert_eq!(parsed["data"]["can_discover"], false);
    }

    #[tokio::test]
    async fn capabilities_includes_rate_limits() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, false, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        let rate_limits = parsed["data"]["rate_limits"]
            .as_array()
            .expect("rate_limits array");
        assert_eq!(rate_limits.len(), 5);
        assert_eq!(parsed["data"]["recommended_max_actions"]["replies"], 5);
        assert_eq!(parsed["data"]["recommended_max_actions"]["tweets"], 6);
        assert_eq!(parsed["data"]["recommended_max_actions"]["threads"], 1);
    }

    #[tokio::test]
    async fn direct_tools_all_unavailable_when_no_x_client() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, false, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let dt = &parsed["data"]["direct_tools"];
        assert_eq!(dt["x_client_available"], false);
        assert!(dt["authenticated_user_id"].is_null());
        let tools = dt["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 21);
        for tool in tools {
            assert_eq!(tool["available"], false);
            assert!(tool["reason"].is_string());
        }
    }

    #[tokio::test]
    async fn direct_tools_all_available_with_x_client_and_user() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Basic")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1")).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let dt = &parsed["data"]["direct_tools"];
        assert_eq!(dt["x_client_available"], true);
        assert_eq!(dt["authenticated_user_id"], "u1");
        let tools = dt["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 21);
        for tool in tools {
            assert_eq!(
                tool["available"], true,
                "tool {} should be available",
                tool["name"]
            );
        }
    }

    #[tokio::test]
    async fn direct_tools_search_unavailable_on_free_tier() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Free")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1")).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let tools = parsed["data"]["direct_tools"]["tools"]
            .as_array()
            .expect("tools array");
        let search = tools
            .iter()
            .find(|t| t["name"] == "x_search_tweets")
            .expect("find search tool");
        assert_eq!(search["available"], false);
        assert!(search["reason"].as_str().unwrap().contains("Basic or Pro"));
    }
}
