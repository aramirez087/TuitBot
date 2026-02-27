//! Stateless write operations over `&dyn XApiClient`.
//!
//! Raw posting, replying, quoting, deleting, and thread operations.
//! No policy enforcement, no audit logging, no mutation recording.
//! Those concerns belong in the workflow layer (AD-04).

use super::ToolkitError;
use crate::x_api::types::PostedTweet;
use crate::x_api::XApiClient;

/// Post a new tweet, optionally with media.
pub async fn post_tweet(
    client: &dyn XApiClient,
    text: &str,
    media_ids: Option<&[String]>,
) -> Result<PostedTweet, ToolkitError> {
    super::validate_tweet_length(text)?;
    match media_ids {
        Some(ids) if !ids.is_empty() => Ok(client.post_tweet_with_media(text, ids).await?),
        _ => Ok(client.post_tweet(text).await?),
    }
}

/// Reply to an existing tweet, optionally with media.
pub async fn reply_to_tweet(
    client: &dyn XApiClient,
    text: &str,
    in_reply_to_id: &str,
    media_ids: Option<&[String]>,
) -> Result<PostedTweet, ToolkitError> {
    super::validate_tweet_length(text)?;
    super::validate_id(in_reply_to_id, "in_reply_to_id")?;
    match media_ids {
        Some(ids) if !ids.is_empty() => Ok(client
            .reply_to_tweet_with_media(text, in_reply_to_id, ids)
            .await?),
        _ => Ok(client.reply_to_tweet(text, in_reply_to_id).await?),
    }
}

/// Post a quote tweet referencing another tweet.
pub async fn quote_tweet(
    client: &dyn XApiClient,
    text: &str,
    quoted_tweet_id: &str,
) -> Result<PostedTweet, ToolkitError> {
    super::validate_tweet_length(text)?;
    super::validate_id(quoted_tweet_id, "quoted_tweet_id")?;
    Ok(client.quote_tweet(text, quoted_tweet_id).await?)
}

/// Delete a tweet by ID.
pub async fn delete_tweet(client: &dyn XApiClient, tweet_id: &str) -> Result<bool, ToolkitError> {
    super::validate_id(tweet_id, "tweet_id")?;
    Ok(client.delete_tweet(tweet_id).await?)
}

/// Post a thread (ordered sequence of tweets).
///
/// Validates all tweet lengths up front. Chains replies sequentially.
/// On partial failure, returns `ToolkitError::ThreadPartialFailure`
/// with the IDs of successfully posted tweets.
pub async fn post_thread(
    client: &dyn XApiClient,
    tweets: &[String],
    media_ids: Option<&[Vec<String>]>,
) -> Result<Vec<String>, ToolkitError> {
    if tweets.is_empty() {
        return Err(ToolkitError::InvalidInput {
            message: "thread must contain at least one tweet".into(),
        });
    }

    for (i, text) in tweets.iter().enumerate() {
        super::validate_tweet_length(text).map_err(|_| ToolkitError::InvalidInput {
            message: format!(
                "tweet {i} too long: {} characters (max {})",
                text.len(),
                super::MAX_TWEET_LENGTH
            ),
        })?;
    }

    let total = tweets.len();
    let mut posted_ids: Vec<String> = Vec::with_capacity(total);

    for (i, text) in tweets.iter().enumerate() {
        let tweet_media = media_ids.and_then(|m| m.get(i)).map(|v| v.as_slice());

        let result = if i == 0 {
            match tweet_media {
                Some(ids) if !ids.is_empty() => client.post_tweet_with_media(text, ids).await,
                _ => client.post_tweet(text).await,
            }
        } else {
            let prev = &posted_ids[i - 1];
            match tweet_media {
                Some(ids) if !ids.is_empty() => {
                    client.reply_to_tweet_with_media(text, prev, ids).await
                }
                _ => client.reply_to_tweet(text, prev).await,
            }
        };

        match result {
            Ok(posted) => posted_ids.push(posted.id),
            Err(e) => {
                let posted = posted_ids.len();
                return Err(ToolkitError::ThreadPartialFailure {
                    posted_ids,
                    failed_index: i,
                    posted,
                    total,
                    source: Box::new(e),
                });
            }
        }
    }

    Ok(posted_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::XApiError;
    use crate::x_api::types::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
        async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "t1".into(),
                text: text.into(),
            })
        }
        async fn reply_to_tweet(&self, text: &str, _: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "t2".into(),
                text: text.into(),
            })
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
    }

    #[tokio::test]
    async fn post_tweet_success() {
        let r = post_tweet(&MockClient, "Hello", None).await.unwrap();
        assert_eq!(r.id, "t1");
    }

    #[tokio::test]
    async fn post_tweet_too_long() {
        let text = "a".repeat(281);
        let e = post_tweet(&MockClient, &text, None).await.unwrap_err();
        assert!(matches!(e, ToolkitError::TweetTooLong { .. }));
    }

    #[tokio::test]
    async fn reply_to_tweet_success() {
        let r = reply_to_tweet(&MockClient, "Nice", "t0", None)
            .await
            .unwrap();
        assert_eq!(r.id, "t2");
    }

    #[tokio::test]
    async fn reply_to_tweet_empty_id() {
        let e = reply_to_tweet(&MockClient, "Hi", "", None)
            .await
            .unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn delete_tweet_empty_id() {
        let e = delete_tweet(&MockClient, "").await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn post_thread_success() {
        let tweets = vec!["First".into(), "Second".into()];
        let ids = post_thread(&MockClient, &tweets, None).await.unwrap();
        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn post_thread_empty() {
        let e = post_thread(&MockClient, &[], None).await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn post_thread_tweet_too_long() {
        let tweets = vec!["ok".into(), "a".repeat(281)];
        let e = post_thread(&MockClient, &tweets, None).await.unwrap_err();
        assert!(matches!(e, ToolkitError::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn post_thread_partial_failure() {
        struct PartialClient {
            calls: AtomicUsize,
        }
        #[async_trait::async_trait]
        impl XApiClient for PartialClient {
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
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(PostedTweet {
                    id: format!("t{n}"),
                    text: "ok".into(),
                })
            }
            async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
                Err(XApiError::ApiError {
                    status: 500,
                    message: "fail".into(),
                })
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
        }

        let client = PartialClient {
            calls: AtomicUsize::new(0),
        };
        let tweets = vec!["First".into(), "Second".into(), "Third".into()];
        let e = post_thread(&client, &tweets, None).await.unwrap_err();
        match e {
            ToolkitError::ThreadPartialFailure {
                posted_ids,
                failed_index,
                posted,
                total,
                ..
            } => {
                assert_eq!(posted_ids, vec!["t0"]);
                assert_eq!(failed_index, 1);
                assert_eq!(posted, 1);
                assert_eq!(total, 3);
            }
            other => panic!("expected ThreadPartialFailure, got {other:?}"),
        }
    }
}
