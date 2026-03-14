//! Factory for building [`Tweet`] instances with realistic test data.
//!
//! # Example
//! ```rust
//! use tuitbot_core::testing::TweetFactory;
//!
//! let tweet = TweetFactory::new().build();
//! let viral = TweetFactory::new().with_likes(10_000).with_retweets(500).build();
//! let thread_reply = TweetFactory::new()
//!     .with_text("Continuing from my last point…")
//!     .with_conversation_id("1000000000000000001")
//!     .build();
//! ```

use crate::x_api::types::{PublicMetrics, Tweet};

static COUNTER: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1_000_000_000_000_000_001);

fn next_id() -> String {
    COUNTER
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .to_string()
}

/// Builder for [`Tweet`] test instances.
pub struct TweetFactory {
    id: Option<String>,
    text: String,
    author_id: String,
    created_at: String,
    likes: u64,
    retweets: u64,
    replies: u64,
    quotes: u64,
    conversation_id: Option<String>,
}

impl Default for TweetFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl TweetFactory {
    pub fn new() -> Self {
        Self {
            id: None,
            text: "This is a test tweet with realistic content for unit testing.".to_string(),
            author_id: "123456789".to_string(),
            created_at: "2026-03-14T00:00:00.000Z".to_string(),
            likes: 42,
            retweets: 7,
            replies: 3,
            quotes: 1,
            conversation_id: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn with_author_id(mut self, author_id: impl Into<String>) -> Self {
        self.author_id = author_id.into();
        self
    }

    pub fn with_created_at(mut self, ts: impl Into<String>) -> Self {
        self.created_at = ts.into();
        self
    }

    pub fn with_likes(mut self, likes: u64) -> Self {
        self.likes = likes;
        self
    }

    pub fn with_retweets(mut self, retweets: u64) -> Self {
        self.retweets = retweets;
        self
    }

    pub fn with_replies(mut self, replies: u64) -> Self {
        self.replies = replies;
        self
    }

    pub fn with_conversation_id(mut self, conv_id: impl Into<String>) -> Self {
        self.conversation_id = Some(conv_id.into());
        self
    }

    /// Build the [`Tweet`] with all accumulated settings.
    pub fn build(self) -> Tweet {
        let id = self.id.unwrap_or_else(next_id);
        let conversation_id = self.conversation_id.or_else(|| Some(id.clone()));
        Tweet {
            id,
            text: self.text,
            author_id: self.author_id,
            created_at: self.created_at,
            public_metrics: PublicMetrics {
                retweet_count: self.retweets,
                reply_count: self.replies,
                like_count: self.likes,
                quote_count: self.quotes,
                impression_count: 0,
                bookmark_count: 0,
            },
            conversation_id,
        }
    }

    /// Build multiple unique tweets (text is suffixed with index).
    pub fn build_many(count: usize) -> Vec<Tweet> {
        (0..count)
            .map(|i| {
                TweetFactory::new()
                    .with_text(format!("Test tweet #{i}: building in public every day."))
                    .with_likes((i as u64) * 10)
                    .build()
            })
            .collect()
    }
}


mod tests {
    use super::*;

    #[test]
    fn builds_tweet_with_defaults() {
        let tweet = TweetFactory::new().build();
        assert!(!tweet.id.is_empty());
        assert!(!tweet.text.is_empty());
        assert_eq!(tweet.public_metrics.like_count, 42);
    }

    #[test]
    fn builds_tweet_with_overrides() {
        let tweet = TweetFactory::new()
            .with_text("Custom text")
            .with_author_id("999")
            .with_likes(1000)
            .with_retweets(50)
            .build();
        assert_eq!(tweet.text, "Custom text");
        assert_eq!(tweet.author_id, "999");
        assert_eq!(tweet.public_metrics.like_count, 1000);
        assert_eq!(tweet.public_metrics.retweet_count, 50);
    }

    #[test]
    fn builds_many_unique_ids() {
        let tweets = TweetFactory::build_many(5);
        assert_eq!(tweets.len(), 5);
        let ids: std::collections::HashSet<_> = tweets.iter().map(|t| &t.id).collect();
        assert_eq!(ids.len(), 5, "all IDs should be unique");
    }

    #[test]
    fn conversation_id_defaults_to_tweet_id() {
        let tweet = TweetFactory::new().with_id("42").build();
        assert_eq!(tweet.conversation_id, Some("42".to_string()));
    }

    #[test]
    fn explicit_conversation_id_is_preserved() {
        let tweet = TweetFactory::new()
            .with_id("99")
            .with_conversation_id("root-100")
            .build();
        assert_eq!(tweet.conversation_id, Some("root-100".to_string()));
    }
}
