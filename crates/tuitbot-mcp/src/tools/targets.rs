//! Target accounts tool: list_target_accounts.

use serde::Serialize;

use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

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
pub async fn list_target_accounts(pool: &DbPool) -> String {
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
            serde_json::to_string_pretty(&out)
                .unwrap_or_else(|e| format!("Error serializing target accounts: {e}"))
        }
        Err(e) => format!("Error fetching target accounts: {e}"),
    }
}
