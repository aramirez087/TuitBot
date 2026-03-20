//! Strategy endpoints — weekly reports, history, and strategy inputs.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::config::Config;
use tuitbot_core::storage::strategy;

use crate::account::{require_mutate, AccountContext};
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

/// `GET /api/strategy/current` — current week's report for the requesting account.
pub async fn current(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let config = load_config(&state)?;
    let report = tuitbot_core::strategy::report::get_or_compute_current_for(
        &state.db,
        &config,
        &ctx.account_id,
    )
    .await?;
    Ok(Json(report_to_json(report)))
}

/// `GET /api/strategy/history` — recent weekly reports for trend view.
pub async fn history(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Value>, ApiError> {
    let reports =
        strategy::get_recent_reports_for(&state.db, &ctx.account_id, params.limit).await?;
    let items: Vec<Value> = reports.into_iter().map(report_to_json).collect();
    Ok(Json(json!(items)))
}

/// `POST /api/strategy/refresh` — force recompute the current week's report for the requesting account.
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let config = load_config(&state)?;
    let report =
        tuitbot_core::strategy::report::refresh_current_for(&state.db, &config, &ctx.account_id)
            .await?;
    Ok(Json(report_to_json(report)))
}

/// `GET /api/strategy/inputs` — current strategy inputs (pillars, keywords, targets, topics).
pub async fn inputs(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Value>, ApiError> {
    let config = load_config(&state)?;

    let targets = tuitbot_core::storage::target_accounts::get_active_target_accounts_for(
        &state.db,
        &ctx.account_id,
    )
    .await?;
    let target_usernames: Vec<String> = targets.into_iter().map(|t| t.username).collect();

    Ok(Json(json!({
        "content_pillars": config.business.content_pillars,
        "industry_topics": config.business.effective_industry_topics(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_history_limit_is_12() {
        assert_eq!(default_history_limit(), 12);
    }

    #[test]
    fn history_query_default_limit() {
        let json = "{}";
        let q: HistoryQuery = serde_json::from_str(json).expect("deser");
        assert_eq!(q.limit, 12);
    }

    #[test]
    fn history_query_custom_limit() {
        let json = r#"{"limit": 5}"#;
        let q: HistoryQuery = serde_json::from_str(json).expect("deser");
        assert_eq!(q.limit, 5);
    }

    #[test]
    fn report_to_json_parses_arrays() {
        let report = strategy::StrategyReportRow {
            id: 1,
            week_start: "2026-03-09".into(),
            week_end: "2026-03-15".into(),
            replies_sent: 10,
            tweets_posted: 5,
            threads_posted: 2,
            target_replies: 3,
            follower_start: 100,
            follower_end: 120,
            follower_delta: 20,
            avg_reply_score: 75.0,
            avg_tweet_score: 80.0,
            reply_acceptance_rate: 0.85,
            estimated_follow_conversion: 0.02,
            top_topics_json: r#"["rust","wasm"]"#.into(),
            bottom_topics_json: "[]".into(),
            top_content_json: "[]".into(),
            recommendations_json: r#"["post more"]"#.into(),
            created_at: "2026-03-15T10:00:00Z".into(),
        };
        let val = report_to_json(report);
        assert_eq!(val["replies_sent"], 10);
        assert_eq!(val["follower_delta"], 20);
        assert!(val["top_topics"].is_array());
        assert_eq!(val["top_topics"][0], "rust");
        assert!(val["recommendations"].is_array());
    }

    #[test]
    fn report_to_json_handles_invalid_json() {
        let report = strategy::StrategyReportRow {
            id: 1,
            week_start: "2026-03-09".into(),
            week_end: "2026-03-15".into(),
            replies_sent: 0,
            tweets_posted: 0,
            threads_posted: 0,
            target_replies: 0,
            follower_start: 0,
            follower_end: 0,
            follower_delta: 0,
            avg_reply_score: 0.0,
            avg_tweet_score: 0.0,
            reply_acceptance_rate: 0.0,
            estimated_follow_conversion: 0.0,
            top_topics_json: "invalid".into(),
            bottom_topics_json: "also-invalid".into(),
            top_content_json: "nope".into(),
            recommendations_json: "bad".into(),
            created_at: "2026-03-15T10:00:00Z".into(),
        };
        let val = report_to_json(report);
        // Should fall back to empty arrays for invalid JSON
        assert!(val["top_topics"].is_array());
        assert_eq!(val["top_topics"].as_array().unwrap().len(), 0);
    }
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
