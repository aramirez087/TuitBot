use super::*;
use std::sync::Mutex;
use tokio::sync::oneshot;

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

    async fn execute_tweet(&self, content: &str, _media_ids: &[String]) -> Result<String, String> {
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
    let (tx, rx) = super::super::queue::create_posting_queue();
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    let exec_clone = executor.clone();
    let handle = tokio::spawn(async move {
        run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
    });

    let (result_tx, result_rx) = oneshot::channel();
    tx.send(PostAction::Reply {
        tweet_id: "t1".to_string(),
        content: "hello".to_string(),
        media_ids: vec![],
        result_tx: Some(result_tx),
    })
    .await
    .expect("send failed");

    let result = result_rx.await.expect("oneshot recv");
    assert_eq!(result, Ok("reply-id-123".to_string()));

    cancel.cancel();
    handle.await.expect("join");
    assert_eq!(executor.call_count(), 1);
}

#[tokio::test]
async fn process_tweet_action() {
    let executor = Arc::new(MockExecutor::new());
    let (tx, rx) = super::super::queue::create_posting_queue();
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    let exec_clone = executor.clone();
    let handle = tokio::spawn(async move {
        run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
    });

    let (result_tx, result_rx) = oneshot::channel();
    tx.send(PostAction::Tweet {
        content: "my tweet".to_string(),
        media_ids: vec![],
        result_tx: Some(result_tx),
    })
    .await
    .expect("send failed");

    let result = result_rx.await.expect("oneshot recv");
    assert_eq!(result, Ok("tweet-id-456".to_string()));

    cancel.cancel();
    handle.await.expect("join");
}

#[tokio::test]
async fn process_thread_tweet_action() {
    let executor = Arc::new(MockExecutor::new());
    let (tx, rx) = super::super::queue::create_posting_queue();
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    let exec_clone = executor.clone();
    let handle = tokio::spawn(async move {
        run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
    });

    let (result_tx, result_rx) = oneshot::channel();
    tx.send(PostAction::ThreadTweet {
        content: "thread part 2".to_string(),
        in_reply_to: "prev-id".to_string(),
        media_ids: vec![],
        result_tx: Some(result_tx),
    })
    .await
    .expect("send failed");

    let result = result_rx.await.expect("oneshot recv");
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    let exec_clone = executor.clone();
    let handle = tokio::spawn(async move {
        run_posting_queue(rx, exec_clone, Duration::ZERO, cancel_clone).await;
    });

    let (result_tx, result_rx) = oneshot::channel();
    tx.send(PostAction::Tweet {
        content: "will fail".to_string(),
        media_ids: vec![],
        result_tx: Some(result_tx),
    })
    .await
    .expect("send failed");

    let result = result_rx.await.expect("oneshot recv");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "mock error");

    cancel.cancel();
    handle.await.expect("join");
}

#[tokio::test]
async fn channel_close_exits_consumer() {
    let executor = Arc::new(MockExecutor::new());
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    tx.send(PostAction::Reply {
        tweet_id: "t1".to_string(),
        content: "hello".to_string(),
        media_ids: vec![],
        result_tx: Some(result_tx),
    })
    .await
    .expect("send");

    let result = result_rx.await.expect("recv");
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
    assert!(is_rate_limit_error_test("Rate limit exceeded"));
    assert!(is_rate_limit_error_test("Error 429: Too Many Requests"));
    assert!(is_rate_limit_error_test("too many requests"));
    assert!(is_rate_limit_error_test("Forbidden: 403"));
    assert!(is_rate_limit_error_test("forbidden"));
    assert!(!is_rate_limit_error_test("Internal server error"));
    assert!(!is_rate_limit_error_test("Not found"));
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
fn is_rate_limit_error_case_insensitive() {
    assert!(is_rate_limit_error_test("RATE LIMIT"));
    assert!(is_rate_limit_error_test("Rate Limit Exceeded"));
    assert!(is_rate_limit_error_test("TOO MANY REQUESTS"));
    assert!(is_rate_limit_error_test("FORBIDDEN"));
}

#[test]
fn is_rate_limit_error_partial_match() {
    assert!(is_rate_limit_error_test(
        "error: rate limit exceeded for endpoint"
    ));
    assert!(is_rate_limit_error_test(
        "HTTP 429 Too Many Requests from API"
    ));
    assert!(is_rate_limit_error_test(
        "403 Forbidden: write scope missing"
    ));
}

#[test]
fn is_rate_limit_error_unrelated() {
    assert!(!is_rate_limit_error_test("success"));
    assert!(!is_rate_limit_error_test(""));
    assert!(!is_rate_limit_error_test("network timeout"));
    assert!(!is_rate_limit_error_test("invalid json"));
}

#[test]
fn randomized_delay_zero_min_nonzero_max() {
    let min = Duration::ZERO;
    let max = Duration::from_millis(100);
    let d = randomized_delay(min, max);
    assert!(d <= max);
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
fn is_rate_limit_error_empty_string() {
    assert!(!is_rate_limit_error_test(""));
}

#[test]
fn is_rate_limit_error_whitespace_only() {
    assert!(!is_rate_limit_error_test("   "));
}

#[test]
fn is_rate_limit_error_mixed_case_429() {
    assert!(is_rate_limit_error_test("Error code: 429"));
    assert!(is_rate_limit_error_test("Got a 429 response"));
    assert!(is_rate_limit_error_test("HTTP/1.1 429 Too Many Requests"));
}

#[test]
fn is_rate_limit_error_mixed_case_403() {
    assert!(is_rate_limit_error_test("403 Forbidden"));
    assert!(is_rate_limit_error_test("Response: 403"));
}

#[test]
fn is_rate_limit_error_long_message() {
    let long = format!(
        "This is a very long error message that mentions rate limit somewhere in the middle {}",
        "x".repeat(500)
    );
    assert!(is_rate_limit_error_test(&long));
}

#[tokio::test]
async fn failed_reply_sends_error_back() {
    let executor = Arc::new(MockExecutor::failing());
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
    let (tx, rx) = super::super::queue::create_posting_queue();
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
