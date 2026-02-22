//! Serialized posting queue for concurrent automation loops.
//!
//! All loops funnel post actions through a single bounded MPSC channel,
//! preventing race conditions and ensuring rate limits are respected
//! globally. A single consumer task processes actions sequentially with
//! configurable delays between posts.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

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
        /// Optional channel to receive the result (posted tweet ID or error).
        result_tx: Option<oneshot::Sender<Result<String, String>>>,
    },
    /// Post a new original tweet.
    Tweet {
        /// The tweet content.
        content: String,
        /// Optional channel to receive the result.
        result_tx: Option<oneshot::Sender<Result<String, String>>>,
    },
    /// Post a tweet as part of a thread (reply to previous tweet in thread).
    ThreadTweet {
        /// The tweet content.
        content: String,
        /// The ID of the previous tweet in the thread.
        in_reply_to: String,
        /// Optional channel to receive the result.
        result_tx: Option<oneshot::Sender<Result<String, String>>>,
    },
}

impl std::fmt::Debug for PostAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostAction::Reply {
                tweet_id, content, ..
            } => f
                .debug_struct("Reply")
                .field("tweet_id", tweet_id)
                .field("content_len", &content.len())
                .finish(),
            PostAction::Tweet { content, .. } => f
                .debug_struct("Tweet")
                .field("content_len", &content.len())
                .finish(),
            PostAction::ThreadTweet {
                content,
                in_reply_to,
                ..
            } => f
                .debug_struct("ThreadTweet")
                .field("in_reply_to", in_reply_to)
                .field("content_len", &content.len())
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
    async fn execute_reply(&self, tweet_id: &str, content: &str) -> Result<String, String>;

    /// Post a new original tweet. Returns the posted tweet ID.
    async fn execute_tweet(&self, content: &str) -> Result<String, String>;
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
    async fn queue_reply(&self, tweet_id: &str, content: &str) -> Result<i64, String>;

    /// Queue a tweet for human review. Returns the queue item ID.
    async fn queue_tweet(&self, content: &str) -> Result<i64, String>;
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
    run_posting_queue_with_approval(receiver, executor, None, min_delay, cancel).await;
}

/// Run the posting queue consumer loop with optional approval mode.
pub async fn run_posting_queue_with_approval(
    mut receiver: mpsc::Receiver<PostAction>,
    executor: Arc<dyn PostExecutor>,
    approval_queue: Option<Arc<dyn ApprovalQueue>>,
    min_delay: Duration,
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

        execute_or_queue(action, &executor, &approval_queue).await;

        if !min_delay.is_zero() {
            tokio::time::sleep(min_delay).await;
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

/// Route a post action: queue for approval if approval mode is on, otherwise execute.
async fn execute_or_queue(
    action: PostAction,
    executor: &Arc<dyn PostExecutor>,
    approval_queue: &Option<Arc<dyn ApprovalQueue>>,
) {
    if let Some(queue) = approval_queue {
        queue_for_approval(action, queue).await;
    } else {
        execute_and_respond(action, executor).await;
    }
}

/// Queue a post action for human approval instead of posting.
async fn queue_for_approval(action: PostAction, queue: &Arc<dyn ApprovalQueue>) {
    let (result, result_tx) = match action {
        PostAction::Reply {
            tweet_id,
            content,
            result_tx,
        } => {
            tracing::info!(tweet_id = %tweet_id, "Queuing reply for approval");
            let r = queue
                .queue_reply(&tweet_id, &content)
                .await
                .map(|id| format!("queued:{id}"));
            (r, result_tx)
        }
        PostAction::Tweet {
            content, result_tx, ..
        } => {
            tracing::info!("Queuing tweet for approval");
            let r = queue
                .queue_tweet(&content)
                .await
                .map(|id| format!("queued:{id}"));
            (r, result_tx)
        }
        PostAction::ThreadTweet {
            content,
            in_reply_to,
            result_tx,
        } => {
            tracing::info!(in_reply_to = %in_reply_to, "Queuing thread tweet for approval");
            let r = queue
                .queue_reply(&in_reply_to, &content)
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
async fn execute_and_respond(action: PostAction, executor: &Arc<dyn PostExecutor>) {
    let (result, result_tx) = match action {
        PostAction::Reply {
            tweet_id,
            content,
            result_tx,
        } => {
            tracing::debug!(tweet_id = %tweet_id, "Executing reply action");
            let r = executor.execute_reply(&tweet_id, &content).await;
            (r, result_tx)
        }
        PostAction::Tweet {
            content, result_tx, ..
        } => {
            tracing::debug!("Executing tweet action");
            let r = executor.execute_tweet(&content).await;
            (r, result_tx)
        }
        PostAction::ThreadTweet {
            content,
            in_reply_to,
            result_tx,
        } => {
            tracing::debug!(in_reply_to = %in_reply_to, "Executing thread tweet action");
            let r = executor.execute_reply(&in_reply_to, &content).await;
            (r, result_tx)
        }
    };

    match &result {
        Ok(id) => tracing::info!(tweet_id = %id, "Post action succeeded"),
        Err(e) => tracing::warn!(error = %e, "Post action failed"),
    }

    if let Some(tx) = result_tx {
        // Ignore send error (receiver may have been dropped).
        let _ = tx.send(result);
    }
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
        async fn execute_reply(&self, tweet_id: &str, content: &str) -> Result<String, String> {
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

        async fn execute_tweet(&self, content: &str) -> Result<String, String> {
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
            result_tx: None,
        })
        .await
        .expect("send");
        tx.send(PostAction::Tweet {
            content: "queued2".to_string(),
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
        async fn queue_reply(&self, tweet_id: &str, content: &str) -> Result<i64, String> {
            self.items.lock().expect("lock").push((
                "reply".to_string(),
                tweet_id.to_string(),
                content.to_string(),
            ));
            Ok(self.item_count() as i64)
        }

        async fn queue_tweet(&self, content: &str) -> Result<i64, String> {
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
                cancel_clone,
            )
            .await;
        });

        let (result_tx, result_rx) = oneshot::channel();
        tx.send(PostAction::Reply {
            tweet_id: "t1".to_string(),
            content: "hello".to_string(),
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
                cancel_clone,
            )
            .await;
        });

        tx.send(PostAction::Tweet {
            content: "my tweet".to_string(),
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
