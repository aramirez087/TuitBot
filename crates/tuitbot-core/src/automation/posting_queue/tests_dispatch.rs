#[cfg(test)]
mod tests_dispatch {
    use super::super::*;
    use std::sync::Mutex;

            result_tx: None,
        };
        let debug = format!("{action:?}");
        assert!(debug.contains("ThreadTweet"));
        assert!(debug.contains("prev-123"));
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

        let (result_tx, result_rx) = oneshot::channel();
        tx.send(PostAction::ThreadTweet {
            content: "thread part 2".to_string(),
            in_reply_to: "prev-id".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result = result_rx.await.expect("recv");
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("queued:"));

        assert_eq!(executor.call_count(), 0);
        assert_eq!(approval.item_count(), 1);

        cancel.cancel();
        handle.await.expect("join");
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
        // This edge case: min.is_zero() && max.is_zero() is false, min < max
        let d = randomized_delay(min, max);
        assert!(d <= max);
    }

    #[test]
    fn queue_capacity_is_100() {
        assert_eq!(QUEUE_CAPACITY, 100);
    }

    #[test]
    fn create_posting_queue_returns_valid_channel() {
        let (tx, _rx) = create_posting_queue();
        // Verify the sender is usable
        assert!(!tx.is_closed());
    }

    #[test]
    fn post_action_debug_reply_with_media() {
        let action = PostAction::Reply {
            tweet_id: "t1".to_string(),
            content: "hello".to_string(),
            media_ids: vec!["m1".to_string(), "m2".to_string()],
            result_tx: None,
        };
        let debug = format!("{action:?}");
        assert!(debug.contains("Reply"));
        assert!(debug.contains("media_count"));
        assert!(debug.contains("2"));
    }

    // =========================================================================
    // Additional edge case tests for coverage push
    // =========================================================================

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
        assert!(is_rate_limit_error("Error code: 429"));
        assert!(is_rate_limit_error("Got a 429 response"));
        assert!(is_rate_limit_error("HTTP/1.1 429 Too Many Requests"));
    }

    #[test]
    fn is_rate_limit_error_mixed_case_403() {
        assert!(is_rate_limit_error("403 Forbidden"));
        assert!(is_rate_limit_error("Response: 403"));
    }

    #[test]
    fn is_rate_limit_error_long_message() {
        let long = format!(
            "This is a very long error message that mentions rate limit somewhere in the middle {}",
            "x".repeat(500)
        );
        assert!(is_rate_limit_error(&long));
    }

    #[test]
    fn randomized_delay_very_close_values() {
        let min = Duration::from_millis(99);
        let max = Duration::from_millis(100);
        let d = randomized_delay(min, max);
        assert!(d >= min && d <= max);
    }

    #[test]
    fn randomized_delay_large_range() {
        let min = Duration::from_millis(1);
        let max = Duration::from_millis(10000);
        for _ in 0..50 {
            let d = randomized_delay(min, max);
            assert!(d >= min && d <= max);
        }
    }

    #[test]
    fn post_action_debug_reply_empty_media() {
        let action = PostAction::Reply {
            tweet_id: "tweet-abc".to_string(),
            content: "reply content here".to_string(),
            media_ids: vec![],
            result_tx: None,
        };
        let debug = format!("{action:?}");
        assert!(debug.contains("Reply"));
        assert!(debug.contains("tweet-abc"));
        assert!(debug.contains("content_len"));
        assert!(debug.contains("media_count"));
    }

    #[test]
    fn post_action_debug_tweet_with_multiple_media() {
        let action = PostAction::Tweet {
            content: "test".to_string(),
            media_ids: vec!["m1".to_string(), "m2".to_string(), "m3".to_string()],
            result_tx: None,
        };
        let debug = format!("{action:?}");
        assert!(debug.contains("Tweet"));
        assert!(debug.contains("3"));
    }

    #[test]
    fn post_action_debug_thread_tweet_long_content() {
        let content = "x".repeat(500);
        let action = PostAction::ThreadTweet {
            content: content.clone(),
            in_reply_to: "prev".to_string(),
            media_ids: vec![],
            result_tx: None,
        };
        let debug = format!("{action:?}");
        assert!(debug.contains("ThreadTweet"));
        assert!(debug.contains("500"));
    }

    #[tokio::test]
    async fn failed_reply_sends_error_back() {
        let executor = Arc::new(MockExecutor::failing());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel();
        tx.send(PostAction::Reply {
            tweet_id: "t1".to_string(),
            content: "will fail".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result = result_rx.await.expect("recv");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "mock error");

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn failed_thread_tweet_sends_error_back() {
        let executor = Arc::new(MockExecutor::failing());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel();
        tx.send(PostAction::ThreadTweet {
            content: "thread will fail".to_string(),
            in_reply_to: "prev-id".to_string(),
            media_ids: vec![],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result = result_rx.await.expect("recv");
        assert!(result.is_err());

        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn process_reply_with_media_ids() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        let (result_tx, result_rx) = oneshot::channel();
        tx.send(PostAction::Reply {
            tweet_id: "t1".to_string(),
            content: "reply with media".to_string(),
            media_ids: vec!["media-1".to_string(), "media-2".to_string()],
            result_tx: Some(result_tx),
        })
        .await
        .expect("send");

        let result = result_rx.await.expect("recv");
        assert!(result.is_ok());

        cancel.cancel();
        handle.await.expect("join");
        assert_eq!(executor.call_count(), 1);
    }

    #[tokio::test]
    async fn multiple_mixed_actions_processed() {
        let executor = Arc::new(MockExecutor::new());
        let (tx, rx) = create_posting_queue();
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let exec_clone = executor.clone();
        let handle = tokio::spawn(async move {
            run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
        });

        // Send a mix of action types
        tx.send(PostAction::Tweet {
            content: "tweet-1".to_string(),
            media_ids: vec![],
            result_tx: None,
        })
        .await
        .expect("send");

        tx.send(PostAction::Reply {
            tweet_id: "t1".to_string(),
            content: "reply-1".to_string(),
            media_ids: vec![],
            result_tx: None,
        })
        .await
        .expect("send");

        tx.send(PostAction::ThreadTweet {
            content: "thread-1".to_string(),
            in_reply_to: "prev".to_string(),
            media_ids: vec![],
            result_tx: None,
        })
        .await
        .expect("send");

        tokio::time::sleep(Duration::from_millis(100)).await;

        cancel.cancel();
        handle.await.expect("join");

        let calls = executor.calls();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0].0, "tweet");
        assert_eq!(calls[1].0, "reply");
        assert_eq!(calls[2].0, "reply"); // ThreadTweet uses execute_reply
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
}
}
