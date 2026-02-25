//! AI assist endpoints for on-demand content generation.
//!
//! These are stateless: they generate content and return it without posting.
//! The user decides what to do with the results.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use tuitbot_core::content::ContentGenerator;
use tuitbot_core::storage;

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_generator(state: &AppState) -> Result<&Arc<ContentGenerator>, ApiError> {
    state.content_generator.as_ref().ok_or(ApiError::BadRequest(
        "LLM not configured — set llm.provider and llm.api_key in config.toml".to_string(),
    ))
}

// ---------------------------------------------------------------------------
// POST /api/assist/tweet
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistTweetRequest {
    pub topic: String,
}

#[derive(Serialize)]
pub struct AssistTweetResponse {
    pub content: String,
    pub topic: String,
}

pub async fn assist_tweet(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AssistTweetRequest>,
) -> Result<Json<AssistTweetResponse>, ApiError> {
    let gen = get_generator(&state)?;

    let output = gen
        .generate_tweet(&body.topic)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistTweetResponse {
        content: output.text,
        topic: body.topic,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/assist/reply
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistReplyRequest {
    pub tweet_text: String,
    pub tweet_author: String,
    #[serde(default)]
    pub mention_product: bool,
}

#[derive(Serialize)]
pub struct AssistReplyResponse {
    pub content: String,
}

pub async fn assist_reply(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AssistReplyRequest>,
) -> Result<Json<AssistReplyResponse>, ApiError> {
    let gen = get_generator(&state)?;

    let output = gen
        .generate_reply(&body.tweet_text, &body.tweet_author, body.mention_product)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistReplyResponse {
        content: output.text,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/assist/thread
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistThreadRequest {
    pub topic: String,
}

#[derive(Serialize)]
pub struct AssistThreadResponse {
    pub tweets: Vec<String>,
    pub topic: String,
}

pub async fn assist_thread(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AssistThreadRequest>,
) -> Result<Json<AssistThreadResponse>, ApiError> {
    let gen = get_generator(&state)?;

    let output = gen
        .generate_thread(&body.topic)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistThreadResponse {
        tweets: output.tweets,
        topic: body.topic,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/assist/improve
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistImproveRequest {
    pub draft: String,
    #[serde(default)]
    pub context: Option<String>,
}

#[derive(Serialize)]
pub struct AssistImproveResponse {
    pub content: String,
}

pub async fn assist_improve(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AssistImproveRequest>,
) -> Result<Json<AssistImproveResponse>, ApiError> {
    let gen = get_generator(&state)?;

    // Use the tweet generation path with the draft as the "topic",
    // rephrased as an improvement request.
    let prompt = if let Some(ctx) = &body.context {
        format!(
            "Rewrite and improve this draft tweet in the brand voice. Context: {}. Draft: {}",
            ctx, body.draft
        )
    } else {
        format!(
            "Rewrite and improve this draft tweet in the brand voice. Draft: {}",
            body.draft
        )
    };

    let output = gen
        .generate_tweet(&prompt)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssistImproveResponse {
        content: output.text,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/assist/topics
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AssistTopicsResponse {
    pub topics: Vec<TopicRecommendation>,
}

#[derive(Serialize)]
pub struct TopicRecommendation {
    pub topic: String,
    pub score: f64,
}

pub async fn assist_topics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AssistTopicsResponse>, ApiError> {
    let top = storage::analytics::get_top_topics(&state.db, 10).await?;

    let topics = top
        .into_iter()
        .map(|cs| TopicRecommendation {
            topic: cs.topic,
            score: cs.avg_performance,
        })
        .collect();

    Ok(Json(AssistTopicsResponse { topics }))
}

// ---------------------------------------------------------------------------
// GET /api/assist/optimal-times
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct OptimalTimesResponse {
    pub times: Vec<OptimalTime>,
}

#[derive(Serialize)]
pub struct OptimalTime {
    pub hour: u32,
    pub avg_engagement: f64,
    pub post_count: i64,
}

pub async fn assist_optimal_times(
    State(state): State<Arc<AppState>>,
) -> Result<Json<OptimalTimesResponse>, ApiError> {
    let rows = storage::analytics::get_optimal_posting_times(&state.db).await?;

    let times = rows
        .into_iter()
        .map(|r| OptimalTime {
            hour: r.hour as u32,
            avg_engagement: r.avg_engagement,
            post_count: r.post_count,
        })
        .collect();

    Ok(Json(OptimalTimesResponse { times }))
}

// ---------------------------------------------------------------------------
// GET /api/assist/mode
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ModeResponse {
    pub mode: String,
    pub approval_mode: bool,
}

pub async fn get_mode(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ModeResponse>), ApiError> {
    // Read mode from config file.
    let config = tuitbot_core::config::Config::load(Some(&state.config_path.to_string_lossy()))
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(ModeResponse {
            mode: config.mode.to_string(),
            approval_mode: config.effective_approval_mode(),
        }),
    ))
}
