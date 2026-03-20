//! Scoring module tests.

use super::*;
use crate::config::ScoringConfig;
use chrono::{Duration, Utc};

fn default_scoring_config() -> ScoringConfig {
    ScoringConfig {
        threshold: 60,
        keyword_relevance_max: 25.0,
        follower_count_max: 15.0,
        recency_max: 10.0,
        engagement_rate_max: 15.0,
        reply_count_max: 15.0,
        content_type_max: 10.0,
    }
}

fn test_tweet(now: chrono::DateTime<Utc>) -> TweetData {
    TweetData {
        text: "Building amazing Rust CLI tools for developers".to_string(),
        created_at: (now - Duration::minutes(10)).to_rfc3339(),
        likes: 20,
        retweets: 5,
        replies: 3,
        author_username: "devuser".to_string(),
        author_followers: 5000,
        has_media: false,
        is_quote_tweet: false,
    }
}

mod edge_cases;
mod engine;
mod helpers;
