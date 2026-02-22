//! Tweet scoring engine for reply-worthiness evaluation.
//!
//! Combines four independent signals (keyword relevance, follower score,
//! recency, engagement rate) into a total score (0-100) with a configurable
//! threshold for the REPLY/SKIP verdict.
//!
//! All scoring is purely heuristic -- no LLM calls.

pub mod signals;

use crate::config::ScoringConfig;
use chrono::{DateTime, Utc};

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
    /// Whether the total score meets the configured threshold.
    pub meets_threshold: bool,
}

/// Scoring engine that combines all signals into a unified score.
pub struct ScoringEngine {
    config: ScoringConfig,
    keywords: Vec<String>,
}

impl ScoringEngine {
    /// Create a new scoring engine with the given config and keywords.
    ///
    /// Keywords should be the combined list of `product_keywords` and
    /// `competitor_keywords` from the business profile.
    pub fn new(config: ScoringConfig, keywords: Vec<String>) -> Self {
        Self { config, keywords }
    }

    /// Score a tweet using all four signals.
    ///
    /// Uses the current time for recency scoring.
    pub fn score_tweet(&self, tweet: &TweetData) -> TweetScore {
        self.score_tweet_at(tweet, Utc::now())
    }

    /// Score a tweet using all four signals with a specific time reference.
    ///
    /// Accepts `now` for deterministic testing.
    pub fn score_tweet_at(&self, tweet: &TweetData, now: DateTime<Utc>) -> TweetScore {
        let keyword_relevance = signals::keyword_relevance(
            &tweet.text,
            &self.keywords,
            self.config.keyword_relevance_max,
        );

        let follower =
            signals::follower_score(tweet.author_followers, self.config.follower_count_max);

        let recency = signals::recency_score_at(&tweet.created_at, self.config.recency_max, now);

        let engagement = signals::engagement_rate(
            tweet.likes,
            tweet.retweets,
            tweet.replies,
            tweet.author_followers,
            self.config.engagement_rate_max,
        );

        let total = (keyword_relevance + follower + recency + engagement).clamp(0.0, 100.0);
        let meets_threshold = total >= self.config.threshold as f32;

        TweetScore {
            total,
            keyword_relevance,
            follower,
            recency,
            engagement,
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

/// Find which keywords matched a tweet (case-insensitive).
///
/// Returns the subset of keywords present in the tweet text.
/// Used for display purposes -- the actual scoring uses weighted counts.
pub fn find_matched_keywords(tweet_text: &str, keywords: &[String]) -> Vec<String> {
    let text_lower = tweet_text.to_lowercase();
    keywords
        .iter()
        .filter(|kw| text_lower.contains(&kw.to_lowercase()))
        .cloned()
        .collect()
}

/// Format a follower count for display.
///
/// Examples: 500 -> "500", 1200 -> "1.2K", 45300 -> "45.3K", 1200000 -> "1.2M".
pub fn format_follower_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

/// Format a tweet's age for display.
///
/// Parses the ISO-8601 timestamp and returns a human-readable duration
/// like "12 minutes", "2 hours", "1 day". Returns "unknown" on parse failure.
pub fn format_tweet_age(created_at: &str) -> String {
    format_tweet_age_at(created_at, Utc::now())
}

/// Format a tweet's age relative to a specific time (for testability).
pub fn format_tweet_age_at(created_at: &str, now: DateTime<Utc>) -> String {
    let created = match created_at.parse::<DateTime<Utc>>() {
        Ok(dt) => dt,
        Err(_) => return "unknown".to_string(),
    };

    let duration = now - created;
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();

    if minutes < 1 {
        let secs = duration.num_seconds().max(0);
        format!("{secs} seconds")
    } else if minutes < 60 {
        format!("{minutes} minutes")
    } else if hours < 24 {
        format!("{hours} hours")
    } else {
        format!("{days} days")
    }
}

/// Truncate text for display, appending "..." if truncated.
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
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

        format!(
            "Tweet: \"{}\" by @{} ({} followers)\n\
             Score: {:.0}/100\n\
             \x20 Keyword relevance:  {:.0}/{}  (matched: {})\n\
             \x20 Author reach:       {:.0}/{}  ({} followers, log scale)\n\
             \x20 Recency:            {:.0}/{}  (posted {} ago)\n\
             \x20 Engagement rate:    {:.0}/{}  ({:.1}% engagement vs 1.5% baseline)\n\
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
            verdict,
            config.threshold,
        )
    }
}

impl std::fmt::Display for TweetScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Score: {:.0}/100 [kw:{:.0} fol:{:.0} rec:{:.0} eng:{:.0}] {}",
            self.total,
            self.keyword_relevance,
            self.follower,
            self.recency,
            self.engagement,
            if self.meets_threshold {
                "REPLY"
            } else {
                "SKIP"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScoringConfig;
    use chrono::Duration;

    fn default_scoring_config() -> ScoringConfig {
        ScoringConfig {
            threshold: 70,
            keyword_relevance_max: 40.0,
            follower_count_max: 20.0,
            recency_max: 15.0,
            engagement_rate_max: 25.0,
        }
    }

    fn test_tweet(now: DateTime<Utc>) -> TweetData {
        TweetData {
            text: "Building amazing Rust CLI tools for developers".to_string(),
            created_at: (now - Duration::minutes(10)).to_rfc3339(),
            likes: 20,
            retweets: 5,
            replies: 3,
            author_username: "devuser".to_string(),
            author_followers: 5000,
        }
    }

    // --- ScoringEngine tests ---

    #[test]
    fn score_total_is_sum_of_signals() {
        let config = default_scoring_config();
        let keywords = vec!["rust".to_string(), "cli".to_string()];
        let engine = ScoringEngine::new(config, keywords);
        let now = Utc::now();
        let tweet = test_tweet(now);

        let score = engine.score_tweet_at(&tweet, now);
        let expected_total =
            score.keyword_relevance + score.follower + score.recency + score.engagement;
        assert!((score.total - expected_total).abs() < 0.01);
    }

    #[test]
    fn score_total_clamped_to_100() {
        // Use very high max values to force total > 100
        let config = ScoringConfig {
            threshold: 70,
            keyword_relevance_max: 80.0,
            follower_count_max: 80.0,
            recency_max: 80.0,
            engagement_rate_max: 80.0,
        };
        let keywords = vec!["rust".to_string()];
        let engine = ScoringEngine::new(config, keywords);
        let now = Utc::now();
        let tweet = test_tweet(now);

        let score = engine.score_tweet_at(&tweet, now);
        assert!(score.total <= 100.0);
    }

    #[test]
    fn score_meets_threshold_above() {
        let config = ScoringConfig {
            threshold: 30,
            ..default_scoring_config()
        };
        let keywords = vec!["rust".to_string(), "cli".to_string()];
        let engine = ScoringEngine::new(config, keywords);
        let now = Utc::now();
        let tweet = test_tweet(now);

        let score = engine.score_tweet_at(&tweet, now);
        assert!(score.total >= 30.0);
        assert!(score.meets_threshold);
    }

    #[test]
    fn score_meets_threshold_below() {
        let config = ScoringConfig {
            threshold: 99,
            ..default_scoring_config()
        };
        let keywords = vec!["nonexistent".to_string()];
        let engine = ScoringEngine::new(config, keywords);
        let now = Utc::now();
        let mut tweet = test_tweet(now);
        tweet.created_at = (now - Duration::hours(12)).to_rfc3339();
        tweet.likes = 0;
        tweet.retweets = 0;
        tweet.replies = 0;

        let score = engine.score_tweet_at(&tweet, now);
        assert!(!score.meets_threshold);
    }

    #[test]
    fn score_with_no_keywords() {
        let config = default_scoring_config();
        let engine = ScoringEngine::new(config, vec![]);
        let now = Utc::now();
        let tweet = test_tweet(now);

        let score = engine.score_tweet_at(&tweet, now);
        assert_eq!(score.keyword_relevance, 0.0);
    }

    // --- find_matched_keywords tests ---

    #[test]
    fn find_matched_some() {
        let keywords = vec!["rust".to_string(), "python".to_string(), "cli".to_string()];
        let matched = find_matched_keywords("Building a Rust CLI tool", &keywords);
        assert!(matched.contains(&"rust".to_string()));
        assert!(matched.contains(&"cli".to_string()));
        assert!(!matched.contains(&"python".to_string()));
    }

    #[test]
    fn find_matched_none() {
        let keywords = vec!["java".to_string()];
        let matched = find_matched_keywords("Building a Rust CLI tool", &keywords);
        assert!(matched.is_empty());
    }

    // --- format_follower_count tests ---

    #[test]
    fn format_followers_under_1k() {
        assert_eq!(format_follower_count(500), "500");
    }

    #[test]
    fn format_followers_1k() {
        assert_eq!(format_follower_count(1200), "1.2K");
    }

    #[test]
    fn format_followers_45k() {
        assert_eq!(format_follower_count(45300), "45.3K");
    }

    #[test]
    fn format_followers_1m() {
        assert_eq!(format_follower_count(1_200_000), "1.2M");
    }

    // --- format_tweet_age tests ---

    #[test]
    fn format_age_seconds() {
        let now = Utc::now();
        let created = (now - Duration::seconds(30)).to_rfc3339();
        assert_eq!(format_tweet_age_at(&created, now), "30 seconds");
    }

    #[test]
    fn format_age_minutes() {
        let now = Utc::now();
        let created = (now - Duration::minutes(12)).to_rfc3339();
        assert_eq!(format_tweet_age_at(&created, now), "12 minutes");
    }

    #[test]
    fn format_age_hours() {
        let now = Utc::now();
        let created = (now - Duration::hours(3)).to_rfc3339();
        assert_eq!(format_tweet_age_at(&created, now), "3 hours");
    }

    #[test]
    fn format_age_days() {
        let now = Utc::now();
        let created = (now - Duration::days(2)).to_rfc3339();
        assert_eq!(format_tweet_age_at(&created, now), "2 days");
    }

    #[test]
    fn format_age_invalid() {
        assert_eq!(format_tweet_age_at("bad", Utc::now()), "unknown");
    }

    // --- truncate_text tests ---

    #[test]
    fn truncate_short_text() {
        assert_eq!(truncate_text("short", 50), "short");
    }

    #[test]
    fn truncate_long_text() {
        let text = "This is a very long tweet that needs to be truncated for display";
        let result = truncate_text(text, 20);
        assert_eq!(result, "This is a very long ...");
        assert!(result.len() <= 23); // 20 + "..."
    }

    // --- format_breakdown tests ---

    #[test]
    fn format_breakdown_contains_verdict() {
        let config = default_scoring_config();
        let now = Utc::now();
        let tweet = test_tweet(now);
        let score = TweetScore {
            total: 75.0,
            keyword_relevance: 30.0,
            follower: 15.0,
            recency: 12.0,
            engagement: 18.0,
            meets_threshold: true,
        };

        let output = score.format_breakdown(&config, &tweet, &["rust".to_string()]);
        assert!(output.contains("REPLY"));
        assert!(output.contains("75/100"));
        assert!(output.contains("@devuser"));
    }

    #[test]
    fn format_breakdown_skip_verdict() {
        let config = default_scoring_config();
        let now = Utc::now();
        let tweet = test_tweet(now);
        let score = TweetScore {
            total: 40.0,
            keyword_relevance: 10.0,
            follower: 10.0,
            recency: 10.0,
            engagement: 10.0,
            meets_threshold: false,
        };

        let output = score.format_breakdown(&config, &tweet, &[]);
        assert!(output.contains("SKIP"));
        assert!(output.contains("40/100"));
    }

    // --- Display impl tests ---

    #[test]
    fn display_impl() {
        let score = TweetScore {
            total: 75.0,
            keyword_relevance: 30.0,
            follower: 15.0,
            recency: 12.0,
            engagement: 18.0,
            meets_threshold: true,
        };
        let display = format!("{score}");
        assert!(display.contains("75/100"));
        assert!(display.contains("REPLY"));
    }
}
