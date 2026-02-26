//! Mutation audit query tools: get_recent_mutations, get_mutation_detail.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage::mutation_audit;
use tuitbot_core::storage::DbPool;

use crate::tools::response::{ToolMeta, ToolResponse};

/// Slim projection of a mutation audit entry for the query response.
#[derive(Serialize)]
struct MutationSummary {
    correlation_id: String,
    tool_name: String,
    status: String,
    params_summary: String,
    result_summary: Option<String>,
    error_message: Option<String>,
    elapsed_ms: Option<i64>,
    created_at: String,
    completed_at: Option<String>,
}

/// Get recent mutation audit entries with optional filters.
pub async fn get_recent_mutations(
    pool: &DbPool,
    limit: u32,
    tool_name: Option<&str>,
    status: Option<&str>,
    config: &Config,
) -> String {
    let start = Instant::now();
    let effective_limit = limit.clamp(1, 100);

    match mutation_audit::get_recent(pool, effective_limit, tool_name, status).await {
        Ok(entries) => {
            let summaries: Vec<MutationSummary> = entries
                .into_iter()
                .map(|e| MutationSummary {
                    correlation_id: e.correlation_id,
                    tool_name: e.tool_name,
                    status: e.status,
                    params_summary: e.params_summary,
                    result_summary: e.result_summary,
                    error_message: e.error_message,
                    elapsed_ms: e.elapsed_ms,
                    created_at: e.created_at,
                    completed_at: e.completed_at,
                })
                .collect();

            let count = summaries.len();
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());

            #[derive(Serialize)]
            struct Response {
                mutations: Vec<MutationSummary>,
                count: usize,
            }
            ToolResponse::success(Response {
                mutations: summaries,
                count,
            })
            .with_meta(meta)
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching mutation audit: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Get a single mutation's full detail by correlation ID.
pub async fn get_mutation_detail(pool: &DbPool, correlation_id: &str, config: &Config) -> String {
    let start = Instant::now();

    match mutation_audit::get_by_correlation_id(pool, correlation_id).await {
        Ok(Some(entry)) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(entry).with_meta(meta).to_json()
        }
        Ok(None) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::error(
                crate::tools::response::ErrorCode::InvalidInput,
                format!("No mutation found with correlation_id: {correlation_id}"),
            )
            .with_meta(meta)
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching mutation detail: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
