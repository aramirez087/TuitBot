//! CRUD operations for rate limit state tracking.
//!
//! Rate limits are stored in SQLite so they persist across restarts.
//! The check and reset logic uses transactions for atomicity.

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

/// Initialize rate limit rows from configuration.
///
/// Uses `INSERT OR IGNORE` so existing counters are preserved across restarts.
/// Only inserts rows for action types that do not already exist.
pub async fn init_rate_limits(
    pool: &DbPool,
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
             (action_type, request_count, period_start, max_requests, period_seconds) \
             VALUES (?, 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?, ?)",
        )
        .bind(action_type)
        .bind(max_requests)
        .bind(period_seconds)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    }

    Ok(())
}

/// Initialize the MCP mutation rate limit row.
///
/// Uses `INSERT OR IGNORE` so an existing counter is preserved across restarts.
pub async fn init_mcp_rate_limit(pool: &DbPool, max_per_hour: u32) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO rate_limits \
         (action_type, request_count, period_start, max_requests, period_seconds) \
         VALUES ('mcp_mutation', 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?, 3600)",
    )
    .bind(i64::from(max_per_hour))
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
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
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let row = sqlx::query_as::<_, RateLimit>("SELECT * FROM rate_limits WHERE action_type = ?")
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
             WHERE action_type = ?",
        )
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

/// Atomically check and increment the rate limit counter within a single transaction.
///
/// Returns `Ok(true)` if the action was permitted and the counter was incremented.
/// Returns `Ok(false)` if the rate limit was reached.
/// Resets the period if expired before checking.
pub async fn check_and_increment_rate_limit(
    pool: &DbPool,
    action_type: &str,
) -> Result<bool, StorageError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let row = sqlx::query_as::<_, RateLimit>("SELECT * FROM rate_limits WHERE action_type = ?")
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
             WHERE action_type = ?",
        )
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
            "UPDATE rate_limits SET request_count = request_count + 1 WHERE action_type = ?",
        )
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

/// Increment the request counter for an action type.
///
/// Called after a successful action to record usage.
pub async fn increment_rate_limit(pool: &DbPool, action_type: &str) -> Result<(), StorageError> {
    sqlx::query("UPDATE rate_limits SET request_count = request_count + 1 WHERE action_type = ?")
        .bind(action_type)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
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

/// Get daily usage counts for reply, tweet, and thread actions.
///
/// Reads from the rate limits table and extracts only the three
/// user-facing action types.
pub async fn get_daily_usage(pool: &DbPool) -> Result<DailyUsage, StorageError> {
    let limits = get_all_rate_limits(pool).await?;

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

/// Fetch all rate limit entries, ordered by action type.
///
/// Used for status reporting and debugging.
pub async fn get_all_rate_limits(pool: &DbPool) -> Result<Vec<RateLimit>, StorageError> {
    sqlx::query_as::<_, RateLimit>("SELECT * FROM rate_limits ORDER BY action_type")
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
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
}
