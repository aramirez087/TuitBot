//! Action log tools: get_action_log, get_action_counts.

use serde::Serialize;

use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

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
pub async fn get_action_log(pool: &DbPool, since_hours: u32, action_type: Option<&str>) -> String {
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
            serde_json::to_string_pretty(&out)
                .unwrap_or_else(|e| format!("Error serializing action log: {e}"))
        }
        Err(e) => format!("Error fetching action log: {e}"),
    }
}

/// Get action counts grouped by type.
pub async fn get_action_counts(pool: &DbPool, since_hours: u32) -> String {
    let since = chrono::Utc::now() - chrono::Duration::hours(i64::from(since_hours));
    let since_str = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    match storage::action_log::get_action_counts_since(pool, &since_str).await {
        Ok(counts) => serde_json::to_string_pretty(&counts)
            .unwrap_or_else(|e| format!("Error serializing action counts: {e}")),
        Err(e) => format!("Error fetching action counts: {e}"),
    }
}
