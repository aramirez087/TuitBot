//! `propose_and_queue_replies` — safety-check, then queue or execute replies.
//!
//! Delegates to `tuitbot_core::workflow::queue` for the actual logic,
//! adding only MCP response envelope wrapping, policy gate check, and telemetry.

use std::sync::Arc;
use std::time::Instant;

use tuitbot_core::mcp_policy::McpPolicyEvaluator;
use tuitbot_core::workflow::queue::{self, QueueInput};
use tuitbot_core::workflow::{ProposeResult, QueueItem, WorkflowError};

use crate::requests::ProposeItem;
use crate::state::SharedState;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};
use crate::tools::workflow::policy_gate::{self, GateResult};

/// Execute the `propose_and_queue_replies` composite tool.
pub async fn execute(state: &SharedState, items: &[ProposeItem], mention_product: bool) -> String {
    let start = Instant::now();

    if items.is_empty() {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::error(ErrorCode::InvalidInput, "items must not be empty.")
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
    }

    // Global policy gate check (MCP-specific)
    let params = serde_json::json!({
        "item_count": items.len(),
        "mention_product": mention_product,
    })
    .to_string();
    match policy_gate::check_policy(state, "propose_and_queue_replies", &params, start).await {
        GateResult::EarlyReturn(r) => return r,
        GateResult::Proceed => {}
    }

    // Build LLM provider Arc (for auto-generation)
    let llm: Option<Arc<dyn tuitbot_core::llm::LlmProvider>> =
        state.llm_provider.as_ref().map(|_| {
            Arc::new(crate::tools::workflow::content::ArcProvider {
                state: Arc::clone(state),
            }) as Arc<dyn tuitbot_core::llm::LlmProvider>
        });

    // Convert ProposeItem → QueueItem
    let queue_items: Vec<QueueItem> = items
        .iter()
        .map(|i| QueueItem {
            candidate_id: i.candidate_id.clone(),
            pre_drafted_text: i.pre_drafted_text.clone(),
        })
        .collect();

    // Delegate to core workflow step
    let x_client = state.x_client.as_deref();
    let result = queue::execute(
        &state.pool,
        x_client,
        llm.as_ref(),
        &state.config,
        QueueInput {
            items: queue_items,
            mention_product,
        },
    )
    .await;

    // Record batch mutation for rate limiting
    let _ = McpPolicyEvaluator::record_mutation(
        &state.pool,
        "propose_and_queue_replies",
        &state.config.mcp_policy.rate_limits,
    )
    .await;

    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(results) => {
            let has_error = results
                .iter()
                .any(|r| matches!(r, ProposeResult::Blocked { .. }));
            crate::tools::workflow::telemetry::record(
                &state.pool,
                "propose_and_queue_replies",
                "composite_mutation",
                elapsed,
                !has_error
                    || results
                        .iter()
                        .any(|r| !matches!(r, ProposeResult::Blocked { .. })),
                None,
                Some("allow"),
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
        WorkflowError::XNotConfigured => ErrorCode::XNotConfigured,
        WorkflowError::LlmNotConfigured => ErrorCode::LlmNotConfigured,
        WorkflowError::Llm(_) => ErrorCode::LlmError,
        WorkflowError::Database(_) | WorkflowError::Storage(_) => ErrorCode::DbError,
        WorkflowError::Toolkit(te) => match te {
            tuitbot_core::toolkit::ToolkitError::XApi(_) => ErrorCode::XApiError,
            _ => ErrorCode::XApiError,
        },
    }
}
