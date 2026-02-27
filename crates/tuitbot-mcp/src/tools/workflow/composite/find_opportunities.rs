//! `find_reply_opportunities` — search, score, and rank tweets in one call.
//!
//! Delegates to `tuitbot_core::workflow::discover` for the actual logic,
//! adding only MCP response envelope wrapping and telemetry.

use std::time::Instant;

use tuitbot_core::workflow::discover::{self, DiscoverInput};
use tuitbot_core::workflow::WorkflowError;

use crate::state::SharedState;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

/// Execute the `find_reply_opportunities` composite tool.
pub async fn execute(
    state: &SharedState,
    query: Option<&str>,
    min_score: Option<f64>,
    limit: Option<u32>,
    since_id: Option<&str>,
) -> String {
    let start = Instant::now();

    // Require X client
    let x_client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::x_not_configured()
                .with_meta(ToolMeta::new(elapsed))
                .to_json();
        }
    };

    // Delegate to core workflow step
    let result = discover::execute(
        &state.pool,
        x_client,
        &state.config,
        DiscoverInput {
            query: query.map(String::from),
            min_score,
            limit,
            since_id: since_id.map(String::from),
        },
    )
    .await;

    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => {
            let total = output.candidates.len();
            crate::tools::workflow::telemetry::record(
                &state.pool,
                "find_reply_opportunities",
                "composite",
                elapsed,
                true,
                None,
                None,
                None,
            )
            .await;
            ToolResponse::success(serde_json::json!({
                "candidates": output.candidates,
                "total_searched": total,
                "query": output.query_used,
                "threshold": output.threshold,
            }))
            .with_meta(ToolMeta::new(elapsed).with_workflow(
                state.config.mode.to_string(),
                state.config.effective_approval_mode(),
            ))
            .to_json()
        }
        Err(e) => {
            let code = workflow_error_to_code(&e);
            crate::tools::workflow::telemetry::record(
                &state.pool,
                "find_reply_opportunities",
                "composite",
                elapsed,
                false,
                Some(code.as_str()),
                None,
                None,
            )
            .await;
            ToolResponse::error(code, e.to_string())
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
    }
}

/// Map a `WorkflowError` to an MCP `ErrorCode`.
fn workflow_error_to_code(e: &WorkflowError) -> ErrorCode {
    match e {
        WorkflowError::InvalidInput(_) => ErrorCode::InvalidInput,
        WorkflowError::XNotConfigured => ErrorCode::XNotConfigured,
        WorkflowError::LlmNotConfigured => ErrorCode::LlmNotConfigured,
        WorkflowError::Llm(_) => ErrorCode::LlmError,
        WorkflowError::Database(_) | WorkflowError::Storage(_) => ErrorCode::DbError,
        WorkflowError::Toolkit(te) => match te {
            tuitbot_core::toolkit::ToolkitError::XApi(_) => ErrorCode::XApiError,
            tuitbot_core::toolkit::ToolkitError::InvalidInput { .. } => ErrorCode::InvalidInput,
            tuitbot_core::toolkit::ToolkitError::TweetTooLong { .. } => ErrorCode::InvalidInput,
            _ => ErrorCode::XApiError,
        },
    }
}
