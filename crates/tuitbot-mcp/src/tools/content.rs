//! Content generation tools: generate reply, tweet, thread.

use std::sync::Arc;

use tuitbot_core::config::BusinessProfile;
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::llm::{GenerationParams, LlmProvider, LlmResponse};
use tuitbot_core::LlmError;

use crate::state::AppState;

/// A thin LlmProvider that delegates to the provider inside shared AppState.
///
/// This lets us pass a `Box<dyn LlmProvider>` to `ContentGenerator` while
/// the actual provider lives inside `Arc<AppState>`.
pub(crate) struct ArcProvider {
    pub(crate) state: Arc<AppState>,
}

#[async_trait::async_trait]
impl LlmProvider for ArcProvider {
    fn name(&self) -> &str {
        self.state
            .llm_provider
            .as_ref()
            .map(|p| p.name())
            .unwrap_or("none")
    }

    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        params: &GenerationParams,
    ) -> Result<LlmResponse, LlmError> {
        match &self.state.llm_provider {
            Some(provider) => provider.complete(system, user_message, params).await,
            None => Err(LlmError::NotConfigured),
        }
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        match &self.state.llm_provider {
            Some(provider) => provider.health_check().await,
            None => Err(LlmError::NotConfigured),
        }
    }
}

/// Generate a reply to a tweet via LLM.
pub async fn generate_reply(
    state: &Arc<AppState>,
    business: &BusinessProfile,
    tweet_text: &str,
    tweet_author: &str,
    mention_product: bool,
) -> String {
    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, business.clone());

    match gen
        .generate_reply(tweet_text, tweet_author, mention_product)
        .await
    {
        Ok(output) => serde_json::json!({
            "reply": output.text,
            "char_count": output.text.len(),
        })
        .to_string(),
        Err(e) => format!("Error generating reply: {e}"),
    }
}

/// Generate an original tweet via LLM.
pub async fn generate_tweet(
    state: &Arc<AppState>,
    business: &BusinessProfile,
    topic: &str,
) -> String {
    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, business.clone());

    match gen.generate_tweet(topic).await {
        Ok(output) => serde_json::json!({
            "tweet": output.text,
            "char_count": output.text.len(),
        })
        .to_string(),
        Err(e) => format!("Error generating tweet: {e}"),
    }
}

/// Generate a multi-tweet thread via LLM.
pub async fn generate_thread(
    state: &Arc<AppState>,
    business: &BusinessProfile,
    topic: &str,
) -> String {
    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, business.clone());

    match gen.generate_thread(topic).await {
        Ok(output) => serde_json::json!({
            "thread": output.tweets,
            "tweet_count": output.tweets.len(),
        })
        .to_string(),
        Err(e) => format!("Error generating thread: {e}"),
    }
}
