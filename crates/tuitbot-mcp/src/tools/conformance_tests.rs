//! Conformance tests for all kernel tool functions.
//!
//! Validates that every kernel read/write/engage tool produces a valid
//! ToolResponse envelope, and that error paths produce correct ErrorCode
//! values with accurate retryable flags and retry_after_ms fields.

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::contract::ProviderError;
    use crate::kernel::{engage, read, utils, write};
    use crate::provider::SocialReadProvider;
    use tuitbot_core::error::XApiError;
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;

    // ── Shared assertion helpers ─────────────────────────────────────

    fn assert_conformant_success(json: &str, tool: &str) {
        let parsed: Value =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("{tool}: invalid JSON: {e}"));
        assert!(
            parsed["success"].as_bool().unwrap_or(false),
            "{tool}: expected success=true"
        );
        assert!(parsed.get("data").is_some(), "{tool}: missing 'data' field");
        assert!(parsed.get("meta").is_some(), "{tool}: missing 'meta' field");
        assert_eq!(
            parsed["meta"]["tool_version"], "1.0",
            "{tool}: tool_version mismatch"
        );
        assert!(
            parsed["meta"]["elapsed_ms"].is_number(),
            "{tool}: elapsed_ms not a number"
        );
    }

    fn assert_conformant_error(json: &str, tool: &str, expected_code: &str) {
        let parsed: Value =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("{tool}: invalid JSON: {e}"));
        assert!(
            !parsed["success"].as_bool().unwrap_or(true),
            "{tool}: expected success=false"
        );
        assert!(
            parsed.get("error").is_some(),
            "{tool}: missing 'error' field"
        );
        assert_eq!(
            parsed["error"]["code"].as_str().unwrap_or(""),
            expected_code,
            "{tool}: error code mismatch"
        );
        // Verify retryable flag matches ErrorCode::is_retryable
        let retryable = parsed["error"]["retryable"].as_bool().unwrap_or(false);
        let code: crate::contract::ErrorCode =
            serde_json::from_value(parsed["error"]["code"].clone())
                .unwrap_or_else(|e| panic!("{tool}: unknown error code: {e}"));
        assert_eq!(
            retryable,
            code.is_retryable(),
            "{tool}: retryable flag mismatch for {expected_code}"
        );
    }

    // ── Mock provider (success) ──────────────────────────────────────

    struct MockProvider;

    #[async_trait::async_trait]
    impl SocialReadProvider for MockProvider {
        async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError> {
            Ok(Tweet {
                id: tweet_id.to_string(),
                text: "Mock tweet".to_string(),
                author_id: "author_1".to_string(),
                created_at: "2026-02-25T00:00:00Z".to_string(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            })
        }

        async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError> {
            Ok(User {
                id: "u1".to_string(),
                username: username.to_string(),
                name: "Mock User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn search_tweets(
            &self,
            _q: &str,
            _max: u32,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
            Ok(SearchResponse {
                data: vec![Tweet {
                    id: "s1".to_string(),
                    text: "Found".to_string(),
                    author_id: "a1".to_string(),
                    created_at: String::new(),
                    public_metrics: PublicMetrics::default(),
                    conversation_id: None,
                }],
                includes: None,
                meta: SearchMeta {
                    newest_id: Some("s1".to_string()),
                    oldest_id: Some("s1".to_string()),
                    result_count: 1,
                    next_token: None,
                },
            })
        }

        async fn get_user_mentions(
            &self,
            _uid: &str,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<MentionResponse, ProviderError> {
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

        async fn get_user_tweets(
            &self,
            uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
            Ok(SearchResponse {
                data: vec![Tweet {
                    id: "ut1".to_string(),
                    text: "User tweet".to_string(),
                    author_id: uid.to_string(),
                    created_at: String::new(),
                    public_metrics: PublicMetrics::default(),
                    conversation_id: None,
                }],
                includes: None,
                meta: SearchMeta {
                    newest_id: Some("ut1".to_string()),
                    oldest_id: Some("ut1".to_string()),
                    result_count: 1,
                    next_token: None,
                },
            })
        }

        async fn get_home_timeline(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
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

        async fn get_me(&self) -> Result<User, ProviderError> {
            Ok(User {
                id: "me_1".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn get_followers(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<UsersResponse, ProviderError> {
            Ok(UsersResponse {
                data: vec![User {
                    id: "f1".to_string(),
                    username: "follower1".to_string(),
                    name: "Follower".to_string(),
                    public_metrics: UserMetrics::default(),
                }],
                meta: UsersMeta {
                    result_count: 1,
                    next_token: None,
                },
            })
        }

        async fn get_following(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<UsersResponse, ProviderError> {
            Ok(UsersResponse {
                data: vec![],
                meta: UsersMeta {
                    result_count: 0,
                    next_token: None,
                },
            })
        }

        async fn get_user_by_id(&self, user_id: &str) -> Result<User, ProviderError> {
            Ok(User {
                id: user_id.to_string(),
                username: "iduser".to_string(),
                name: "ID User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn get_liked_tweets(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
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

        async fn get_bookmarks(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
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

        async fn get_users_by_ids(
            &self,
            user_ids: &[&str],
        ) -> Result<UsersResponse, ProviderError> {
            let users = user_ids
                .iter()
                .map(|id| User {
                    id: id.to_string(),
                    username: format!("user_{id}"),
                    name: format!("User {id}"),
                    public_metrics: UserMetrics::default(),
                })
                .collect::<Vec<_>>();
            let count = users.len() as u32;
            Ok(UsersResponse {
                data: users,
                meta: UsersMeta {
                    result_count: count,
                    next_token: None,
                },
            })
        }

        async fn get_tweet_liking_users(
            &self,
            _tid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<UsersResponse, ProviderError> {
            Ok(UsersResponse {
                data: vec![],
                meta: UsersMeta {
                    result_count: 0,
                    next_token: None,
                },
            })
        }
    }

    // ── Error provider ───────────────────────────────────────────────

    struct ErrorProvider;

    #[async_trait::async_trait]
    impl SocialReadProvider for ErrorProvider {
        async fn get_tweet(&self, _tid: &str) -> Result<Tweet, ProviderError> {
            Err(ProviderError::Other {
                message: "not found".to_string(),
            })
        }

        async fn get_user_by_username(&self, _u: &str) -> Result<User, ProviderError> {
            Err(ProviderError::AuthExpired)
        }

        async fn search_tweets(
            &self,
            _q: &str,
            _max: u32,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
            Err(ProviderError::RateLimited {
                retry_after: Some(60),
            })
        }

        async fn get_me(&self) -> Result<User, ProviderError> {
            Err(ProviderError::AuthExpired)
        }

        async fn get_followers(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<UsersResponse, ProviderError> {
            Err(ProviderError::Network {
                message: "timeout".to_string(),
            })
        }
    }

    // ── Mock XApiClient (success) ────────────────────────────────────

    struct MockXApiClient;

    #[async_trait::async_trait]
    impl XApiClient for MockXApiClient {
        async fn search_tweets(
            &self,
            _q: &str,
            _max: u32,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            unimplemented!()
        }

        async fn get_mentions(
            &self,
            _uid: &str,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            unimplemented!()
        }

        async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "posted_1".to_string(),
                text: text.to_string(),
            })
        }

        async fn reply_to_tweet(
            &self,
            text: &str,
            _reply_to: &str,
        ) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "reply_1".to_string(),
                text: text.to_string(),
            })
        }

        async fn get_tweet(&self, _id: &str) -> Result<Tweet, XApiError> {
            unimplemented!()
        }

        async fn get_me(&self) -> Result<User, XApiError> {
            unimplemented!()
        }

        async fn get_user_tweets(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            unimplemented!()
        }

        async fn get_user_by_username(&self, _u: &str) -> Result<User, XApiError> {
            unimplemented!()
        }

        async fn quote_tweet(&self, text: &str, _quoted: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "quote_1".to_string(),
                text: text.to_string(),
            })
        }

        async fn like_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn follow_user(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unfollow_user(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(false)
        }

        async fn retweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unretweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(false)
        }

        async fn delete_tweet(&self, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unlike_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(false)
        }

        async fn bookmark_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unbookmark_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(false)
        }
    }

    // ══════════════════════════════════════════════════════════════════
    // Read tool conformance
    // ══════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn conformance_read_get_tweet() {
        let json = read::get_tweet(&MockProvider, "t1").await;
        assert_conformant_success(&json, "get_tweet");
    }

    #[tokio::test]
    async fn conformance_read_get_user_by_username() {
        let json = read::get_user_by_username(&MockProvider, "alice").await;
        assert_conformant_success(&json, "get_user_by_username");
    }

    #[tokio::test]
    async fn conformance_read_search_tweets() {
        let json = read::search_tweets(&MockProvider, "rust", 10, None, None).await;
        assert_conformant_success(&json, "search_tweets");
    }

    #[tokio::test]
    async fn conformance_read_get_user_mentions() {
        let json = read::get_user_mentions(&MockProvider, "u1", None, None).await;
        assert_conformant_success(&json, "get_user_mentions");
    }

    #[tokio::test]
    async fn conformance_read_get_user_tweets() {
        let json = read::get_user_tweets(&MockProvider, "u1", 10, None).await;
        assert_conformant_success(&json, "get_user_tweets");
    }

    #[tokio::test]
    async fn conformance_read_get_home_timeline() {
        let json = read::get_home_timeline(&MockProvider, "u1", 20, None).await;
        assert_conformant_success(&json, "get_home_timeline");
    }

    #[tokio::test]
    async fn conformance_read_get_me() {
        let json = utils::get_me(&MockProvider).await;
        assert_conformant_success(&json, "get_me");
    }

    #[tokio::test]
    async fn conformance_read_get_followers() {
        let json = read::get_followers(&MockProvider, "u1", 10, None).await;
        assert_conformant_success(&json, "get_followers");
    }

    #[tokio::test]
    async fn conformance_read_get_following() {
        let json = read::get_following(&MockProvider, "u1", 10, None).await;
        assert_conformant_success(&json, "get_following");
    }

    #[tokio::test]
    async fn conformance_read_get_user_by_id() {
        let json = read::get_user_by_id(&MockProvider, "u42").await;
        assert_conformant_success(&json, "get_user_by_id");
    }

    #[tokio::test]
    async fn conformance_read_get_liked_tweets() {
        let json = read::get_liked_tweets(&MockProvider, "u1", 10, None).await;
        assert_conformant_success(&json, "get_liked_tweets");
    }

    #[tokio::test]
    async fn conformance_read_get_bookmarks() {
        let json = read::get_bookmarks(&MockProvider, "u1", 10, None).await;
        assert_conformant_success(&json, "get_bookmarks");
    }

    #[tokio::test]
    async fn conformance_read_get_users_by_ids() {
        let json = read::get_users_by_ids(&MockProvider, &["u1", "u2"]).await;
        assert_conformant_success(&json, "get_users_by_ids");
    }

    #[tokio::test]
    async fn conformance_read_get_tweet_liking_users() {
        let json = read::get_tweet_liking_users(&MockProvider, "t1", 10, None).await;
        assert_conformant_success(&json, "get_tweet_liking_users");
    }

    // ══════════════════════════════════════════════════════════════════
    // Write tool conformance
    // ══════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn conformance_write_post_tweet() {
        let json = write::post_tweet(&MockXApiClient, "Hello!", None).await;
        assert_conformant_success(&json, "post_tweet");
    }

    #[tokio::test]
    async fn conformance_write_reply_to_tweet() {
        let json = write::reply_to_tweet(&MockXApiClient, "Great!", "t1", None).await;
        assert_conformant_success(&json, "reply_to_tweet");
    }

    #[tokio::test]
    async fn conformance_write_quote_tweet() {
        let json = write::quote_tweet(&MockXApiClient, "So true!", "t1").await;
        assert_conformant_success(&json, "quote_tweet");
    }

    #[tokio::test]
    async fn conformance_write_delete_tweet() {
        let json = write::delete_tweet(&MockXApiClient, "t1").await;
        assert_conformant_success(&json, "delete_tweet");
    }

    #[tokio::test]
    async fn conformance_write_post_thread() {
        let tweets = vec!["First".to_string(), "Second".to_string()];
        let json = write::post_thread(&MockXApiClient, &tweets, None).await;
        assert_conformant_success(&json, "post_thread");
    }

    // ══════════════════════════════════════════════════════════════════
    // Engage tool conformance
    // ══════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn conformance_engage_like_tweet() {
        let json = engage::like_tweet(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "like_tweet");
    }

    #[tokio::test]
    async fn conformance_engage_unlike_tweet() {
        let json = engage::unlike_tweet(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "unlike_tweet");
    }

    #[tokio::test]
    async fn conformance_engage_follow_user() {
        let json = engage::follow_user(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "follow_user");
    }

    #[tokio::test]
    async fn conformance_engage_unfollow_user() {
        let json = engage::unfollow_user(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "unfollow_user");
    }

    #[tokio::test]
    async fn conformance_engage_retweet() {
        let json = engage::retweet(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "retweet");
    }

    #[tokio::test]
    async fn conformance_engage_unretweet() {
        let json = engage::unretweet(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "unretweet");
    }

    #[tokio::test]
    async fn conformance_engage_bookmark_tweet() {
        let json = engage::bookmark_tweet(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "bookmark_tweet");
    }

    #[tokio::test]
    async fn conformance_engage_unbookmark_tweet() {
        let json = engage::unbookmark_tweet(&MockXApiClient, "u1", "t1").await;
        assert_conformant_success(&json, "unbookmark_tweet");
    }

    // ══════════════════════════════════════════════════════════════════
    // Error path conformance
    // ══════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn conformance_error_rate_limited() {
        let json = read::search_tweets(&ErrorProvider, "test", 10, None, None).await;
        assert_conformant_error(&json, "search_tweets/rate_limited", "x_rate_limited");
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed["error"]["retry_after_ms"], 60000,
            "retry_after_ms should be 60000 for 60s retry"
        );
    }

    #[tokio::test]
    async fn conformance_error_auth_expired() {
        let json = read::get_user_by_username(&ErrorProvider, "nobody").await;
        assert_conformant_error(&json, "get_user_by_username/auth_expired", "x_auth_expired");
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(
            parsed["error"].get("retry_after_ms").is_none(),
            "auth errors should not have retry_after_ms"
        );
    }

    #[tokio::test]
    async fn conformance_error_network() {
        let json = read::get_followers(&ErrorProvider, "u1", 10, None).await;
        assert_conformant_error(&json, "get_followers/network", "x_network_error");
    }

    #[tokio::test]
    async fn conformance_error_other() {
        let json = read::get_tweet(&ErrorProvider, "missing").await;
        assert_conformant_error(&json, "get_tweet/other", "x_api_error");
    }

    #[tokio::test]
    async fn conformance_error_get_me_auth_expired() {
        let json = utils::get_me(&ErrorProvider).await;
        assert_conformant_error(&json, "get_me/auth_expired", "x_auth_expired");
    }

    // ══════════════════════════════════════════════════════════════════
    // Aggregate conformance test
    // ══════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn conformance_all_kernel_tools_produce_valid_envelope() {
        let mut results: Vec<(&str, bool)> = Vec::new();

        // Read tools (14)
        let tools: Vec<(&str, String)> = vec![
            ("get_tweet", read::get_tweet(&MockProvider, "t1").await),
            (
                "get_user_by_username",
                read::get_user_by_username(&MockProvider, "a").await,
            ),
            (
                "search_tweets",
                read::search_tweets(&MockProvider, "q", 10, None, None).await,
            ),
            (
                "get_user_mentions",
                read::get_user_mentions(&MockProvider, "u1", None, None).await,
            ),
            (
                "get_user_tweets",
                read::get_user_tweets(&MockProvider, "u1", 10, None).await,
            ),
            (
                "get_home_timeline",
                read::get_home_timeline(&MockProvider, "u1", 20, None).await,
            ),
            ("get_me", utils::get_me(&MockProvider).await),
            (
                "get_followers",
                read::get_followers(&MockProvider, "u1", 10, None).await,
            ),
            (
                "get_following",
                read::get_following(&MockProvider, "u1", 10, None).await,
            ),
            (
                "get_user_by_id",
                read::get_user_by_id(&MockProvider, "u1").await,
            ),
            (
                "get_liked_tweets",
                read::get_liked_tweets(&MockProvider, "u1", 10, None).await,
            ),
            (
                "get_bookmarks",
                read::get_bookmarks(&MockProvider, "u1", 10, None).await,
            ),
            (
                "get_users_by_ids",
                read::get_users_by_ids(&MockProvider, &["u1"]).await,
            ),
            (
                "get_tweet_liking_users",
                read::get_tweet_liking_users(&MockProvider, "t1", 10, None).await,
            ),
        ];
        for (name, json) in &tools {
            let parsed: Value = serde_json::from_str(json).unwrap_or_default();
            let valid = parsed["success"].as_bool().unwrap_or(false)
                && parsed.get("meta").is_some()
                && parsed["meta"]["tool_version"] == "1.0";
            results.push((name, valid));
        }

        // Write tools (5)
        let write_tools: Vec<(&str, String)> = vec![
            (
                "post_tweet",
                write::post_tweet(&MockXApiClient, "Hi", None).await,
            ),
            (
                "reply_to_tweet",
                write::reply_to_tweet(&MockXApiClient, "Re", "t1", None).await,
            ),
            (
                "quote_tweet",
                write::quote_tweet(&MockXApiClient, "QT", "t1").await,
            ),
            (
                "delete_tweet",
                write::delete_tweet(&MockXApiClient, "t1").await,
            ),
            (
                "post_thread",
                write::post_thread(&MockXApiClient, &["A".to_string()], None).await,
            ),
        ];
        for (name, json) in &write_tools {
            let parsed: Value = serde_json::from_str(json).unwrap_or_default();
            let valid = parsed["success"].as_bool().unwrap_or(false)
                && parsed.get("meta").is_some()
                && parsed["meta"]["tool_version"] == "1.0";
            results.push((name, valid));
        }

        // Engage tools (8)
        let engage_tools: Vec<(&str, String)> = vec![
            (
                "like_tweet",
                engage::like_tweet(&MockXApiClient, "u1", "t1").await,
            ),
            (
                "unlike_tweet",
                engage::unlike_tweet(&MockXApiClient, "u1", "t1").await,
            ),
            (
                "follow_user",
                engage::follow_user(&MockXApiClient, "u1", "t1").await,
            ),
            (
                "unfollow_user",
                engage::unfollow_user(&MockXApiClient, "u1", "t1").await,
            ),
            (
                "retweet",
                engage::retweet(&MockXApiClient, "u1", "t1").await,
            ),
            (
                "unretweet",
                engage::unretweet(&MockXApiClient, "u1", "t1").await,
            ),
            (
                "bookmark_tweet",
                engage::bookmark_tweet(&MockXApiClient, "u1", "t1").await,
            ),
            (
                "unbookmark_tweet",
                engage::unbookmark_tweet(&MockXApiClient, "u1", "t1").await,
            ),
        ];
        for (name, json) in &engage_tools {
            let parsed: Value = serde_json::from_str(json).unwrap_or_default();
            let valid = parsed["success"].as_bool().unwrap_or(false)
                && parsed.get("meta").is_some()
                && parsed["meta"]["tool_version"] == "1.0";
            results.push((name, valid));
        }

        let total = results.len();
        let passed = results.iter().filter(|(_, v)| *v).count();
        let rate = passed as f64 / total as f64 * 100.0;

        // Write conformance report
        let mut md = String::from("# Session 09 — Kernel Conformance Results\n\n");
        md.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));
        md.push_str(&format!(
            "**Conformance rate:** {passed}/{total} ({rate:.1}%)\n\n"
        ));
        md.push_str("| Tool | Conformant |\n");
        md.push_str("|------|------------|\n");
        for (name, valid) in &results {
            md.push_str(&format!(
                "| {} | {} |\n",
                name,
                if *valid { "PASS" } else { "FAIL" }
            ));
        }

        let artifacts_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs/roadmap/artifacts");
        std::fs::create_dir_all(&artifacts_dir).expect("create artifacts dir");
        std::fs::write(artifacts_dir.join("session-09-conformance-results.md"), &md)
            .expect("write conformance results");

        assert_eq!(
            rate,
            100.0,
            "Conformance rate {rate:.1}% < 100%. Failures: {:?}",
            results
                .iter()
                .filter(|(_, v)| !v)
                .map(|(n, _)| *n)
                .collect::<Vec<_>>()
        );
    }
}
