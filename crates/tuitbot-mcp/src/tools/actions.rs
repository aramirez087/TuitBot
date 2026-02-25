//! Action log tools: get_action_log, get_action_counts.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

#[derive(Serialize)]
struct ActionLogOut {
    id: i64,
    action_type: String,
    status: String,
    message: Option<String>,
    metadata: Option<String>,
    created_at: String,
}

/// Get recent action log entries.
pub async fn get_action_log(
    pool: &DbPool,
    since_hours: u32,
    action_type: Option<&str>,
    config: &Config,
) -> String {
    let start = Instant::now();
    let since = chrono::Utc::now() - chrono::Duration::hours(i64::from(since_hours));
    let since_str = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    match storage::action_log::get_actions_since(pool, &since_str, action_type).await {
        Ok(entries) => {
            let out: Vec<ActionLogOut> = entries
                .into_iter()
                .map(|e| ActionLogOut {
                    id: e.id,
                    action_type: e.action_type,
                    status: e.status,
                    message: e.message,
                    metadata: e.metadata,
                    created_at: e.created_at,
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
            ToolResponse::db_error(format!("Error fetching action log: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Get action counts grouped by type.
pub async fn get_action_counts(pool: &DbPool, since_hours: u32, config: &Config) -> String {
    let start = Instant::now();
    let since = chrono::Utc::now() - chrono::Duration::hours(i64::from(since_hours));
    let since_str = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    match storage::action_log::get_action_counts_since(pool, &since_str).await {
        Ok(counts) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(counts).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching action counts: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
