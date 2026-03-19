use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::automation::circuit_breaker::CircuitBreaker;
use crate::automation::posting_queue::queue::{ApprovalQueue, PostAction, PostExecutor, QUEUE_CAPACITY, create_posting_queue};

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
pub fn is_rate_limit_error(msg: &str) -> bool {
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
pub fn randomized_delay(min: Duration, max: Duration) -> Duration {
    if min >= max || min.is_zero() && max.is_zero() {
        return min;
    }
    let min_ms = min.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    Duration::from_millis(rand::rng().random_range(min_ms..=max_ms))
}

