//! Rate limit tracking and enforcement for action quotas.
//!
//! Tracks usage of actions (replies, tweets, threads) within time periods,
//! stored in SQLite for persistence across restarts. Check and reset logic
//! uses transactions for atomicity.

pub mod queries;
pub mod tracker;

pub use crate::mcp_policy::types::{PolicyRateLimit, RateLimitDimension};
pub use queries::{
    check_policy_rate_limits, check_policy_rate_limits_for, get_all_rate_limits,
    get_all_rate_limits_for, get_daily_usage, get_daily_usage_for, init_policy_rate_limits,
    init_policy_rate_limits_for, record_policy_rate_limits, record_policy_rate_limits_for,
    ActionUsage, DailyUsage,
};
pub use tracker::{
    check_and_increment_rate_limit, check_and_increment_rate_limit_for, check_rate_limit,
    check_rate_limit_for, increment_rate_limit, increment_rate_limit_for,
};

use super::DbPool;
use crate::config::{IntervalsConfig, LimitsConfig};
use crate::error::StorageError;

use super::accounts::DEFAULT_ACCOUNT_ID;

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

#[cfg(test)]
mod tests;
