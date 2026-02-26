//! Golden fixture tests for tool response schema drift detection.
//!
//! Captures the structural shape of tool response `data` fields for each
//! tool family. On first run, generates a golden JSON file; on subsequent
//! runs, asserts the shape hasn't drifted.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::contract::ProviderError;
    use crate::kernel::{engage, read, write};
    use crate::provider::SocialReadProvider;
    use tuitbot_core::error::XApiError;
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;

    // ── Golden fixture types ─────────────────────────────────────────

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct GoldenFixtures {
        version: String,
        generated: String,
        families: BTreeMap<String, FixtureFamily>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct FixtureFamily {
        description: String,
        tools: Vec<String>,
        data_keys: Vec<String>,
        has_pagination: bool,
        sample_shape: Value,
    }

    // ── Mock provider ────────────────────────────────────────────────

    struct MockProvider;

    #[async_trait::async_trait]
    impl SocialReadProvider for MockProvider {
        async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError> {
            Ok(Tweet {
                id: tweet_id.to_string(),
                text: "Mock tweet".to_string(),
                author_id: "author_1".to_string(),
                created_at: "2026-02-25T00:00:00Z".to_string(),
                public_metrics: PublicMetrics {
                    like_count: 5,
                    retweet_count: 1,
                    reply_count: 0,
                    quote_count: 0,
                    impression_count: 100,
                    bookmark_count: 0,
                },
                conversation_id: None,
            })
        }

        async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError> {
            Ok(User {
                id: "u1".to_string(),
                username: username.to_string(),
                name: "Mock User".to_string(),
                public_metrics: UserMetrics {
                    followers_count: 100,
                    following_count: 50,
                    tweet_count: 200,
                },
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
                    created_at: "2026-02-25T00:00:00Z".to_string(),
                    public_metrics: PublicMetrics::default(),
                    conversation_id: None,
                }],
                includes: Some(Includes {
                    users: vec![User {
                        id: "a1".to_string(),
                        username: "user1".to_string(),
                        name: "User 1".to_string(),
                        public_metrics: UserMetrics::default(),
                    }],
                }),
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
            let count = user_ids.len() as u32;
            Ok(UsersResponse {
                data: user_ids
                    .iter()
                    .map(|id| User {
                        id: id.to_string(),
                        username: format!("user_{id}"),
                        name: format!("User {id}"),
                        public_metrics: UserMetrics::default(),
                    })
                    .collect(),
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

    // ── Mock XApiClient ──────────────────────────────────────────────

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
        async fn retweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
        async fn bookmark_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
    }

    // ── Error provider ───────────────────────────────────────────────

    struct ErrorProvider;

    #[async_trait::async_trait]
    impl SocialReadProvider for ErrorProvider {
        async fn get_tweet(&self, _tid: &str) -> Result<Tweet, ProviderError> {
            Err(ProviderError::RateLimited {
                retry_after: Some(30),
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
    }

    // ── Shape extraction ─────────────────────────────────────────────

    fn extract_keys(val: &Value) -> Vec<String> {
        match val {
            Value::Object(map) => map.keys().cloned().collect(),
            _ => vec![],
        }
    }

    fn shape_of(val: &Value) -> Value {
        match val {
            Value::Object(map) => {
                let shape: BTreeMap<String, Value> =
                    map.iter().map(|(k, v)| (k.clone(), shape_of(v))).collect();
                serde_json::to_value(shape).unwrap()
            }
            Value::Array(arr) => {
                if let Some(first) = arr.first() {
                    serde_json::json!([shape_of(first)])
                } else {
                    serde_json::json!([])
                }
            }
            Value::String(_) => serde_json::json!("string"),
            Value::Number(_) => serde_json::json!("number"),
            Value::Bool(_) => serde_json::json!("boolean"),
            Value::Null => serde_json::json!("null"),
        }
    }

    // ── Artifact paths ───────────────────────────────────────────────

    fn artifacts_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs/roadmap/artifacts")
    }

    // ── Golden fixture generation ────────────────────────────────────

    async fn generate_fixtures() -> GoldenFixtures {
        let mut families = BTreeMap::new();

        // 1. read_single_tweet
        {
            let json = read::get_tweet(&MockProvider, "t1").await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let data = &parsed["data"];
            families.insert(
                "read_single_tweet".to_string(),
                FixtureFamily {
                    description: "Single tweet by ID".to_string(),
                    tools: vec!["get_tweet".to_string()],
                    data_keys: extract_keys(data),
                    has_pagination: false,
                    sample_shape: shape_of(data),
                },
            );
        }

        // 2. read_single_user
        {
            let json = read::get_user_by_username(&MockProvider, "alice").await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let data = &parsed["data"];
            families.insert(
                "read_single_user".to_string(),
                FixtureFamily {
                    description: "Single user lookup".to_string(),
                    tools: vec![
                        "get_user_by_username".to_string(),
                        "get_user_by_id".to_string(),
                        "get_me".to_string(),
                    ],
                    data_keys: extract_keys(data),
                    has_pagination: false,
                    sample_shape: shape_of(data),
                },
            );
        }

        // 3. read_tweet_list
        {
            let json = read::search_tweets(&MockProvider, "q", 10, None, None).await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let data = &parsed["data"];
            let has_pagination = parsed
                .get("meta")
                .and_then(|m| m.get("pagination"))
                .is_some();
            families.insert(
                "read_tweet_list".to_string(),
                FixtureFamily {
                    description: "Paginated tweet list".to_string(),
                    tools: vec![
                        "search_tweets".to_string(),
                        "get_user_mentions".to_string(),
                        "get_user_tweets".to_string(),
                        "get_home_timeline".to_string(),
                        "get_liked_tweets".to_string(),
                        "get_bookmarks".to_string(),
                    ],
                    data_keys: extract_keys(data),
                    has_pagination,
                    sample_shape: shape_of(data),
                },
            );
        }

        // 4. read_users_list
        {
            let json = read::get_followers(&MockProvider, "u1", 10, None).await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let data = &parsed["data"];
            let has_pagination = parsed
                .get("meta")
                .and_then(|m| m.get("pagination"))
                .is_some();
            families.insert(
                "read_users_list".to_string(),
                FixtureFamily {
                    description: "Paginated user list".to_string(),
                    tools: vec![
                        "get_followers".to_string(),
                        "get_following".to_string(),
                        "get_users_by_ids".to_string(),
                        "get_tweet_liking_users".to_string(),
                    ],
                    data_keys: extract_keys(data),
                    has_pagination,
                    sample_shape: shape_of(data),
                },
            );
        }

        // 5. write_result
        {
            let json = write::post_tweet(&MockXApiClient, "Hello!", None).await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let data = &parsed["data"];
            families.insert(
                "write_result".to_string(),
                FixtureFamily {
                    description: "Write operation result".to_string(),
                    tools: vec![
                        "post_tweet".to_string(),
                        "reply_to_tweet".to_string(),
                        "quote_tweet".to_string(),
                    ],
                    data_keys: extract_keys(data),
                    has_pagination: false,
                    sample_shape: shape_of(data),
                },
            );
        }

        // 6. engage_result
        {
            let json = engage::like_tweet(&MockXApiClient, "u1", "t1").await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let data = &parsed["data"];
            families.insert(
                "engage_result".to_string(),
                FixtureFamily {
                    description: "Engage action result".to_string(),
                    tools: vec![
                        "like_tweet".to_string(),
                        "retweet".to_string(),
                        "bookmark_tweet".to_string(),
                    ],
                    data_keys: extract_keys(data),
                    has_pagination: false,
                    sample_shape: shape_of(data),
                },
            );
        }

        // 7. error_rate_limited
        {
            let json = read::get_tweet(&ErrorProvider, "t1").await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let error = &parsed["error"];
            families.insert(
                "error_rate_limited".to_string(),
                FixtureFamily {
                    description: "Rate-limited error shape".to_string(),
                    tools: vec!["any_tool_via_error_provider".to_string()],
                    data_keys: extract_keys(error),
                    has_pagination: false,
                    sample_shape: shape_of(error),
                },
            );
        }

        // 8. error_auth
        {
            let json = read::get_user_by_username(&ErrorProvider, "u").await;
            let parsed: Value = serde_json::from_str(&json).unwrap();
            let error = &parsed["error"];
            families.insert(
                "error_auth".to_string(),
                FixtureFamily {
                    description: "Auth expired error shape".to_string(),
                    tools: vec!["any_tool_via_error_provider".to_string()],
                    data_keys: extract_keys(error),
                    has_pagination: false,
                    sample_shape: shape_of(error),
                },
            );
        }

        GoldenFixtures {
            version: "1.0".to_string(),
            generated: chrono::Utc::now().to_rfc3339(),
            families,
        }
    }

    // ── Snapshot test ────────────────────────────────────────────────

    #[tokio::test]
    async fn golden_snapshot_matches() {
        let fixtures = generate_fixtures().await;
        let json = serde_json::to_string_pretty(&fixtures).unwrap();

        let dir = artifacts_dir();
        std::fs::create_dir_all(&dir).expect("create artifacts dir");
        let fixture_path = dir.join("session-09-golden-fixtures.json");

        if fixture_path.exists() {
            let existing = std::fs::read_to_string(&fixture_path).expect("read golden file");
            let existing_fixtures: GoldenFixtures =
                serde_json::from_str(&existing).expect("parse golden file");

            // Compare structural shapes (ignore generated timestamp)
            for (family_name, expected) in &existing_fixtures.families {
                let actual = fixtures.families.get(family_name).unwrap_or_else(|| {
                    panic!("Missing family in generated fixtures: {family_name}")
                });
                assert_eq!(
                    actual.data_keys, expected.data_keys,
                    "Data keys drifted for family {family_name}"
                );
                assert_eq!(
                    actual.sample_shape, expected.sample_shape,
                    "Shape drifted for family {family_name}"
                );
                assert_eq!(
                    actual.has_pagination, expected.has_pagination,
                    "Pagination flag drifted for family {family_name}"
                );
            }

            // Also check no families were removed
            for family_name in fixtures.families.keys() {
                assert!(
                    existing_fixtures.families.contains_key(family_name),
                    "New family {family_name} not in golden file — update snapshot"
                );
            }
        } else {
            // First run: write the golden file
            std::fs::write(&fixture_path, &json).expect("write golden file");
        }

        // Always write the golden report
        write_golden_report(&fixtures, &dir);
    }

    fn write_golden_report(fixtures: &GoldenFixtures, dir: &PathBuf) {
        let mut md = String::from("# Session 09 — Schema Golden Report\n\n");
        md.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));
        md.push_str("| Family | Tools | Keys | Pagination | Status |\n");
        md.push_str("|--------|-------|------|------------|--------|\n");
        for (name, family) in &fixtures.families {
            md.push_str(&format!(
                "| {} | {} | {} | {} | PASS |\n",
                name,
                family.tools.len(),
                family.data_keys.len(),
                if family.has_pagination { "yes" } else { "no" },
            ));
        }
        md.push_str(&format!(
            "\n**Total families:** {}\n",
            fixtures.families.len()
        ));

        std::fs::write(dir.join("session-09-schema-golden-report.md"), &md)
            .expect("write golden report");
    }

    // ── Individual family validation tests ───────────────────────────

    #[tokio::test]
    async fn golden_single_tweet_has_required_keys() {
        let json = read::get_tweet(&MockProvider, "t1").await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let data = &parsed["data"];
        for key in ["id", "text", "author_id", "created_at", "public_metrics"] {
            assert!(data.get(key).is_some(), "get_tweet missing key: {key}");
        }
    }

    #[tokio::test]
    async fn golden_single_user_has_required_keys() {
        let json = read::get_user_by_username(&MockProvider, "alice").await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let data = &parsed["data"];
        for key in ["id", "username", "name", "public_metrics"] {
            assert!(
                data.get(key).is_some(),
                "get_user_by_username missing key: {key}"
            );
        }
    }

    #[tokio::test]
    async fn golden_tweet_list_has_data_and_meta() {
        let json = read::search_tweets(&MockProvider, "q", 10, None, None).await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["data"]["data"].is_array(), "missing data.data[]");
        assert!(parsed["data"]["meta"].is_object(), "missing data.meta");
        assert!(
            parsed["meta"]["pagination"].is_object(),
            "missing meta.pagination"
        );
    }

    #[tokio::test]
    async fn golden_users_list_has_data_and_meta() {
        let json = read::get_followers(&MockProvider, "u1", 10, None).await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["data"]["data"].is_array(), "missing data.data[]");
        assert!(parsed["data"]["meta"].is_object(), "missing data.meta");
        assert!(
            parsed["meta"]["pagination"].is_object(),
            "missing meta.pagination"
        );
    }

    #[tokio::test]
    async fn golden_write_result_has_id_and_text() {
        let json = write::post_tweet(&MockXApiClient, "Hello!", None).await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["data"]["id"].is_string(), "missing data.id");
        assert!(parsed["data"]["text"].is_string(), "missing data.text");
    }

    #[tokio::test]
    async fn golden_engage_result_has_action_and_id() {
        let json = engage::like_tweet(&MockXApiClient, "u1", "t1").await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["data"]["liked"].is_boolean(), "missing data.liked");
        assert!(
            parsed["data"]["tweet_id"].is_string(),
            "missing data.tweet_id"
        );
    }

    #[tokio::test]
    async fn golden_error_rate_limited_has_retry_after() {
        let json = read::get_tweet(&ErrorProvider, "t1").await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(
            parsed["error"]["retry_after_ms"].is_number(),
            "missing error.retry_after_ms"
        );
        assert_eq!(parsed["error"]["code"], "x_rate_limited");
        assert!(parsed["error"]["retryable"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn golden_error_auth_no_retry_after() {
        let json = read::get_user_by_username(&ErrorProvider, "u").await;
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["error"]["code"], "x_auth_expired");
        assert!(!parsed["error"]["retryable"].as_bool().unwrap());
        assert!(
            parsed["error"].get("retry_after_ms").is_none(),
            "auth error should not have retry_after_ms"
        );
    }
}
