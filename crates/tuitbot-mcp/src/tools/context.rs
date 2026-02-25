//! MCP tools for context-aware intelligence.
//!
//! Wraps `tuitbot_core::context` functions with the unified
//! [`ToolResponse`] envelope and timing metadata.

use std::time::Instant;

use crate::state::SharedState;
use crate::tools::response::{ToolMeta, ToolResponse};

/// Get a rich context profile for an author including interaction history,
/// conversation records, response metrics, topic affinity, and risk signals.
pub async fn get_author_context(state: &SharedState, identifier: &str) -> String {
    let start = Instant::now();
    match tuitbot_core::context::author::get_author_context(&state.pool, identifier, &state.config)
        .await
    {
        Ok(ctx) => ToolResponse::success(ctx)
            .with_meta(ToolMeta::new(start.elapsed().as_millis() as u64))
            .to_json(),
        Err(e) => ToolResponse::error("context_error", e.to_string(), false)
            .with_meta(ToolMeta::new(start.elapsed().as_millis() as u64))
            .to_json(),
    }
}

/// Produce an explainable engagement recommendation for a given author,
/// tweet text, and optional campaign objective.
pub async fn recommend_engagement(
    state: &SharedState,
    author_username: &str,
    tweet_text: &str,
    campaign_objective: Option<&str>,
) -> String {
    let start = Instant::now();
    match tuitbot_core::context::engagement::recommend_engagement(
        &state.pool,
        author_username,
        tweet_text,
        campaign_objective,
        &state.config,
    )
    .await
    {
        Ok(rec) => ToolResponse::success(rec)
            .with_meta(ToolMeta::new(start.elapsed().as_millis() as u64))
            .to_json(),
        Err(e) => ToolResponse::error("recommendation_error", e.to_string(), false)
            .with_meta(ToolMeta::new(start.elapsed().as_millis() as u64))
            .to_json(),
    }
}

/// Get a time-windowed topic performance snapshot with ranked topics
/// and double-down/reduce recommendations.
pub async fn topic_performance_snapshot(pool: &tuitbot_core::storage::DbPool, days: u32) -> String {
    let start = Instant::now();
    match tuitbot_core::context::topics::get_topic_snapshot(pool, days).await {
        Ok(snapshot) => ToolResponse::success(snapshot)
            .with_meta(ToolMeta::new(start.elapsed().as_millis() as u64))
            .to_json(),
        Err(e) => ToolResponse::error("topic_error", e.to_string(), false)
            .with_meta(ToolMeta::new(start.elapsed().as_millis() as u64))
            .to_json(),
    }
}
