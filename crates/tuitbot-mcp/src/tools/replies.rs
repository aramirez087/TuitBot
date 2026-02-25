//! Replies tools: get_recent_replies, get_reply_count_today.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

#[derive(Serialize)]
struct ReplySentOut {
    id: i64,
    target_tweet_id: String,
    reply_tweet_id: Option<String>,
    reply_content: String,
    llm_provider: Option<String>,
    llm_model: Option<String>,
    created_at: String,
    status: String,
    error_message: Option<String>,
}

/// Get replies sent within a time window.
pub async fn get_recent_replies(pool: &DbPool, since_hours: u32, config: &Config) -> String {
    let start = Instant::now();
    let since = chrono::Utc::now() - chrono::Duration::hours(i64::from(since_hours));
    let since_str = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    match storage::replies::get_replies_since(pool, &since_str).await {
        Ok(replies) => {
            let out: Vec<ReplySentOut> = replies
                .into_iter()
                .map(|r| ReplySentOut {
                    id: r.id,
                    target_tweet_id: r.target_tweet_id,
                    reply_tweet_id: r.reply_tweet_id,
                    reply_content: r.reply_content,
                    llm_provider: r.llm_provider,
                    llm_model: r.llm_model,
                    created_at: r.created_at,
                    status: r.status,
                    error_message: r.error_message,
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
            ToolResponse::db_error(format!("Error fetching replies: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Get count of replies sent today.
pub async fn get_reply_count_today(pool: &DbPool, config: &Config) -> String {
    let start = Instant::now();

    match storage::replies::count_replies_today(pool).await {
        Ok(count) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({ "replies_today": count }))
                .with_meta(meta)
                .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error counting replies today: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
