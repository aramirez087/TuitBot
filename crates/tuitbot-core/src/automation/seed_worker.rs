//! Low-priority background worker that pre-computes draft seeds from content nodes.
//!
//! Processes pending content nodes by extracting tweetable hooks via LLM,
//! then stores them as draft seeds with a cold-start engagement weight.

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::context::winning_dna::COLD_START_WEIGHT;
use crate::llm::{GenerationParams, LlmProvider};
use crate::storage::watchtower::{self, ContentNode};
use crate::storage::DbPool;
use crate::workflow::WorkflowError;

use super::scheduler::LoopScheduler;

/// Default number of content nodes to process per worker tick.
pub const SEED_BATCH_SIZE: u32 = 5;

/// Default interval between worker ticks (5 minutes).
pub const SEED_WORKER_INTERVAL_SECS: u64 = 300;

/// Background worker that extracts draft seeds from ingested content nodes.
pub struct SeedWorker {
    db: DbPool,
    llm: Arc<dyn LlmProvider>,
    batch_size: u32,
}

impl SeedWorker {
    /// Create a new seed worker.
    pub fn new(db: DbPool, llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            db,
            llm,
            batch_size: SEED_BATCH_SIZE,
        }
    }

    /// Run the seed worker loop until cancellation.
    ///
    /// On each tick:
    /// 1. Query up to `batch_size` content nodes with status='pending'
    /// 2. For each node, generate 1-3 draft seeds via LLM
    /// 3. Store seeds in `draft_seeds` with default weight
    /// 4. Mark node as 'processed'
    ///
    /// Uses a long interval (5 minutes) and yields between batches
    /// to stay low-priority.
    pub async fn run(&self, cancel: CancellationToken, scheduler: LoopScheduler) {
        tracing::info!("Seed worker started");

        loop {
            tokio::select! {
                () = cancel.cancelled() => {
                    tracing::debug!("Seed worker cancelled");
                    return;
                }
                () = scheduler.tick() => {}
            }

            if cancel.is_cancelled() {
                return;
            }

            match self.process_batch().await {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!(seeds = count, "Seed worker generated seeds");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Seed worker batch failed");
                }
            }
        }
    }

    /// Process a batch of pending content nodes.
    async fn process_batch(&self) -> Result<u32, WorkflowError> {
        let nodes = watchtower::get_pending_content_nodes(&self.db, self.batch_size).await?;

        if nodes.is_empty() {
            return Ok(0);
        }

        let mut total_seeds = 0u32;

        for node in &nodes {
            match self.process_node(node).await {
                Ok(count) => {
                    total_seeds += count;
                    watchtower::mark_node_processed(&self.db, node.id).await?;
                }
                Err(e) => {
                    tracing::warn!(
                        node_id = node.id,
                        path = %node.relative_path,
                        error = %e,
                        "Failed to process node, will retry next tick"
                    );
                    // Leave as 'pending' for retry
                }
            }
            // Yield between nodes to avoid starving other tasks
            tokio::task::yield_now().await;
        }

        Ok(total_seeds)
    }

    /// Public test accessor for `process_node`.
    #[cfg(test)]
    pub async fn process_node_for_test(&self, node: &ContentNode) -> Result<u32, WorkflowError> {
        self.process_node(node).await
    }

    /// Process a single content node, extracting hooks via LLM.
    ///
    /// Returns the number of seeds generated.
    async fn process_node(&self, node: &ContentNode) -> Result<u32, WorkflowError> {
        let body = if node.body_text.len() > 2000 {
            &node.body_text[..2000]
        } else {
            &node.body_text
        };

        let title_hint = node
            .title
            .as_deref()
            .map(|t| format!("Title: {t}\n"))
            .unwrap_or_default();

        let system = "You are an expert at extracting tweetable hooks from written content. \
            Given a note/article, identify 1-3 distinct angles that could each become a \
            standalone tweet. For each, output a one-line hook (max 200 chars) and suggest \
            a tweet format (list, tip, question, contrarian_take, storytelling, before_after).\n\n\
            Format your response as:\n\
            HOOK: <hook text>\n\
            FORMAT: <format name>\n\
            ---";

        let user_message = format!("{title_hint}Content:\n{body}");

        let params = GenerationParams {
            max_tokens: 400,
            temperature: 0.7,
            ..Default::default()
        };

        let resp = self.llm.complete(system, &user_message, &params).await?;
        let seeds = parse_seed_response(&resp.text);

        let mut count = 0u32;
        for (hook, format_name) in &seeds {
            if hook.len() > 200 || hook.is_empty() {
                continue;
            }
            let archetype = if format_name.is_empty() {
                None
            } else {
                Some(format_name.as_str())
            };
            watchtower::insert_draft_seed_with_weight(
                &self.db,
                node.id,
                hook,
                archetype,
                COLD_START_WEIGHT,
            )
            .await?;
            count += 1;
        }

        Ok(count)
    }
}

/// Parse the LLM response into (hook, format) pairs.
///
/// Expects blocks separated by `---`, each containing `HOOK:` and `FORMAT:` lines.
fn parse_seed_response(text: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut current_hook = String::new();
    let mut current_format = String::new();

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed == "---" {
            if !current_hook.is_empty() {
                results.push((current_hook.clone(), current_format.clone()));
                current_hook.clear();
                current_format.clear();
            }
            continue;
        }

        if let Some(hook) = trimmed.strip_prefix("HOOK:") {
            current_hook = hook.trim().to_string();
        } else if let Some(fmt) = trimmed.strip_prefix("FORMAT:") {
            current_format = fmt.trim().to_string();
        }
    }

    // Capture the last block
    if !current_hook.is_empty() {
        results.push((current_hook, current_format));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seed_response_single() {
        let text = "HOOK: Rust's ownership model prevents data races at compile time\nFORMAT: tip";
        let seeds = parse_seed_response(text);
        assert_eq!(seeds.len(), 1);
        assert_eq!(
            seeds[0].0,
            "Rust's ownership model prevents data races at compile time"
        );
        assert_eq!(seeds[0].1, "tip");
    }

    #[test]
    fn parse_seed_response_multiple() {
        let text = "\
HOOK: Most people think async is hard. It's not—the tooling is.
FORMAT: contrarian_take
---
HOOK: 3 things I learned about error handling in Rust
FORMAT: list
---
HOOK: What's your favorite Rust crate for web development?
FORMAT: question";

        let seeds = parse_seed_response(text);
        assert_eq!(seeds.len(), 3);
        assert!(seeds[0].0.contains("async"));
        assert_eq!(seeds[0].1, "contrarian_take");
        assert!(seeds[1].0.contains("error handling"));
        assert_eq!(seeds[1].1, "list");
        assert!(seeds[2].0.contains("favorite Rust crate"));
        assert_eq!(seeds[2].1, "question");
    }

    #[test]
    fn parse_seed_response_empty() {
        let seeds = parse_seed_response("");
        assert!(seeds.is_empty());
    }

    #[test]
    fn parse_seed_response_no_format() {
        let text = "HOOK: A standalone hook without format";
        let seeds = parse_seed_response(text);
        assert_eq!(seeds.len(), 1);
        assert_eq!(seeds[0].0, "A standalone hook without format");
        assert_eq!(seeds[0].1, "");
    }

    #[test]
    fn parse_seed_response_trailing_separator() {
        let text = "HOOK: First hook\nFORMAT: tip\n---\nHOOK: Second hook\nFORMAT: list\n---";
        let seeds = parse_seed_response(text);
        assert_eq!(seeds.len(), 2);
    }

    #[tokio::test]
    async fn seed_worker_process_node_with_mock_llm() {
        use crate::error::LlmError;
        use crate::llm::LlmResponse;
        use crate::storage::init_test_db;

        struct MockLlm;

        #[async_trait::async_trait]
        impl LlmProvider for MockLlm {
            fn name(&self) -> &str {
                "mock"
            }

            async fn complete(
                &self,
                _system: &str,
                _user_message: &str,
                _params: &GenerationParams,
            ) -> Result<LlmResponse, LlmError> {
                Ok(LlmResponse {
                    text: "HOOK: Testing is the most underrated skill\nFORMAT: tip\n---\nHOOK: Why I switched to Rust\nFORMAT: storytelling".to_string(),
                    usage: crate::llm::TokenUsage::default(),
                    model: "mock".to_string(),
                })
            }

            async fn health_check(&self) -> Result<(), LlmError> {
                Ok(())
            }
        }

        let pool = init_test_db().await.expect("init db");
        let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
            .await
            .expect("insert source");
        watchtower::upsert_content_node(
            &pool,
            source_id,
            "test.md",
            "hash1",
            Some("Test Note"),
            "Here is some content about testing in Rust.",
            None,
            None,
        )
        .await
        .expect("upsert node");

        let node = watchtower::get_pending_content_nodes(&pool, 1)
            .await
            .expect("get nodes");
        assert_eq!(node.len(), 1);

        let worker = SeedWorker::new(pool.clone(), Arc::new(MockLlm));
        let count = worker.process_node(&node[0]).await.expect("process node");
        assert_eq!(count, 2);

        // Verify seeds were stored
        let seeds = watchtower::get_seeds_for_context(&pool, 10)
            .await
            .expect("get seeds");
        assert_eq!(seeds.len(), 2);
        assert!(seeds[0].seed_text.contains("Testing"));
    }
}
