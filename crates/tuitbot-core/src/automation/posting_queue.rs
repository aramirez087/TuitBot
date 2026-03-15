//! Serialized posting queue for concurrent automation loops.
//!
//! All loops funnel post actions through a single bounded MPSC channel,
//! preventing race conditions and ensuring rate limits are respected
//! globally. A single consumer task processes actions sequentially with
//! configurable delays between posts.

use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use super::circuit_breaker::CircuitBreaker;

/// Default bounded channel capacity for the posting queue.
pub const QUEUE_CAPACITY: usize = 100;

/// An action to be executed by the posting queue consumer.
///
/// Each variant optionally includes a oneshot sender so the caller can
/// await the result (e.g., the posted tweet ID or an error message).
pub enum PostAction {
    /// Reply to an existing tweet.
    Reply {
        /// The ID of the tweet to reply to.
        tweet_id: String,
        /// The reply content.
        content: String,
        /// Media IDs to attach (already uploaded to X API).
        media_ids: Vec<String>,
        /// Optional channel to receive the result (posted tweet ID or error).
        result_tx: Option<oneshot::Sender<Result<String, String>>>,
    },
    /// Post a new original tweet.
    Tweet {
        /// The tweet content.
        content: String,
        /// Media IDs to attach (already uploaded to X API).
        media_ids: Vec<String>,
        /// Optional channel to receive the result.
        result_tx: Option<oneshot::Sender<Result<String, String>>>,
    },
    /// Post a tweet as part of a thread (reply to previous tweet in thread).
    ThreadTweet {
        /// The tweet content.
        content: String,
        /// The ID of the previous tweet in the thread.
        in_reply_to: String,
        /// Media IDs to attach (already uploaded to X API).
        media_ids: Vec<String>,
        /// Optional channel to receive the result.
        result_tx: Option<oneshot::Sender<Result<String, String>>>,
    },
}

impl std::fmt::Debug for PostAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostAction::Reply {
                tweet_id,
                content,
                media_ids,
                ..
            } => f
                .debug_struct("Reply")
                .field("tweet_id", tweet_id)
                .field("content_len", &content.len())
                .field("media_count", &media_ids.len())
                .finish(),
            PostAction::Tweet {
                content, media_ids, ..
            } => f
                .debug_struct("Tweet")
                .field("content_len", &content.len())
                .field("media_count", &media_ids.len())
                .finish(),
            PostAction::ThreadTweet {
                content,
                in_reply_to,
                media_ids,
                ..
            } => f
                .debug_struct("ThreadTweet")
                .field("in_reply_to", in_reply_to)
                .field("content_len", &content.len())
                .field("media_count", &media_ids.len())
                .finish(),
        }
    }
}

/// Trait for executing post actions against the X API.
///
/// This trait decouples the posting queue from the actual API client,
/// allowing the queue to be tested with mock executors.
#[async_trait::async_trait]
pub trait PostExecutor: Send + Sync {
    /// Post a reply to a specific tweet. Returns the posted tweet ID.
    async fn execute_reply(
        &self,
        tweet_id: &str,
        content: &str,
        media_ids: &[String],
    ) -> Result<String, String>;

    /// Post a new original tweet. Returns the posted tweet ID.
    async fn execute_tweet(&self, content: &str, media_ids: &[String]) -> Result<String, String>;
}

/// Create a bounded posting queue channel.
///
/// Returns `(sender, receiver)`. Clone the sender for each automation loop.
/// Pass the receiver to [`run_posting_queue`].
pub fn create_posting_queue() -> (mpsc::Sender<PostAction>, mpsc::Receiver<PostAction>) {
    mpsc::channel(QUEUE_CAPACITY)
}

/// Trait for queueing actions for human approval instead of posting.
#[async_trait::async_trait]
pub trait ApprovalQueue: Send + Sync {
    /// Queue a reply for human review. Returns the queue item ID.
    async fn queue_reply(
        &self,
        tweet_id: &str,
        content: &str,
        media_paths: &[String],
    ) -> Result<i64, String>;

    /// Queue a tweet for human review. Returns the queue item ID.
    async fn queue_tweet(&self, content: &str, media_paths: &[String]) -> Result<i64, String>;
}

/// Run the posting queue consumer loop.
///
/// Processes actions sequentially with `min_delay` between each post.
/// On cancellation, drains remaining actions in the channel before exiting.
///
/// When `approval_queue` is `Some`, actions are queued for human review
/// instead of being posted directly.
pub async fn run_posting_queue(
    receiver: mpsc::Receiver<PostAction>,
    executor: Arc<dyn PostExecutor>,
    min_delay: Duration,
    cancel: CancellationToken,
) {
    run_posting_queue_with_approval(receiver, executor, None, min_delay, min_delay, None, cancel)
        .await;
}

/// Run the posting queue consumer loop with optional approval mode.
///
/// Delay between posts is randomized uniformly in `[min_delay, max_delay]`.
/// If a `circuit_breaker` is provided, mutations are gated: the queue blocks
/// while the breaker is Open, and errors/successes are recorded.
pub async fn run_posting_queue_with_approval(
    mut receiver: mpsc::Receiver<PostAction>,
    executor: Arc<dyn PostExecutor>,
    approval_queue: Option<Arc<dyn ApprovalQueue>>,
    min_delay: Duration,
    max_delay: Duration,
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    cancel: CancellationToken,
) {
    tracing::info!("Posting queue consumer started");

    loop {
        let action = tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                tracing::info!("Posting queue received cancellation, draining remaining actions");
                break;
            }
            action = receiver.recv() => {
                match action {
                    Some(a) => a,
                    None => {
                        tracing::info!("Posting queue channel closed");
                        break;
                    }
                }
            }
        };

        // Gate on circuit breaker (only for direct execution, not approval queue).
        if approval_queue.is_none() {
            if let Some(ref cb) = circuit_breaker {
                if !cb.should_allow_mutation().await {
                    tracing::warn!("Circuit breaker open — waiting before posting");
                    if !cb.wait_until_closed(&cancel).await {
                        tracing::info!("Cancelled while waiting for circuit breaker");
                        break;
                    }
                }
            }
        }

        let result = execute_or_queue(action, &executor, &approval_queue).await;

        // Record result in circuit breaker.
        if approval_queue.is_none() {
            if let Some(ref cb) = circuit_breaker {
                match result {
                    PostResult::Success => {
                        cb.record_success().await;
                    }
                    PostResult::Error(ref msg) if is_rate_limit_error(msg) => {
                        cb.record_error().await;
                    }
                    _ => {}
                }
            }
        }

        let delay = randomized_delay(min_delay, max_delay);
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }

    // Drain remaining actions after cancellation or channel close.
    let mut drained = 0u32;
    while let Ok(action) = receiver.try_recv() {
        execute_or_queue(action, &executor, &approval_queue).await;
        drained += 1;
    }

    if drained > 0 {
        tracing::info!(
            count = drained,
            "Drained remaining actions from posting queue"
        );
    }

    tracing::info!("Posting queue consumer stopped");
}

/// Whether an error message indicates a rate limit or forbidden response.
fn is_rate_limit_error(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("429")
        || lower.contains("forbidden")
        || lower.contains("403")
}

/// Outcome of a post action (for circuit breaker tracking).
enum PostResult {
    Success,
    Error(String),
    Queued,
}

/// Route a post action: queue for approval if approval mode is on, otherwise execute.
async fn execute_or_queue(
    action: PostAction,
    executor: &Arc<dyn PostExecutor>,
    approval_queue: &Option<Arc<dyn ApprovalQueue>>,
) -> PostResult {
    if let Some(queue) = approval_queue {
        queue_for_approval(action, queue).await;
        PostResult::Queued
    } else {
        execute_and_respond(action, executor).await
    }
}

/// Queue a post action for human approval instead of posting.
async fn queue_for_approval(action: PostAction, queue: &Arc<dyn ApprovalQueue>) {
    let (result, result_tx) = match action {
        PostAction::Reply {
            tweet_id,
            content,
            media_ids: _,
            result_tx,
        } => {
            tracing::info!(tweet_id = %tweet_id, "Queuing reply for approval");
            let r = queue
                .queue_reply(&tweet_id, &content, &[])
                .await
                .map(|id| format!("queued:{id}"));
            (r, result_tx)
        }
        PostAction::Tweet {
            content,
            media_ids: _,
            result_tx,
        } => {
            tracing::info!("Queuing tweet for approval");
            let r = queue
                .queue_tweet(&content, &[])
                .await
                .map(|id| format!("queued:{id}"));
            (r, result_tx)
        }
        PostAction::ThreadTweet {
            content,
            in_reply_to,
            media_ids: _,
            result_tx,
        } => {
            tracing::info!(in_reply_to = %in_reply_to, "Queuing thread tweet for approval");
            let r = queue
                .queue_reply(&in_reply_to, &content, &[])
                .await
                .map(|id| format!("queued:{id}"));
            (r, result_tx)
        }
    };

    match &result {
        Ok(id) => tracing::info!(queue_id = %id, "Action queued for approval"),
        Err(e) => tracing::warn!(error = %e, "Failed to queue action for approval"),
    }

    if let Some(tx) = result_tx {
        let _ = tx.send(result);
    }
}

/// Execute a single post action and send the result back via oneshot.
async fn execute_and_respond(action: PostAction, executor: &Arc<dyn PostExecutor>) -> PostResult {
    let (result, result_tx) = match action {
        PostAction::Reply {
            tweet_id,
            content,
            media_ids,
            result_tx,
        } => {
            tracing::debug!(tweet_id = %tweet_id, "Executing reply action");
            let r = executor
                .execute_reply(&tweet_id, &content, &media_ids)
                .await;
            (r, result_tx)
        }
        PostAction::Tweet {
            content,
            media_ids,
            result_tx,
        } => {
            tracing::debug!("Executing tweet action");
            let r = executor.execute_tweet(&content, &media_ids).await;
            (r, result_tx)
        }
        PostAction::ThreadTweet {
            content,
            in_reply_to,
            media_ids,
            result_tx,
        } => {
            tracing::debug!(in_reply_to = %in_reply_to, "Executing thread tweet action");
            let r = executor
                .execute_reply(&in_reply_to, &content, &media_ids)
                .await;
            (r, result_tx)
        }
    };

    let post_result = match &result {
        Ok(id) => {
            tracing::info!(tweet_id = %id, "Post action succeeded");
            PostResult::Success
        }
        Err(e) => {
            tracing::warn!(error = %e, "Post action failed");
            PostResult::Error(e.clone())
        }
    };

    if let Some(tx) = result_tx {
        // Ignore send error (receiver may have been dropped).
        let _ = tx.send(result);
    }

    post_result
}

/// Compute a randomized delay between `min` and `max`.
fn randomized_delay(min: Duration, max: Duration) -> Duration {
    if min >= max || min.is_zero() && max.is_zero() {
        return min;
    }
    let min_ms = min.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    Duration::from_millis(rand::thread_rng().gen_range(min_ms..=max_ms))
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let (tx, rx) = create_posting_queue();
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
        let (tx, rx) = create_posting_queue();
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

    #[test]
    fn post_action_debug_thread_tweet_variant() {
        let action = PostAction::ThreadTweet {
            content: "thread part".to_string(),
            in_reply_to: "prev-123".to_string(),
            media_ids: vec![],
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
