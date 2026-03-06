//! Discovery feed endpoints for browsing scored tweets and composing replies.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tuitbot_core::config::Config;
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::storage::{self, approval_queue};

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn get_generator(
    state: &AppState,
    account_id: &str,
) -> Result<Arc<ContentGenerator>, ApiError> {
    let generators = state.content_generators.lock().await;
    generators
        .get(account_id)
        .cloned()
        .ok_or(ApiError::BadRequest(
            "LLM not configured — set llm.provider and llm.api_key in config.toml".to_string(),
        ))
}

// ---------------------------------------------------------------------------
// GET /api/discovery/feed
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    pub max_score: Option<f64>,
    pub keyword: Option<String>,
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
    ctx: AccountContext,
    Query(q): Query<FeedQuery>,
) -> Result<Json<Vec<DiscoveryTweet>>, ApiError> {
    let rows = storage::tweets::get_discovery_feed_filtered_for(
        &state.db,
        &ctx.account_id,
        q.min_score,
        q.max_score,
        q.keyword.as_deref(),
        q.limit,
    )
    .await?;

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
// GET /api/discovery/keywords
// ---------------------------------------------------------------------------

pub async fn keywords(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Vec<String>>, ApiError> {
    let kws = storage::tweets::get_distinct_keywords_for(&state.db, &ctx.account_id).await?;
    Ok(Json(kws))
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
    ctx: AccountContext,
    Path(tweet_id): Path<String>,
    Json(body): Json<ComposeReplyRequest>,
) -> Result<Json<ComposeReplyResponse>, ApiError> {
    let gen = get_generator(&state, &ctx.account_id).await?;

    // Fetch the tweet content from discovered_tweets.
    let tweet = storage::tweets::get_tweet_by_id_for(&state.db, &ctx.account_id, &tweet_id)
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
    ctx: AccountContext,
    Path(tweet_id): Path<String>,
    Json(body): Json<QueueReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Block posting unless the backend can actually post.
    let config: Config = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();
    let can_post = match config.x_api.provider_backend.as_str() {
        "x_api" => true,
        "scraper" => state.data_dir.join("scraper_session.json").exists(),
        _ => false,
    };
    if !can_post {
        return Err(ApiError::BadRequest(
            "Direct posting requires X API credentials or an imported browser session. \
             Configure in Settings → X API."
                .to_string(),
        ));
    }

    if body.content.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "content must not be empty".to_string(),
        ));
    }

    // Look up author from discovered_tweets.
    let target_author = storage::tweets::get_tweet_by_id_for(&state.db, &ctx.account_id, &tweet_id)
        .await?
        .map(|t| t.author_username)
        .unwrap_or_default();

    let queue_id = approval_queue::enqueue_for(
        &state.db,
        &ctx.account_id,
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
    storage::approval_queue::update_status_for(&state.db, &ctx.account_id, queue_id, "approved")
        .await?;

    Ok(Json(json!({
        "approval_queue_id": queue_id,
        "tweet_id": tweet_id,
        "status": "queued_for_posting"
    })))
}
