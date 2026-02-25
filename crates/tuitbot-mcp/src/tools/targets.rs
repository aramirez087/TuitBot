//! Target accounts tool: list_target_accounts.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

#[derive(Serialize)]
struct TargetAccountOut {
    account_id: String,
    username: String,
    followed_at: Option<String>,
    first_engagement_at: Option<String>,
    total_replies_sent: i64,
    last_reply_at: Option<String>,
    status: String,
}

/// List all active target accounts with engagement stats.
pub async fn list_target_accounts(pool: &DbPool, config: &Config) -> String {
    let start = Instant::now();

    match storage::target_accounts::get_active_target_accounts(pool).await {
        Ok(accounts) => {
            let out: Vec<TargetAccountOut> = accounts
                .into_iter()
                .map(|a| TargetAccountOut {
                    account_id: a.account_id,
                    username: a.username,
                    followed_at: a.followed_at,
                    first_engagement_at: a.first_engagement_at,
                    total_replies_sent: a.total_replies_sent,
                    last_reply_at: a.last_reply_at,
                    status: a.status,
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
            ToolResponse::db_error(format!("Error fetching target accounts: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
