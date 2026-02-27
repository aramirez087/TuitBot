//! Stateless read operations over `&dyn XApiClient`.
//!
//! Each function validates inputs, delegates to the client trait,
//! and maps errors to `ToolkitError`. No state, no DB, no policy.

use super::ToolkitError;
use crate::x_api::types::{MentionResponse, SearchResponse, Tweet, User, UsersResponse};
use crate::x_api::XApiClient;

/// Get a single tweet by ID.
pub async fn get_tweet(client: &dyn XApiClient, tweet_id: &str) -> Result<Tweet, ToolkitError> {
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.get_tweet(tweet_id).await?)
}

/// Look up a user by username.
pub async fn get_user_by_username(
    client: &dyn XApiClient,
    username: &str,
) -> Result<User, ToolkitError> {
    super::validate_id(username, "username")?;
    Ok(client.get_user_by_username(username).await?)
}

/// Get a user by their numeric ID.
pub async fn get_user_by_id(client: &dyn XApiClient, user_id: &str) -> Result<User, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client.get_user_by_id(user_id).await?)
}

/// Get the authenticated user's profile.
pub async fn get_me(client: &dyn XApiClient) -> Result<User, ToolkitError> {
    Ok(client.get_me().await?)
}

/// Search recent tweets matching a query.
pub async fn search_tweets(
    client: &dyn XApiClient,
    query: &str,
    max_results: u32,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> Result<SearchResponse, ToolkitError> {
    if query.is_empty() {
        return Err(ToolkitError::InvalidInput {
            message: "query must not be empty".into(),
        });
    }
    Ok(client
        .search_tweets(query, max_results, since_id, pagination_token)
        .await?)
}

/// Get mentions for a user.
pub async fn get_mentions(
    client: &dyn XApiClient,
    user_id: &str,
    since_id: Option<&str>,
    pagination_token: Option<&str>,
) -> Result<MentionResponse, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client
        .get_mentions(user_id, since_id, pagination_token)
        .await?)
}

/// Get recent tweets from a specific user.
pub async fn get_user_tweets(
    client: &dyn XApiClient,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> Result<SearchResponse, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client
        .get_user_tweets(user_id, max_results, pagination_token)
        .await?)
}

/// Get the authenticated user's home timeline.
pub async fn get_home_timeline(
    client: &dyn XApiClient,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> Result<SearchResponse, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client
        .get_home_timeline(user_id, max_results, pagination_token)
        .await?)
}

/// Get followers of a user.
pub async fn get_followers(
    client: &dyn XApiClient,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> Result<UsersResponse, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client
        .get_followers(user_id, max_results, pagination_token)
        .await?)
}

/// Get accounts a user is following.
pub async fn get_following(
    client: &dyn XApiClient,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> Result<UsersResponse, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client
        .get_following(user_id, max_results, pagination_token)
        .await?)
}

/// Get tweets liked by a user.
pub async fn get_liked_tweets(
    client: &dyn XApiClient,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> Result<SearchResponse, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client
        .get_liked_tweets(user_id, max_results, pagination_token)
        .await?)
}

/// Get the authenticated user's bookmarks.
pub async fn get_bookmarks(
    client: &dyn XApiClient,
    user_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> Result<SearchResponse, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    Ok(client
        .get_bookmarks(user_id, max_results, pagination_token)
        .await?)
}

/// Get multiple users by their IDs (1-100).
pub async fn get_users_by_ids(
    client: &dyn XApiClient,
    user_ids: &[&str],
) -> Result<UsersResponse, ToolkitError> {
    if user_ids.is_empty() || user_ids.len() > 100 {
        return Err(ToolkitError::InvalidInput {
            message: format!("user_ids must contain 1-100 IDs, got {}", user_ids.len()),
        });
    }
    Ok(client.get_users_by_ids(user_ids).await?)
}

/// Get users who liked a specific tweet.
pub async fn get_tweet_liking_users(
    client: &dyn XApiClient,
    tweet_id: &str,
    max_results: u32,
    pagination_token: Option<&str>,
) -> Result<UsersResponse, ToolkitError> {
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client
        .get_tweet_liking_users(tweet_id, max_results, pagination_token)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::XApiError;
    use crate::x_api::types::*;

    struct MockClient;

    #[async_trait::async_trait]
    impl XApiClient for MockClient {
        async fn search_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Ok(empty_search())
        }
        async fn get_mentions(
            &self,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            Ok(empty_search())
        }
        async fn post_tweet(&self, _: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "t1".into(),
                text: "t".into(),
            })
        }
        async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "t2".into(),
                text: "t".into(),
            })
        }
        async fn get_tweet(&self, id: &str) -> Result<Tweet, XApiError> {
            Ok(Tweet {
                id: id.to_string(),
                text: "hello".into(),
                author_id: "a1".into(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            })
        }
        async fn get_me(&self) -> Result<User, XApiError> {
            Ok(test_user("me"))
        }
        async fn get_user_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Ok(empty_search())
        }
        async fn get_user_by_username(&self, u: &str) -> Result<User, XApiError> {
            Ok(test_user(u))
        }
    }

    fn test_user(id: &str) -> User {
        User {
            id: id.into(),
            username: id.into(),
            name: "Test".into(),
            public_metrics: UserMetrics::default(),
        }
    }

    fn empty_search() -> SearchResponse {
        SearchResponse {
            data: vec![],
            includes: None,
            meta: SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 0,
                next_token: None,
            },
        }
    }

    #[tokio::test]
    async fn get_tweet_success() {
        let t = get_tweet(&MockClient, "123").await.unwrap();
        assert_eq!(t.id, "123");
    }

    #[tokio::test]
    async fn get_tweet_empty_id() {
        let e = get_tweet(&MockClient, "").await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn get_user_by_username_success() {
        let u = get_user_by_username(&MockClient, "alice").await.unwrap();
        assert_eq!(u.username, "alice");
    }

    #[tokio::test]
    async fn get_user_by_username_empty() {
        let e = get_user_by_username(&MockClient, "").await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn search_tweets_success() {
        let r = search_tweets(&MockClient, "rust", 10, None, None)
            .await
            .unwrap();
        assert_eq!(r.meta.result_count, 0);
    }

    #[tokio::test]
    async fn search_tweets_empty_query() {
        let e = search_tweets(&MockClient, "", 10, None, None)
            .await
            .unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn get_me_success() {
        let u = get_me(&MockClient).await.unwrap();
        assert_eq!(u.id, "me");
    }

    #[tokio::test]
    async fn get_users_by_ids_empty() {
        let e = get_users_by_ids(&MockClient, &[]).await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn get_users_by_ids_over_100() {
        let ids: Vec<&str> = (0..101).map(|_| "x").collect();
        let e = get_users_by_ids(&MockClient, &ids).await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn x_api_error_maps_to_toolkit_error() {
        struct FailClient;
        #[async_trait::async_trait]
        impl XApiClient for FailClient {
            async fn search_tweets(
                &self,
                _: &str,
                _: u32,
                _: Option<&str>,
                _: Option<&str>,
            ) -> Result<SearchResponse, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
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
                Err(XApiError::ApiError {
                    status: 404,
                    message: "Not found".into(),
                })
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

        let e = get_tweet(&FailClient, "123").await.unwrap_err();
        assert!(matches!(
            e,
            ToolkitError::XApi(XApiError::ApiError { status: 404, .. })
        ));

        let e = search_tweets(&FailClient, "q", 10, None, None)
            .await
            .unwrap_err();
        assert!(matches!(
            e,
            ToolkitError::XApi(XApiError::RateLimited { .. })
        ));
    }
}
