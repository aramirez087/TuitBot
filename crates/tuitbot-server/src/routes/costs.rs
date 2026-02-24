//! Cost tracking endpoints — LLM usage summaries and breakdowns.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use tuitbot_core::storage::llm_usage;

use crate::error::ApiError;
use crate::state::AppState;

/// Query parameters for endpoints that accept a `days` window.
#[derive(Deserialize)]
pub struct DaysQuery {
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 {
    30
}

/// `GET /api/costs/summary` — cost totals across time windows.
pub async fn summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<llm_usage::CostSummary>, ApiError> {
    let summary = llm_usage::get_cost_summary(&state.db).await?;
    Ok(Json(summary))
}

/// `GET /api/costs/daily?days=30` — per-day cost data for charts.
pub async fn daily(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DaysQuery>,
) -> Result<Json<Vec<llm_usage::DailyCostSummary>>, ApiError> {
    let data = llm_usage::get_daily_costs(&state.db, params.days).await?;
    Ok(Json(data))
}

/// `GET /api/costs/by-model?days=30` — cost breakdown by provider + model.
pub async fn by_model(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DaysQuery>,
) -> Result<Json<Vec<llm_usage::ModelCostBreakdown>>, ApiError> {
    let data = llm_usage::get_model_breakdown(&state.db, params.days).await?;
    Ok(Json(data))
}

/// `GET /api/costs/by-type?days=30` — cost breakdown by generation type.
pub async fn by_type(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DaysQuery>,
) -> Result<Json<Vec<llm_usage::TypeCostBreakdown>>, ApiError> {
    let data = llm_usage::get_type_breakdown(&state.db, params.days).await?;
    Ok(Json(data))
}
