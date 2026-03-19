//! PostAction enum and queue creation.

use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

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

/// Create a bounded posting queue channel.
///
/// Returns `(sender, receiver)`. Clone the sender for each automation loop.
/// Pass the receiver to [`run_posting_queue`](crate::automation::posting_queue::run_posting_queue).
pub fn create_posting_queue() -> (mpsc::Sender<PostAction>, mpsc::Receiver<PostAction>) {
    mpsc::channel(QUEUE_CAPACITY)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
