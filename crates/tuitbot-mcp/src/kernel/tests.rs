//! Kernel tests using mock providers and clients.
//!
//! Proves the kernel tools work through the provider/client boundary
//! without any real `XApiClient`, `AppState`, or database dependency.

use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;
use tuitbot_core::error::XApiError;
use tuitbot_core::x_api::types::{self, *};
use tuitbot_core::x_api::XApiClient;

use super::{engage, read, utils, write};

// ── Mock provider (success) ─────────────────────────────────────────

struct MockProvider;

#[async_trait::async_trait]
impl SocialReadProvider for MockProvider {
    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError> {
        Ok(Tweet {
            id: tweet_id.to_string(),
            text: "Mock tweet content".to_string(),
            author_id: "mock_author".to_string(),
            created_at: "2026-02-25T00:00:00Z".to_string(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError> {
        Ok(User {
            id: "mock_user_id".to_string(),
            username: username.to_string(),
            name: "Mock User".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }

    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "search_1".to_string(),
                text: "Found tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("search_1".to_string()),
                oldest_id: Some("search_1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_user_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "mention_1".to_string(),
                text: "@user hello".to_string(),
                author_id: "a2".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("mention_1".to_string()),
                oldest_id: Some("mention_1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_user_tweets(
        &self,
        user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "user_tweet_1".to_string(),
                text: "My tweet".to_string(),
                author_id: user_id.to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("user_tweet_1".to_string()),
                oldest_id: Some("user_tweet_1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_home_timeline(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "timeline_1".to_string(),
                text: "Timeline tweet".to_string(),
                author_id: "t1".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("timeline_1".to_string()),
                oldest_id: Some("timeline_1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_me(&self) -> Result<User, ProviderError> {
        Ok(User {
            id: "me_123".to_string(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }

    async fn get_followers(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<types::UsersResponse, ProviderError> {
        Ok(types::UsersResponse {
            data: vec![User {
                id: "f1".to_string(),
                username: "follower1".to_string(),
                name: "Follower One".to_string(),
                public_metrics: UserMetrics::default(),
            }],
            meta: types::UsersMeta {
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_following(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<types::UsersResponse, ProviderError> {
        Ok(types::UsersResponse {
            data: vec![User {
                id: "fw1".to_string(),
                username: "following1".to_string(),
                name: "Following One".to_string(),
                public_metrics: UserMetrics::default(),
            }],
            meta: types::UsersMeta {
                result_count: 1,
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
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "liked_1".to_string(),
                text: "Liked tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("liked_1".to_string()),
                oldest_id: Some("liked_1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_bookmarks(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "bk_1".to_string(),
                text: "Bookmarked tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("bk_1".to_string()),
                oldest_id: Some("bk_1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_users_by_ids(
        &self,
        user_ids: &[&str],
    ) -> Result<types::UsersResponse, ProviderError> {
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
        Ok(types::UsersResponse {
            data: users,
            meta: types::UsersMeta {
                result_count: count,
                next_token: None,
            },
        })
    }

    async fn get_tweet_liking_users(
        &self,
        _tweet_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<types::UsersResponse, ProviderError> {
        Ok(types::UsersResponse {
            data: vec![User {
                id: "lu1".to_string(),
                username: "liker1".to_string(),
                name: "Liker One".to_string(),
                public_metrics: UserMetrics::default(),
            }],
            meta: types::UsersMeta {
                result_count: 1,
                next_token: None,
            },
        })
    }
}

// ── Mock provider (errors) ──────────────────────────────────────────

struct ErrorProvider;

#[async_trait::async_trait]
impl SocialReadProvider for ErrorProvider {
    async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, ProviderError> {
        Err(ProviderError::Other {
            message: "not found".to_string(),
        })
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<User, ProviderError> {
        Err(ProviderError::AuthExpired)
    }

    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::RateLimited {
            retry_after: Some(60),
        })
    }

    async fn get_me(&self) -> Result<User, ProviderError> {
        Err(ProviderError::AuthExpired)
    }
}

// ── Mock X API client (success) ─────────────────────────────────────

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
        _in_reply_to: &str,
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

    async fn get_user_by_username(&self, _username: &str) -> Result<User, XApiError> {
        unimplemented!()
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

    async fn quote_tweet(&self, text: &str, _quoted: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "quote_1".to_string(),
            text: text.to_string(),
        })
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

// ── Mock X API client (errors) ──────────────────────────────────────

struct ErrorXApiClient;

#[async_trait::async_trait]
impl XApiClient for ErrorXApiClient {
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

    async fn post_tweet(&self, _text: &str) -> Result<PostedTweet, XApiError> {
        Err(XApiError::RateLimited {
            retry_after: Some(30),
        })
    }

    async fn reply_to_tweet(
        &self,
        _text: &str,
        _in_reply_to: &str,
    ) -> Result<PostedTweet, XApiError> {
        Err(XApiError::Forbidden {
            message: "not allowed".to_string(),
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

    async fn get_user_by_username(&self, _username: &str) -> Result<User, XApiError> {
        unimplemented!()
    }

    async fn like_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Err(XApiError::AuthExpired)
    }

    async fn follow_user(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Err(XApiError::Forbidden {
            message: "not allowed to follow".to_string(),
        })
    }

    async fn unlike_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Err(XApiError::AuthExpired)
    }

    async fn bookmark_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Err(XApiError::Forbidden {
            message: "not allowed to bookmark".to_string(),
        })
    }

    async fn unbookmark_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Err(XApiError::RateLimited {
            retry_after: Some(30),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════
// Read tests (original)
// ══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_tweet_success() {
    let json = read::get_tweet(&MockProvider, "t42").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "t42");
    assert_eq!(parsed["data"]["text"], "Mock tweet content");
    assert!(parsed["meta"]["elapsed_ms"].is_number());
}

#[tokio::test]
async fn get_user_by_username_success() {
    let json = read::get_user_by_username(&MockProvider, "alice").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["username"], "alice");
    assert_eq!(parsed["data"]["id"], "mock_user_id");
}

#[tokio::test]
async fn search_tweets_success() {
    let json = read::search_tweets(&MockProvider, "rust", 10, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["meta"]["result_count"], 1);
    assert_eq!(parsed["data"]["data"][0]["text"], "Found tweet");
}

#[tokio::test]
async fn search_tweets_with_pagination() {
    let json = read::search_tweets(&MockProvider, "rust", 10, Some("s1"), Some("next")).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
}

// ── Read error tests ────────────────────────────────────────────────

#[tokio::test]
async fn get_tweet_error() {
    let json = read::get_tweet(&ErrorProvider, "missing").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_api_error");
    assert_eq!(parsed["error"]["retryable"], false);
}

#[tokio::test]
async fn get_user_by_username_auth_expired() {
    let json = read::get_user_by_username(&ErrorProvider, "nobody").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_auth_expired");
    assert_eq!(parsed["error"]["retryable"], false);
}

#[tokio::test]
async fn search_tweets_rate_limited() {
    let json = read::search_tweets(&ErrorProvider, "test", 10, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_rate_limited");
    assert_eq!(parsed["error"]["retryable"], true);
    assert!(parsed["error"]["message"].as_str().unwrap().contains("60s"));
}

// ── Envelope structure tests ────────────────────────────────────────

#[tokio::test]
async fn response_always_has_meta() {
    let json = read::get_tweet(&MockProvider, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["meta"].is_object());
    assert_eq!(parsed["meta"]["tool_version"], "1.0");
}

#[tokio::test]
async fn error_response_has_meta() {
    let json = read::get_tweet(&ErrorProvider, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["meta"].is_object());
    assert_eq!(parsed["meta"]["tool_version"], "1.0");
}

// ══════════════════════════════════════════════════════════════════════
// New read tests (mentions, user_tweets, home_timeline, get_me)
// ══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_user_mentions_success() {
    let json = read::get_user_mentions(&MockProvider, "user_123", None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["id"], "mention_1");
    assert_eq!(parsed["data"]["meta"]["result_count"], 1);
}

#[tokio::test]
async fn get_user_tweets_success() {
    let json = read::get_user_tweets(&MockProvider, "user_456", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["id"], "user_tweet_1");
    assert_eq!(parsed["data"]["data"][0]["author_id"], "user_456");
}

#[tokio::test]
async fn get_home_timeline_success() {
    let json = read::get_home_timeline(&MockProvider, "user_789", 20, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["id"], "timeline_1");
    assert_eq!(parsed["data"]["data"][0]["text"], "Timeline tweet");
}

// ── Utils: get_me tests ─────────────────────────────────────────────

#[tokio::test]
async fn get_me_success() {
    let json = utils::get_me(&MockProvider).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "me_123");
    assert_eq!(parsed["data"]["username"], "testuser");
}

#[tokio::test]
async fn get_me_auth_expired() {
    let json = utils::get_me(&ErrorProvider).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_auth_expired");
}

// ── Utils: check_tweet_length tests ─────────────────────────────────

#[test]
fn check_tweet_length_ok() {
    let start = std::time::Instant::now();
    assert!(utils::check_tweet_length("Hello world", start).is_none());
}

#[test]
fn check_tweet_length_too_long() {
    let start = std::time::Instant::now();
    let text = "a".repeat(281);
    let err = utils::check_tweet_length(&text, start).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&err).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
}

// ══════════════════════════════════════════════════════════════════════
// Write tests
// ══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn write_post_tweet_success() {
    let json = write::post_tweet(&MockXApiClient, "Hello!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "posted_1");
    assert_eq!(parsed["data"]["text"], "Hello!");
}

#[tokio::test]
async fn write_post_tweet_too_long() {
    let long_text = "a".repeat(281);
    let json = write::post_tweet(&MockXApiClient, &long_text, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
}

#[tokio::test]
async fn write_post_tweet_api_error() {
    let json = write::post_tweet(&ErrorXApiClient, "Hello!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_rate_limited");
    assert_eq!(parsed["error"]["retryable"], true);
}

#[tokio::test]
async fn write_reply_to_tweet_success() {
    let json = write::reply_to_tweet(&MockXApiClient, "Great!", "t123", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "reply_1");
}

#[tokio::test]
async fn write_quote_tweet_success() {
    let json = write::quote_tweet(&MockXApiClient, "So true!", "t456").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "quote_1");
}

#[tokio::test]
async fn write_delete_tweet_success() {
    let json = write::delete_tweet(&MockXApiClient, "t789").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["deleted"], true);
    assert_eq!(parsed["data"]["tweet_id"], "t789");
}

#[tokio::test]
async fn write_post_thread_empty() {
    let json = write::post_thread(&MockXApiClient, &[], None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "invalid_input");
}

#[tokio::test]
async fn write_post_thread_success() {
    let tweets = vec!["First tweet".to_string(), "Second tweet".to_string()];
    let json = write::post_thread(&MockXApiClient, &tweets, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["tweet_count"], 2);
    assert!(parsed["data"]["root_tweet_id"].is_string());
}

#[tokio::test]
async fn write_post_thread_too_long_tweet() {
    let tweets = vec!["OK".to_string(), "a".repeat(281)];
    let json = write::post_thread(&MockXApiClient, &tweets, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
    assert_eq!(parsed["error"]["tweet_index"], 1);
}

// ══════════════════════════════════════════════════════════════════════
// Engage tests
// ══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn engage_like_tweet_success() {
    let json = engage::like_tweet(&MockXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["liked"], true);
    assert_eq!(parsed["data"]["tweet_id"], "t1");
}

#[tokio::test]
async fn engage_like_tweet_auth_error() {
    let json = engage::like_tweet(&ErrorXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_auth_expired");
}

#[tokio::test]
async fn engage_follow_user_success() {
    let json = engage::follow_user(&MockXApiClient, "u1", "target_1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["following"], true);
    assert_eq!(parsed["data"]["target_user_id"], "target_1");
}

#[tokio::test]
async fn engage_follow_user_forbidden_error() {
    let json = engage::follow_user(&ErrorXApiClient, "u1", "target_1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_forbidden");
    assert_eq!(parsed["error"]["retryable"], false);
}

#[tokio::test]
async fn engage_unfollow_user_success() {
    let json = engage::unfollow_user(&MockXApiClient, "u1", "target_1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["following"], false);
}

#[tokio::test]
async fn engage_retweet_success() {
    let json = engage::retweet(&MockXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["retweeted"], true);
}

#[tokio::test]
async fn engage_unretweet_success() {
    let json = engage::unretweet(&MockXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["retweeted"], false);
}

// ══════════════════════════════════════════════════════════════════════
// New read tests (followers, following, user_by_id, liked_tweets,
//                 bookmarks, users_by_ids, tweet_liking_users)
// ══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_followers_success() {
    let json = read::get_followers(&MockProvider, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["username"], "follower1");
    assert_eq!(parsed["data"]["meta"]["result_count"], 1);
}

#[tokio::test]
async fn get_following_success() {
    let json = read::get_following(&MockProvider, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["username"], "following1");
}

#[tokio::test]
async fn get_user_by_id_success() {
    let json = read::get_user_by_id(&MockProvider, "u42").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "u42");
    assert_eq!(parsed["data"]["username"], "iduser");
}

#[tokio::test]
async fn get_liked_tweets_success() {
    let json = read::get_liked_tweets(&MockProvider, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["id"], "liked_1");
}

#[tokio::test]
async fn get_bookmarks_success() {
    let json = read::get_bookmarks(&MockProvider, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["id"], "bk_1");
}

#[tokio::test]
async fn get_users_by_ids_success() {
    let json = read::get_users_by_ids(&MockProvider, &["u1", "u2"]).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn get_users_by_ids_empty_input_error() {
    let json = read::get_users_by_ids(&MockProvider, &[]).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "invalid_input");
}

#[tokio::test]
async fn get_tweet_liking_users_success() {
    let json = read::get_tweet_liking_users(&MockProvider, "t1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["username"], "liker1");
}

// ══════════════════════════════════════════════════════════════════════
// New engage tests (unlike, bookmark, unbookmark)
// ══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn engage_unlike_tweet_success() {
    let json = engage::unlike_tweet(&MockXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["liked"], false);
    assert_eq!(parsed["data"]["tweet_id"], "t1");
}

#[tokio::test]
async fn engage_unlike_tweet_auth_error() {
    let json = engage::unlike_tweet(&ErrorXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_auth_expired");
}

#[tokio::test]
async fn engage_bookmark_tweet_success() {
    let json = engage::bookmark_tweet(&MockXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["bookmarked"], true);
    assert_eq!(parsed["data"]["tweet_id"], "t1");
}

#[tokio::test]
async fn engage_bookmark_tweet_forbidden_error() {
    let json = engage::bookmark_tweet(&ErrorXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_forbidden");
}

#[tokio::test]
async fn engage_unbookmark_tweet_success() {
    let json = engage::unbookmark_tweet(&MockXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["bookmarked"], false);
}

#[tokio::test]
async fn engage_unbookmark_tweet_rate_limited() {
    let json = engage::unbookmark_tweet(&ErrorXApiClient, "u1", "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_rate_limited");
    assert_eq!(parsed["error"]["retryable"], true);
}
