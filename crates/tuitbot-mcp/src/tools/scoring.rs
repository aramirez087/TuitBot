//! Scoring tool: score a tweet for reply-worthiness.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::scoring::{ScoringEngine, TweetData};

use super::response::{ToolMeta, ToolResponse};

#[derive(Serialize)]
struct ScoreOut {
    total: f32,
    keyword_relevance: f32,
    follower: f32,
    recency: f32,
    engagement: f32,
    reply_count: f32,
    content_type: f32,
    meets_threshold: bool,
}

/// Input for scoring a tweet.
pub struct ScoreTweetInput<'a> {
    pub text: &'a str,
    pub author_username: &'a str,
    pub author_followers: u64,
    pub likes: u64,
    pub retweets: u64,
    pub replies: u64,
    pub created_at: &'a str,
}

/// Score a tweet using the 6-signal scoring engine.
pub fn score_tweet(config: &Config, input: &ScoreTweetInput<'_>) -> String {
    let start = Instant::now();

    let keywords: Vec<String> = config
        .business
        .product_keywords
        .iter()
        .chain(config.business.competitor_keywords.iter())
        .chain(config.business.industry_topics.iter())
        .cloned()
        .collect();

    let engine = ScoringEngine::new(config.scoring.clone(), keywords);

    let tweet_data = TweetData {
        text: input.text.to_string(),
        created_at: input.created_at.to_string(),
        likes: input.likes,
        retweets: input.retweets,
        replies: input.replies,
        author_username: input.author_username.to_string(),
        author_followers: input.author_followers,
        has_media: false,
        is_quote_tweet: false,
    };

    let score = engine.score_tweet(&tweet_data);

    let out = ScoreOut {
        total: score.total,
        keyword_relevance: score.keyword_relevance,
        follower: score.follower,
        recency: score.recency,
        engagement: score.engagement,
        reply_count: score.reply_count,
        content_type: score.content_type,
        meets_threshold: score.meets_threshold,
    };

    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::success(out)
        .with_meta(ToolMeta::new(elapsed))
        .to_json()
}
