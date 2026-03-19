#[cfg(test)]
mod tests_dispatch {
    use super::super::*;
    use super::super::dispatch::{is_rate_limit_error, randomized_delay};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::sync::oneshot;
    use tokio_util::sync::CancellationToken;

    /// Mock approval queue that records all queued actions.
    struct MockApprovalQueue {
        items: Mutex<Vec<(String, String)>>,
    }

    impl MockApprovalQueue {
        fn new() -> Self {
            Self {
                items: Mutex::new(Vec::new()),
            }
        }

        fn item_count(&self) -> usize {
            self.items.lock().expect("lock poisoned").len()
        }
    }

    #[async_trait::async_trait]
    impl ApprovalQueue for MockApprovalQueue {
        async fn queue_reply(
            &self,
            tweet_id: &str,
            content: &str,
            _media_paths: &[String],
        ) -> Result<i64, String> {
            self.items
                .lock()
                .expect("lock poisoned")
                .push(("reply".to_string(), format!("{tweet_id}:{content}")));
            Ok(42)
        }

        async fn queue_tweet(&self, content: &str, _media_paths: &[String]) -> Result<i64, String> {
            self.items
                .lock()
                .expect("lock poisoned")
                .push(("tweet".to_string(), content.to_string()));
            Ok(43)
        }
    }

    /// Mock executor for approval mode tests.
    struct MockExecutor {
        calls: Mutex<Vec<(String, String)>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.lock().expect("lock poisoned").len()
        }
    }

    #[async_trait::async_trait]
    impl PostExecutor for MockExecutor {
        async fn execute_reply(
            &self,
            tweet_id: &str,
            content: &str,
            _media_ids: &[String],
        ) -> Result<String, String> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push(("reply".to_string(), format!("{tweet_id}:{content}")));
            Ok("reply-id-123".to_string())
        }

        async fn execute_tweet(
            &self,
            content: &str,
            _media_ids: &[String],
        ) -> Result<String, String> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push(("tweet".to_string(), content.to_string()));
            Ok("tweet-id-456".to_string())
        }
    }

    #[tokio::test]
    async fn approval_mode_queues_instead_of_posting() {
        let executor = Arc::new(MockExecutor::new());
        let approval = Arc::new(MockApprovalQueue::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let approval_clone = approval.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue_with_approval(
                rx,
                exec_clone,
                Some(approval_clone),
                Duration::ZERO,
                Duration::ZERO,
                None,
                cancel_clone,
            )
            .await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::Reply {
            tweet_id: "t123".to_string(),
            content: "needs approval".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result: Result<String, String> = result_rx.await.expect("recv");
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("queued:"));

        assert_eq!(executor.call_count(), 0);
        assert_eq!(approval.item_count(), 1);

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn approval_mode_queues_tweets() {
        let executor = Arc::new(MockExecutor::new());
        let approval = Arc::new(MockApprovalQueue::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let approval_clone = approval.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue_with_approval(
                rx,
                exec_clone,
                Some(approval_clone),
                Duration::ZERO,
                Duration::ZERO,
                None,
                cancel_clone,
            )
            .await;
        });

        tx.send(PostAction::Tweet {
            content: "my tweet".to_string(),
            media_ids: vec![],
            result_tx: None,
        })
        .await
        .expect("send");

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(executor.call_count(), 0);
        assert_eq!(approval.item_count(), 1);

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn approval_mode_queues_thread_tweets() {
        let executor = Arc::new(MockExecutor::new());
        let approval = Arc::new(MockApprovalQueue::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let approval_clone = approval.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue_with_approval(
                rx,
                exec_clone,
                Some(approval_clone),
                Duration::ZERO,
                Duration::ZERO,
                None,
                cancel_clone,
            )
            .await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::ThreadTweet {
            content: "thread part 2".to_string(),
            in_reply_to: "prev-id".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result: Result<String, String> = result_rx.await.expect("recv");
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("queued:"));

        assert_eq!(executor.call_count(), 0);
        assert_eq!(approval.item_count(), 1);

        cancel.cancel();
        handle.await.expect("join");
    }

    #[test]
    fn is_rate_limit_error_detects_rate_limit() {
        assert!(is_rate_limit_error("rate limit"));
        assert!(is_rate_limit_error("rate limit exceeded"));
        assert!(is_rate_limit_error("too many requests"));
        assert!(is_rate_limit_error("HTTP 429"));
        assert!(is_rate_limit_error("429"));
        assert!(is_rate_limit_error("forbidden"));
        assert!(is_rate_limit_error("403"));
    }

    #[test]
    fn is_rate_limit_error_case_insensitive() {
        assert!(is_rate_limit_error("RATE LIMIT"));
        assert!(is_rate_limit_error("Rate Limit Exceeded"));
        assert!(is_rate_limit_error("TOO MANY REQUESTS"));
        assert!(is_rate_limit_error("FORBIDDEN"));
    }

    #[test]
    fn is_rate_limit_error_partial_match() {
        assert!(is_rate_limit_error(
            "error: rate limit exceeded for endpoint"
        ));
        assert!(is_rate_limit_error("HTTP 429 Too Many Requests from API"));
        assert!(is_rate_limit_error("403 Forbidden: write scope missing"));
    }

    #[test]
    fn is_rate_limit_error_unrelated() {
        assert!(!is_rate_limit_error("success"));
        assert!(!is_rate_limit_error(""));
        assert!(!is_rate_limit_error("network timeout"));
        assert!(!is_rate_limit_error("invalid json"));
    }

    #[test]
    fn randomized_delay_zero_min_nonzero_max() {
        let min = Duration::ZERO;
        let max = Duration::from_millis(100);
        let d = randomized_delay(min, max);
        assert!(d <= max);
    }

    #[test]
    fn queue_capacity_is_100() {
        assert_eq!(QUEUE_CAPACITY, 100);
    }

    #[test]
    fn is_rate_limit_error_empty_string() {
        assert!(!is_rate_limit_error(""));
    }

    #[test]
    fn is_rate_limit_error_whitespace_only() {
        assert!(!is_rate_limit_error("   "));
    }

    #[test]
    fn is_rate_limit_error_mixed_case_429() {
        assert!(is_rate_limit_error("ERROR 429 service busy"));
    }

    #[test]
    fn is_rate_limit_error_mixed_case_403() {
        assert!(is_rate_limit_error("access forbidden by policy"));
    }

    #[test]
    fn is_rate_limit_error_long_message() {
        let long_msg = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Rate limit exceeded.";
        assert!(is_rate_limit_error(long_msg));
    }

    #[tokio::test]
    async fn failed_reply_sends_error_back() {
        struct FailingExecutor;

        #[async_trait::async_trait]
        impl PostExecutor for FailingExecutor {
            async fn execute_reply(
                &self,
                _tweet_id: &str,
                _content: &str,
                _media_ids: &[String],
            ) -> Result<String, String> {
                Err("API error".to_string())
            }

            async fn execute_tweet(
                &self,
                _content: &str,
                _media_ids: &[String],
            ) -> Result<String, String> {
                Err("API error".to_string())
            }
        }

        let executor = Arc::new(FailingExecutor);
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::Reply {
            tweet_id: "t1".to_string(),
            content: "will fail".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result: Result<String, String> = result_rx.await.expect("oneshot recv");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "API error");

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn failed_thread_tweet_sends_error_back() {
        struct FailingExecutor;

        #[async_trait::async_trait]
        impl PostExecutor for FailingExecutor {
            async fn execute_reply(
                &self,
                _tweet_id: &str,
                _content: &str,
                _media_ids: &[String],
            ) -> Result<String, String> {
                Err("API error".to_string())
            }

            async fn execute_tweet(
                &self,
                _content: &str,
                _media_ids: &[String],
            ) -> Result<String, String> {
                Err("API error".to_string())
            }
        }

        let executor = Arc::new(FailingExecutor);
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::ThreadTweet {
            content: "thread part".to_string(),
            in_reply_to: "prev".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result: Result<String, String> = result_rx.await.expect("oneshot recv");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "API error");

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn process_reply_with_media_ids() {
        struct MockExecutorMedia;

        #[async_trait::async_trait]
        impl PostExecutor for MockExecutorMedia {
            async fn execute_reply(
                &self,
                _tweet_id: &str,
                _content: &str,
                media_ids: &[String],
            ) -> Result<String, String> {
                if media_ids.len() == 2 {
                    Ok("reply-with-media".to_string())
                } else {
                    Err("unexpected media count".to_string())
                }
            }

            async fn execute_tweet(
                &self,
                _content: &str,
                _media_ids: &[String],
            ) -> Result<String, String> {
                Ok("tweet-id".to_string())
            }
        }

        let executor = Arc::new(MockExecutorMedia);
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::Reply {
            tweet_id: "t1".to_string(),
            content: "reply".to_string(),
            media_ids: vec!["m1".to_string(), "m2".to_string()],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result: Result<String, String> = result_rx.await.expect("oneshot recv");
        assert_eq!(result, Ok("reply-with-media".to_string()));

        cancel.cancel();
        handle.await.expect("join");
    }

    #[test]
    fn randomized_delay_in_range() {
        let min = Duration::from_millis(10);
        let max = Duration::from_millis(100);

        for _ in 0..100 {
            let d = randomized_delay(min, max);
            assert!(d >= min);
            assert!(d <= max);
        }
    }

    #[test]
    fn randomized_delay_returns_min_when_equal() {
        let min = Duration::from_millis(50);
        let max = Duration::from_millis(50);
        let d = randomized_delay(min, max);
        assert_eq!(d, min);
    }

    #[test]
    fn randomized_delay_returns_zero_when_both_zero() {
        let min = Duration::ZERO;
        let max = Duration::ZERO;
        let d = randomized_delay(min, max);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn randomized_delay_returns_min_when_min_greater() {
        let min = Duration::from_millis(100);
        let max = Duration::from_millis(50);
        let d = randomized_delay(min, max);
        assert_eq!(d, min);
    }

}
