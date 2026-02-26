//! Minimal benchmark-optimized mock providers.
//!
//! These are kept local (not shared via test_mocks) because they
//! implement a minimal subset of methods for speed benchmarking.

use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;
use tuitbot_core::error::XApiError;
use tuitbot_core::x_api::types::*;
use tuitbot_core::x_api::XApiClient;

pub struct BenchMockProvider;

#[async_trait::async_trait]
impl SocialReadProvider for BenchMockProvider {
    async fn get_tweet(&self, tid: &str) -> Result<Tweet, ProviderError> {
        Ok(Tweet {
            id: tid.to_string(),
            text: "Mock".to_string(),
            author_id: "a1".to_string(),
            created_at: "2026-02-25T00:00:00Z".to_string(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        })
    }
    async fn get_user_by_username(&self, u: &str) -> Result<User, ProviderError> {
        Ok(User {
            id: "u1".to_string(),
            username: u.to_string(),
            name: "Mock".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }
    async fn search_tweets(
        &self,
        _q: &str,
        _m: u32,
        _s: Option<&str>,
        _p: Option<&str>,
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
    async fn get_followers(
        &self,
        _u: &str,
        _m: u32,
        _p: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Ok(UsersResponse {
            data: vec![],
            meta: UsersMeta {
                result_count: 0,
                next_token: None,
            },
        })
    }
    async fn get_user_by_id(&self, uid: &str) -> Result<User, ProviderError> {
        Ok(User {
            id: uid.to_string(),
            username: "bench".to_string(),
            name: "Bench".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }
    async fn get_me(&self) -> Result<User, ProviderError> {
        Ok(User {
            id: "me".to_string(),
            username: "bench".to_string(),
            name: "Bench".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }
}

pub struct BenchMockXApiClient;

#[async_trait::async_trait]
impl XApiClient for BenchMockXApiClient {
    async fn search_tweets(
        &self,
        _q: &str,
        _m: u32,
        _s: Option<&str>,
        _p: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        unimplemented!()
    }
    async fn get_mentions(
        &self,
        _u: &str,
        _s: Option<&str>,
        _p: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        unimplemented!()
    }
    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "p1".to_string(),
            text: text.to_string(),
        })
    }
    async fn reply_to_tweet(&self, text: &str, _r: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "r1".to_string(),
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
        _u: &str,
        _m: u32,
        _p: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        unimplemented!()
    }
    async fn get_user_by_username(&self, _u: &str) -> Result<User, XApiError> {
        unimplemented!()
    }
}
