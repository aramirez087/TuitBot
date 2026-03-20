//! Tweet scoring engine for reply-worthiness evaluation.
//!
//! Combines six independent signals (keyword relevance, follower score,
//! recency, engagement rate, reply count, content type) into a total score
//! (0-100) with a configurable threshold for the REPLY/SKIP verdict.
//!
//! All scoring is purely heuristic — no LLM calls.

pub mod signals;

mod engine;
mod weights;

pub use engine::ScoringEngine;
pub use weights::{
    find_matched_keywords, format_follower_count, format_tweet_age, format_tweet_age_at,
    truncate_text,
};

use crate::config::ScoringConfig;

/// Input data for scoring a tweet.
///
/// This struct decouples the scoring engine from specific API types,
/// allowing the engine to be used with any data source.
#[derive(Debug, Clone)]
pub struct TweetData {
    /// The tweet text content.
    pub text: String,
    /// ISO-8601 timestamp of when the tweet was created.
    pub created_at: String,
    /// Number of likes on the tweet.
    pub likes: u64,
    /// Number of retweets.
    pub retweets: u64,
    /// Number of replies.
    pub replies: u64,
    /// Author's username (for display).
    pub author_username: String,
    /// Author's follower count.
    pub author_followers: u64,
    /// Whether the tweet has attached media (images, video, etc.).
    #[allow(dead_code)]
    pub has_media: bool,
    /// Whether the tweet is a quote tweet.
    #[allow(dead_code)]
    pub is_quote_tweet: bool,
}

/// Per-signal score breakdown for a tweet.
#[derive(Debug, Clone)]
pub struct TweetScore {
    /// Total score (0-100), clamped.
    pub total: f32,
    /// Keyword relevance signal score.
    pub keyword_relevance: f32,
    /// Author follower count signal score.
    pub follower: f32,
    /// Tweet recency signal score.
    pub recency: f32,
    /// Engagement rate signal score.
    pub engagement: f32,
    /// Reply count signal score (fewer replies = higher).
    pub reply_count: f32,
    /// Content type signal score (text-only = max).
    pub content_type: f32,
    /// Whether the total score meets the configured threshold.
    pub meets_threshold: bool,
}

impl TweetScore {
    /// Format a human-readable breakdown of the score.
    ///
    /// Shows the total score, per-signal breakdown with context,
    /// and the REPLY/SKIP verdict.
    pub fn format_breakdown(
        &self,
        config: &ScoringConfig,
        tweet: &TweetData,
        matched_keywords: &[String],
    ) -> String {
        let truncated = truncate_text(&tweet.text, 50);
        let formatted_followers = format_follower_count(tweet.author_followers);
        let age = format_tweet_age(&tweet.created_at);
        let matched_list = if matched_keywords.is_empty() {
            "none".to_string()
        } else {
            matched_keywords.join(", ")
        };

        let total_engagement = tweet.likes + tweet.retweets + tweet.replies;
        let followers_for_rate = tweet.author_followers.max(1) as f64;
        let rate_pct = (total_engagement as f64 / followers_for_rate) * 100.0;

        let verdict = if self.meets_threshold {
            "REPLY"
        } else {
            "SKIP"
        };

        let reply_count_display = tweet.replies;

        format!(
            "Tweet: \"{}\" by @{} ({} followers)\n\
             Score: {:.0}/100\n\
             \x20 Keyword relevance:  {:.0}/{}  (matched: {})\n\
             \x20 Author reach:       {:.0}/{}  ({} followers, bell curve)\n\
             \x20 Recency:            {:.0}/{}  (posted {} ago)\n\
             \x20 Engagement rate:    {:.0}/{}  ({:.1}% engagement vs 1.5% baseline)\n\
             \x20 Reply count:        {:.0}/{}  ({} existing replies)\n\
             \x20 Content type:       {:.0}/{}  ({})\n\
             Verdict: {} (threshold: {})",
            truncated,
            tweet.author_username,
            formatted_followers,
            self.total,
            self.keyword_relevance,
            config.keyword_relevance_max as u32,
            matched_list,
            self.follower,
            config.follower_count_max as u32,
            formatted_followers,
            self.recency,
            config.recency_max as u32,
            age,
            self.engagement,
            config.engagement_rate_max as u32,
            rate_pct,
            self.reply_count,
            config.reply_count_max as u32,
            reply_count_display,
            self.content_type,
            config.content_type_max as u32,
            if tweet.has_media || tweet.is_quote_tweet {
                "media/quote"
            } else {
                "text-only"
            },
            verdict,
            config.threshold,
        )
    }
}

impl std::fmt::Display for TweetScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Score: {:.0}/100 [kw:{:.0} fol:{:.0} rec:{:.0} eng:{:.0} rep:{:.0} ct:{:.0}] {}",
            self.total,
            self.keyword_relevance,
            self.follower,
            self.recency,
            self.engagement,
            self.reply_count,
            self.content_type,
            if self.meets_threshold {
                "REPLY"
            } else {
                "SKIP"
            }
        )
    }
}

#[cfg(test)]
mod signals_tests;
#[cfg(test)]
mod tests;
