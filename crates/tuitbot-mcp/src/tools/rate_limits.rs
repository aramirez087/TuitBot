//! Rate limits tool: get_rate_limits.

use serde::Serialize;

use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

#[derive(Serialize)]
struct RateLimitOut {
    action_type: String,
    request_count: i64,
    max_requests: i64,
    period_start: String,
    period_seconds: i64,
    remaining: i64,
}

/// Get current rate limit status for all action types.
pub async fn get_rate_limits(pool: &DbPool) -> String {
    match storage::rate_limits::get_all_rate_limits(pool).await {
        Ok(limits) => {
            let out: Vec<RateLimitOut> = limits
                .into_iter()
                .map(|l| {
                    let remaining = (l.max_requests - l.request_count).max(0);
                    RateLimitOut {
                        action_type: l.action_type,
                        request_count: l.request_count,
                        max_requests: l.max_requests,
                        period_start: l.period_start,
                        period_seconds: l.period_seconds,
                        remaining,
                    }
                })
                .collect();
            serde_json::to_string_pretty(&out)
                .unwrap_or_else(|e| format!("Error serializing rate limits: {e}"))
        }
        Err(e) => format!("Error fetching rate limits: {e}"),
    }
}
