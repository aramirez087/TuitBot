//! `draft_replies_for_candidates` — generate reply drafts for discovered tweets.
//!
//! Delegates to `tuitbot_core::workflow::draft` for the actual logic,
//! adding only MCP response envelope wrapping and telemetry.

use std::sync::Arc;
use std::time::Instant;

use tuitbot_core::workflow::draft::{self, DraftInput};
use tuitbot_core::workflow::WorkflowError;

use crate::state::SharedState;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

/// Execute the `draft_replies_for_candidates` composite tool.
pub async fn execute(
    state: &SharedState,
    candidate_ids: &[String],
    archetype_str: Option<&str>,
    mention_product: bool,
) -> String {
    let start = Instant::now();

    // Validate input before checking provider
    if candidate_ids.is_empty() {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::error(ErrorCode::InvalidInput, "candidate_ids must not be empty.")
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
    }

    // Need LLM provider — wrap in Arc for the workflow step
    let llm: Arc<dyn tuitbot_core::llm::LlmProvider> = match &state.llm_provider {
        Some(_provider) => Arc::new(crate::tools::workflow::content::ArcProvider {
            state: Arc::clone(state),
        }),
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(ErrorCode::LlmNotConfigured, "No LLM provider configured.")
                .with_meta(ToolMeta::new(elapsed))
                .to_json();
        }
    };

    // Delegate to core workflow step
    let result = draft::execute(
        &state.pool,
        &llm,
        &state.config,
        DraftInput {
            candidate_ids: candidate_ids.to_vec(),
            archetype: archetype_str.map(String::from),
            mention_product,
        },
    )
    .await;

    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(results) => {
            crate::tools::workflow::telemetry::record(
                &state.pool,
                "draft_replies_for_candidates",
                "composite",
                elapsed,
                true,
                None,
                None,
                None,
            )
            .await;
            ToolResponse::success(&results)
                .with_meta(ToolMeta::new(elapsed).with_workflow(
                    state.config.mode.to_string(),
                    state.config.effective_approval_mode(),
                ))
                .to_json()
        }
        Err(e) => {
            let code = workflow_error_to_code(&e);
            ToolResponse::error(code, e.to_string())
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
    }
}

fn workflow_error_to_code(e: &WorkflowError) -> ErrorCode {
    match e {
        WorkflowError::InvalidInput(_) => ErrorCode::InvalidInput,
        WorkflowError::LlmNotConfigured => ErrorCode::LlmNotConfigured,
        WorkflowError::Llm(_) => ErrorCode::LlmError,
        WorkflowError::Database(_) | WorkflowError::Storage(_) => ErrorCode::DbError,
        _ => ErrorCode::XApiError,
    }
}
