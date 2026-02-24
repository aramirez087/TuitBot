//! Content endpoints (tweets, threads, calendar, compose, scheduled content).

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tuitbot_core::config::Config;
use tuitbot_core::storage::{approval_queue, replies, scheduled_content, threads};

use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::WsEvent;

// ---------------------------------------------------------------------------
// Existing endpoints
// ---------------------------------------------------------------------------

/// Query parameters for the tweets endpoint.
#[derive(Deserialize)]
pub struct TweetsQuery {
    /// Maximum number of tweets to return (default: 50).
    #[serde(default = "default_tweet_limit")]
    pub limit: u32,
}

fn default_tweet_limit() -> u32 {
    50
}

/// Query parameters for the threads endpoint.
#[derive(Deserialize)]
pub struct ThreadsQuery {
    /// Maximum number of threads to return (default: 20).
    #[serde(default = "default_thread_limit")]
    pub limit: u32,
}

fn default_thread_limit() -> u32 {
    20
}

/// `GET /api/content/tweets` — recent original tweets posted.
pub async fn list_tweets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TweetsQuery>,
) -> Result<Json<Value>, ApiError> {
    let tweets = threads::get_recent_original_tweets(&state.db, params.limit).await?;
    Ok(Json(json!(tweets)))
}

/// `GET /api/content/threads` — recent threads posted.
pub async fn list_threads(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ThreadsQuery>,
) -> Result<Json<Value>, ApiError> {
    let threads = threads::get_recent_threads(&state.db, params.limit).await?;
    Ok(Json(json!(threads)))
}

/// Request body for composing a manual tweet.
#[derive(Deserialize)]
pub struct ComposeTweetRequest {
    /// The tweet text.
    pub text: String,
    /// Optional ISO 8601 timestamp to schedule the tweet.
    pub scheduled_for: Option<String>,
}

/// `POST /api/content/tweets` — compose and queue a manual tweet.
pub async fn compose_tweet(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ComposeTweetRequest>,
) -> Result<Json<Value>, ApiError> {
    let text = body.text.trim();
    if text.is_empty() {
        return Err(ApiError::BadRequest("text is required".to_string()));
    }

    // Check if approval mode is enabled.
    let approval_mode = read_approval_mode(&state)?;

    if approval_mode {
        let id = approval_queue::enqueue(
            &state.db, "tweet", "", // no target tweet
            "", // no target author
            text, "", // no topic
            "", // no archetype
            0.0,
        )
        .await?;

        let _ = state.event_tx.send(WsEvent::ApprovalQueued {
            id,
            action_type: "tweet".to_string(),
            content: text.to_string(),
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else {
        // Without X API client in AppState, we can only acknowledge the intent.
        Ok(Json(json!({
            "status": "accepted",
            "text": text,
            "scheduled_for": body.scheduled_for,
        })))
    }
}

/// Request body for composing a manual thread.
#[derive(Deserialize)]
pub struct ComposeThreadRequest {
    /// The tweets forming the thread.
    pub tweets: Vec<String>,
    /// Optional ISO 8601 timestamp to schedule the thread.
    pub scheduled_for: Option<String>,
}

/// `POST /api/content/threads` — compose and queue a manual thread.
pub async fn compose_thread(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ComposeThreadRequest>,
) -> Result<Json<Value>, ApiError> {
    if body.tweets.is_empty() {
        return Err(ApiError::BadRequest(
            "tweets array must not be empty".to_string(),
        ));
    }

    let approval_mode = read_approval_mode(&state)?;
    let combined = body.tweets.join("\n---\n");

    if approval_mode {
        let id =
            approval_queue::enqueue(&state.db, "thread", "", "", &combined, "", "", 0.0).await?;

        let _ = state.event_tx.send(WsEvent::ApprovalQueued {
            id,
            action_type: "thread".to_string(),
            content: combined,
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else {
        Ok(Json(json!({
            "status": "accepted",
            "tweet_count": body.tweets.len(),
            "scheduled_for": body.scheduled_for,
        })))
    }
}

// ---------------------------------------------------------------------------
// Calendar endpoints
// ---------------------------------------------------------------------------

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
    Query(params): Query<CalendarQuery>,
) -> Result<Json<Value>, ApiError> {
    let from = &params.from;
    let to = &params.to;

    let mut items: Vec<CalendarItem> = Vec::new();

    // Tweets
    let tweets = threads::get_tweets_in_range(&state.db, from, to).await?;
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
    let thread_list = threads::get_threads_in_range(&state.db, from, to).await?;
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
    let reply_list = replies::get_replies_in_range(&state.db, from, to).await?;
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
    let pending = approval_queue::get_by_statuses(&state.db, &["pending"], None).await?;
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
    let scheduled = scheduled_content::get_in_range(&state.db, from, to).await?;
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
pub async fn schedule(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let config = read_config(&state)?;

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

/// Request body for the unified compose endpoint.
#[derive(Deserialize)]
pub struct ComposeRequest {
    /// Content type: "tweet" or "thread".
    pub content_type: String,
    /// Content text (string for tweet, JSON array string for thread).
    pub content: String,
    /// Optional ISO 8601 timestamp to schedule the content.
    pub scheduled_for: Option<String>,
}

/// `POST /api/content/compose` — compose manual content (tweet or thread).
pub async fn compose(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ComposeRequest>,
) -> Result<Json<Value>, ApiError> {
    let content = body.content.trim().to_string();
    if content.is_empty() {
        return Err(ApiError::BadRequest("content is required".to_string()));
    }

    match body.content_type.as_str() {
        "tweet" => {
            if content.len() > 280 {
                return Err(ApiError::BadRequest(
                    "tweet content must not exceed 280 characters".to_string(),
                ));
            }
        }
        "thread" => {
            // Validate that content is a JSON array of strings
            let tweets: Result<Vec<String>, _> = serde_json::from_str(&content);
            match tweets {
                Ok(ref t) if t.is_empty() => {
                    return Err(ApiError::BadRequest(
                        "thread must contain at least one tweet".to_string(),
                    ));
                }
                Ok(ref t) => {
                    for (i, tweet) in t.iter().enumerate() {
                        if tweet.len() > 280 {
                            return Err(ApiError::BadRequest(format!(
                                "tweet {} exceeds 280 characters",
                                i + 1
                            )));
                        }
                    }
                }
                Err(_) => {
                    return Err(ApiError::BadRequest(
                        "thread content must be a JSON array of strings".to_string(),
                    ));
                }
            }
        }
        _ => {
            return Err(ApiError::BadRequest(
                "content_type must be 'tweet' or 'thread'".to_string(),
            ));
        }
    }

    let approval_mode = read_approval_mode(&state)?;

    if approval_mode {
        let id =
            approval_queue::enqueue(&state.db, &body.content_type, "", "", &content, "", "", 0.0)
                .await?;

        let _ = state.event_tx.send(WsEvent::ApprovalQueued {
            id,
            action_type: body.content_type,
            content: content.clone(),
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else {
        let id = scheduled_content::insert(
            &state.db,
            &body.content_type,
            &content,
            body.scheduled_for.as_deref(),
        )
        .await?;

        let _ = state.event_tx.send(WsEvent::ContentScheduled {
            id,
            content_type: body.content_type,
            scheduled_for: body.scheduled_for,
        });

        Ok(Json(json!({
            "status": "scheduled",
            "id": id,
        })))
    }
}

/// Request body for editing a scheduled content item.
#[derive(Deserialize)]
pub struct EditScheduledRequest {
    /// Updated content text.
    pub content: Option<String>,
    /// Updated scheduled time.
    pub scheduled_for: Option<String>,
}

/// `PATCH /api/content/scheduled/{id}` — edit a scheduled content item.
pub async fn edit_scheduled(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<EditScheduledRequest>,
) -> Result<Json<Value>, ApiError> {
    let item = scheduled_content::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("scheduled content {id} not found")))?;

    if item.status != "scheduled" {
        return Err(ApiError::BadRequest(
            "can only edit items with status 'scheduled'".to_string(),
        ));
    }

    let new_content = body.content.as_deref().unwrap_or(&item.content);
    let new_scheduled_for = match &body.scheduled_for {
        Some(t) => Some(t.as_str()),
        None => item.scheduled_for.as_deref(),
    };

    scheduled_content::update_content(&state.db, id, new_content, new_scheduled_for).await?;

    let updated = scheduled_content::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("scheduled content {id} not found")))?;

    Ok(Json(json!(updated)))
}

/// `DELETE /api/content/scheduled/{id}` — cancel a scheduled content item.
pub async fn cancel_scheduled(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let item = scheduled_content::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("scheduled content {id} not found")))?;

    if item.status != "scheduled" {
        return Err(ApiError::BadRequest(
            "can only cancel items with status 'scheduled'".to_string(),
        ));
    }

    scheduled_content::cancel(&state.db, id).await?;

    Ok(Json(json!({
        "status": "cancelled",
        "id": id,
    })))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read `approval_mode` from the config file.
fn read_approval_mode(state: &AppState) -> Result<bool, ApiError> {
    let config = read_config(state)?;
    Ok(config.approval_mode)
}

/// Read the full config from the config file.
fn read_config(state: &AppState) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).unwrap_or_default();
    let config: Config = toml::from_str(&contents).unwrap_or_default();
    Ok(config)
}
