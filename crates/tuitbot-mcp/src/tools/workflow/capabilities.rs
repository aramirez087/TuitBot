//! Capabilities tool: get_capabilities.
//!
//! Reports the detected API tier, boolean capability flags, rate-limit
//! remaining counts, recommended max actions, scope analysis, endpoint
//! group availability, and actionable guidance so agents can plan
//! before taking actions.

use std::collections::BTreeSet;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;
use tuitbot_core::x_api::scopes::{self, ScopeAnalysis};

use crate::provider::{self, capabilities::ProviderCapabilities};
use crate::spec::SPEC_ENDPOINTS;
use crate::tools::response::{ToolMeta, ToolResponse};

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
    auth: AuthInfo,
    scope_analysis: Option<ScopeAnalysis>,
    endpoint_groups: Vec<EndpointGroupStatus>,
    rate_limits: Vec<RateLimitEntry>,
    recommended_max_actions: RecommendedMax,
    direct_tools: DirectToolsMap,
    provider: ProviderCapabilities,
    guidance: Vec<String>,
}

#[derive(Serialize)]
struct AuthInfo {
    mode: &'static str,
    x_client_available: bool,
    authenticated_user_id: Option<String>,
    token_scopes_available: bool,
}

#[derive(Serialize)]
struct EndpointGroupStatus {
    group: String,
    total_endpoints: usize,
    available_endpoints: usize,
    required_scopes: Vec<String>,
    missing_scopes: Vec<String>,
    fully_available: bool,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    requires_scopes: Vec<String>,
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

/// Build endpoint group status from spec metadata and granted scopes.
fn compute_endpoint_groups(granted: &BTreeSet<String>) -> Vec<EndpointGroupStatus> {
    // Collect unique groups from spec endpoints.
    let mut groups: std::collections::BTreeMap<String, (BTreeSet<String>, usize)> =
        std::collections::BTreeMap::new();

    for ep in SPEC_ENDPOINTS {
        let entry = groups
            .entry(ep.group.to_string())
            .or_insert_with(|| (BTreeSet::new(), 0));
        for scope in ep.scopes {
            entry.0.insert((*scope).to_string());
        }
        entry.1 += 1;
    }

    groups
        .into_iter()
        .map(|(group, (required_scopes, total))| {
            let missing: Vec<String> = required_scopes
                .iter()
                .filter(|s| !granted.contains(s.as_str()))
                .cloned()
                .collect();
            let available = if missing.is_empty() { total } else { 0 };
            EndpointGroupStatus {
                group,
                total_endpoints: total,
                available_endpoints: available,
                required_scopes: required_scopes.into_iter().collect(),
                missing_scopes: missing.clone(),
                fully_available: missing.is_empty(),
            }
        })
        .collect()
}

/// Generate actionable guidance based on current state.
fn compute_guidance(
    x_available: bool,
    has_user_id: bool,
    scopes_available: bool,
    scope_analysis: Option<&ScopeAnalysis>,
    tier: &str,
    llm_available: bool,
) -> Vec<String> {
    let mut guidance = Vec::new();

    if !x_available {
        guidance.push(
            "X API client not configured. Run `tuitbot auth` to authenticate with OAuth 2.0."
                .to_string(),
        );
        return guidance;
    }

    if !has_user_id {
        guidance.push(
            "Authenticated user ID not available. Token may be valid but get_me() failed. \
             Check network connectivity or re-authenticate with `tuitbot auth`."
                .to_string(),
        );
    }

    if !scopes_available {
        guidance.push(
            "OAuth scopes not available for analysis. Token was loaded without scope metadata. \
             Re-authenticate with `tuitbot auth` to capture granted scopes."
                .to_string(),
        );
    }

    if let Some(analysis) = scope_analysis {
        if !analysis.all_required_present {
            let missing = analysis.missing.join(", ");
            guidance.push(format!(
                "Missing required scopes: {missing}. \
                 Re-authenticate with `tuitbot auth` to request all required permissions."
            ));
        }
        for feat in &analysis.degraded_features {
            guidance.push(format!(
                "{} is degraded — missing: {}.",
                feat.feature,
                feat.missing_scopes.join(", ")
            ));
        }
    }

    let tier_lower = tier.to_lowercase();
    if tier_lower == "free" {
        guidance.push(
            "Free tier detected. Search, mentions, and discovery are unavailable. \
             Upgrade to Basic ($100/mo) or Pro for full functionality."
                .to_string(),
        );
    } else if tier_lower == "unknown" {
        guidance.push(
            "API tier not yet detected. Run a search or wait for the next discovery cycle \
             to trigger tier detection."
                .to_string(),
        );
    }

    if !llm_available {
        guidance.push(
            "LLM provider not configured. Content generation tools (generate_reply, \
             generate_tweet, generate_thread) will not work. Configure an LLM in config.toml."
                .to_string(),
        );
    }

    if guidance.is_empty() {
        guidance.push("All systems operational. No issues detected.".to_string());
    }

    guidance
}

/// Build a capabilities JSON response.
pub async fn get_capabilities(
    pool: &DbPool,
    config: &Config,
    llm_available: bool,
    x_available: bool,
    user_id: Option<&str>,
    granted_scopes: &[String],
) -> String {
    let start = Instant::now();

    // 1. Read persisted tier and its timestamp.
    let (tier_str, tier_detected_at) =
        match storage::cursors::get_cursor_with_timestamp(pool, "api_tier").await {
            Ok(Some((value, ts))) => (value, Some(ts)),
            _ => ("unknown".to_string(), None),
        };

    // 2. Derive capabilities from tier.
    let tier_lower = tier_str.to_lowercase();
    let can_post_tweets = true;
    let can_reply = tier_lower != "free";
    let can_search = tier_lower == "basic" || tier_lower == "pro";
    let can_discover = can_search;

    // 3. Scope analysis (only if scopes were loaded from token).
    let scopes_available = !granted_scopes.is_empty();
    let scope_analysis = if scopes_available {
        Some(scopes::analyze_scopes(granted_scopes))
    } else {
        None
    };

    // 4. Endpoint group availability.
    let granted_set: BTreeSet<String> = granted_scopes.iter().cloned().collect();
    let endpoint_groups = compute_endpoint_groups(&granted_set);

    // 5. Auth info.
    let auth = AuthInfo {
        mode: "oauth2_user_context",
        x_client_available: x_available,
        authenticated_user_id: user_id.map(|s| s.to_string()),
        token_scopes_available: scopes_available,
    };

    // 6. Read rate limits and compute remaining (accounting for period expiry).
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

    // 7. Build direct tools availability map.
    let has_user_id = user_id.is_some();
    let not_configured = "X API client not configured. Run `tuitbot auth` to authenticate.";
    let no_user_id = "Authenticated user ID not available.";
    let needs_search = "Requires Basic or Pro tier for search.";

    let mut direct_tools_entries = Vec::new();

    // Read tools
    let read_tools: &[(&str, bool, Option<&str>, &[&str])] = &[
        (
            "get_tweet_by_id",
            x_available,
            None,
            &["tweet.read", "users.read"],
        ),
        ("x_get_user_by_username", x_available, None, &["users.read"]),
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
            &["tweet.read", "users.read"],
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
            &["tweet.read", "users.read"],
        ),
        (
            "x_get_user_tweets",
            x_available,
            None,
            &["tweet.read", "users.read"],
        ),
        (
            "x_get_followers",
            x_available,
            None,
            &["follows.read", "users.read"],
        ),
        (
            "x_get_following",
            x_available,
            None,
            &["follows.read", "users.read"],
        ),
        ("x_get_user_by_id", x_available, None, &["users.read"]),
        (
            "x_get_liked_tweets",
            x_available,
            None,
            &["like.read", "users.read"],
        ),
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
            &["bookmark.read", "users.read"],
        ),
        ("x_get_users_by_ids", x_available, None, &["users.read"]),
        (
            "x_get_tweet_liking_users",
            x_available,
            None,
            &["tweet.read", "users.read"],
        ),
    ];

    // Mutation tools
    let mutation_tools: &[(&str, bool, Option<&str>, &[&str])] = &[
        (
            "x_post_tweet",
            x_available,
            None,
            &["tweet.read", "tweet.write", "users.read"],
        ),
        (
            "x_reply_to_tweet",
            x_available,
            None,
            &["tweet.read", "tweet.write", "users.read"],
        ),
        (
            "x_quote_tweet",
            x_available,
            None,
            &["tweet.read", "tweet.write", "users.read"],
        ),
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
            &["like.read", "like.write", "users.read"],
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
            &["like.read", "like.write", "users.read"],
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
            &["follows.read", "follows.write", "users.read"],
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
            &["follows.read", "follows.write", "users.read"],
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
            &["bookmark.read", "bookmark.write", "users.read"],
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
            &["bookmark.read", "bookmark.write", "users.read"],
        ),
    ];

    for (name, available, reason, tool_scopes) in read_tools.iter().chain(mutation_tools.iter()) {
        direct_tools_entries.push(DirectToolEntry {
            name: name.to_string(),
            available: *available,
            reason: if *available {
                None
            } else {
                Some(reason.unwrap_or(not_configured).to_string())
            },
            requires_scopes: tool_scopes.iter().map(|s| (*s).to_string()).collect(),
        });
    }

    let direct_tools = DirectToolsMap {
        x_client_available: x_available,
        authenticated_user_id: user_id.map(|s| s.to_string()),
        tools: direct_tools_entries,
    };

    let backend = provider::parse_backend(&config.x_api.provider_backend);
    let provider_caps = match backend {
        provider::ProviderBackend::XApi => ProviderCapabilities::x_api(),
        provider::ProviderBackend::Scraper => {
            ProviderCapabilities::scraper(config.x_api.scraper_allow_mutations)
        }
    };

    // 8. Actionable guidance.
    let guidance = compute_guidance(
        x_available,
        has_user_id,
        scopes_available,
        scope_analysis.as_ref(),
        &tier_str,
        llm_available,
    );

    let out = Capabilities {
        tier: tier_str,
        tier_detected_at,
        can_post_tweets,
        can_reply,
        can_search,
        can_discover,
        approval_mode: config.approval_mode,
        llm_available,
        auth,
        scope_analysis,
        endpoint_groups,
        rate_limits: rate_entries,
        recommended_max_actions: RecommendedMax {
            replies: reply_remaining,
            tweets: tweet_remaining,
            threads: thread_remaining,
        },
        direct_tools,
        provider: provider_caps,
        guidance,
    };

    let elapsed = start.elapsed().as_millis() as u64;
    let meta = ToolMeta::new(elapsed)
        .with_workflow(config.mode.to_string(), config.effective_approval_mode());

    ToolResponse::success(out).with_meta(meta).to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuitbot_core::config::Config;
    use tuitbot_core::storage;
    use tuitbot_core::x_api::scopes::REQUIRED_SCOPES;

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

    fn all_scopes() -> Vec<String> {
        REQUIRED_SCOPES.iter().map(|s| (*s).to_string()).collect()
    }

    #[tokio::test]
    async fn capabilities_returns_valid_json() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, false, None, &[]).await;
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
        let result = get_capabilities(&pool, &config, false, true, Some("u1"), &all_scopes()).await;
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
        let result = get_capabilities(&pool, &config, true, false, None, &[]).await;
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
        let result = get_capabilities(&pool, &config, true, false, None, &[]).await;
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
        let result = get_capabilities(&pool, &config, true, false, None, &[]).await;
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
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
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
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
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

    // ── Scope analysis tests ──────────────────────────────────────────

    #[tokio::test]
    async fn scope_analysis_present_with_full_scopes() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let sa = &parsed["data"]["scope_analysis"];
        assert!(sa.is_object(), "scope_analysis should be present");
        assert_eq!(sa["all_required_present"], true);
        assert_eq!(sa["missing"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn scope_analysis_absent_without_scopes() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &[]).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed["data"]["scope_analysis"].is_null());
    }

    #[tokio::test]
    async fn scope_analysis_reports_missing_scopes() {
        let pool = setup_db().await;
        let config = Config::default();
        let partial: Vec<String> = vec!["tweet.read".to_string(), "users.read".to_string()];
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &partial).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let sa = &parsed["data"]["scope_analysis"];
        assert_eq!(sa["all_required_present"], false);
        let missing = sa["missing"].as_array().unwrap();
        assert!(!missing.is_empty());
        // Should be missing bookmark.read, bookmark.write, follows.read, etc.
        assert!(missing.iter().any(|m| m == "bookmark.read"));
    }

    // ── Endpoint group tests ──────────────────────────────────────────

    #[tokio::test]
    async fn endpoint_groups_present() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let groups = parsed["data"]["endpoint_groups"]
            .as_array()
            .expect("endpoint_groups array");
        assert!(!groups.is_empty());
        // Should have groups: tweets, users, lists, mutes, blocks, spaces
        let group_names: Vec<&str> = groups
            .iter()
            .map(|g| g["group"].as_str().unwrap())
            .collect();
        assert!(group_names.contains(&"tweets"));
        assert!(group_names.contains(&"lists"));
        assert!(group_names.contains(&"spaces"));
    }

    #[tokio::test]
    async fn endpoint_groups_show_missing_with_partial_scopes() {
        let pool = setup_db().await;
        let config = Config::default();
        // Only tweet.read and users.read — lists/mutes/blocks/spaces need more
        let partial: Vec<String> = vec!["tweet.read".to_string(), "users.read".to_string()];
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &partial).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let groups = parsed["data"]["endpoint_groups"]
            .as_array()
            .expect("endpoint_groups array");
        let lists = groups
            .iter()
            .find(|g| g["group"] == "lists")
            .expect("lists group");
        assert_eq!(lists["fully_available"], false);
        let missing = lists["missing_scopes"].as_array().unwrap();
        assert!(missing.iter().any(|m| m == "list.read"));
    }

    // ── Auth info tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn auth_info_present() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let auth = &parsed["data"]["auth"];
        assert_eq!(auth["mode"], "oauth2_user_context");
        assert_eq!(auth["x_client_available"], true);
        assert_eq!(auth["authenticated_user_id"], "u1");
        assert_eq!(auth["token_scopes_available"], true);
    }

    // ── Guidance tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn guidance_reports_no_x_client() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, false, None, &[]).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let guidance = parsed["data"]["guidance"].as_array().unwrap();
        assert!(guidance
            .iter()
            .any(|g| g.as_str().unwrap().contains("tuitbot auth")));
    }

    #[tokio::test]
    async fn guidance_all_operational() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Basic")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let guidance = parsed["data"]["guidance"].as_array().unwrap();
        assert!(guidance
            .iter()
            .any(|g| g.as_str().unwrap().contains("All systems operational")));
    }

    #[tokio::test]
    async fn guidance_reports_free_tier() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Free")
            .await
            .expect("set tier");
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let guidance = parsed["data"]["guidance"].as_array().unwrap();
        assert!(guidance
            .iter()
            .any(|g| g.as_str().unwrap().contains("Free tier")));
    }

    #[tokio::test]
    async fn guidance_reports_missing_scopes() {
        let pool = setup_db().await;
        storage::cursors::set_cursor(&pool, "api_tier", "Basic")
            .await
            .expect("set tier");
        let config = Config::default();
        let partial: Vec<String> = vec!["tweet.read".to_string(), "users.read".to_string()];
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &partial).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let guidance = parsed["data"]["guidance"].as_array().unwrap();
        assert!(guidance
            .iter()
            .any(|g| g.as_str().unwrap().contains("Missing required scopes")));
    }

    // ── Direct tools scope metadata ──────────────────────────────────

    #[tokio::test]
    async fn direct_tools_include_scope_metadata() {
        let pool = setup_db().await;
        let config = Config::default();
        let result = get_capabilities(&pool, &config, true, true, Some("u1"), &all_scopes()).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        let tools = parsed["data"]["direct_tools"]["tools"]
            .as_array()
            .expect("tools array");
        let post_tweet = tools
            .iter()
            .find(|t| t["name"] == "x_post_tweet")
            .expect("find x_post_tweet");
        let scopes = post_tweet["requires_scopes"].as_array().unwrap();
        assert!(scopes.iter().any(|s| s == "tweet.write"));
    }
}
