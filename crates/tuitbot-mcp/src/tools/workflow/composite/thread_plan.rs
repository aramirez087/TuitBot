//! `generate_thread_plan` — plan and generate a thread with analysis.
//!
//! Delegates to `tuitbot_core::workflow::thread_plan` for the actual logic,
//! adding only MCP response envelope wrapping and telemetry.

use std::sync::Arc;
use std::time::Instant;

use tuitbot_core::workflow::thread_plan::{self, ThreadPlanInput};
use tuitbot_core::workflow::WorkflowError;

use crate::state::SharedState;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

/// Execute the `generate_thread_plan` composite tool.
pub async fn execute(
    state: &SharedState,
    topic: &str,
    objective: Option<&str>,
    target_audience: Option<&str>,
    structure_str: Option<&str>,
) -> String {
    let start = Instant::now();

    // Require LLM provider
    let llm: Arc<dyn tuitbot_core::llm::LlmProvider> = match &state.llm_provider {
        Some(_) => Arc::new(crate::tools::workflow::content::ArcProvider {
            state: Arc::clone(state),
        }),
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                ErrorCode::LlmNotConfigured,
                "No LLM provider configured. Set up the [llm] section in config.toml.",
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    // Delegate to core workflow step
    let result = thread_plan::execute(
        &llm,
        &state.config,
        ThreadPlanInput {
            topic: topic.to_string(),
            objective: objective.map(String::from),
            target_audience: target_audience.map(String::from),
            structure: structure_str.map(String::from),
        },
    )
    .await;

    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => {
            crate::tools::workflow::telemetry::record(
                &state.pool,
                "generate_thread_plan",
                "composite",
                elapsed,
                true,
                None,
                None,
                None,
            )
            .await;
            ToolResponse::success(serde_json::json!({
                "thread_tweets": output.thread_tweets,
                "tweet_count": output.tweet_count,
                "structure_used": output.structure_used,
                "hook_analysis": {
                    "type": output.hook_type,
                    "first_tweet_preview": output.first_tweet_preview,
                },
                "estimated_performance": output.estimated_performance,
                "objective_alignment": output.objective_alignment,
                "target_audience": output.target_audience,
                "topic_relevance": output.topic_relevance,
            }))
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
        WorkflowError::LlmNotConfigured => ErrorCode::LlmNotConfigured,
        WorkflowError::Llm(_) => ErrorCode::LlmError,
        WorkflowError::InvalidInput(_) => ErrorCode::InvalidInput,
        _ => ErrorCode::XApiError,
    }
}
