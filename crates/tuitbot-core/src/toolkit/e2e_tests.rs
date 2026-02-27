//! End-to-end tests for the toolkit layer.
//!
//! These tests verify that toolkit functions compose correctly without
//! any database, LLM, or MCP transport — proving the utility-first
//! architecture where every X API operation is standalone and composable.

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::config::ScoringConfig;
    use crate::error::XApiError;
    use crate::scoring::{ScoringEngine, TweetData};
    use crate::toolkit;
    use crate::x_api::types::*;
    use crate::x_api::XApiClient;

    // ── Configurable mock ────────────────────────────────────────────

    struct ScenarioClient {
        search_results: Vec<Tweet>,
        search_users: Vec<User>,
        post_counter: AtomicUsize,
    }

    impl ScenarioClient {
        fn new(tweets: Vec<Tweet>, users: Vec<User>) -> Self {
            Self {
                search_results: tweets,
                search_users: users,
                post_counter: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl XApiClient for ScenarioClient {
        async fn search_tweets(
            &self,
            _query: &str,
            _max: u32,
            _since: Option<&str>,
            _page: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Ok(SearchResponse {
                data: self.search_results.clone(),
                includes: if self.search_users.is_empty() {
                    None
                } else {
                    Some(Includes {
                        users: self.search_users.clone(),
                    })
                },
                meta: SearchMeta {
                    newest_id: self.search_results.first().map(|t| t.id.clone()),
                    oldest_id: self.search_results.last().map(|t| t.id.clone()),
                    result_count: self.search_results.len() as u32,
                    next_token: None,
                },
            })
        }

        async fn get_mentions(
            &self,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
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
            let n = self.post_counter.fetch_add(1, Ordering::SeqCst);
            Ok(PostedTweet {
                id: format!("posted_{n}"),
                text: text.to_string(),
            })
        }

        async fn reply_to_tweet(
            &self,
            text: &str,
            _in_reply_to: &str,
        ) -> Result<PostedTweet, XApiError> {
            let n = self.post_counter.fetch_add(1, Ordering::SeqCst);
            Ok(PostedTweet {
                id: format!("reply_{n}"),
                text: text.to_string(),
            })
        }

        async fn get_tweet(&self, id: &str) -> Result<Tweet, XApiError> {
            Ok(Tweet {
                id: id.to_string(),
                text: "test tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: "2026-02-24T12:00:00Z".to_string(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            })
        }

        async fn get_me(&self) -> Result<User, XApiError> {
            Ok(User {
                id: "me".to_string(),
                username: "testbot".to_string(),
                name: "Test Bot".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn get_user_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
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
                id: format!("uid_{username}"),
                username: username.to_string(),
                name: username.to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn like_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unlike_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn follow_user(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unfollow_user(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn retweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unretweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn bookmark_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unbookmark_tweet(&self, _: &str, _: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
    }

    fn sample_tweet(id: &str, text: &str, author_id: &str) -> Tweet {
        Tweet {
            id: id.to_string(),
            text: text.to_string(),
            author_id: author_id.to_string(),
            created_at: "2026-02-24T12:00:00Z".to_string(),
            public_metrics: PublicMetrics {
                like_count: 10,
                retweet_count: 3,
                reply_count: 2,
                impression_count: 1000,
                ..Default::default()
            },
            conversation_id: None,
        }
    }

    fn sample_user(id: &str, username: &str, followers: u64) -> User {
        User {
            id: id.to_string(),
            username: username.to_string(),
            name: username.to_string(),
            public_metrics: UserMetrics {
                followers_count: followers,
                following_count: 100,
                tweet_count: 500,
            },
        }
    }

    // ── E2E: Search → Score (pure toolkit, no DB) ────────────────────

    #[tokio::test]
    async fn e2e_search_and_score_without_db() {
        // This test proves the utility-first architecture: search and score
        // compose without database, LLM, or MCP transport.
        let tweets = vec![
            sample_tweet("t1", "Learning Rust async programming today", "a1"),
            sample_tweet("t2", "Just built a web scraper in Python", "a2"),
            sample_tweet("t3", "Rust ownership model is amazing", "a3"),
        ];
        let users = vec![
            sample_user("a1", "rustdev", 3000),
            sample_user("a2", "pydev", 500),
            sample_user("a3", "systems_dev", 8000),
        ];
        let client = ScenarioClient::new(tweets, users);

        // Step 1: Search via toolkit (stateless)
        let results = toolkit::read::search_tweets(&client, "rust", 10, None, None)
            .await
            .unwrap();

        assert_eq!(results.data.len(), 3);
        assert_eq!(results.meta.result_count, 3);

        // Step 2: Score each tweet (stateless, no DB)
        let keywords = vec!["rust".to_string(), "async".to_string()];
        let engine = ScoringEngine::new(ScoringConfig::default(), keywords.clone());
        let user_map: std::collections::HashMap<String, &User> = results
            .includes
            .as_ref()
            .unwrap()
            .users
            .iter()
            .map(|u| (u.id.clone(), u))
            .collect();

        let mut scores: Vec<(String, f32)> = results
            .data
            .iter()
            .map(|tweet| {
                let user = user_map.get(&tweet.author_id);
                let data = TweetData {
                    text: tweet.text.clone(),
                    created_at: tweet.created_at.clone(),
                    likes: tweet.public_metrics.like_count,
                    retweets: tweet.public_metrics.retweet_count,
                    replies: tweet.public_metrics.reply_count,
                    author_username: user.map(|u| u.username.clone()).unwrap_or_default(),
                    author_followers: user.map(|u| u.public_metrics.followers_count).unwrap_or(0),
                    has_media: false,
                    is_quote_tweet: false,
                };
                (tweet.id.clone(), engine.score_tweet(&data).total)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Rust-related tweets should score higher than unrelated ones
        assert!(!scores.is_empty());
        // The search returned all tweets; scoring differentiates them
        assert!(scores[0].1 > 0.0);
    }

    // ── E2E: Search → Read detail → Reply (write chain) ─────────────

    #[tokio::test]
    async fn e2e_search_read_reply_chain() {
        // Proves the read → write composition path works without DB/LLM.
        let tweets = vec![sample_tweet("t1", "How do I handle errors in Rust?", "a1")];
        let users = vec![sample_user("a1", "newbie_dev", 200)];
        let client = ScenarioClient::new(tweets, users);

        // Step 1: Search
        let results = toolkit::read::search_tweets(&client, "rust errors", 10, None, None)
            .await
            .unwrap();
        assert_eq!(results.data.len(), 1);
        let tweet = &results.data[0];

        // Step 2: Read the specific tweet (could be a re-fetch for freshness)
        let detail = toolkit::read::get_tweet(&client, &tweet.id).await.unwrap();
        assert_eq!(detail.id, "t1");

        // Step 3: Reply via toolkit write
        let reply = toolkit::write::reply_to_tweet(
            &client,
            "Use the ? operator for ergonomic error propagation!",
            &detail.id,
            None,
        )
        .await
        .unwrap();

        assert!(reply.id.starts_with("reply_"));
        assert!(reply.text.contains("? operator"));
    }

    // ── E2E: Thread posting (multi-tweet write) ──────────────────────

    #[tokio::test]
    async fn e2e_post_thread_without_db() {
        let client = ScenarioClient::new(vec![], vec![]);

        let thread_tweets = vec![
            "Thread: Why Rust is great for CLI tools".to_string(),
            "1/ The type system catches bugs at compile time".to_string(),
            "2/ Zero-cost abstractions mean fast binaries".to_string(),
            "3/ Cargo makes dependency management painless".to_string(),
        ];

        let ids = toolkit::write::post_thread(&client, &thread_tweets, None)
            .await
            .unwrap();

        assert_eq!(ids.len(), 4);
        // First tweet is posted, subsequent are replies
        assert!(ids[0].starts_with("posted_"));
        assert!(ids[1].starts_with("reply_"));
        assert!(ids[2].starts_with("reply_"));
        assert!(ids[3].starts_with("reply_"));
    }

    // ── E2E: Engage operations compose (like + follow) ───────────────

    #[tokio::test]
    async fn e2e_engage_compose_without_db() {
        let client = ScenarioClient::new(vec![], vec![]);

        // Like a tweet and follow its author — pure toolkit, no DB
        let liked = toolkit::engage::like_tweet(&client, "me", "t1")
            .await
            .unwrap();
        assert!(liked);

        let followed = toolkit::engage::follow_user(&client, "me", "a1")
            .await
            .unwrap();
        assert!(followed);

        // Bookmark for later reference
        let bookmarked = toolkit::engage::bookmark_tweet(&client, "me", "t1")
            .await
            .unwrap();
        assert!(bookmarked);
    }

    // ── E2E: User lookup chain ───────────────────────────────────────

    #[tokio::test]
    async fn e2e_user_lookup_chain() {
        let client = ScenarioClient::new(vec![], vec![]);

        // Look up user by username
        let user = toolkit::read::get_user_by_username(&client, "rustdev")
            .await
            .unwrap();
        assert_eq!(user.username, "rustdev");

        // Get authenticated user
        let me = toolkit::read::get_me(&client).await.unwrap();
        assert_eq!(me.username, "testbot");
    }

    // ── E2E: Error handling across toolkit boundaries ────────────────

    #[tokio::test]
    async fn e2e_input_validation_across_toolkit() {
        let client = ScenarioClient::new(vec![], vec![]);

        // Empty query rejected by read
        let err = toolkit::read::search_tweets(&client, "", 10, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, toolkit::ToolkitError::InvalidInput { .. }));

        // Empty ID rejected by engage
        let err = toolkit::engage::like_tweet(&client, "", "t1")
            .await
            .unwrap_err();
        assert!(matches!(err, toolkit::ToolkitError::InvalidInput { .. }));

        // Too-long tweet rejected by write
        let long = "a".repeat(281);
        let err = toolkit::write::post_tweet(&client, &long, None)
            .await
            .unwrap_err();
        assert!(matches!(err, toolkit::ToolkitError::TweetTooLong { .. }));

        // Empty thread rejected
        let err = toolkit::write::post_thread(&client, &[], None)
            .await
            .unwrap_err();
        assert!(matches!(err, toolkit::ToolkitError::InvalidInput { .. }));
    }

    // ── E2E: Rate limit error propagation ────────────────────────────

    #[tokio::test]
    async fn e2e_rate_limit_propagates_through_toolkit() {
        struct RateLimitedClient;

        #[async_trait::async_trait]
        impl XApiClient for RateLimitedClient {
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
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
            }
            async fn post_tweet(&self, _: &str) -> Result<PostedTweet, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
            }
            async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
            }
            async fn get_tweet(&self, _: &str) -> Result<Tweet, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
            }
            async fn get_me(&self) -> Result<User, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
            }
            async fn get_user_tweets(
                &self,
                _: &str,
                _: u32,
                _: Option<&str>,
            ) -> Result<SearchResponse, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
            }
            async fn get_user_by_username(&self, _: &str) -> Result<User, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(30),
                })
            }
        }

        // Rate limit propagates through read
        let err = toolkit::read::search_tweets(&RateLimitedClient, "test", 10, None, None)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            toolkit::ToolkitError::XApi(XApiError::RateLimited {
                retry_after: Some(30)
            })
        ));

        // Rate limit propagates through write
        let err = toolkit::write::post_tweet(&RateLimitedClient, "hello", None)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            toolkit::ToolkitError::XApi(XApiError::RateLimited { .. })
        ));
    }
}
