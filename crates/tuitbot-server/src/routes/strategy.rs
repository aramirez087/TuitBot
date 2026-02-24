//! Strategy endpoints — weekly reports, history, and strategy inputs.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::config::Config;
use tuitbot_core::storage::strategy;

use crate::error::ApiError;
use crate::state::AppState;

/// Query parameters for the history endpoint.
#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    pub limit: u32,
}

fn default_history_limit() -> u32 {
    12
}

/// `GET /api/strategy/current` — current week's report (computed on-the-fly if missing).
pub async fn current(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let config = load_config(&state)?;
    let report = tuitbot_core::strategy::report::get_or_compute_current(&state.db, &config).await?;
    Ok(Json(report_to_json(report)))
}

/// `GET /api/strategy/history` — recent weekly reports for trend view.
pub async fn history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Value>, ApiError> {
    let reports = strategy::get_recent_reports(&state.db, params.limit).await?;
    let items: Vec<Value> = reports.into_iter().map(report_to_json).collect();
    Ok(Json(json!(items)))
}

/// `POST /api/strategy/refresh` — force recompute the current week's report.
pub async fn refresh(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let config = load_config(&state)?;
    let report = tuitbot_core::strategy::report::refresh_current(&state.db, &config).await?;
    Ok(Json(report_to_json(report)))
}

/// `GET /api/strategy/inputs` — current strategy inputs (pillars, keywords, targets, topics).
pub async fn inputs(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let config = load_config(&state)?;

    let targets =
        tuitbot_core::storage::target_accounts::get_active_target_accounts(&state.db).await?;
    let target_usernames: Vec<String> = targets.into_iter().map(|t| t.username).collect();

    Ok(Json(json!({
        "content_pillars": config.business.content_pillars,
        "industry_topics": config.business.industry_topics,
        "product_keywords": config.business.product_keywords,
        "competitor_keywords": config.business.competitor_keywords,
        "target_accounts": target_usernames,
    })))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_config(state: &AppState) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            state.config_path.display()
        ))
    })?;
    let config: Config = toml::from_str(&contents)
        .map_err(|e| ApiError::BadRequest(format!("failed to parse config: {e}")))?;
    Ok(config)
}

fn report_to_json(report: strategy::StrategyReportRow) -> Value {
    let top_topics: Value =
        serde_json::from_str(&report.top_topics_json).unwrap_or_else(|_| json!([]));
    let bottom_topics: Value =
        serde_json::from_str(&report.bottom_topics_json).unwrap_or_else(|_| json!([]));
    let top_content: Value =
        serde_json::from_str(&report.top_content_json).unwrap_or_else(|_| json!([]));
    let recommendations: Value =
        serde_json::from_str(&report.recommendations_json).unwrap_or_else(|_| json!([]));

    json!({
        "id": report.id,
        "week_start": report.week_start,
        "week_end": report.week_end,
        "replies_sent": report.replies_sent,
        "tweets_posted": report.tweets_posted,
        "threads_posted": report.threads_posted,
        "target_replies": report.target_replies,
        "follower_start": report.follower_start,
        "follower_end": report.follower_end,
        "follower_delta": report.follower_delta,
        "avg_reply_score": report.avg_reply_score,
        "avg_tweet_score": report.avg_tweet_score,
        "reply_acceptance_rate": report.reply_acceptance_rate,
        "estimated_follow_conversion": report.estimated_follow_conversion,
        "top_topics": top_topics,
        "bottom_topics": bottom_topics,
        "top_content": top_content,
        "recommendations": recommendations,
        "created_at": report.created_at,
    })
}
