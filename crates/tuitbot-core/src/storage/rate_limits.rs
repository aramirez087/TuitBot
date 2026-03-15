//! CRUD operations for rate limit state tracking.
//!
//! Rate limits are stored in SQLite so they persist across restarts.
//! The check and reset logic uses transactions for atomicity.

use super::accounts::DEFAULT_ACCOUNT_ID;
use super::DbPool;
use crate::config::{IntervalsConfig, LimitsConfig};
use crate::error::StorageError;
use chrono::{DateTime, Utc};

/// A rate limit entry tracking usage for a specific action type.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct RateLimit {
    /// Action type: reply, tweet, thread, search, mention_check.
    pub action_type: String,
    /// Number of requests made in the current period.
    pub request_count: i64,
    /// ISO-8601 UTC timestamp when the current period started.
    pub period_start: String,
    /// Maximum requests allowed per period.
    pub max_requests: i64,
    /// Period length in seconds.
    pub period_seconds: i64,
}

/// Initialize rate limit rows from configuration for a specific account.
///
/// Uses `INSERT OR IGNORE` so existing counters are preserved across restarts.
/// Only inserts rows for action types that do not already exist.
pub async fn init_rate_limits_for(
    pool: &DbPool,
    account_id: &str,
    config: &LimitsConfig,
    intervals: &IntervalsConfig,
) -> Result<(), StorageError> {
    // Suppress unused variable warning -- intervals is reserved for future per-interval limits
    let _ = intervals;

    let defaults: Vec<(&str, i64, i64)> = vec![
        ("reply", i64::from(config.max_replies_per_day), 86400),
        ("tweet", i64::from(config.max_tweets_per_day), 86400),
        ("thread", i64::from(config.max_threads_per_week), 604800),
        ("search", 300, 900),
        ("mention_check", 180, 900),
    ];

    for (action_type, max_requests, period_seconds) in defaults {
        sqlx::query(
            "INSERT OR IGNORE INTO rate_limits \
             (account_id, action_type, request_count, period_start, max_requests, period_seconds) \
             VALUES (?, ?, 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?, ?)",
        )
        .bind(account_id)
        .bind(action_type)
        .bind(max_requests)
        .bind(period_seconds)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    }

    Ok(())
}

/// Initialize rate limit rows from configuration.
///
/// Uses `INSERT OR IGNORE` so existing counters are preserved across restarts.
/// Only inserts rows for action types that do not already exist.
pub async fn init_rate_limits(
    pool: &DbPool,
    config: &LimitsConfig,
    intervals: &IntervalsConfig,
) -> Result<(), StorageError> {
    init_rate_limits_for(pool, DEFAULT_ACCOUNT_ID, config, intervals).await
}

/// Initialize the MCP mutation rate limit row for a specific account.
///
/// Uses `INSERT OR IGNORE` so an existing counter is preserved across restarts.
pub async fn init_mcp_rate_limit_for(
    pool: &DbPool,
    account_id: &str,
    max_per_hour: u32,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO rate_limits \
         (account_id, action_type, request_count, period_start, max_requests, period_seconds) \
         VALUES (?, 'mcp_mutation', 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?, 3600)",
    )
    .bind(account_id)
    .bind(i64::from(max_per_hour))
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Initialize the MCP mutation rate limit row.
///
/// Uses `INSERT OR IGNORE` so an existing counter is preserved across restarts.
pub async fn init_mcp_rate_limit(pool: &DbPool, max_per_hour: u32) -> Result<(), StorageError> {
    init_mcp_rate_limit_for(pool, DEFAULT_ACCOUNT_ID, max_per_hour).await
}

/// Check whether the rate limit for an action type allows another request for a specific account.
///
/// Within a single transaction:
/// 1. Fetches the rate limit row.
/// 2. Resets the counter if the period has expired.
/// 3. Returns `true` if under the limit, `false` if at or over.
///
/// Does NOT increment the counter -- call `increment_rate_limit` after the action succeeds.
pub async fn check_rate_limit_for(
    pool: &DbPool,
    account_id: &str,
    action_type: &str,
) -> Result<bool, StorageError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let row = sqlx::query_as::<_, RateLimit>(
        "SELECT action_type, request_count, period_start, max_requests, period_seconds \
         FROM rate_limits WHERE account_id = ? AND action_type = ?",
    )
    .bind(account_id)
    .bind(action_type)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let limit = match row {
        Some(l) => l,
        None => {
            tx.commit()
                .await
                .map_err(|e| StorageError::Connection { source: e })?;
            return Ok(true);
        }
    };

    let now = Utc::now();
    let period_start = limit.period_start.parse::<DateTime<Utc>>().unwrap_or(now);

    let elapsed = now.signed_duration_since(period_start).num_seconds();

    if elapsed >= limit.period_seconds {
        sqlx::query(
            "UPDATE rate_limits SET request_count = 0, \
             period_start = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
             WHERE account_id = ? AND action_type = ?",
        )
        .bind(account_id)
        .bind(action_type)
        .execute(&mut *tx)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

        tx.commit()
            .await
            .map_err(|e| StorageError::Connection { source: e })?;
        return Ok(true);
    }

    let allowed = limit.request_count < limit.max_requests;

    tx.commit()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    Ok(allowed)
}

/// Check whether the rate limit for an action type allows another request.
///
/// Within a single transaction:
/// 1. Fetches the rate limit row.
/// 2. Resets the counter if the period has expired.
/// 3. Returns `true` if under the limit, `false` if at or over.
///
/// Does NOT increment the counter -- call `increment_rate_limit` after the action succeeds.
pub async fn check_rate_limit(pool: &DbPool, action_type: &str) -> Result<bool, StorageError> {
    check_rate_limit_for(pool, DEFAULT_ACCOUNT_ID, action_type).await
}

/// Atomically check and increment the rate limit counter for a specific account within a single transaction.
///
/// Returns `Ok(true)` if the action was permitted and the counter was incremented.
/// Returns `Ok(false)` if the rate limit was reached.
/// Resets the period if expired before checking.
pub async fn check_and_increment_rate_limit_for(
    pool: &DbPool,
    account_id: &str,
    action_type: &str,
) -> Result<bool, StorageError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let row = sqlx::query_as::<_, RateLimit>(
        "SELECT action_type, request_count, period_start, max_requests, period_seconds \
         FROM rate_limits WHERE account_id = ? AND action_type = ?",
    )
    .bind(account_id)
    .bind(action_type)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let limit = match row {
        Some(l) => l,
        None => {
            tx.commit()
                .await
                .map_err(|e| StorageError::Connection { source: e })?;
            return Ok(true);
        }
    };

    let now = Utc::now();
    let period_start = limit.period_start.parse::<DateTime<Utc>>().unwrap_or(now);

    let elapsed = now.signed_duration_since(period_start).num_seconds();

    let current_count = if elapsed >= limit.period_seconds {
        sqlx::query(
            "UPDATE rate_limits SET request_count = 0, \
             period_start = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
             WHERE account_id = ? AND action_type = ?",
        )
        .bind(account_id)
        .bind(action_type)
        .execute(&mut *tx)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
        0
    } else {
        limit.request_count
    };

    if current_count < limit.max_requests {
        sqlx::query(
            "UPDATE rate_limits SET request_count = request_count + 1 \
             WHERE account_id = ? AND action_type = ?",
        )
        .bind(account_id)
        .bind(action_type)
        .execute(&mut *tx)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

        tx.commit()
            .await
            .map_err(|e| StorageError::Connection { source: e })?;
        Ok(true)
    } else {
        tx.commit()
            .await
            .map_err(|e| StorageError::Connection { source: e })?;
        Ok(false)
    }
}

/// Atomically check and increment the rate limit counter within a single transaction.
///
/// Returns `Ok(true)` if the action was permitted and the counter was incremented.
/// Returns `Ok(false)` if the rate limit was reached.
/// Resets the period if expired before checking.
pub async fn check_and_increment_rate_limit(
    pool: &DbPool,
    action_type: &str,
) -> Result<bool, StorageError> {
    check_and_increment_rate_limit_for(pool, DEFAULT_ACCOUNT_ID, action_type).await
}

/// Increment the request counter for an action type for a specific account.
///
/// Called after a successful action to record usage.
pub async fn increment_rate_limit_for(
    pool: &DbPool,
    account_id: &str,
    action_type: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE rate_limits SET request_count = request_count + 1 \
         WHERE account_id = ? AND action_type = ?",
    )
    .bind(account_id)
    .bind(action_type)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Increment the request counter for an action type.
///
/// Called after a successful action to record usage.
pub async fn increment_rate_limit(pool: &DbPool, action_type: &str) -> Result<(), StorageError> {
    increment_rate_limit_for(pool, DEFAULT_ACCOUNT_ID, action_type).await
}

/// Usage count for a single action type.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActionUsage {
    pub used: i64,
    pub max: i64,
}

/// Daily action usage summary for the activity feed rate limit display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyUsage {
    pub replies: ActionUsage,
    pub tweets: ActionUsage,
    pub threads: ActionUsage,
}

/// Get daily usage counts for reply, tweet, and thread actions for a specific account.
///
/// Reads from the rate limits table and extracts only the three
/// user-facing action types.
pub async fn get_daily_usage_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<DailyUsage, StorageError> {
    let limits = get_all_rate_limits_for(pool, account_id).await?;

    let mut usage = DailyUsage {
        replies: ActionUsage { used: 0, max: 0 },
        tweets: ActionUsage { used: 0, max: 0 },
        threads: ActionUsage { used: 0, max: 0 },
    };

    for limit in limits {
        let target = match limit.action_type.as_str() {
            "reply" => &mut usage.replies,
            "tweet" => &mut usage.tweets,
            "thread" => &mut usage.threads,
            _ => continue,
        };
        target.used = limit.request_count;
        target.max = limit.max_requests;
    }

    Ok(usage)
}

/// Get daily usage counts for reply, tweet, and thread actions.
///
/// Reads from the rate limits table and extracts only the three
/// user-facing action types.
pub async fn get_daily_usage(pool: &DbPool) -> Result<DailyUsage, StorageError> {
    get_daily_usage_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Fetch all rate limit entries for a specific account, ordered by action type.
///
/// Used for status reporting and debugging.
pub async fn get_all_rate_limits_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<RateLimit>, StorageError> {
    sqlx::query_as::<_, RateLimit>(
        "SELECT action_type, request_count, period_start, max_requests, period_seconds \
         FROM rate_limits WHERE account_id = ? ORDER BY action_type",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Fetch all rate limit entries, ordered by action type.
///
/// Used for status reporting and debugging.
pub async fn get_all_rate_limits(pool: &DbPool) -> Result<Vec<RateLimit>, StorageError> {
    get_all_rate_limits_for(pool, DEFAULT_ACCOUNT_ID).await
}

// ---------------------------------------------------------------------------
// Per-dimension rate limits (v2 policy engine)
// ---------------------------------------------------------------------------

use crate::mcp_policy::types::{PolicyRateLimit, RateLimitDimension};

/// Initialize rate limit rows for v2 policy rate limits for a specific account.
///
/// Uses `INSERT OR IGNORE` so existing counters are preserved.
pub async fn init_policy_rate_limits_for(
    pool: &DbPool,
    account_id: &str,
    limits: &[PolicyRateLimit],
) -> Result<(), StorageError> {
    for limit in limits {
        sqlx::query(
            "INSERT OR IGNORE INTO rate_limits \
             (account_id, action_type, request_count, period_start, max_requests, period_seconds) \
             VALUES (?, ?, 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?, ?)",
        )
        .bind(account_id)
        .bind(&limit.key)
        .bind(i64::from(limit.max_count))
        .bind(limit.period_seconds as i64)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    }
    Ok(())
}

/// Initialize rate limit rows for v2 policy rate limits.
///
/// Uses `INSERT OR IGNORE` so existing counters are preserved.
pub async fn init_policy_rate_limits(
    pool: &DbPool,
    limits: &[PolicyRateLimit],
) -> Result<(), StorageError> {
    init_policy_rate_limits_for(pool, DEFAULT_ACCOUNT_ID, limits).await
}

/// Check all applicable rate limits for a tool invocation for a specific account.
///
/// Returns the key of the first exceeded limit, or `None` if all pass.
pub async fn check_policy_rate_limits_for(
    pool: &DbPool,
    account_id: &str,
    tool_name: &str,
    category: &str,
    limits: &[PolicyRateLimit],
) -> Result<Option<String>, StorageError> {
    for limit in limits {
        let matches = match limit.dimension {
            RateLimitDimension::Tool => limit.match_value == tool_name,
            RateLimitDimension::Category => limit.match_value == category,
            RateLimitDimension::EngagementType => limit.match_value == tool_name,
            RateLimitDimension::Global => true,
        };

        if !matches {
            continue;
        }

        let allowed = check_rate_limit_for(pool, account_id, &limit.key).await?;
        if !allowed {
            return Ok(Some(limit.key.clone()));
        }
    }
    Ok(None)
}

/// Check all applicable rate limits for a tool invocation.
///
/// Returns the key of the first exceeded limit, or `None` if all pass.
pub async fn check_policy_rate_limits(
    pool: &DbPool,
    tool_name: &str,
    category: &str,
    limits: &[PolicyRateLimit],
) -> Result<Option<String>, StorageError> {
    check_policy_rate_limits_for(pool, DEFAULT_ACCOUNT_ID, tool_name, category, limits).await
}

/// Increment all applicable rate limit counters for a specific account after a successful mutation.
pub async fn record_policy_rate_limits_for(
    pool: &DbPool,
    account_id: &str,
    tool_name: &str,
    category: &str,
    limits: &[PolicyRateLimit],
) -> Result<(), StorageError> {
    for limit in limits {
        let matches = match limit.dimension {
            RateLimitDimension::Tool => limit.match_value == tool_name,
            RateLimitDimension::Category => limit.match_value == category,
            RateLimitDimension::EngagementType => limit.match_value == tool_name,
            RateLimitDimension::Global => true,
        };

        if matches {
            // Best-effort: if the row doesn't exist yet, skip it
            let _ = increment_rate_limit_for(pool, account_id, &limit.key).await;
        }
    }
    Ok(())
}

/// Increment all applicable rate limit counters after a successful mutation.
pub async fn record_policy_rate_limits(
    pool: &DbPool,
    tool_name: &str,
    category: &str,
    limits: &[PolicyRateLimit],
) -> Result<(), StorageError> {
    record_policy_rate_limits_for(pool, DEFAULT_ACCOUNT_ID, tool_name, category, limits).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    fn test_limits_config() -> LimitsConfig {
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

    fn test_intervals_config() -> IntervalsConfig {
        IntervalsConfig {
            mentions_check_seconds: 300,
            discovery_search_seconds: 600,
            content_post_window_seconds: 14400,
            thread_interval_seconds: 604800,
        }
    }

    #[tokio::test]
    async fn init_creates_all_rate_limit_rows() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init rate limits");

        let limits = get_all_rate_limits(&pool).await.expect("get limits");
        assert_eq!(limits.len(), 5);

        let reply = limits
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply");
        assert_eq!(reply.max_requests, 3);
        assert_eq!(reply.period_seconds, 86400);
        assert_eq!(reply.request_count, 0);

        let thread = limits
            .iter()
            .find(|l| l.action_type == "thread")
            .expect("thread");
        assert_eq!(thread.max_requests, 1);
        assert_eq!(thread.period_seconds, 604800);
    }

    #[tokio::test]
    async fn init_preserves_existing_counters() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("first init");

        // Increment reply counter
        increment_rate_limit(&pool, "reply")
            .await
            .expect("increment");

        // Re-init should preserve the counter
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("second init");

        let limits = get_all_rate_limits(&pool).await.expect("get limits");
        let reply = limits
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply");
        assert_eq!(reply.request_count, 1, "counter should be preserved");
    }

    #[tokio::test]
    async fn check_rate_limit_allows_under_max() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        assert!(check_rate_limit(&pool, "reply").await.expect("check"));
    }

    #[tokio::test]
    async fn check_rate_limit_blocks_at_max() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        // Fill up the reply limit (max = 3)
        for _ in 0..3 {
            increment_rate_limit(&pool, "reply").await.expect("inc");
        }

        assert!(!check_rate_limit(&pool, "reply").await.expect("check"));
    }

    #[tokio::test]
    async fn check_rate_limit_resets_expired_period() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        // Fill up and set period_start to 25 hours ago
        for _ in 0..3 {
            increment_rate_limit(&pool, "reply").await.expect("inc");
        }
        sqlx::query(
            "UPDATE rate_limits SET period_start = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-25 hours') \
             WHERE action_type = 'reply'",
        )
        .execute(&pool)
        .await
        .expect("backdate");

        // Should reset and allow
        assert!(check_rate_limit(&pool, "reply").await.expect("check"));

        // Verify counter was reset
        let limits = get_all_rate_limits(&pool).await.expect("get");
        let reply = limits
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply");
        assert_eq!(reply.request_count, 0);
    }

    #[tokio::test]
    async fn check_rate_limit_unknown_type_allows() {
        let pool = init_test_db().await.expect("init db");
        assert!(check_rate_limit(&pool, "nonexistent").await.expect("check"));
    }

    #[tokio::test]
    async fn check_and_increment_works() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        // Should succeed 3 times (max_replies = 3) then fail
        assert!(check_and_increment_rate_limit(&pool, "reply")
            .await
            .expect("1"));
        assert!(check_and_increment_rate_limit(&pool, "reply")
            .await
            .expect("2"));
        assert!(check_and_increment_rate_limit(&pool, "reply")
            .await
            .expect("3"));
        assert!(!check_and_increment_rate_limit(&pool, "reply")
            .await
            .expect("4"));

        let limits = get_all_rate_limits(&pool).await.expect("get");
        let reply = limits
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply");
        assert_eq!(reply.request_count, 3);
    }

    #[tokio::test]
    async fn increment_rate_limit_works() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        increment_rate_limit(&pool, "tweet").await.expect("inc");
        increment_rate_limit(&pool, "tweet").await.expect("inc");

        let limits = get_all_rate_limits(&pool).await.expect("get");
        let tweet = limits
            .iter()
            .find(|l| l.action_type == "tweet")
            .expect("tweet");
        assert_eq!(tweet.request_count, 2);
    }

    #[tokio::test]
    async fn get_all_rate_limits_ordered() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        let limits = get_all_rate_limits(&pool).await.expect("get");
        let types: Vec<&str> = limits.iter().map(|l| l.action_type.as_str()).collect();
        let mut sorted = types.clone();
        sorted.sort();
        assert_eq!(types, sorted, "should be sorted by action_type");
    }

    #[tokio::test]
    async fn daily_usage_returns_correct_counts() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        increment_rate_limit(&pool, "reply").await.expect("inc");
        increment_rate_limit(&pool, "reply").await.expect("inc");
        increment_rate_limit(&pool, "tweet").await.expect("inc");

        let usage = get_daily_usage(&pool).await.expect("get usage");

        assert_eq!(usage.replies.used, 2);
        assert_eq!(usage.replies.max, 3);
        assert_eq!(usage.tweets.used, 1);
        assert_eq!(usage.tweets.max, 2);
        assert_eq!(usage.threads.used, 0);
        assert_eq!(usage.threads.max, 1);
    }

    #[tokio::test]
    async fn init_mcp_rate_limit_creates_row() {
        let pool = init_test_db().await.expect("init db");
        init_mcp_rate_limit(&pool, 60).await.expect("init mcp");

        let limits = get_all_rate_limits(&pool).await.expect("get");
        let mcp = limits
            .iter()
            .find(|l| l.action_type == "mcp_mutation")
            .expect("mcp_mutation row");
        assert_eq!(mcp.max_requests, 60);
        assert_eq!(mcp.period_seconds, 3600);
        assert_eq!(mcp.request_count, 0);
    }

    #[tokio::test]
    async fn init_mcp_rate_limit_preserves_existing() {
        let pool = init_test_db().await.expect("init db");
        init_mcp_rate_limit(&pool, 60).await.expect("init mcp");
        increment_rate_limit(&pool, "mcp_mutation")
            .await
            .expect("inc");

        // Re-init should not reset counter
        init_mcp_rate_limit(&pool, 60).await.expect("re-init mcp");
        let limits = get_all_rate_limits(&pool).await.expect("get");
        let mcp = limits
            .iter()
            .find(|l| l.action_type == "mcp_mutation")
            .expect("mcp_mutation");
        assert_eq!(mcp.request_count, 1);
    }

    #[tokio::test]
    async fn check_and_increment_resets_expired_period() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        // Fill up reply limit (max = 3)
        for _ in 0..3 {
            check_and_increment_rate_limit(&pool, "reply")
                .await
                .expect("inc");
        }
        assert!(!check_and_increment_rate_limit(&pool, "reply")
            .await
            .expect("blocked"));

        // Backdate period_start to 25 hours ago
        sqlx::query(
            "UPDATE rate_limits SET period_start = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-25 hours') \
             WHERE action_type = 'reply'",
        )
        .execute(&pool)
        .await
        .expect("backdate");

        // Should reset and allow again
        assert!(check_and_increment_rate_limit(&pool, "reply")
            .await
            .expect("after reset"));

        let limits = get_all_rate_limits(&pool).await.expect("get");
        let reply = limits
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply");
        // Counter should be 1 (reset to 0, then incremented)
        assert_eq!(reply.request_count, 1);
    }

    #[tokio::test]
    async fn check_and_increment_unknown_type_allows() {
        let pool = init_test_db().await.expect("init db");
        assert!(check_and_increment_rate_limit(&pool, "nonexistent")
            .await
            .expect("check"));
    }

    #[tokio::test]
    async fn daily_usage_zero_when_no_data() {
        let pool = init_test_db().await.expect("init db");
        let usage = get_daily_usage(&pool).await.expect("get usage");
        assert_eq!(usage.replies.used, 0);
        assert_eq!(usage.replies.max, 0);
        assert_eq!(usage.tweets.used, 0);
        assert_eq!(usage.threads.used, 0);
    }

    #[tokio::test]
    async fn get_all_rate_limits_empty_when_no_init() {
        let pool = init_test_db().await.expect("init db");
        let limits = get_all_rate_limits(&pool).await.expect("get");
        assert!(limits.is_empty());
    }

    #[tokio::test]
    async fn init_policy_rate_limits_creates_rows() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![
            PolicyRateLimit {
                key: "mcp:like_tweet:hourly".to_string(),
                dimension: RateLimitDimension::Tool,
                match_value: "like_tweet".to_string(),
                max_count: 10,
                period_seconds: 3600,
            },
            PolicyRateLimit {
                key: "mcp:engagement:daily".to_string(),
                dimension: RateLimitDimension::Category,
                match_value: "engagement".to_string(),
                max_count: 50,
                period_seconds: 86400,
            },
        ];

        init_policy_rate_limits(&pool, &limits)
            .await
            .expect("init policy");

        let all = get_all_rate_limits(&pool).await.expect("get");
        assert_eq!(all.len(), 2);

        let like = all
            .iter()
            .find(|l| l.action_type == "mcp:like_tweet:hourly")
            .expect("like row");
        assert_eq!(like.max_requests, 10);
        assert_eq!(like.period_seconds, 3600);
    }

    #[tokio::test]
    async fn check_policy_rate_limits_allows_under_limit() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![PolicyRateLimit {
            key: "mcp:like_tweet:hourly".to_string(),
            dimension: RateLimitDimension::Tool,
            match_value: "like_tweet".to_string(),
            max_count: 5,
            period_seconds: 3600,
        }];

        init_policy_rate_limits(&pool, &limits).await.expect("init");

        let exceeded = check_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
            .await
            .expect("check");
        assert!(exceeded.is_none());
    }

    #[tokio::test]
    async fn check_policy_rate_limits_blocks_at_limit() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![PolicyRateLimit {
            key: "mcp:like_tweet:hourly".to_string(),
            dimension: RateLimitDimension::Tool,
            match_value: "like_tweet".to_string(),
            max_count: 2,
            period_seconds: 3600,
        }];

        init_policy_rate_limits(&pool, &limits).await.expect("init");

        // Fill up the limit
        for _ in 0..2 {
            increment_rate_limit(&pool, "mcp:like_tweet:hourly")
                .await
                .expect("inc");
        }

        let exceeded = check_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
            .await
            .expect("check");
        assert_eq!(
            exceeded,
            Some("mcp:like_tweet:hourly".to_string()),
            "should report the exceeded limit key"
        );
    }

    #[tokio::test]
    async fn check_policy_rate_limits_skips_non_matching_dimension() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![PolicyRateLimit {
            key: "mcp:follow:hourly".to_string(),
            dimension: RateLimitDimension::Tool,
            match_value: "follow_user".to_string(),
            max_count: 1,
            period_seconds: 3600,
        }];

        init_policy_rate_limits(&pool, &limits).await.expect("init");

        // Fill up the follow limit
        increment_rate_limit(&pool, "mcp:follow:hourly")
            .await
            .expect("inc");

        // Check with a different tool name -- should not match
        let exceeded = check_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
            .await
            .expect("check");
        assert!(exceeded.is_none());
    }

    #[tokio::test]
    async fn check_policy_rate_limits_global_matches_any_tool() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![PolicyRateLimit {
            key: "mcp:global:hourly".to_string(),
            dimension: RateLimitDimension::Global,
            match_value: String::new(),
            max_count: 1,
            period_seconds: 3600,
        }];

        init_policy_rate_limits(&pool, &limits).await.expect("init");
        increment_rate_limit(&pool, "mcp:global:hourly")
            .await
            .expect("inc");

        let exceeded = check_policy_rate_limits(&pool, "any_tool", "any_cat", &limits)
            .await
            .expect("check");
        assert_eq!(exceeded, Some("mcp:global:hourly".to_string()));
    }

    #[tokio::test]
    async fn check_policy_rate_limits_category_dimension() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![PolicyRateLimit {
            key: "mcp:engagement:daily".to_string(),
            dimension: RateLimitDimension::Category,
            match_value: "engagement".to_string(),
            max_count: 1,
            period_seconds: 86400,
        }];

        init_policy_rate_limits(&pool, &limits).await.expect("init");
        increment_rate_limit(&pool, "mcp:engagement:daily")
            .await
            .expect("inc");

        let exceeded = check_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
            .await
            .expect("check");
        assert_eq!(exceeded, Some("mcp:engagement:daily".to_string()));
    }

    #[tokio::test]
    async fn record_policy_rate_limits_increments_matching() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![
            PolicyRateLimit {
                key: "mcp:like_tweet:hourly".to_string(),
                dimension: RateLimitDimension::Tool,
                match_value: "like_tweet".to_string(),
                max_count: 10,
                period_seconds: 3600,
            },
            PolicyRateLimit {
                key: "mcp:engagement:daily".to_string(),
                dimension: RateLimitDimension::Category,
                match_value: "engagement".to_string(),
                max_count: 50,
                period_seconds: 86400,
            },
            PolicyRateLimit {
                key: "mcp:follow:hourly".to_string(),
                dimension: RateLimitDimension::Tool,
                match_value: "follow_user".to_string(),
                max_count: 5,
                period_seconds: 3600,
            },
        ];

        init_policy_rate_limits(&pool, &limits).await.expect("init");

        record_policy_rate_limits(&pool, "like_tweet", "engagement", &limits)
            .await
            .expect("record");

        let all = get_all_rate_limits(&pool).await.expect("get");

        let like = all
            .iter()
            .find(|l| l.action_type == "mcp:like_tweet:hourly")
            .expect("like");
        assert_eq!(like.request_count, 1, "tool match should increment");

        let engagement = all
            .iter()
            .find(|l| l.action_type == "mcp:engagement:daily")
            .expect("engagement");
        assert_eq!(
            engagement.request_count, 1,
            "category match should increment"
        );

        let follow = all
            .iter()
            .find(|l| l.action_type == "mcp:follow:hourly")
            .expect("follow");
        assert_eq!(
            follow.request_count, 0,
            "non-matching tool should not increment"
        );
    }

    #[tokio::test]
    async fn init_rate_limits_for_different_accounts() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits_for(
            &pool,
            "acct_a",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init a");
        init_rate_limits_for(
            &pool,
            "acct_b",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init b");

        increment_rate_limit_for(&pool, "acct_a", "reply")
            .await
            .expect("inc a");

        let limits_a = get_all_rate_limits_for(&pool, "acct_a")
            .await
            .expect("get a");
        let reply_a = limits_a
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply a");
        assert_eq!(reply_a.request_count, 1);

        let limits_b = get_all_rate_limits_for(&pool, "acct_b")
            .await
            .expect("get b");
        let reply_b = limits_b
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply b");
        assert_eq!(reply_b.request_count, 0, "account b should be unaffected");
    }

    // ================================================================
    // Account-scoped `_for` variant isolation tests
    // ================================================================

    #[tokio::test]
    async fn check_rate_limit_for_account_isolation() {
        let pool = init_test_db().await.expect("init db");

        init_rate_limits_for(
            &pool,
            "acct_rl_a",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init a");
        init_rate_limits_for(
            &pool,
            "acct_rl_b",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init b");

        // Fill acct_a's reply limit (max = 3)
        for _ in 0..3 {
            increment_rate_limit_for(&pool, "acct_rl_a", "reply")
                .await
                .expect("inc a");
        }

        // acct_a should be blocked
        assert!(!check_rate_limit_for(&pool, "acct_rl_a", "reply")
            .await
            .expect("check a"));

        // acct_b should still be allowed
        assert!(check_rate_limit_for(&pool, "acct_rl_b", "reply")
            .await
            .expect("check b"));
    }

    #[tokio::test]
    async fn get_daily_usage_for_account_isolation() {
        let pool = init_test_db().await.expect("init db");

        init_rate_limits_for(
            &pool,
            "acct_du_a",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init a");
        init_rate_limits_for(
            &pool,
            "acct_du_b",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init b");

        // Increment acct_a's reply and tweet counters
        increment_rate_limit_for(&pool, "acct_du_a", "reply")
            .await
            .expect("inc a reply");
        increment_rate_limit_for(&pool, "acct_du_a", "reply")
            .await
            .expect("inc a reply2");
        increment_rate_limit_for(&pool, "acct_du_a", "tweet")
            .await
            .expect("inc a tweet");

        // Increment acct_b's thread counter
        increment_rate_limit_for(&pool, "acct_du_b", "thread")
            .await
            .expect("inc b thread");

        let usage_a = get_daily_usage_for(&pool, "acct_du_a")
            .await
            .expect("usage a");
        assert_eq!(usage_a.replies.used, 2);
        assert_eq!(usage_a.replies.max, 3);
        assert_eq!(usage_a.tweets.used, 1);
        assert_eq!(usage_a.tweets.max, 2);
        assert_eq!(usage_a.threads.used, 0);

        let usage_b = get_daily_usage_for(&pool, "acct_du_b")
            .await
            .expect("usage b");
        assert_eq!(usage_b.replies.used, 0);
        assert_eq!(usage_b.tweets.used, 0);
        assert_eq!(usage_b.threads.used, 1);
        assert_eq!(usage_b.threads.max, 1);
    }

    // =========================================================================
    // Additional edge case tests for coverage push
    // =========================================================================

    #[tokio::test]
    async fn increment_nonexistent_action_type_no_error() {
        let pool = init_test_db().await.expect("init db");
        // Incrementing a nonexistent type should not error (0 rows affected)
        increment_rate_limit(&pool, "nonexistent")
            .await
            .expect("increment nonexistent");
    }

    #[tokio::test]
    async fn multiple_action_types_independent() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        increment_rate_limit(&pool, "reply")
            .await
            .expect("inc reply");
        increment_rate_limit(&pool, "reply")
            .await
            .expect("inc reply");
        increment_rate_limit(&pool, "tweet")
            .await
            .expect("inc tweet");

        let limits = get_all_rate_limits(&pool).await.expect("get");
        let reply = limits
            .iter()
            .find(|l| l.action_type == "reply")
            .expect("reply");
        let tweet = limits
            .iter()
            .find(|l| l.action_type == "tweet")
            .expect("tweet");
        let thread = limits
            .iter()
            .find(|l| l.action_type == "thread")
            .expect("thread");
        let search = limits
            .iter()
            .find(|l| l.action_type == "search")
            .expect("search");
        let mention = limits
            .iter()
            .find(|l| l.action_type == "mention_check")
            .expect("mention");

        assert_eq!(reply.request_count, 2);
        assert_eq!(tweet.request_count, 1);
        assert_eq!(thread.request_count, 0);
        assert_eq!(search.request_count, 0);
        assert_eq!(mention.request_count, 0);
    }

    #[tokio::test]
    async fn check_rate_limit_just_under_max() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        // max_replies = 3, increment to 2 (just under)
        increment_rate_limit(&pool, "reply").await.expect("inc");
        increment_rate_limit(&pool, "reply").await.expect("inc");

        assert!(check_rate_limit(&pool, "reply").await.expect("check"));

        // Now at 3 (at max)
        increment_rate_limit(&pool, "reply").await.expect("inc");
        assert!(!check_rate_limit(&pool, "reply").await.expect("check"));
    }

    #[tokio::test]
    async fn check_and_increment_all_action_types() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        // Test each action type independently
        for action in &["reply", "tweet", "thread", "search", "mention_check"] {
            let allowed = check_and_increment_rate_limit(&pool, action)
                .await
                .expect("check and inc");
            assert!(allowed, "{action} should be allowed initially");
        }

        let limits = get_all_rate_limits(&pool).await.expect("get");
        for limit in &limits {
            assert_eq!(
                limit.request_count, 1,
                "{} should have count 1",
                limit.action_type
            );
        }
    }

    #[tokio::test]
    async fn daily_usage_reflects_all_increments() {
        let pool = init_test_db().await.expect("init db");
        init_rate_limits(&pool, &test_limits_config(), &test_intervals_config())
            .await
            .expect("init");

        // Fill to various levels
        for _ in 0..3 {
            increment_rate_limit(&pool, "reply").await.expect("inc");
        }
        for _ in 0..2 {
            increment_rate_limit(&pool, "tweet").await.expect("inc");
        }
        increment_rate_limit(&pool, "thread").await.expect("inc");

        let usage = get_daily_usage(&pool).await.expect("usage");
        assert_eq!(usage.replies.used, 3);
        assert_eq!(usage.replies.max, 3);
        assert_eq!(usage.tweets.used, 2);
        assert_eq!(usage.tweets.max, 2);
        assert_eq!(usage.threads.used, 1);
        assert_eq!(usage.threads.max, 1);
    }

    #[tokio::test]
    async fn init_mcp_rate_limit_different_max() {
        let pool = init_test_db().await.expect("init db");
        init_mcp_rate_limit(&pool, 100).await.expect("init");

        let limits = get_all_rate_limits(&pool).await.expect("get");
        let mcp = limits
            .iter()
            .find(|l| l.action_type == "mcp_mutation")
            .expect("mcp");
        assert_eq!(mcp.max_requests, 100);
        assert_eq!(mcp.period_seconds, 3600);
    }

    #[tokio::test]
    async fn mcp_rate_limit_check_and_increment() {
        let pool = init_test_db().await.expect("init db");
        init_mcp_rate_limit(&pool, 2).await.expect("init");

        assert!(check_and_increment_rate_limit(&pool, "mcp_mutation")
            .await
            .expect("1"));
        assert!(check_and_increment_rate_limit(&pool, "mcp_mutation")
            .await
            .expect("2"));
        assert!(!check_and_increment_rate_limit(&pool, "mcp_mutation")
            .await
            .expect("3"));
    }

    #[tokio::test]
    async fn policy_rate_limits_multiple_dimensions() {
        let pool = init_test_db().await.expect("init db");
        let limits = vec![
            PolicyRateLimit {
                key: "tool:like:hourly".to_string(),
                dimension: RateLimitDimension::Tool,
                match_value: "like_tweet".to_string(),
                max_count: 5,
                period_seconds: 3600,
            },
            PolicyRateLimit {
                key: "cat:eng:daily".to_string(),
                dimension: RateLimitDimension::Category,
                match_value: "engagement".to_string(),
                max_count: 10,
                period_seconds: 86400,
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
        let cat = all
            .iter()
            .find(|l| l.action_type == "cat:eng:daily")
            .expect("cat");
        assert_eq!(cat.request_count, 1);
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
        assert_eq!(like.request_count, 1, "like should be incremented");
    }

    #[tokio::test]
    async fn init_mcp_rate_limit_for_different_accounts() {
        let pool = init_test_db().await.expect("init db");
        init_mcp_rate_limit_for(&pool, "acct_mcp_a", 50)
            .await
            .expect("init a");
        init_mcp_rate_limit_for(&pool, "acct_mcp_b", 100)
            .await
            .expect("init b");

        let limits_a = get_all_rate_limits_for(&pool, "acct_mcp_a")
            .await
            .expect("get a");
        let mcp_a = limits_a
            .iter()
            .find(|l| l.action_type == "mcp_mutation")
            .expect("mcp a");
        assert_eq!(mcp_a.max_requests, 50);

        let limits_b = get_all_rate_limits_for(&pool, "acct_mcp_b")
            .await
            .expect("get b");
        let mcp_b = limits_b
            .iter()
            .find(|l| l.action_type == "mcp_mutation")
            .expect("mcp b");
        assert_eq!(mcp_b.max_requests, 100);
    }

    #[tokio::test]
    async fn check_rate_limit_for_different_accounts_isolated() {
        let pool = init_test_db().await.expect("init db");

        init_rate_limits_for(
            &pool,
            "acct_ci_a",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init a");
        init_rate_limits_for(
            &pool,
            "acct_ci_b",
            &test_limits_config(),
            &test_intervals_config(),
        )
        .await
        .expect("init b");

        // Exhaust acct_a's tweet limit (max = 2)
        assert!(
            check_and_increment_rate_limit_for(&pool, "acct_ci_a", "tweet")
                .await
                .expect("1")
        );
        assert!(
            check_and_increment_rate_limit_for(&pool, "acct_ci_a", "tweet")
                .await
                .expect("2")
        );
        assert!(
            !check_and_increment_rate_limit_for(&pool, "acct_ci_a", "tweet")
                .await
                .expect("3")
        );

        // acct_b should still be allowed
        assert!(
            check_and_increment_rate_limit_for(&pool, "acct_ci_b", "tweet")
                .await
                .expect("b1")
        );

        // Verify counts
        let limits_a = get_all_rate_limits_for(&pool, "acct_ci_a")
            .await
            .expect("get a");
        let tweet_a = limits_a
            .iter()
            .find(|l| l.action_type == "tweet")
            .expect("tweet a");
        assert_eq!(tweet_a.request_count, 2);

        let limits_b = get_all_rate_limits_for(&pool, "acct_ci_b")
            .await
            .expect("get b");
        let tweet_b = limits_b
            .iter()
            .find(|l| l.action_type == "tweet")
            .expect("tweet b");
        assert_eq!(tweet_b.request_count, 1);
    }
}
