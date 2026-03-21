//! Rate limit queries and policy operations.

use super::super::DbPool;
use super::{RateLimit, DEFAULT_ACCOUNT_ID};
use crate::error::StorageError;
use crate::mcp_policy::types::PolicyRateLimit;

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

/// Check policy rate limits for a specific action and dimension for a specific account.
///
/// Returns the first limit that was exceeded, or None if all are within limits.
pub async fn check_policy_rate_limits_for(
    pool: &DbPool,
    account_id: &str,
    action: &str,
    dimension: &str,
    limits: &[PolicyRateLimit],
) -> Result<Option<String>, StorageError> {
    for limit in limits {
        if limit.match_value != action
            && limit.match_value != dimension
            && !limit.match_value.is_empty()
        {
            continue;
        }

        let row = sqlx::query_as::<_, RateLimit>(
            "SELECT action_type, request_count, period_start, max_requests, period_seconds \
             FROM rate_limits WHERE account_id = ? AND action_type = ?",
        )
        .bind(account_id)
        .bind(&limit.key)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

        if let Some(rate_limit) = row {
            if rate_limit.request_count >= rate_limit.max_requests {
                return Ok(Some(limit.key.clone()));
            }
        }
    }
    Ok(None)
}

/// Check policy rate limits for a specific action and dimension.
///
/// Returns the first limit that was exceeded, or None if all are within limits.
pub async fn check_policy_rate_limits(
    pool: &DbPool,
    action: &str,
    dimension: &str,
    limits: &[PolicyRateLimit],
) -> Result<Option<String>, StorageError> {
    check_policy_rate_limits_for(pool, DEFAULT_ACCOUNT_ID, action, dimension, limits).await
}

/// Record usage for policy rate limits that match the given action and dimension for a specific account.
///
/// Only increments limits whose match_value matches the action or dimension.
pub async fn record_policy_rate_limits_for(
    pool: &DbPool,
    account_id: &str,
    action: &str,
    dimension: &str,
    limits: &[PolicyRateLimit],
) -> Result<(), StorageError> {
    for limit in limits {
        if limit.match_value != action
            && limit.match_value != dimension
            && !limit.match_value.is_empty()
        {
            continue;
        }

        sqlx::query("UPDATE rate_limits SET request_count = request_count + 1 WHERE account_id = ? AND action_type = ?")
            .bind(account_id)
            .bind(&limit.key)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;
    }
    Ok(())
}

/// Record usage for policy rate limits that match the given action and dimension.
///
/// Only increments limits whose match_value matches the action or dimension.
pub async fn record_policy_rate_limits(
    pool: &DbPool,
    action: &str,
    dimension: &str,
    limits: &[PolicyRateLimit],
) -> Result<(), StorageError> {
    record_policy_rate_limits_for(pool, DEFAULT_ACCOUNT_ID, action, dimension, limits).await
}
