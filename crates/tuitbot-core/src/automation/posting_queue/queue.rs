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
