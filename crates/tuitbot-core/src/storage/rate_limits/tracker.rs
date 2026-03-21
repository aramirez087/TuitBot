//! Rate limit checking and increment operations.

use super::super::DbPool;
use super::{RateLimit, DEFAULT_ACCOUNT_ID};
use crate::error::StorageError;
use chrono::{DateTime, Utc};

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
        // Period expired: reset and allow
        sqlx::query(
            "UPDATE rate_limits SET request_count = 0, period_start = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
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

    // Period active: check count
    let under_limit = limit.request_count < limit.max_requests;
    tx.commit()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;
    Ok(under_limit)
}

/// Check whether the rate limit for an action type allows another request for the default account.
pub async fn check_rate_limit(pool: &DbPool, action_type: &str) -> Result<bool, StorageError> {
    check_rate_limit_for(pool, DEFAULT_ACCOUNT_ID, action_type).await
}

/// Check rate limit and increment in a single transaction for a specific account.
///
/// Returns `Ok(true)` if the limit allowed the action (counter incremented).
/// Returns `Ok(false)` if the limit was exceeded (counter not incremented, period not reset).
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

    if elapsed >= limit.period_seconds {
        // Period expired: reset and increment
        sqlx::query(
            "UPDATE rate_limits SET request_count = 1, period_start = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
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

    // Period active: check count and maybe increment
    if limit.request_count < limit.max_requests {
        sqlx::query("UPDATE rate_limits SET request_count = request_count + 1 WHERE account_id = ? AND action_type = ?")
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
        // Over limit
        tx.commit()
            .await
            .map_err(|e| StorageError::Connection { source: e })?;
        Ok(false)
    }
}

/// Check rate limit and increment in a single transaction for the default account.
pub async fn check_and_increment_rate_limit(
    pool: &DbPool,
    action_type: &str,
) -> Result<bool, StorageError> {
    check_and_increment_rate_limit_for(pool, DEFAULT_ACCOUNT_ID, action_type).await
}

/// Increment the rate limit counter for an action type for a specific account.
///
/// Used after an action succeeds, to record that it was performed.
/// Does NOT check the limit -- call `check_rate_limit` before this.
pub async fn increment_rate_limit_for(
    pool: &DbPool,
    account_id: &str,
    action_type: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE rate_limits SET request_count = request_count + 1 WHERE account_id = ? AND action_type = ?")
        .bind(account_id)
        .bind(action_type)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Increment the rate limit counter for an action type for the default account.
pub async fn increment_rate_limit(pool: &DbPool, action_type: &str) -> Result<(), StorageError> {
    increment_rate_limit_for(pool, DEFAULT_ACCOUNT_ID, action_type).await
}
