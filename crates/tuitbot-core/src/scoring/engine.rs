//! Scoring engine — combines all signals into a unified tweet score.

use chrono::{DateTime, Utc};

use crate::config::ScoringConfig;

use super::signals;
use super::{TweetData, TweetScore};

/// Scoring engine that combines all signals into a unified score.
pub struct ScoringEngine {
    pub(super) config: ScoringConfig,
    pub(super) keywords: Vec<String>,
}

impl ScoringEngine {
    /// Create a new scoring engine with the given config and keywords.
    ///
    /// Keywords should be the combined list of `product_keywords` and
    /// `competitor_keywords` from the business profile.
    pub fn new(config: ScoringConfig, keywords: Vec<String>) -> Self {
        Self { config, keywords }
    }

    /// Score a tweet using all six signals.
    ///
    /// Uses the current time for recency scoring.
    pub fn score_tweet(&self, tweet: &TweetData) -> TweetScore {
        self.score_tweet_at(tweet, Utc::now())
    }

    /// Score a tweet using all six signals with a specific time reference.
    ///
    /// Accepts `now` for deterministic testing.
    pub fn score_tweet_at(&self, tweet: &TweetData, now: DateTime<Utc>) -> TweetScore {
        let keyword_relevance = signals::keyword_relevance(
            &tweet.text,
            &self.keywords,
            self.config.keyword_relevance_max,
        );

        let follower = signals::targeted_follower_score(
            tweet.author_followers,
            self.config.follower_count_max,
        );

        let recency = signals::recency_score_at(&tweet.created_at, self.config.recency_max, now);

        let engagement = signals::engagement_rate(
            tweet.likes,
            tweet.retweets,
            tweet.replies,
            tweet.author_followers,
            self.config.engagement_rate_max,
        );

        let reply_count = signals::reply_count_score(tweet.replies, self.config.reply_count_max);

        let content_type = signals::content_type_score(
            tweet.has_media,
            tweet.is_quote_tweet,
            self.config.content_type_max,
        );

        let total =
            (keyword_relevance + follower + recency + engagement + reply_count + content_type)
                .clamp(0.0, 100.0);
        let meets_threshold = total >= self.config.threshold as f32;

        tracing::debug!(
            author = %tweet.author_username,
            total = format!("{:.0}", total),
            keyword = format!("{:.0}", keyword_relevance),
            follower = format!("{:.0}", follower),
            recency = format!("{:.0}", recency),
            engagement = format!("{:.0}", engagement),
            reply = format!("{:.0}", reply_count),
            content = format!("{:.0}", content_type),
            meets = meets_threshold,
            "Scored tweet",
        );

        TweetScore {
            total,
            keyword_relevance,
            follower,
            recency,
            engagement,
            reply_count,
            content_type,
            meets_threshold,
        }
    }

    /// Return the configured keywords.
    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }

    /// Return the scoring configuration.
    pub fn config(&self) -> &ScoringConfig {
        &self.config
    }
}
