//! Task 3.5 — content_loop comprehensive tests: safety guardrails + publisher.
//!
//! Covers:
//!  - Safety guardrail enforcement (daily tweet cap, thread cap)
//!  - publisher: try_post_scheduled happy path and dry-run
//!  - Dry-run vs live posting semantics
//!  - Single topic, no topics
//!  - Posted result captures correct topic and content

#[cfg(test)]
mod tests {
    use super::super::test_mocks::{make_topics, MockGenerator, MockSafety, MockStorage};
    use super::super::{ContentLoop, ContentResult};
    use crate::automation::loop_helpers::{ContentLoopError, ContentStorage};
    use std::sync::Arc;
    use std::sync::Mutex;

    // -------------------------------------------------------------------------
    // Safety guardrail: daily tweet limit (6/day)
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn safety_blocks_tweet_when_can_tweet_false() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet content".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: false,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            0,
            false,
        );
        let result = content.run_once(Some("Rust")).await;
        assert!(
            matches!(result, ContentResult::RateLimited),
            "expected RateLimited when can_tweet=false, got {result:?}"
        );
    }

    #[tokio::test]
    async fn safety_allows_tweet_when_thread_blocked() {
        // Thread blocked but tweet allowed — tweet path should succeed.
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet about Rust".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: false,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            0,
            false,
        );
        let result = content.run_once(Some("Rust")).await;
        assert!(
            matches!(result, ContentResult::Posted { .. }),
            "tweet should post even when thread is rate-limited"
        );
    }

    #[tokio::test]
    async fn safety_blocks_when_both_limits_hit() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: false,
                can_thread: false,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            0,
            false,
        );
        let result = content.run_once(None).await;
        assert!(matches!(result, ContentResult::RateLimited));
    }

    // -------------------------------------------------------------------------
    // Safety guardrail: no topics configured
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn no_topics_returns_no_topics_regardless_of_safety() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Vec::new(),
            0,
            false,
        );
        let result = content.run_once(None).await;
        assert!(matches!(result, ContentResult::NoTopics));
    }

    #[tokio::test]
    async fn single_topic_gets_used() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet about single topic".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            vec!["only_topic".to_string()],
            0,
            false,
        );
        let result = content.run_once(Some("only_topic")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1);
    }

    // -------------------------------------------------------------------------
    // Dry-run semantics
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn dry_run_logs_action_but_does_not_write_tweet() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "dry run tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            0,
            true,
        );
        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 0, "dry-run must not persist tweets");
        assert_eq!(storage.action_count(), 1, "dry-run must log action");
    }

    #[tokio::test]
    async fn live_run_writes_tweet_and_logs_action() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "live tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            0,
            false,
        );
        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1, "live run must persist tweet");
        assert_eq!(storage.action_count(), 1, "live run must log action");
    }

    // -------------------------------------------------------------------------
    // Publisher: try_post_scheduled
    // -------------------------------------------------------------------------

    struct ScheduledMockStorage {
        pub scheduled: Mutex<Option<(i64, String, String)>>,
        pub posted_scheduled: Mutex<Vec<(String, String)>>,
        pub marked_posted: Mutex<Vec<i64>>,
        pub actions: Mutex<Vec<(String, String, String)>>,
    }

    impl ScheduledMockStorage {
        fn with_item(id: i64, content_type: &str, content: &str) -> Arc<Self> {
            Arc::new(Self {
                scheduled: Mutex::new(Some((id, content_type.to_string(), content.to_string()))),
                posted_scheduled: Mutex::new(Vec::new()),
                marked_posted: Mutex::new(Vec::new()),
                actions: Mutex::new(Vec::new()),
            })
        }

        fn empty() -> Arc<Self> {
            Arc::new(Self {
                scheduled: Mutex::new(None),
                posted_scheduled: Mutex::new(Vec::new()),
                marked_posted: Mutex::new(Vec::new()),
                actions: Mutex::new(Vec::new()),
            })
        }
    }

    #[async_trait::async_trait]
    impl ContentStorage for ScheduledMockStorage {
        async fn last_tweet_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }
        async fn last_thread_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }
        async fn todays_tweet_times(
            &self,
        ) -> Result<Vec<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(Vec::new())
        }
        async fn post_tweet(&self, topic: &str, content: &str) -> Result<(), ContentLoopError> {
            self.posted_scheduled
                .lock()
                .expect("lock")
                .push((topic.to_string(), content.to_string()));
            Ok(())
        }
        async fn create_thread(&self, _t: &str, _c: usize) -> Result<String, ContentLoopError> {
            Ok("t1".to_string())
        }
        async fn update_thread_status(
            &self,
            _id: &str,
            _s: &str,
            _c: usize,
            _r: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn store_thread_tweet(
            &self,
            _id: &str,
            _pos: usize,
            _tid: &str,
            _c: &str,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn log_action(&self, a: &str, s: &str, m: &str) -> Result<(), ContentLoopError> {
            self.actions
                .lock()
                .expect("lock")
                .push((a.to_string(), s.to_string(), m.to_string()));
            Ok(())
        }
        async fn next_scheduled_item(
            &self,
        ) -> Result<Option<(i64, String, String)>, ContentLoopError> {
            Ok(self.scheduled.lock().expect("lock").clone())
        }
        async fn mark_scheduled_posted(
            &self,
            id: i64,
            _tid: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            self.marked_posted.lock().expect("lock").push(id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn try_post_scheduled_returns_none_when_queue_empty() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            ScheduledMockStorage::empty(),
            make_topics(),
            0,
            false,
        );
        let result = content.try_post_scheduled().await;
        assert!(result.is_none(), "empty queue → None");
    }

    #[tokio::test]
    async fn try_post_scheduled_posts_tweet_type() {
        let storage = ScheduledMockStorage::with_item(42, "tweet", "Scheduled tweet content");
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            0,
            false,
        );
        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), ContentResult::Posted { .. }));
        let posts = storage.posted_scheduled.lock().expect("lock");
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].1, "Scheduled tweet content");
    }

    #[tokio::test]
    async fn try_post_scheduled_dry_run_does_not_post() {
        let storage = ScheduledMockStorage::with_item(7, "tweet", "Dry scheduled tweet");
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            0,
            true,
        );
        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        let posts = storage.posted_scheduled.lock().expect("lock");
        assert_eq!(posts.len(), 0, "dry-run must not post scheduled tweet");
    }

    // -------------------------------------------------------------------------
    // Publisher: scheduled thread posting
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn try_post_scheduled_thread_posts_via_thread_poster() {
        use crate::automation::thread_loop::test_mocks::MockPoster;
        let tweets = serde_json::to_string(&vec!["Tweet 1", "Tweet 2", "Tweet 3"]).unwrap();
        let storage = ScheduledMockStorage::with_item(99, "thread", &tweets);
        let poster = Arc::new(MockPoster::new());

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            0,
            false,
        )
        .with_thread_poster(poster.clone());

        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), ContentResult::Posted { .. }));
        assert_eq!(poster.posted_count(), 3, "all 3 tweets should be posted");
        let marked = storage.marked_posted.lock().expect("lock");
        assert_eq!(marked.len(), 1);
        assert_eq!(marked[0], 99);
    }

    #[tokio::test]
    async fn try_post_scheduled_thread_fails_without_poster() {
        let tweets = serde_json::to_string(&vec!["Tweet 1", "Tweet 2"]).unwrap();
        let storage = ScheduledMockStorage::with_item(10, "thread", &tweets);

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            0,
            false,
        );
        // No .with_thread_poster() → should fail

        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        match result.unwrap() {
            ContentResult::Failed { error } => {
                assert!(
                    error.contains("No thread poster"),
                    "expected 'No thread poster' error, got: {error}"
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn try_post_scheduled_thread_fails_on_unparseable_content() {
        use crate::automation::thread_loop::test_mocks::MockPoster;
        let storage = ScheduledMockStorage::with_item(11, "thread", "not json at all");
        let poster = Arc::new(MockPoster::new());

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            0,
            false,
        )
        .with_thread_poster(poster);

        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        match result.unwrap() {
            ContentResult::Failed { error } => {
                assert!(
                    error.contains("Cannot parse"),
                    "expected parse error, got: {error}"
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn try_post_scheduled_thread_fails_on_empty_tweets() {
        use crate::automation::thread_loop::test_mocks::MockPoster;
        let tweets: Vec<String> = vec![];
        let json = serde_json::to_string(&tweets).unwrap();
        let storage = ScheduledMockStorage::with_item(12, "thread", &json);
        let poster = Arc::new(MockPoster::new());

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            0,
            false,
        )
        .with_thread_poster(poster);

        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        match result.unwrap() {
            ContentResult::Failed { error } => {
                assert!(
                    error.contains("no tweets"),
                    "expected 'no tweets' error, got: {error}"
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn try_post_scheduled_thread_partial_failure() {
        use crate::automation::thread_loop::test_mocks::MockPoster;
        let tweets = serde_json::to_string(&vec!["Tweet 1", "Tweet 2", "Tweet 3"]).unwrap();
        let storage = ScheduledMockStorage::with_item(13, "thread", &tweets);
        let poster = Arc::new(MockPoster::failing_at(1)); // fail on second tweet

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            0,
            false,
        )
        .with_thread_poster(poster);

        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        match result.unwrap() {
            ContentResult::Failed { error } => {
                assert!(error.contains("Thread failed at tweet 2/3"), "got: {error}");
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    // ScheduledMockStorage variant that returns an error from next_scheduled_item
    struct FailingScheduledStorage;

    #[async_trait::async_trait]
    impl ContentStorage for FailingScheduledStorage {
        async fn last_tweet_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }
        async fn last_thread_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }
        async fn todays_tweet_times(
            &self,
        ) -> Result<Vec<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(Vec::new())
        }
        async fn post_tweet(&self, _: &str, _: &str) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn create_thread(&self, _: &str, _: usize) -> Result<String, ContentLoopError> {
            Ok("t1".to_string())
        }
        async fn update_thread_status(
            &self,
            _: &str,
            _: &str,
            _: usize,
            _: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn store_thread_tweet(
            &self,
            _: &str,
            _: usize,
            _: &str,
            _: &str,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn log_action(&self, _: &str, _: &str, _: &str) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn next_scheduled_item(
            &self,
        ) -> Result<Option<(i64, String, String)>, ContentLoopError> {
            Err(ContentLoopError::StorageError("db unavailable".to_string()))
        }
        async fn mark_scheduled_posted(
            &self,
            _: i64,
            _: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn try_post_scheduled_returns_none_on_storage_error() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(FailingScheduledStorage),
            make_topics(),
            0,
            false,
        );
        let result = content.try_post_scheduled().await;
        assert!(result.is_none(), "storage error → None (graceful fallback)");
    }

    // ScheduledMockStorage variant where post_tweet fails
    struct FailingPostScheduledStorage {
        scheduled: Mutex<Option<(i64, String, String)>>,
    }

    impl FailingPostScheduledStorage {
        fn with_item(id: i64, content: &str) -> Arc<Self> {
            Arc::new(Self {
                scheduled: Mutex::new(Some((id, "tweet".to_string(), content.to_string()))),
            })
        }
    }

    #[async_trait::async_trait]
    impl ContentStorage for FailingPostScheduledStorage {
        async fn last_tweet_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }
        async fn last_thread_time(
            &self,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(None)
        }
        async fn todays_tweet_times(
            &self,
        ) -> Result<Vec<chrono::DateTime<chrono::Utc>>, ContentLoopError> {
            Ok(Vec::new())
        }
        async fn post_tweet(&self, _: &str, _: &str) -> Result<(), ContentLoopError> {
            Err(ContentLoopError::PostFailed(
                "X API unavailable".to_string(),
            ))
        }
        async fn create_thread(&self, _: &str, _: usize) -> Result<String, ContentLoopError> {
            Ok("t1".to_string())
        }
        async fn update_thread_status(
            &self,
            _: &str,
            _: &str,
            _: usize,
            _: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn store_thread_tweet(
            &self,
            _: &str,
            _: usize,
            _: &str,
            _: &str,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn log_action(&self, _: &str, _: &str, _: &str) -> Result<(), ContentLoopError> {
            Ok(())
        }
        async fn next_scheduled_item(
            &self,
        ) -> Result<Option<(i64, String, String)>, ContentLoopError> {
            Ok(self.scheduled.lock().expect("lock").clone())
        }
        async fn mark_scheduled_posted(
            &self,
            _: i64,
            _: Option<&str>,
        ) -> Result<(), ContentLoopError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn try_post_scheduled_tweet_failure_returns_failed() {
        let storage = FailingPostScheduledStorage::with_item(20, "My scheduled tweet");
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "unused".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            0,
            false,
        );
        let result = content.try_post_scheduled().await;
        assert!(result.is_some());
        match result.unwrap() {
            ContentResult::Failed { error } => {
                assert!(
                    error.contains("Scheduled post failed"),
                    "expected 'Scheduled post failed', got: {error}"
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // Scheduler: run_slot_iteration
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn run_slot_iteration_posts_when_no_scheduled_and_safety_allows() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Slot tweet!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = content.run_slot_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1);
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn run_slot_iteration_rate_limited_when_safety_blocks() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: false,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = content.run_slot_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::RateLimited));
    }

    #[tokio::test]
    async fn run_slot_iteration_prioritizes_scheduled_content() {
        let storage = ScheduledMockStorage::with_item(50, "tweet", "Scheduled first!");
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "should not be used".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = content.run_slot_iteration(&mut recent, 3, &mut rng).await;
        match result {
            ContentResult::Posted { topic, content } => {
                assert!(topic.contains("scheduled:50"));
                assert_eq!(content, "Scheduled first!");
            }
            other => panic!("expected Posted with scheduled content, got {other:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // Scheduler: run_iteration with scheduled content priority
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn run_iteration_checks_scheduled_before_elapsed() {
        let storage = ScheduledMockStorage::with_item(60, "tweet", "Iteration scheduled!");
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "should not be used".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        match result {
            ContentResult::Posted { topic, .. } => {
                assert!(
                    topic.contains("scheduled:60"),
                    "scheduled content should take priority"
                );
            }
            other => panic!("expected Posted with scheduled content, got {other:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // Posted result captures correct topic + content
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn posted_result_contains_topic_and_content() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Great content about testing".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            vec!["testing".to_string()],
            0,
            false,
        );
        let result = content.run_once(Some("testing")).await;
        match result {
            ContentResult::Posted { topic, content } => {
                assert_eq!(topic, "testing");
                assert_eq!(content, "Great content about testing");
            }
            other => panic!("expected Posted, got {other:?}"),
        }
    }
}
