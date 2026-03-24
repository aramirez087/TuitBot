//! Analytics endpoints.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::analytics;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

/// Query parameters for the followers endpoint.
#[derive(Deserialize)]
pub struct FollowersQuery {
    /// Number of days of follower snapshots to return (default: 7).
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 {
    7
}

/// Query parameters for the topics endpoint.
#[derive(Deserialize)]
pub struct TopicsQuery {
    /// Maximum number of topics to return (default: 20).
    #[serde(default = "default_topic_limit")]
    pub limit: u32,
}

fn default_topic_limit() -> u32 {
    20
}

/// Query parameters for the recent-performance endpoint.
#[derive(Deserialize)]
pub struct RecentPerformanceQuery {
    /// Maximum number of items to return (default: 20).
    #[serde(default = "default_recent_limit")]
    pub limit: u32,
}

fn default_recent_limit() -> u32 {
    20
}

/// `GET /api/analytics/followers` — follower snapshots over time.
pub async fn followers(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<FollowersQuery>,
) -> Result<Json<Value>, ApiError> {
    let snapshots =
        analytics::get_follower_snapshots_for(&state.db, &ctx.account_id, params.days).await?;
    Ok(Json(json!(snapshots)))
}

/// `GET /api/analytics/performance` — reply and tweet performance summaries.
pub async fn performance(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let avg_reply = analytics::get_avg_reply_engagement_for(&state.db, &ctx.account_id).await?;
    let avg_tweet = analytics::get_avg_tweet_engagement_for(&state.db, &ctx.account_id).await?;
    let (reply_count, tweet_count) =
        analytics::get_performance_counts_for(&state.db, &ctx.account_id).await?;

    Ok(Json(json!({
        "avg_reply_engagement": avg_reply,
        "avg_tweet_engagement": avg_tweet,
        "measured_replies": reply_count,
        "measured_tweets": tweet_count,
    })))
}

/// `GET /api/analytics/topics` — topic performance scores.
pub async fn topics(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<TopicsQuery>,
) -> Result<Json<Value>, ApiError> {
    let scores = analytics::get_top_topics_for(&state.db, &ctx.account_id, params.limit).await?;
    Ok(Json(json!(scores)))
}

/// `GET /api/analytics/summary` — combined analytics dashboard summary.
pub async fn summary(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let data = analytics::get_analytics_summary_for(&state.db, &ctx.account_id).await?;
    Ok(Json(json!(data)))
}

/// `GET /api/analytics/recent-performance` — recent content with performance metrics.
pub async fn recent_performance(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<RecentPerformanceQuery>,
) -> Result<Json<Value>, ApiError> {
    let items =
        analytics::get_recent_performance_items_for(&state.db, &ctx.account_id, params.limit)
            .await?;
    Ok(Json(json!(items)))
}

/// Query parameters for engagement-rate endpoint.
#[derive(Deserialize)]
pub struct EngagementRateQuery {
    /// Maximum number of posts to return (default: 20).
    #[serde(default = "default_engagement_limit")]
    pub limit: u32,
}

fn default_engagement_limit() -> u32 {
    20
}

/// `GET /api/analytics/engagement-rate` — top posts by engagement rate (for charting).
pub async fn engagement_rate(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<EngagementRateQuery>,
) -> Result<Json<Value>, ApiError> {
    let metrics =
        analytics::get_engagement_rate_for(&state.db, &ctx.account_id, params.limit).await?;
    Ok(Json(json!(metrics)))
}

/// Query parameters for reach endpoint.
#[derive(Deserialize)]
pub struct ReachQuery {
    /// Number of days of reach data to return (default: 7).
    #[serde(default = "default_reach_days")]
    pub window: u32,
}

fn default_reach_days() -> u32 {
    7
}

/// `GET /api/analytics/reach` — reach time-series by day (for charting).
pub async fn reach(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<ReachQuery>,
) -> Result<Json<Value>, ApiError> {
    let snapshots = analytics::get_reach_for(&state.db, &ctx.account_id, params.window).await?;
    Ok(Json(json!(snapshots)))
}

/// Query parameters for follower-growth endpoint.
#[derive(Deserialize)]
pub struct FollowerGrowthQuery {
    /// Number of days of follower growth data to return (default: 30).
    #[serde(default = "default_growth_days")]
    pub window: u32,
}

fn default_growth_days() -> u32 {
    30
}

/// `GET /api/analytics/follower-growth` — follower growth time-series with deltas (for charting).
pub async fn follower_growth(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<FollowerGrowthQuery>,
) -> Result<Json<Value>, ApiError> {
    let snapshots =
        analytics::get_follower_growth_for(&state.db, &ctx.account_id, params.window).await?;
    Ok(Json(json!(snapshots)))
}

/// `GET /api/analytics/best-times` — ranked posting time slots by engagement.
pub async fn best_times(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let slots = analytics::get_best_times_for(&state.db, &ctx.account_id).await?;
    Ok(Json(json!(slots)))
}

/// `GET /api/analytics/heatmap` — 7×24 best-time heatmap grid.
pub async fn heatmap(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let grid = analytics::get_heatmap_for(&state.db, &ctx.account_id).await?;
    Ok(Json(json!(grid)))
}

/// `GET /api/analytics/content-breakdown` — content performance by type.
pub async fn content_breakdown(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let breakdown = analytics::get_content_breakdown_for(&state.db, &ctx.account_id).await?;
    Ok(Json(json!(breakdown)))
}
