//! Content generation tools: generate reply, tweet, thread.

use std::sync::Arc;
use std::time::Instant;

use tuitbot_core::config::{BusinessProfile, Config};
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::llm::{GenerationParams, LlmProvider, LlmResponse};
use tuitbot_core::LlmError;

use crate::state::AppState;

use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

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
    config: &Config,
) -> String {
    let start = Instant::now();
    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, business.clone());

    match gen
        .generate_reply(tweet_text, tweet_author, mention_product)
        .await
    {
        Ok(output) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({
                "reply": output.text,
                "char_count": output.text.len(),
            }))
            .with_meta(meta)
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::error(ErrorCode::LlmError, format!("Error generating reply: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Generate an original tweet via LLM.
pub async fn generate_tweet(
    state: &Arc<AppState>,
    business: &BusinessProfile,
    topic: &str,
    config: &Config,
) -> String {
    let start = Instant::now();
    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, business.clone());

    match gen.generate_tweet(topic).await {
        Ok(output) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({
                "tweet": output.text,
                "char_count": output.text.len(),
            }))
            .with_meta(meta)
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::error(ErrorCode::LlmError, format!("Error generating tweet: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Generate a multi-tweet thread via LLM.
pub async fn generate_thread(
    state: &Arc<AppState>,
    business: &BusinessProfile,
    topic: &str,
    config: &Config,
) -> String {
    let start = Instant::now();
    let provider = Box::new(ArcProvider {
        state: Arc::clone(state),
    });
    let gen = ContentGenerator::new(provider, business.clone());

    match gen.generate_thread(topic).await {
        Ok(output) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(serde_json::json!({
                "thread": output.tweets,
                "tweet_count": output.tweets.len(),
            }))
            .with_meta(meta)
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::error(ErrorCode::LlmError, format!("Error generating thread: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::idempotency::IdempotencyStore;
    use tuitbot_core::config::Config;
    use tuitbot_core::error::XApiError;
    use tuitbot_core::storage;
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;

    struct NullX;

    #[async_trait::async_trait]
    impl XApiClient for NullX {
        async fn search_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_mentions(
            &self,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn post_tweet(&self, _: &str) -> Result<PostedTweet, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_tweet(&self, _: &str) -> Result<Tweet, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_me(&self) -> Result<User, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_user_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_user_by_username(&self, _: &str) -> Result<User, XApiError> {
            Err(XApiError::AuthExpired)
        }
    }

    async fn make_state() -> Arc<crate::state::AppState> {
        let pool = storage::init_test_db().await.expect("init test db");
        storage::rate_limits::init_mcp_rate_limit(
            &pool,
            Config::default().mcp_policy.max_mutations_per_hour,
        )
        .await
        .expect("init rate limits");
        Arc::new(crate::state::AppState {
            pool,
            config: Config::default(),
            llm_provider: None,
            x_client: Some(Box::new(NullX)),
            authenticated_user_id: Some("u1".to_string()),
            granted_scopes: vec![],
            idempotency: Arc::new(IdempotencyStore::new()),
        })
    }

    #[test]
    fn arc_provider_name_without_llm() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let state = rt.block_on(make_state());
        let provider = ArcProvider {
            state: Arc::clone(&state),
        };
        assert_eq!(provider.name(), "none");
    }

    #[tokio::test]
    async fn arc_provider_complete_without_llm_returns_not_configured() {
        let state = make_state().await;
        let provider = ArcProvider {
            state: Arc::clone(&state),
        };
        let params = GenerationParams::default();
        let result = provider.complete("system", "user", &params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn arc_provider_health_check_without_llm_returns_not_configured() {
        let state = make_state().await;
        let provider = ArcProvider {
            state: Arc::clone(&state),
        };
        let result = provider.health_check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn generate_reply_without_llm_returns_error_json() {
        let state = make_state().await;
        let result = generate_reply(
            &state,
            &state.config.business,
            "This is a great tweet about Rust!",
            "rustacean",
            false,
            &state.config,
        )
        .await;
        assert!(!result.is_empty());
        assert!(result.contains("error") || result.contains("Error"));
    }

    #[tokio::test]
    async fn generate_tweet_without_llm_returns_error_json() {
        let state = make_state().await;
        let result = generate_tweet(
            &state,
            &state.config.business,
            "Rust programming",
            &state.config,
        )
        .await;
        assert!(!result.is_empty());
        assert!(result.contains("error") || result.contains("Error"));
    }

    #[tokio::test]
    async fn generate_thread_without_llm_returns_error_json() {
        let state = make_state().await;
        let result = generate_thread(
            &state,
            &state.config.business,
            "CLI tools in Rust",
            &state.config,
        )
        .await;
        assert!(!result.is_empty());
        assert!(result.contains("error") || result.contains("Error"));
    }

    #[test]
    fn tool_response_llm_not_configured() {
        let result = ToolResponse::llm_not_configured().to_json();
        assert!(!result.is_empty());
    }
}
