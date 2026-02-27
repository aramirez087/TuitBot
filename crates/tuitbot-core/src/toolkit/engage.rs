//! Stateless engagement operations over `&dyn XApiClient`.
//!
//! Raw like, unlike, follow, unfollow, retweet, unretweet, bookmark, unbookmark.
//! No policy enforcement, no audit logging, no mutation recording (AD-04).

use super::ToolkitError;
use crate::x_api::XApiClient;

/// Like a tweet on behalf of a user.
pub async fn like_tweet(
    client: &dyn XApiClient,
    user_id: &str,
    tweet_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.like_tweet(user_id, tweet_id).await?)
}

/// Unlike a tweet on behalf of a user.
pub async fn unlike_tweet(
    client: &dyn XApiClient,
    user_id: &str,
    tweet_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.unlike_tweet(user_id, tweet_id).await?)
}

/// Follow a user.
pub async fn follow_user(
    client: &dyn XApiClient,
    user_id: &str,
    target_user_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(target_user_id, "target_user_id")?;
    Ok(client.follow_user(user_id, target_user_id).await?)
}

/// Unfollow a user.
pub async fn unfollow_user(
    client: &dyn XApiClient,
    user_id: &str,
    target_user_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(target_user_id, "target_user_id")?;
    Ok(client.unfollow_user(user_id, target_user_id).await?)
}

/// Retweet a tweet on behalf of a user.
pub async fn retweet(
    client: &dyn XApiClient,
    user_id: &str,
    tweet_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.retweet(user_id, tweet_id).await?)
}

/// Undo a retweet on behalf of a user.
pub async fn unretweet(
    client: &dyn XApiClient,
    user_id: &str,
    tweet_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.unretweet(user_id, tweet_id).await?)
}

/// Bookmark a tweet.
pub async fn bookmark_tweet(
    client: &dyn XApiClient,
    user_id: &str,
    tweet_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.bookmark_tweet(user_id, tweet_id).await?)
}

/// Remove a bookmark.
pub async fn unbookmark_tweet(
    client: &dyn XApiClient,
    user_id: &str,
    tweet_id: &str,
) -> Result<bool, ToolkitError> {
    super::validate_id(user_id, "user_id")?;
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.unbookmark_tweet(user_id, tweet_id).await?)
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
            unimplemented!()
        }
        async fn get_mentions(
            &self,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            unimplemented!()
        }
        async fn post_tweet(&self, _: &str) -> Result<PostedTweet, XApiError> {
            unimplemented!()
        }
        async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
            unimplemented!()
        }
        async fn get_tweet(&self, _: &str) -> Result<Tweet, XApiError> {
            unimplemented!()
        }
        async fn get_me(&self) -> Result<User, XApiError> {
            unimplemented!()
        }
        async fn get_user_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            unimplemented!()
        }
        async fn get_user_by_username(&self, _: &str) -> Result<User, XApiError> {
            unimplemented!()
        }
        async fn like_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
        async fn unlike_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(false)
        }
        async fn follow_user(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
        async fn unfollow_user(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(false)
        }
        async fn retweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
        async fn unretweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(false)
        }
        async fn bookmark_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
        async fn unbookmark_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(false)
        }
    }

    #[tokio::test]
    async fn like_tweet_success() {
        assert!(like_tweet(&MockClient, "u1", "t1").await.unwrap());
    }

    #[tokio::test]
    async fn like_tweet_empty_user_id() {
        let e = like_tweet(&MockClient, "", "t1").await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn like_tweet_empty_tweet_id() {
        let e = like_tweet(&MockClient, "u1", "").await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn follow_user_success() {
        assert!(follow_user(&MockClient, "u1", "u2").await.unwrap());
    }

    #[tokio::test]
    async fn unfollow_user_success() {
        assert!(!unfollow_user(&MockClient, "u1", "u2").await.unwrap());
    }

    #[tokio::test]
    async fn retweet_success() {
        assert!(retweet(&MockClient, "u1", "t1").await.unwrap());
    }

    #[tokio::test]
    async fn bookmark_tweet_success() {
        assert!(bookmark_tweet(&MockClient, "u1", "t1").await.unwrap());
    }

    #[tokio::test]
    async fn unbookmark_tweet_success() {
        assert!(!unbookmark_tweet(&MockClient, "u1", "t1").await.unwrap());
    }

    #[tokio::test]
    async fn x_api_error_propagates() {
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
                unimplemented!()
            }
            async fn get_mentions(
                &self,
                _: &str,
                _: Option<&str>,
                _: Option<&str>,
            ) -> Result<MentionResponse, XApiError> {
                unimplemented!()
            }
            async fn post_tweet(&self, _: &str) -> Result<PostedTweet, XApiError> {
                unimplemented!()
            }
            async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
                unimplemented!()
            }
            async fn get_tweet(&self, _: &str) -> Result<Tweet, XApiError> {
                unimplemented!()
            }
            async fn get_me(&self) -> Result<User, XApiError> {
                unimplemented!()
            }
            async fn get_user_tweets(
                &self,
                _: &str,
                _: u32,
                _: Option<&str>,
            ) -> Result<SearchResponse, XApiError> {
                unimplemented!()
            }
            async fn get_user_by_username(&self, _: &str) -> Result<User, XApiError> {
                unimplemented!()
            }
            async fn like_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(60),
                })
            }
        }

        let e = like_tweet(&FailClient, "u1", "t1").await.unwrap_err();
        assert!(matches!(
            e,
            ToolkitError::XApi(XApiError::RateLimited { .. })
        ));
    }
}
