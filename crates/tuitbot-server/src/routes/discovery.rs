//! Discovery feed endpoints for browsing scored tweets and composing replies.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tuitbot_core::storage::{self, approval_queue};

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// GET /api/discovery/feed
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    #[serde(default = "default_feed_limit")]
    pub limit: u32,
}

fn default_min_score() -> f64 {
    50.0
}
fn default_feed_limit() -> u32 {
    20
}

#[derive(Serialize)]
pub struct DiscoveryTweet {
    pub id: String,
    pub author_username: String,
    pub content: String,
    pub relevance_score: f64,
    pub matched_keyword: Option<String>,
    pub like_count: i64,
    pub retweet_count: i64,
    pub reply_count: i64,
    pub replied_to: bool,
    pub discovered_at: String,
}

pub async fn feed(
    State(state): State<Arc<AppState>>,
    Query(q): Query<FeedQuery>,
) -> Result<Json<Vec<DiscoveryTweet>>, ApiError> {
    let rows = storage::tweets::get_discovery_feed(&state.db, q.min_score, q.limit).await?;

    let tweets = rows
        .into_iter()
        .map(|t| DiscoveryTweet {
            id: t.id,
            author_username: t.author_username,
            content: t.content,
            relevance_score: t.relevance_score.unwrap_or(0.0),
            matched_keyword: t.matched_keyword,
            like_count: t.like_count,
            retweet_count: t.retweet_count,
            reply_count: t.reply_count,
            replied_to: t.replied_to != 0,
            discovered_at: t.discovered_at,
        })
        .collect();

    Ok(Json(tweets))
}

// ---------------------------------------------------------------------------
// POST /api/discovery/{tweet_id}/compose-reply
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ComposeReplyRequest {
    #[serde(default)]
    pub mention_product: bool,
}

#[derive(Serialize)]
pub struct ComposeReplyResponse {
    pub content: String,
    pub tweet_id: String,
}

pub async fn compose_reply(
    State(state): State<Arc<AppState>>,
    Path(tweet_id): Path<String>,
    Json(body): Json<ComposeReplyRequest>,
) -> Result<Json<ComposeReplyResponse>, ApiError> {
    let gen = state
        .content_generator
        .as_ref()
        .ok_or(ApiError::BadRequest(
            "LLM not configured — set llm.provider and llm.api_key in config.toml".to_string(),
        ))?;

    // Fetch the tweet content from discovered_tweets.
    let tweet = storage::tweets::get_tweet_by_id(&state.db, &tweet_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Tweet {tweet_id} not found in discovered tweets"))
        })?;

    let output = gen
        .generate_reply(&tweet.content, &tweet.author_username, body.mention_product)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ComposeReplyResponse {
        content: output.text,
        tweet_id,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/discovery/{tweet_id}/queue-reply
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct QueueReplyRequest {
    pub content: String,
}

pub async fn queue_reply(
    State(state): State<Arc<AppState>>,
    Path(tweet_id): Path<String>,
    Json(body): Json<QueueReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    if body.content.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "content must not be empty".to_string(),
        ));
    }

    // Look up author from discovered_tweets.
    let target_author = storage::tweets::get_tweet_by_id(&state.db, &tweet_id)
        .await?
        .map(|t| t.author_username)
        .unwrap_or_default();

    let queue_id = approval_queue::enqueue(
        &state.db,
        "reply",
        &tweet_id,
        &target_author,
        &body.content,
        "",  // topic
        "",  // archetype
        0.0, // score
        "[]",
    )
    .await?;

    // Auto-approve for immediate posting.
    storage::approval_queue::update_status(&state.db, queue_id, "approved").await?;

    Ok(Json(json!({
        "approval_queue_id": queue_id,
        "tweet_id": tweet_id,
        "status": "queued_for_posting"
    })))
}
