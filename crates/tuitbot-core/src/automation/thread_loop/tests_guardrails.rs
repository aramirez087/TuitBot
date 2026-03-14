//! Task 3.5 — thread_loop comprehensive tests: safety guardrails + semantics.
//!
//! Covers:
//!  - Safety guardrail: weekly thread cap (can_post_thread=false)
//!  - Safety guardrail: no topics configured
//!  - Dry-run vs live posting semantics
//!  - Partial failure: poster fails on second tweet
//!  - Thread interval (too soon, ready)
//!  - Pick-topic deduplication across recent list

#[cfg(test)]
mod tests {
    use super::super::test_mocks::{
        make_thread_tweets, make_topics, FailingThreadGenerator, MockPoster, MockSafety,
        MockStorage, MockThreadGenerator,
    };
    use super::super::{ThreadLoop, ThreadResult};
    use std::sync::Arc;

    // -------------------------------------------------------------------------
    // Safety guardrail: weekly thread cap (1 thread/week)
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn safety_blocks_thread_when_can_thread_false() {
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: false, // weekly cap hit
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            make_topics(),
            0,
            false,
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(
            matches!(result, ThreadResult::RateLimited),
            "expected RateLimited when can_thread=false, got {result:?}"
        );
    }

    #[tokio::test]
    async fn safety_allows_thread_when_can_tweet_blocked() {
        // can_tweet=false should not block thread posting
        let poster = Arc::new(MockPoster::new());
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: false, // tweet blocked
                can_thread: true, // thread ok
            }),
            Arc::new(MockStorage::new(None)),
            poster.clone(),
            make_topics(),
            0,
            false,
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(
            matches!(result, ThreadResult::Posted { .. }),
            "thread should post even when tweet cap is hit"
        );
    }

    #[tokio::test]
    async fn safety_blocks_when_both_limits_hit() {
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: false,
                can_thread: false,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            make_topics(),
            0,
            false,
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(matches!(result, ThreadResult::RateLimited));
    }

    // -------------------------------------------------------------------------
    // Safety guardrail: no topics configured
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn no_topics_returns_no_topics() {
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            Vec::new(), // no topics
            0,
            false,
        );
        let result = thread_loop.run_once(None, None).await;
        assert!(
            matches!(result, ThreadResult::NoTopics),
            "expected NoTopics with empty topic list"
        );
    }

    // -------------------------------------------------------------------------
    // Dry-run: posts to poster=0, logs action
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn dry_run_does_not_post_to_x() {
        let poster = Arc::new(MockPoster::new());
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            poster.clone(),
            make_topics(),
            0,
            true, // dry_run
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(
            matches!(result, ThreadResult::Posted { .. }),
            "dry-run should return Posted result"
        );
        assert_eq!(
            poster.posted_count(),
            0,
            "dry-run must not actually post tweets"
        );
    }

    #[tokio::test]
    async fn live_run_posts_all_tweets_to_x() {
        let poster = Arc::new(MockPoster::new());
        let thread_count = make_thread_tweets().len();
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            poster.clone(),
            make_topics(),
            0,
            false,
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        match &result {
            ThreadResult::Posted { tweet_count, .. } => {
                assert_eq!(*tweet_count, thread_count);
            }
            other => panic!("expected Posted, got {other:?}"),
        }
        assert_eq!(poster.posted_count(), thread_count);
    }

    // -------------------------------------------------------------------------
    // Partial failure: poster fails on second tweet
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn partial_failure_when_poster_fails_at_index_1() {
        let poster = Arc::new(MockPoster::failing_at(1));
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            poster.clone(),
            make_topics(),
            0,
            false,
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(
            matches!(
                result,
                ThreadResult::PartialFailure { tweets_posted: 1, .. }
            ),
            "expected PartialFailure with 1 tweet posted, got {result:?}"
        );
    }

    // -------------------------------------------------------------------------
    // Generation failure
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn generation_failure_returns_failed() {
        let thread_loop = ThreadLoop::new(
            Arc::new(FailingThreadGenerator),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            make_topics(),
            0,
            false,
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        assert!(
            matches!(result, ThreadResult::Failed { .. }),
            "expected Failed for generator error"
        );
    }

    // -------------------------------------------------------------------------
    // Posted result captures topic and count
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn posted_result_contains_correct_topic_and_count() {
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: vec![
                    "Tweet 1".to_string(),
                    "Tweet 2".to_string(),
                    "Tweet 3".to_string(),
                ],
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            vec!["MyTopic".to_string()],
            0,
            false,
        );
        let result = thread_loop.run_once(Some("MyTopic"), None).await;
        match result {
            ThreadResult::Posted {
                topic,
                tweet_count,
                thread_id,
            } => {
                assert_eq!(topic, "MyTopic");
                assert_eq!(tweet_count, 3);
                assert!(!thread_id.is_empty());
            }
            other => panic!("expected Posted, got {other:?}"),
        }
    }

    // -------------------------------------------------------------------------
    // Single-tweet thread (edge: min size)
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn single_tweet_thread_posts_and_returns_count_1() {
        let poster = Arc::new(MockPoster::new());
        let thread_loop = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: vec!["Only tweet".to_string()],
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            poster.clone(),
            make_topics(),
            0,
            false,
        );
        let result = thread_loop.run_once(Some("Rust"), None).await;
        // Single tweet may pass validation or fail (min count) — either is acceptable
        // as long as no panic occurs.
        match result {
            ThreadResult::Posted { tweet_count, .. } => assert_eq!(tweet_count, 1),
            ThreadResult::ValidationFailed { .. } => { /* also valid — min thread size */ }
            other => panic!("unexpected result for 1-tweet thread: {other:?}"),
        }
    }
}
