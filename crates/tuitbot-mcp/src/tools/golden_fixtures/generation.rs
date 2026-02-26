use std::collections::BTreeMap;

use serde_json::Value;

use super::{extract_keys, shape_of, FixtureFamily, GoldenFixtures};
use crate::contract::ProviderError;
use crate::kernel::{engage, read, write};
use crate::provider::SocialReadProvider;
use crate::tools::test_mocks::MockXApiClient;
use tuitbot_core::x_api::types::*;

// Golden fixtures need richer mock data (includes, non-default metrics)
// to produce meaningful shapes, so we use a local provider here.
pub(super) struct GoldenMockProvider;

#[async_trait::async_trait]
impl SocialReadProvider for GoldenMockProvider {
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

    async fn get_users_by_ids(&self, user_ids: &[&str]) -> Result<UsersResponse, ProviderError> {
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

// Golden fixtures also need an error provider with rate-limited get_tweet.
pub(super) struct GoldenErrorProvider;

#[async_trait::async_trait]
impl SocialReadProvider for GoldenErrorProvider {
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

pub async fn generate_fixtures() -> GoldenFixtures {
    let mut families = BTreeMap::new();

    // 1. read_single_tweet
    {
        let json = read::get_tweet(&GoldenMockProvider, "t1").await;
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
        let json = read::get_user_by_username(&GoldenMockProvider, "alice").await;
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
        let json = read::search_tweets(&GoldenMockProvider, "q", 10, None, None).await;
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
        let json = read::get_followers(&GoldenMockProvider, "u1", 10, None).await;
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
        let json = read::get_tweet(&GoldenErrorProvider, "t1").await;
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
        let json = read::get_user_by_username(&GoldenErrorProvider, "u").await;
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
