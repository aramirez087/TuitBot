//! Replies tools: get_recent_replies, get_reply_count_today.

use serde::Serialize;

use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

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
pub async fn get_recent_replies(pool: &DbPool, since_hours: u32) -> String {
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
            serde_json::to_string_pretty(&out)
                .unwrap_or_else(|e| format!("Error serializing replies: {e}"))
        }
        Err(e) => format!("Error fetching replies: {e}"),
    }
}

/// Get count of replies sent today.
pub async fn get_reply_count_today(pool: &DbPool) -> String {
    match storage::replies::count_replies_today(pool).await {
        Ok(count) => serde_json::json!({ "replies_today": count }).to_string(),
        Err(e) => format!("Error counting replies today: {e}"),
    }
}
