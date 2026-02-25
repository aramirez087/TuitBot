//! Rate limits tool: get_rate_limits.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

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
pub async fn get_rate_limits(pool: &DbPool, config: &Config) -> String {
    let start = Instant::now();

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
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching rate limits: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
