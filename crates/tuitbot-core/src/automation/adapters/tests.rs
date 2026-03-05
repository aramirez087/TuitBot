use std::sync::Arc;

use super::*;
use crate::automation::analytics_loop::{EngagementFetcher, ProfileFetcher};
use crate::automation::loop_helpers::{LoopError, MentionsFetcher, ThreadPoster, TweetSearcher};
use crate::automation::posting_queue::PostExecutor;
use crate::automation::target_loop::{TargetTweetFetcher, TargetUserManager};
use crate::x_api::types::*;
use crate::x_api::{SearchResponse, XApiClient};

/// Mock XApiClient that returns deterministic responses.
/// Used to verify adapters route through toolkit functions.
struct MockXApiClient;

#[async_trait::async_trait]
impl XApiClient for MockXApiClient {
    async fn search_tweets(
        &self,
        query: &str,
        _: u32,
        _: Option<&str>,
        _: Option<&str>,
    ) -> Result<SearchResponse, crate::error::XApiError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "st1".into(),
                text: query.into(),
                author_id: "a1".into(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_mentions(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<&str>,
    ) -> Result<MentionResponse, crate::error::XApiError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "m1".into(),
                text: "mention".into(),
                author_id: "a2".into(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, crate::error::XApiError> {
        Ok(PostedTweet {
            id: "pt1".into(),
            text: text.into(),
        })
    }

    async fn reply_to_tweet(
        &self,
        text: &str,
        _: &str,
    ) -> Result<PostedTweet, crate::error::XApiError> {
        Ok(PostedTweet {
            id: "rt1".into(),
            text: text.into(),
        })
    }

    async fn get_tweet(&self, id: &str) -> Result<Tweet, crate::error::XApiError> {
        Ok(Tweet {
            id: id.into(),
            text: "tweet text".into(),
            author_id: "a1".into(),
            created_at: String::new(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        })
    }

    async fn get_me(&self) -> Result<User, crate::error::XApiError> {
        Ok(User {
            id: "me".into(),
            username: "testuser".into(),
            name: "Test".into(),
            profile_image_url: None,
            public_metrics: UserMetrics::default(),
        })
    }

    async fn get_user_tweets(
        &self,
        _: &str,
        _: u32,
        _: Option<&str>,
    ) -> Result<SearchResponse, crate::error::XApiError> {
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

    async fn get_user_by_username(&self, u: &str) -> Result<User, crate::error::XApiError> {
        Ok(User {
            id: format!("uid_{u}"),
            username: u.into(),
            name: "Test".into(),
            profile_image_url: None,
            public_metrics: UserMetrics::default(),
        })
    }
}

fn mock_client() -> Arc<dyn XApiClient> {
    Arc::new(MockXApiClient) as Arc<dyn XApiClient>
}

// --- TweetSearcher (routes through toolkit::read::search_tweets) ---

#[tokio::test]
async fn search_adapter_routes_through_toolkit() {
    let adapter = XApiSearchAdapter::new(mock_client());
    let tweets = adapter.search_tweets("rust").await.unwrap();
    assert_eq!(tweets.len(), 1);
    assert_eq!(tweets[0].id, "st1");
    assert_eq!(tweets[0].text, "rust");
}

// --- MentionsFetcher (routes through toolkit::read::get_mentions) ---

#[tokio::test]
async fn mentions_adapter_routes_through_toolkit() {
    let adapter = XApiMentionsAdapter::new(mock_client(), "me".into());
    let tweets = adapter.get_mentions(None).await.unwrap();
    assert_eq!(tweets.len(), 1);
    assert_eq!(tweets[0].id, "m1");
}

// --- TargetTweetFetcher (routes through toolkit::read::get_user_tweets) ---

#[tokio::test]
async fn target_adapter_fetch_routes_through_toolkit() {
    let adapter = XApiTargetAdapter::new(mock_client());
    let tweets = adapter.fetch_user_tweets("u1").await.unwrap();
    assert!(tweets.is_empty()); // mock returns empty
}

// --- TargetUserManager (routes through toolkit::read::get_user_by_username) ---

#[tokio::test]
async fn target_adapter_lookup_routes_through_toolkit() {
    let adapter = XApiTargetAdapter::new(mock_client());
    let (id, username) = adapter.lookup_user("alice").await.unwrap();
    assert_eq!(id, "uid_alice");
    assert_eq!(username, "alice");
}

// --- ProfileFetcher (routes through toolkit::read::get_me) ---

#[tokio::test]
async fn profile_adapter_routes_through_toolkit() {
    let adapter = XApiProfileAdapter::new(mock_client());
    let metrics = adapter.get_profile_metrics().await.unwrap();
    // MockXApiClient returns default UserMetrics (all zeros)
    assert_eq!(metrics.follower_count, 0);
}

// --- EngagementFetcher (routes through toolkit::read::get_tweet) ---

#[tokio::test]
async fn engagement_adapter_routes_through_toolkit() {
    let adapter = XApiProfileAdapter::new(mock_client());
    let metrics = adapter.get_tweet_metrics("t123").await.unwrap();
    assert_eq!(metrics.likes, 0);
}

// --- PostExecutor (routes through toolkit::write) ---

#[tokio::test]
async fn post_executor_reply_routes_through_toolkit() {
    let adapter = XApiPostExecutorAdapter::new(mock_client());
    let id = adapter.execute_reply("t0", "hello", &[]).await.unwrap();
    assert_eq!(id, "rt1");
}

#[tokio::test]
async fn post_executor_tweet_routes_through_toolkit() {
    let adapter = XApiPostExecutorAdapter::new(mock_client());
    let id = adapter.execute_tweet("hello", &[]).await.unwrap();
    assert_eq!(id, "pt1");
}

// --- ThreadPoster (routes through toolkit::write) ---

#[tokio::test]
async fn thread_poster_post_routes_through_toolkit() {
    let adapter = XApiThreadPosterAdapter::new(mock_client());
    let id = adapter.post_tweet("thread start").await.unwrap();
    assert_eq!(id, "pt1");
}

#[tokio::test]
async fn thread_poster_reply_routes_through_toolkit() {
    let adapter = XApiThreadPosterAdapter::new(mock_client());
    let id = adapter.reply_to_tweet("t0", "thread cont").await.unwrap();
    assert_eq!(id, "rt1");
}

// --- Error mapping: ToolkitError → LoopError ---

#[tokio::test]
async fn toolkit_error_maps_to_loop_error() {
    struct FailClient;
    #[async_trait::async_trait]
    impl XApiClient for FailClient {
        async fn search_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<SearchResponse, crate::error::XApiError> {
            Err(crate::error::XApiError::RateLimited {
                retry_after: Some(30),
            })
        }
        async fn get_mentions(
            &self,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<MentionResponse, crate::error::XApiError> {
            unimplemented!()
        }
        async fn post_tweet(&self, _: &str) -> Result<PostedTweet, crate::error::XApiError> {
            unimplemented!()
        }
        async fn reply_to_tweet(
            &self,
            _: &str,
            _: &str,
        ) -> Result<PostedTweet, crate::error::XApiError> {
            unimplemented!()
        }
        async fn get_tweet(&self, _: &str) -> Result<Tweet, crate::error::XApiError> {
            unimplemented!()
        }
        async fn get_me(&self) -> Result<User, crate::error::XApiError> {
            unimplemented!()
        }
        async fn get_user_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
        ) -> Result<SearchResponse, crate::error::XApiError> {
            unimplemented!()
        }
        async fn get_user_by_username(&self, _: &str) -> Result<User, crate::error::XApiError> {
            unimplemented!()
        }
    }

    let client: Arc<dyn XApiClient> = Arc::new(FailClient);
    let adapter = XApiSearchAdapter::new(client);
    let err = adapter.search_tweets("q").await.unwrap_err();
    assert!(matches!(
        err,
        LoopError::RateLimited {
            retry_after: Some(30)
        }
    ));
}

// --- Toolkit input validation propagates as LoopError ---

#[tokio::test]
async fn empty_id_triggers_toolkit_validation() {
    let adapter = XApiTargetAdapter::new(mock_client());
    let err = adapter.fetch_user_tweets("").await.unwrap_err();
    assert!(matches!(err, LoopError::Other(_)));
}
