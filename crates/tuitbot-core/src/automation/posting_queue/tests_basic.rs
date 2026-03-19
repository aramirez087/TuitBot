#[cfg(test)]
mod tests_basic {
    use super::super::*;
    use std::sync::Mutex;

    /// Mock executor that records all calls.
    struct MockExecutor {
        calls: Mutex<Vec<(String, String)>>,
        fail: bool,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fail: true,
            }
        }

        fn call_count(&self) -> usize {
            self.calls.lock().expect("lock poisoned").len()
        }

        fn calls(&self) -> Vec<(String, String)> {
            self.calls.lock().expect("lock poisoned").clone()
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
            if self.fail {
                Err("mock error".to_string())
            } else {
                Ok("reply-id-123".to_string())
            }
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
            if self.fail {
                Err("mock error".to_string())
            } else {
                Ok("tweet-id-456".to_string())
            }
        }
    }

    #[tokio::test]
    async fn process_reply_action() {
        let executor = Arc::new(MockExecutor::new());
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
            content: "hello".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send failed");

        let result: Result<String, String> = result_rx.await.expect::<Result<String, String>>("oneshot recv");
        assert_eq!(result, Ok("reply-id-123".to_string()));

        cancel.cancel();
        handle.await.expect("join");
        assert_eq!(executor.call_count(), 1);
    }

    #[tokio::test]
    async fn process_tweet_action() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::Tweet {
            content: "my tweet".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send failed");

        let result: Result<String, String> = result_rx.await.expect::<Result<String, String>>("oneshot recv");
        assert_eq!(result, Ok("tweet-id-456".to_string()));

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn process_thread_tweet_action() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::ThreadTweet {
            content: "thread part 2".to_string(),
            in_reply_to: "prev-id".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send failed");

        let result: Result<String, String> = result_rx.await.expect::<Result<String, String>>("oneshot recv");
        assert_eq!(result, Ok("reply-id-123".to_string()));

        cancel.cancel();
        handle.await.expect("join");

        let calls = executor.calls();
        assert_eq!(calls[0].0, "reply");
        assert!(calls[0].1.contains("prev-id"));
    }

    #[tokio::test]
    async fn result_tx_none_does_not_panic() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        tx.send(PostAction::Tweet {
            content: "fire and forget".to_string(),
            media_ids: vec![],
            result_tx: None,
        })
        .await
        .expect("send failed");

        // Give time for processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        cancel.cancel();
        handle.await.expect("join");
        assert_eq!(executor.call_count(), 1);
    }

    #[tokio::test]
    async fn failed_action_sends_error_back() {
        let executor = Arc::new(MockExecutor::failing());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel::<Result<String, String>>();
        tx.send(PostAction::Tweet {
            content: "will fail".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send failed");

        let result: Result<String, String> = result_rx.await.expect::<Result<String, String>>("oneshot recv");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "mock error");

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn channel_close_exits_consumer() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let handle = tokio::spawn(async move {
            run_posting_queue(rx, executor, Duration::ZERO, cancel).await;
        });

        // Drop sender to close channel
        drop(tx);

        // Consumer should exit cleanly
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn drain_on_cancel() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        // Send actions before starting consumer
        tx.send(PostAction::Tweet {
            content: "queued1".to_string(),
            media_ids: vec![],
            result_tx: None,
        })
        .await
        .expect("send");
        tx.send(PostAction::Tweet {
            content: "queued2".to_string(),
            media_ids: vec![],
            result_tx: None,
        })
        .await
        .expect("send");

        // Cancel immediately
        cancel.cancel();

        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel).await;
        });

        handle.await.expect("join");

        // Both queued actions should have been drained
        assert_eq!(executor.call_count(), 2);
    }

    #[tokio::test]
    async fn multiple_actions_processed_in_order() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        for i in 0..5 {
            tx.send(PostAction::Tweet {
                content: format!("tweet-{i}"),
                media_ids: vec![],
                result_tx: None,
            })
            .await
            .expect("send");
        }

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        cancel.cancel();
        handle.await.expect("join");

        let calls = executor.calls();
        assert_eq!(calls.len(), 5);
        for (i, (action_type, content)) in calls.iter().enumerate() {
            assert_eq!(action_type, "tweet");
            assert_eq!(content, &format!("tweet-{i}"));
        }
    }

    #[test]
    fn post_action_debug_format() {
        let action = PostAction::Reply {
            tweet_id: "123".to_string(),
            content: "hello world".to_string(),
            media_ids: vec![],
            result_tx: None,
        };
        let debug = format!("{action:?}");
        assert!(debug.contains("Reply"));
        assert!(debug.contains("123"));
    }

    // --- Approval queue tests ---

    struct MockApprovalQueue {
        items: Mutex<Vec<(String, String, String)>>,
    }

    impl MockApprovalQueue {
        fn new() -> Self {
            Self {
                items: Mutex::new(Vec::new()),
            }
        }

        fn item_count(&self) -> usize {
            self.items.lock().expect("lock").len()
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
            self.items.lock().expect("lock").push((
                "reply".to_string(),
                tweet_id.to_string(),
                content.to_string(),
            ));
            Ok(self.item_count() as i64)
        }

        async fn queue_tweet(&self, content: &str, _media_paths: &[String]) -> Result<i64, String> {
            self.items.lock().expect("lock").push((
                "tweet".to_string(),
                String::new(),
                content.to_string(),
            ));
            Ok(self.item_count() as i64)
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
            tweet_id: "t1".to_string(),
            content: "hello".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result: Result<String, String> = result_rx.await.expect::<Result<String, String>>("recv");
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("queued:"));

        // Executor should NOT have been called
        assert_eq!(executor.call_count(), 0);
        // Approval queue should have the item
        assert_eq!(approval.item_count(), 1);

        cancel.cancel();
        handle.await.expect("join");
    }

    // --- Pure function tests ---

    #[test]
    fn is_rate_limit_error_detects_rate_limit() {
        assert!(is_rate_limit_error("Rate limit exceeded"));
        assert!(is_rate_limit_error("Error 429: Too Many Requests"));
        assert!(is_rate_limit_error("too many requests"));
        assert!(is_rate_limit_error("Forbidden: 403"));
        assert!(is_rate_limit_error("forbidden"));
        assert!(!is_rate_limit_error("Internal server error"));
        assert!(!is_rate_limit_error("Not found"));
    }

    #[test]
    fn randomized_delay_returns_min_when_equal() {
        let d = randomized_delay(Duration::from_millis(100), Duration::from_millis(100));
        assert_eq!(d, Duration::from_millis(100));
    }

    #[test]
    fn randomized_delay_returns_min_when_min_greater() {
        let d = randomized_delay(Duration::from_millis(200), Duration::from_millis(100));
        assert_eq!(d, Duration::from_millis(200));
    }

    #[test]
    fn randomized_delay_returns_zero_when_both_zero() {
        let d = randomized_delay(Duration::ZERO, Duration::ZERO);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn randomized_delay_in_range() {
        let min = Duration::from_millis(50);
        let max = Duration::from_millis(150);
        for _ in 0..20 {
            let d = randomized_delay(min, max);
            assert!(
                d >= min && d <= max,
                "delay {:?} not in [{:?}, {:?}]",
                d,
                min,
                max
            );
        }
    }

    #[test]
    fn post_action_debug_tweet_variant() {
        let action = PostAction::Tweet {
            content: "hello world".to_string(),
            media_ids: vec!["m1".to_string()],
            result_tx: None,
        };
        let debug = format!("{action:?}");
        assert!(debug.contains("Tweet"));
        assert!(debug.contains("media_count"));
    }
}
