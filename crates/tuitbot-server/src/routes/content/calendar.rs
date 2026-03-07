//! Calendar and schedule endpoints.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tuitbot_core::storage::{approval_queue, replies, scheduled_content, threads};

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

use super::read_effective_config;

/// A unified calendar item merging content from all sources.
#[derive(Debug, Serialize)]
pub struct CalendarItem {
    pub id: i64,
    pub content_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub timestamp: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_score: Option<f64>,
    pub source: String,
}

/// Query parameters for the calendar endpoint.
#[derive(Deserialize)]
pub struct CalendarQuery {
    /// Start of the date range (ISO 8601).
    pub from: String,
    /// End of the date range (ISO 8601).
    pub to: String,
}

/// `GET /api/content/calendar?from=...&to=...` — unified content timeline.
pub async fn calendar(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<CalendarQuery>,
) -> Result<Json<Value>, ApiError> {
    let from = &params.from;
    let to = &params.to;

    let mut items: Vec<CalendarItem> = Vec::new();

    // Tweets
    let tweets = threads::get_tweets_in_range_for(&state.db, &ctx.account_id, from, to).await?;
    for t in tweets {
        items.push(CalendarItem {
            id: t.id,
            content_type: "tweet".to_string(),
            content: t.content,
            target_author: None,
            topic: t.topic,
            timestamp: t.created_at,
            status: t.status,
            performance_score: None,
            source: "autonomous".to_string(),
        });
    }

    // Threads
    let thread_list =
        threads::get_threads_in_range_for(&state.db, &ctx.account_id, from, to).await?;
    for t in thread_list {
        items.push(CalendarItem {
            id: t.id,
            content_type: "thread".to_string(),
            content: t.topic.clone(),
            target_author: None,
            topic: Some(t.topic),
            timestamp: t.created_at,
            status: t.status,
            performance_score: None,
            source: "autonomous".to_string(),
        });
    }

    // Replies
    let reply_list =
        replies::get_replies_in_range_for(&state.db, &ctx.account_id, from, to).await?;
    for r in reply_list {
        items.push(CalendarItem {
            id: r.id,
            content_type: "reply".to_string(),
            content: r.reply_content,
            target_author: Some(r.target_tweet_id),
            topic: None,
            timestamp: r.created_at,
            status: r.status,
            performance_score: None,
            source: "autonomous".to_string(),
        });
    }

    // Approval queue items
    let pending =
        approval_queue::get_by_statuses_for(&state.db, &ctx.account_id, &["pending"], None).await?;
    for a in pending {
        // Only include if the item falls within range
        if a.created_at >= *from && a.created_at <= *to {
            items.push(CalendarItem {
                id: a.id,
                content_type: a.action_type,
                content: a.generated_content,
                target_author: if a.target_author.is_empty() {
                    None
                } else {
                    Some(a.target_author)
                },
                topic: if a.topic.is_empty() {
                    None
                } else {
                    Some(a.topic)
                },
                timestamp: a.created_at,
                status: "pending".to_string(),
                performance_score: None,
                source: "approval".to_string(),
            });
        }
    }

    // Scheduled content
    let scheduled =
        scheduled_content::get_in_range_for(&state.db, &ctx.account_id, from, to).await?;
    for s in scheduled {
        items.push(CalendarItem {
            id: s.id,
            content_type: s.content_type,
            content: s.content,
            target_author: None,
            topic: None,
            timestamp: s.scheduled_for.unwrap_or(s.created_at),
            status: s.status,
            performance_score: None,
            source: "manual".to_string(),
        });
    }

    // Sort by timestamp ascending
    items.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(Json(json!(items)))
}

/// `GET /api/content/schedule` — the configured posting schedule.
pub async fn schedule(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let config = read_effective_config(&state, &ctx.account_id).await?;

    Ok(Json(json!({
        "timezone": config.schedule.timezone,
        "active_hours": {
            "start": config.schedule.active_hours_start,
            "end": config.schedule.active_hours_end,
        },
        "preferred_times": config.schedule.preferred_times,
        "preferred_times_override": config.schedule.preferred_times_override,
        "thread_day": config.schedule.thread_preferred_day,
        "thread_time": config.schedule.thread_preferred_time,
    })))
}
