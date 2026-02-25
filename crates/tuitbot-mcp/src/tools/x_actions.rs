//! Direct X API tool implementations.
//!
//! Provides 11 MCP tools that wrap `XApiClient` methods, giving agents
//! direct access to read and mutate X API resources.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::error::XApiError;

use crate::state::SharedState;

use super::response::{ToolMeta, ToolResponse};

/// Map an `XApiError` to a structured `(code, message, retryable)` triple.
fn x_error_to_response(e: &XApiError) -> (&'static str, String, bool) {
    match e {
        XApiError::RateLimited { retry_after } => (
            "x_rate_limited",
            format!(
                "X API rate limited{}",
                match retry_after {
                    Some(s) => format!(", retry after {s}s"),
                    None => String::new(),
                }
            ),
            true,
        ),
        XApiError::AuthExpired => (
            "x_auth_expired",
            "X API authentication expired. Run `tuitbot auth` to re-authenticate.".to_string(),
            false,
        ),
        XApiError::Forbidden { message } => {
            ("x_forbidden", format!("X API forbidden: {message}"), false)
        }
        XApiError::AccountRestricted { message } => (
            "x_account_restricted",
            format!("X API account restricted: {message}"),
            false,
        ),
        XApiError::Network { source } => (
            "x_network_error",
            format!("X API network error: {source}"),
            true,
        ),
        other => ("x_api_error", other.to_string(), false),
    }
}

/// Return an error response when the X client is not configured.
fn not_configured_response(start: Instant) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(
        "x_not_configured",
        "X API client not available. Run `tuitbot auth` to authenticate.",
        false,
    )
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}

/// Helper: build an error response from an XApiError.
fn error_response(e: &XApiError, start: Instant) -> String {
    let (code, message, retryable) = x_error_to_response(e);
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(code, message, retryable)
        .with_meta(ToolMeta::new(elapsed))
        .to_json()
}

// ── Read Tools ──────────────────────────────────────────────────────────

/// Get a single tweet by ID.
pub async fn get_tweet_by_id(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.get_tweet(tweet_id).await {
        Ok(tweet) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Look up a user by username.
pub async fn get_user_by_username(state: &SharedState, username: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.get_user_by_username(username).await {
        Ok(user) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&user)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Search recent tweets.
pub async fn search_tweets(
    state: &SharedState,
    query: &str,
    max_results: u32,
    since_id: Option<&str>,
) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.search_tweets(query, max_results, since_id).await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Get mentions for the authenticated user.
pub async fn get_user_mentions(state: &SharedState, since_id: Option<&str>) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "x_not_configured",
                "Authenticated user ID not available. X client may not be fully initialized.",
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    match client.get_mentions(user_id, since_id).await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Get recent tweets from a specific user.
pub async fn get_user_tweets(state: &SharedState, user_id: &str, max_results: u32) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.get_user_tweets(user_id, max_results).await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&resp)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

// ── Mutation Tools ──────────────────────────────────────────────────────

/// Post a new tweet.
pub async fn post_tweet(state: &SharedState, text: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"text": text}).to_string();
    match super::policy_gate::check_policy(state, "post_tweet", &params, start).await {
        super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.post_tweet(text).await {
        Ok(tweet) => {
            let _ =
                tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(&state.pool).await;
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Reply to an existing tweet.
pub async fn reply_to_tweet(state: &SharedState, text: &str, in_reply_to_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"text": text, "in_reply_to_id": in_reply_to_id}).to_string();
    match super::policy_gate::check_policy(state, "reply_to_tweet", &params, start).await {
        super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.reply_to_tweet(text, in_reply_to_id).await {
        Ok(tweet) => {
            let _ =
                tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(&state.pool).await;
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Post a quote tweet.
pub async fn quote_tweet(state: &SharedState, text: &str, quoted_tweet_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"text": text, "quoted_tweet_id": quoted_tweet_id}).to_string();
    match super::policy_gate::check_policy(state, "quote_tweet", &params, start).await {
        super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    match client.quote_tweet(text, quoted_tweet_id).await {
        Ok(tweet) => {
            let _ =
                tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(&state.pool).await;
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(&tweet)
                .with_meta(ToolMeta::new(elapsed))
                .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Like a tweet.
pub async fn like_tweet(state: &SharedState, tweet_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"tweet_id": tweet_id}).to_string();
    match super::policy_gate::check_policy(state, "like_tweet", &params, start).await {
        super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "x_not_configured",
                "Authenticated user ID not available.",
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    match client.like_tweet(user_id, tweet_id).await {
        Ok(liked) => {
            let _ =
                tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(&state.pool).await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct LikeResult {
                liked: bool,
                tweet_id: String,
            }
            ToolResponse::success(LikeResult {
                liked,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Follow a user.
pub async fn follow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"target_user_id": target_user_id}).to_string();
    match super::policy_gate::check_policy(state, "follow_user", &params, start).await {
        super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "x_not_configured",
                "Authenticated user ID not available.",
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    match client.follow_user(user_id, target_user_id).await {
        Ok(following) => {
            let _ =
                tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(&state.pool).await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct FollowResult {
                following: bool,
                target_user_id: String,
            }
            ToolResponse::success(FollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

/// Unfollow a user.
pub async fn unfollow_user(state: &SharedState, target_user_id: &str) -> String {
    let start = Instant::now();
    let params = serde_json::json!({"target_user_id": target_user_id}).to_string();
    match super::policy_gate::check_policy(state, "unfollow_user", &params, start).await {
        super::policy_gate::GateResult::EarlyReturn(r) => return r,
        super::policy_gate::GateResult::Proceed => {}
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };
    let user_id = match state.authenticated_user_id.as_deref() {
        Some(id) => id,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "x_not_configured",
                "Authenticated user ID not available.",
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    match client.unfollow_user(user_id, target_user_id).await {
        Ok(following) => {
            let _ =
                tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(&state.pool).await;
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UnfollowResult {
                following: bool,
                target_user_id: String,
            }
            ToolResponse::success(UnfollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => error_response(&e, start),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tuitbot_core::config::Config;
    use tuitbot_core::error::XApiError;
    use tuitbot_core::storage;
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;

    use crate::state::AppState;

    /// Mock X API client for testing.
    struct MockXApiClient;

    #[async_trait::async_trait]
    impl XApiClient for MockXApiClient {
        async fn search_tweets(
            &self,
            _query: &str,
            _max_results: u32,
            _since_id: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Ok(SearchResponse {
                data: vec![Tweet {
                    id: "t1".to_string(),
                    text: "Hello".to_string(),
                    author_id: "a1".to_string(),
                    created_at: String::new(),
                    public_metrics: PublicMetrics::default(),
                    conversation_id: None,
                }],
                includes: None,
                meta: SearchMeta {
                    newest_id: Some("t1".to_string()),
                    oldest_id: Some("t1".to_string()),
                    result_count: 1,
                    next_token: None,
                },
            })
        }

        async fn get_mentions(
            &self,
            _user_id: &str,
            _since_id: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            Ok(SearchResponse {
                data: vec![],
                includes: None,
                meta: SearchMeta {
                    newest_id: None,
                    oldest_id: None,
                    result_count: 0,
                    next_token: None,
                },
            })
        }

        async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "new_1".to_string(),
                text: text.to_string(),
            })
        }

        async fn reply_to_tweet(
            &self,
            text: &str,
            _in_reply_to_id: &str,
        ) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "reply_1".to_string(),
                text: text.to_string(),
            })
        }

        async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, XApiError> {
            Ok(Tweet {
                id: tweet_id.to_string(),
                text: "Test tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: "2026-02-24T00:00:00Z".to_string(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            })
        }

        async fn get_me(&self) -> Result<User, XApiError> {
            Ok(User {
                id: "u1".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn get_user_tweets(
            &self,
            _user_id: &str,
            _max_results: u32,
        ) -> Result<SearchResponse, XApiError> {
            Ok(SearchResponse {
                data: vec![],
                includes: None,
                meta: SearchMeta {
                    newest_id: None,
                    oldest_id: None,
                    result_count: 0,
                    next_token: None,
                },
            })
        }

        async fn get_user_by_username(&self, username: &str) -> Result<User, XApiError> {
            Ok(User {
                id: "u2".to_string(),
                username: username.to_string(),
                name: "Looked Up User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn quote_tweet(
            &self,
            text: &str,
            _quoted_tweet_id: &str,
        ) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "qt_1".to_string(),
                text: text.to_string(),
            })
        }

        async fn like_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn follow_user(
            &self,
            _user_id: &str,
            _target_user_id: &str,
        ) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unfollow_user(
            &self,
            _user_id: &str,
            _target_user_id: &str,
        ) -> Result<bool, XApiError> {
            Ok(false)
        }
    }

    /// Mock that returns errors.
    struct ErrorXApiClient;

    #[async_trait::async_trait]
    impl XApiClient for ErrorXApiClient {
        async fn search_tweets(
            &self,
            _query: &str,
            _max_results: u32,
            _since_id: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Err(XApiError::RateLimited {
                retry_after: Some(30),
            })
        }

        async fn get_mentions(
            &self,
            _user_id: &str,
            _since_id: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }

        async fn post_tweet(&self, _text: &str) -> Result<PostedTweet, XApiError> {
            Err(XApiError::Forbidden {
                message: "not allowed".to_string(),
            })
        }

        async fn reply_to_tweet(
            &self,
            _text: &str,
            _in_reply_to_id: &str,
        ) -> Result<PostedTweet, XApiError> {
            Err(XApiError::AccountRestricted {
                message: "suspended".to_string(),
            })
        }

        async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, XApiError> {
            Err(XApiError::ApiError {
                status: 404,
                message: "not found".to_string(),
            })
        }

        async fn get_me(&self) -> Result<User, XApiError> {
            Err(XApiError::AuthExpired)
        }

        async fn get_user_tweets(
            &self,
            _user_id: &str,
            _max_results: u32,
        ) -> Result<SearchResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }

        async fn get_user_by_username(&self, _username: &str) -> Result<User, XApiError> {
            Err(XApiError::AuthExpired)
        }
    }

    /// Create test state with policy enforcement disabled (for existing X API tests).
    async fn make_state(
        x_client: Option<Box<dyn XApiClient>>,
        user_id: Option<String>,
    ) -> SharedState {
        let mut config = Config::default();
        config.mcp_policy.enforce_for_mutations = false;
        let pool = storage::init_test_db().await.expect("init db");
        Arc::new(AppState {
            pool,
            config,
            llm_provider: None,
            x_client,
            authenticated_user_id: user_id,
        })
    }

    /// Create test state with specific config for policy tests.
    async fn make_state_with_config(
        x_client: Option<Box<dyn XApiClient>>,
        user_id: Option<String>,
        config: Config,
    ) -> SharedState {
        let pool = storage::init_test_db().await.expect("init db");
        tuitbot_core::storage::rate_limits::init_mcp_rate_limit(
            &pool,
            config.mcp_policy.max_mutations_per_hour,
        )
        .await
        .expect("init mcp rate limit");
        Arc::new(AppState {
            pool,
            config,
            llm_provider: None,
            x_client,
            authenticated_user_id: user_id,
        })
    }

    // ── Success path tests ──

    #[tokio::test]
    async fn get_tweet_by_id_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = get_tweet_by_id(&state, "t1").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["id"], "t1");
        assert!(parsed["meta"]["elapsed_ms"].is_number());
    }

    #[tokio::test]
    async fn get_user_by_username_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = get_user_by_username(&state, "testuser").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["username"], "testuser");
    }

    #[tokio::test]
    async fn search_tweets_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = search_tweets(&state, "rust", 10, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["meta"]["result_count"], 1);
    }

    #[tokio::test]
    async fn post_tweet_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = post_tweet(&state, "Hello!").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["id"], "new_1");
    }

    #[tokio::test]
    async fn like_tweet_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = like_tweet(&state, "t1").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["liked"], true);
    }

    #[tokio::test]
    async fn follow_user_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = follow_user(&state, "target1").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["following"], true);
    }

    #[tokio::test]
    async fn unfollow_user_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = unfollow_user(&state, "target1").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["following"], false);
    }

    #[tokio::test]
    async fn quote_tweet_success() {
        let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
        let result = quote_tweet(&state, "Great!", "qt_id").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["id"], "qt_1");
    }

    // ── Error mapping tests ──

    #[tokio::test]
    async fn error_maps_rate_limited() {
        let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
        let result = search_tweets(&state, "test", 10, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_rate_limited");
        assert_eq!(parsed["error"]["retryable"], true);
    }

    #[tokio::test]
    async fn error_maps_auth_expired() {
        let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
        let result = get_user_mentions(&state, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_auth_expired");
        assert_eq!(parsed["error"]["retryable"], false);
    }

    #[tokio::test]
    async fn error_maps_forbidden() {
        let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
        let result = post_tweet(&state, "test").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_forbidden");
    }

    #[tokio::test]
    async fn error_maps_api_error() {
        let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
        let result = get_tweet_by_id(&state, "nonexistent").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_api_error");
    }

    // ── Not configured test ──

    #[tokio::test]
    async fn x_not_configured_when_no_client() {
        let state = make_state(None, None).await;
        let result = get_tweet_by_id(&state, "t1").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_not_configured");
        assert_eq!(parsed["error"]["retryable"], false);
    }

    #[tokio::test]
    async fn like_tweet_no_user_id() {
        let state = make_state(Some(Box::new(MockXApiClient)), None).await;
        let result = like_tweet(&state, "t1").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_not_configured");
    }

    // ── Policy gate tests ──

    use tuitbot_core::config::McpPolicyConfig;

    fn blocked_config() -> Config {
        let mut config = Config::default();
        config.mcp_policy = McpPolicyConfig {
            enforce_for_mutations: true,
            blocked_tools: vec!["post_tweet".to_string()],
            require_approval_for: Vec::new(),
            dry_run_mutations: false,
            max_mutations_per_hour: 20,
        };
        config
    }

    fn approval_config() -> Config {
        let mut config = Config::default();
        config.mcp_policy = McpPolicyConfig {
            enforce_for_mutations: true,
            require_approval_for: vec!["post_tweet".to_string()],
            blocked_tools: Vec::new(),
            dry_run_mutations: false,
            max_mutations_per_hour: 20,
        };
        config
    }

    fn dry_run_config() -> Config {
        let mut config = Config::default();
        config.mcp_policy = McpPolicyConfig {
            enforce_for_mutations: true,
            require_approval_for: Vec::new(),
            blocked_tools: Vec::new(),
            dry_run_mutations: true,
            max_mutations_per_hour: 20,
        };
        config
    }

    fn allowed_config() -> Config {
        let mut config = Config::default();
        config.mcp_policy = McpPolicyConfig {
            enforce_for_mutations: true,
            require_approval_for: Vec::new(),
            blocked_tools: Vec::new(),
            dry_run_mutations: false,
            max_mutations_per_hour: 20,
        };
        config
    }

    fn composer_config() -> Config {
        let mut config = Config::default();
        config.mode = tuitbot_core::config::OperatingMode::Composer;
        config.mcp_policy = McpPolicyConfig {
            enforce_for_mutations: true,
            require_approval_for: Vec::new(),
            blocked_tools: Vec::new(),
            dry_run_mutations: false,
            max_mutations_per_hour: 20,
        };
        config
    }

    #[tokio::test]
    async fn post_tweet_blocked_by_policy() {
        let state = make_state_with_config(
            Some(Box::new(MockXApiClient)),
            Some("u1".into()),
            blocked_config(),
        )
        .await;
        let result = post_tweet(&state, "Hello!").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "policy_denied_blocked");
    }

    #[tokio::test]
    async fn post_tweet_routed_to_approval() {
        let state = make_state_with_config(
            Some(Box::new(MockXApiClient)),
            Some("u1".into()),
            approval_config(),
        )
        .await;
        let result = post_tweet(&state, "Hello!").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["routed_to_approval"], true);
        assert!(parsed["data"]["approval_queue_id"].is_number());
    }

    #[tokio::test]
    async fn post_tweet_allowed_when_not_gated() {
        let state = make_state_with_config(
            Some(Box::new(MockXApiClient)),
            Some("u1".into()),
            allowed_config(),
        )
        .await;
        let result = post_tweet(&state, "Hello!").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["id"], "new_1");
    }

    #[tokio::test]
    async fn dry_run_returns_would_execute() {
        let state = make_state_with_config(
            Some(Box::new(MockXApiClient)),
            Some("u1".into()),
            dry_run_config(),
        )
        .await;
        let result = post_tweet(&state, "Hello!").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["dry_run"], true);
        assert_eq!(parsed["data"]["would_execute"], "post_tweet");
    }

    #[tokio::test]
    async fn composer_mode_forces_approval_for_all() {
        let state = make_state_with_config(
            Some(Box::new(MockXApiClient)),
            Some("u1".into()),
            composer_config(),
        )
        .await;
        // unfollow_user is NOT in require_approval_for, but Composer mode forces approval
        let result = unfollow_user(&state, "target1").await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["routed_to_approval"], true);
    }
}
