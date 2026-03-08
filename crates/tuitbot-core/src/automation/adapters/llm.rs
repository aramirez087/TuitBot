//! LLM adapter implementations.

use std::sync::Arc;

use super::super::loop_helpers::{
    ContentLoopError, LoopError, ReplyGenerator, ReplyOutput, TweetGenerator,
};
use super::super::thread_loop::ThreadGenerator;
use super::helpers::{llm_to_content_error, llm_to_loop_error};
use crate::content::ContentGenerator;
use crate::storage::DbPool;

/// Record LLM usage to the database (fire-and-forget).
pub(super) async fn record_llm_usage(
    pool: &DbPool,
    generation_type: &str,
    provider: &str,
    model: &str,
    input_tokens: u32,
    output_tokens: u32,
) {
    let pricing = crate::llm::pricing::lookup(provider, model);
    let cost = pricing.compute_cost(input_tokens, output_tokens);
    if let Err(e) = crate::storage::llm_usage::insert_llm_usage(
        pool,
        generation_type,
        provider,
        model,
        input_tokens,
        output_tokens,
        cost,
    )
    .await
    {
        tracing::warn!(error = %e, "Failed to record LLM usage");
    }
}

/// Adapts `ContentGenerator` to the `ReplyGenerator` port trait.
pub struct LlmReplyAdapter {
    generator: Arc<ContentGenerator>,
    pool: DbPool,
}

impl LlmReplyAdapter {
    pub fn new(generator: Arc<ContentGenerator>, pool: DbPool) -> Self {
        Self { generator, pool }
    }
}

#[async_trait::async_trait]
impl ReplyGenerator for LlmReplyAdapter {
    async fn generate_reply(
        &self,
        tweet_text: &str,
        author: &str,
        mention_product: bool,
    ) -> Result<String, LoopError> {
        let output = self
            .generator
            .generate_reply(tweet_text, author, mention_product)
            .await
            .map_err(llm_to_loop_error)?;
        record_llm_usage(
            &self.pool,
            "reply",
            &output.provider,
            &output.model,
            output.usage.input_tokens,
            output.usage.output_tokens,
        )
        .await;
        Ok(output.text)
    }
}

/// Vault-aware reply adapter that injects pre-built RAG context into replies.
///
/// The RAG prompt is built once at construction time (by the server/CLI wiring
/// layer) and reused for every reply, avoiding per-tweet DB queries.
pub struct VaultAwareLlmReplyAdapter {
    generator: Arc<ContentGenerator>,
    pool: DbPool,
    /// Pre-built RAG prompt block to inject into every reply.
    rag_prompt: Option<String>,
    /// Pre-built vault citations corresponding to the RAG prompt.
    vault_citations: Vec<crate::context::retrieval::VaultCitation>,
}

impl VaultAwareLlmReplyAdapter {
    pub fn new(
        generator: Arc<ContentGenerator>,
        pool: DbPool,
        rag_prompt: Option<String>,
        vault_citations: Vec<crate::context::retrieval::VaultCitation>,
    ) -> Self {
        Self {
            generator,
            pool,
            rag_prompt,
            vault_citations,
        }
    }
}

#[async_trait::async_trait]
impl ReplyGenerator for VaultAwareLlmReplyAdapter {
    async fn generate_reply(
        &self,
        tweet_text: &str,
        author: &str,
        mention_product: bool,
    ) -> Result<String, LoopError> {
        let output = self
            .generator
            .generate_reply_with_context(
                tweet_text,
                author,
                mention_product,
                None,
                self.rag_prompt.as_deref(),
            )
            .await
            .map_err(llm_to_loop_error)?;
        record_llm_usage(
            &self.pool,
            "reply",
            &output.provider,
            &output.model,
            output.usage.input_tokens,
            output.usage.output_tokens,
        )
        .await;
        Ok(output.text)
    }

    async fn generate_reply_with_rag(
        &self,
        tweet_text: &str,
        author: &str,
        mention_product: bool,
    ) -> Result<ReplyOutput, LoopError> {
        let text = self
            .generate_reply(tweet_text, author, mention_product)
            .await?;
        Ok(ReplyOutput {
            text,
            vault_citations: self.vault_citations.clone(),
        })
    }
}

/// Adapts `ContentGenerator` to the `TweetGenerator` port trait.
pub struct LlmTweetAdapter {
    generator: Arc<ContentGenerator>,
    pool: DbPool,
}

impl LlmTweetAdapter {
    pub fn new(generator: Arc<ContentGenerator>, pool: DbPool) -> Self {
        Self { generator, pool }
    }
}

#[async_trait::async_trait]
impl TweetGenerator for LlmTweetAdapter {
    async fn generate_tweet(&self, topic: &str) -> Result<String, ContentLoopError> {
        let output = self
            .generator
            .generate_tweet(topic)
            .await
            .map_err(llm_to_content_error)?;
        record_llm_usage(
            &self.pool,
            "tweet",
            &output.provider,
            &output.model,
            output.usage.input_tokens,
            output.usage.output_tokens,
        )
        .await;
        Ok(output.text)
    }
}

/// Adapts `ContentGenerator` to the `ThreadGenerator` port trait.
pub struct LlmThreadAdapter {
    generator: Arc<ContentGenerator>,
    pool: DbPool,
}

impl LlmThreadAdapter {
    pub fn new(generator: Arc<ContentGenerator>, pool: DbPool) -> Self {
        Self { generator, pool }
    }
}

#[async_trait::async_trait]
impl ThreadGenerator for LlmThreadAdapter {
    async fn generate_thread(
        &self,
        topic: &str,
        _count: Option<usize>,
    ) -> Result<Vec<String>, ContentLoopError> {
        let output = self
            .generator
            .generate_thread(topic)
            .await
            .map_err(llm_to_content_error)?;
        record_llm_usage(
            &self.pool,
            "thread",
            &output.provider,
            &output.model,
            output.usage.input_tokens,
            output.usage.output_tokens,
        )
        .await;
        Ok(output.tweets)
    }
}
