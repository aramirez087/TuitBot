//! Admin-profile MCP server (all tools including universal requests).
//!
//! Superset of the write profile. Adds `x_get`, `x_post`, `x_put`, `x_delete`
//! for arbitrary X API endpoint access. Only available when explicitly configured.

mod handlers;
mod tools;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::requests::*;
use crate::state::SharedState;
use crate::tools::response::{ToolMeta, ToolResponse};
use crate::tools::workflow;

/// Admin-profile MCP server (all tools including universal requests).
#[derive(Clone)]
pub struct AdminMcpServer {
    state: SharedState,
    tool_router: ToolRouter<Self>,
}

impl AdminMcpServer {
    /// Create a new admin-profile MCP server with the given shared state.
    pub fn new(state: SharedState) -> Self {
        let mut router = Self::core_router();
        router.merge(Self::tools_router());
        router.merge(Self::handlers_router());
        Self {
            state,
            tool_router: router,
        }
    }
}

/// Convert `Option<&[KeyValue]>` to `Option<Vec<(String, String)>>`.
pub(super) fn kv_to_tuples(kv: Option<&[KeyValue]>) -> Option<Vec<(String, String)>> {
    kv.map(|pairs| {
        pairs
            .iter()
            .map(|kv| (kv.key.clone(), kv.value.clone()))
            .collect()
    })
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AdminMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot Admin MCP Server — full X growth assistant with universal request tools. \
                 Includes all write-profile tools plus x_get/x_post/x_put/x_delete for \
                 arbitrary X API endpoint access (policy-gated, audit-recorded mutations)."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tool_router(router = core_router)]
impl AdminMcpServer {
    /// Generate a reply to a tweet using the configured LLM provider. Requires LLM provider.
    #[tool]
    async fn generate_reply(
        &self,
        Parameters(req): Parameters<GenerateReplyRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let mention = req.mention_product.unwrap_or(false);
        let result = workflow::content::generate_reply(
            &self.state,
            &self.state.config.business,
            &req.tweet_text,
            &req.tweet_author,
            mention,
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Generate an original educational tweet using the configured LLM provider. Requires LLM provider.
    #[tool]
    async fn generate_tweet(
        &self,
        Parameters(req): Parameters<TopicRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let topic = req.topic.unwrap_or_else(|| {
            self.state
                .config
                .business
                .effective_industry_topics()
                .first()
                .cloned()
                .unwrap_or_else(|| "general industry trends".to_string())
        });
        let result = workflow::content::generate_tweet(
            &self.state,
            &self.state.config.business,
            &topic,
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Generate a multi-tweet educational thread (5-8 tweets). Requires LLM provider.
    #[tool]
    async fn generate_thread(
        &self,
        Parameters(req): Parameters<TopicRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let topic = req.topic.unwrap_or_else(|| {
            self.state
                .config
                .business
                .effective_industry_topics()
                .first()
                .cloned()
                .unwrap_or_else(|| "general industry trends".to_string())
        });
        let result = workflow::content::generate_thread(
            &self.state,
            &self.state.config.business,
            &topic,
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get current capabilities, tier info, scope analysis, endpoint group availability, rate-limit remaining, and actionable guidance.
    #[tool]
    async fn get_capabilities(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let llm_available = self.state.llm_provider.is_some();
        let x_available = self.state.x_client.is_some();
        let result = workflow::capabilities::get_capabilities(
            &self.state.pool,
            &self.state.config,
            llm_available,
            x_available,
            self.state.authenticated_user_id.as_deref(),
            &self.state.granted_scopes,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get the current operating mode (autopilot or composer) and effective approval mode.
    #[tool]
    async fn get_mode(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = std::time::Instant::now();
        let mode = self.state.config.mode.to_string();
        let approval = self.state.config.effective_approval_mode();
        let elapsed = start.elapsed().as_millis() as u64;
        let meta = ToolMeta::new(elapsed).with_workflow(&mode, approval);
        let result = ToolResponse::success(serde_json::json!({
            "mode": mode,
            "approval_mode": approval,
        }))
        .with_meta(meta)
        .to_json();
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Create a new draft or scheduled tweet/thread. Primary content queue tool in composer mode.
    #[tool]
    async fn compose_tweet(
        &self,
        Parameters(req): Parameters<ComposeTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = std::time::Instant::now();
        let params = serde_json::json!({
            "content": req.content,
            "content_type": req.content_type,
            "scheduled_for": req.scheduled_for,
        })
        .to_string();
        match workflow::policy_gate::check_policy(&self.state, "compose_tweet", &params, start)
            .await
        {
            workflow::policy_gate::GateResult::EarlyReturn(r) => {
                return Ok(CallToolResult::success(vec![Content::text(r)]));
            }
            workflow::policy_gate::GateResult::Proceed => {}
        }
        let content_type = req.content_type.as_deref().unwrap_or("tweet");
        let config = &self.state.config;
        let result = if let Some(scheduled_for) = &req.scheduled_for {
            match tuitbot_core::storage::scheduled_content::insert(
                &self.state.pool,
                content_type,
                &req.content,
                Some(scheduled_for),
            )
            .await
            {
                Ok(id) => {
                    let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                        &self.state.pool,
                        "compose_tweet",
                        &self.state.config.mcp_policy.rate_limits,
                    )
                    .await;
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::success(serde_json::json!({
                        "scheduled_item_id": id,
                        "content_type": content_type,
                        "scheduled_for": scheduled_for,
                    }))
                    .with_meta(meta)
                    .to_json()
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::db_error(format!("Error scheduling content: {e}"))
                        .with_meta(meta)
                        .to_json()
                }
            }
        } else {
            match tuitbot_core::storage::scheduled_content::insert_draft(
                &self.state.pool,
                content_type,
                &req.content,
                "mcp",
            )
            .await
            {
                Ok(id) => {
                    let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                        &self.state.pool,
                        "compose_tweet",
                        &self.state.config.mcp_policy.rate_limits,
                    )
                    .await;
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::success(serde_json::json!({
                        "draft_id": id,
                        "content_type": content_type,
                    }))
                    .with_meta(meta)
                    .to_json()
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::db_error(format!("Error creating draft: {e}"))
                        .with_meta(meta)
                        .to_json()
                }
            }
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

#[cfg(test)]
mod tests;
