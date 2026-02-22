//! Individual scoring signal functions.
//!
//! All functions are pure: same inputs always produce the same outputs.
//! Each signal evaluates one dimension of a tweet's reply-worthiness.

use chrono::{DateTime, Utc};

/// Compute keyword relevance score for a tweet.
///
/// Matches the tweet text (case-insensitive) against the provided keywords.
/// Multi-word keywords (containing spaces) receive 2x weight compared to
/// single-word keywords, since they indicate more specific relevance.
///
/// Returns a score in the range `0.0..=max_score`. Returns 0.0 if keywords is empty.
pub fn keyword_relevance(tweet_text: &str, keywords: &[String], max_score: f32) -> f32 {
    if keywords.is_empty() {
        return 0.0;
    }

    let text_lower = tweet_text.to_lowercase();
    let mut matched_weight: f32 = 0.0;
    let mut max_possible_weight: f32 = 0.0;

    for keyword in keywords {
        let weight = if keyword.contains(' ') { 2.0 } else { 1.0 };
        max_possible_weight += weight;

        if text_lower.contains(&keyword.to_lowercase()) {
            matched_weight += weight;
        }
    }

    if max_possible_weight == 0.0 {
        return 0.0;
    }

    let score = (matched_weight / max_possible_weight) * max_score;
    score.clamp(0.0, max_score)
}

/// Compute follower score using a logarithmic scale.
///
/// Maps follower count to a score where:
/// - 0 followers = 0.0
/// - 100 followers = ~25% of max_score
/// - 1,000 followers = ~50% of max_score
/// - 10,000 followers = ~75% of max_score
/// - 100,000+ followers = max_score
///
/// Uses `log10(max(count, 1)) / 5.0` since `log10(100000) = 5.0`.
pub fn follower_score(follower_count: u64, max_score: f32) -> f32 {
    if follower_count == 0 {
        return 0.0;
    }

    let log_val = (follower_count.max(1) as f64).log10();
    let score = (log_val / 5.0) * max_score as f64;
    (score as f32).clamp(0.0, max_score)
}

/// Compute recency score based on tweet age.
///
/// Uses time brackets with linear interpolation:
/// - 0-5 minutes: 100% of max_score
/// - 5-30 minutes: 80-100% (interpolated)
/// - 30-60 minutes: 50-80% (interpolated)
/// - 1-6 hours: 25-50% (interpolated)
/// - 6+ hours: 0-25% (interpolated, reaching 0 at 12 hours)
///
/// Accepts a `now` parameter for testability.
/// Returns 0.0 if the timestamp fails to parse.
pub fn recency_score_at(tweet_created_at: &str, max_score: f32, now: DateTime<Utc>) -> f32 {
    let created_at = match tweet_created_at.parse::<DateTime<Utc>>() {
        Ok(dt) => dt,
        Err(_) => {
            tracing::warn!(
                timestamp = tweet_created_at,
                "Failed to parse tweet timestamp for recency scoring"
            );
            return 0.0;
        }
    };

    let age_minutes = (now - created_at).num_minutes().max(0) as f64;

    let fraction = if age_minutes <= 5.0 {
        // 0-5 min: 100%
        1.0
    } else if age_minutes <= 30.0 {
        // 5-30 min: 80-100%, linearly interpolated
        let t = (age_minutes - 5.0) / 25.0;
        1.0 - t * 0.2
    } else if age_minutes <= 60.0 {
        // 30-60 min: 50-80%
        let t = (age_minutes - 30.0) / 30.0;
        0.8 - t * 0.3
    } else if age_minutes <= 360.0 {
        // 1-6 hours: 25-50%
        let t = (age_minutes - 60.0) / 300.0;
        0.5 - t * 0.25
    } else {
        // 6+ hours: 0%
        0.0
    };

    (fraction as f32 * max_score).clamp(0.0, max_score)
}

/// Convenience wrapper for `recency_score_at` using the current time.
pub fn recency_score(tweet_created_at: &str, max_score: f32) -> f32 {
    recency_score_at(tweet_created_at, max_score, Utc::now())
}

/// Compute engagement rate score.
///
/// Calculates `(likes + retweets + replies) / max(followers, 1)` and
/// maps it to a score based on a 5% ceiling (rates above 5% get max score).
///
/// The baseline engagement rate on X is ~1.5%; tweets above 5% are
/// considered high-engagement.
pub fn engagement_rate(
    likes: u64,
    retweets: u64,
    replies: u64,
    follower_count: u64,
    max_score: f32,
) -> f32 {
    let total_engagement = (likes + retweets + replies) as f64;
    let followers = follower_count.max(1) as f64;
    let rate = total_engagement / followers;

    let score = (rate / 0.05).min(1.0) * max_score as f64;
    (score as f32).clamp(0.0, max_score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // --- keyword_relevance tests ---

    #[test]
    fn keyword_empty_keywords() {
        assert_eq!(keyword_relevance("some tweet text", &[], 40.0), 0.0);
    }

    #[test]
    fn keyword_no_match() {
        let keywords = vec!["rust".to_string(), "cli".to_string()];
        assert_eq!(keyword_relevance("python is great", &keywords, 40.0), 0.0);
    }

    #[test]
    fn keyword_single_word_match() {
        let keywords = vec!["rust".to_string(), "cli".to_string()];
        // "rust" matches (weight 1), "cli" doesn't. max_possible=2, matched=1
        let score = keyword_relevance("I love Rust programming", &keywords, 40.0);
        assert!((score - 20.0).abs() < 0.01);
    }

    #[test]
    fn keyword_all_match() {
        let keywords = vec!["rust".to_string(), "cli".to_string()];
        let score = keyword_relevance("Building a Rust CLI tool", &keywords, 40.0);
        assert!((score - 40.0).abs() < 0.01);
    }

    #[test]
    fn keyword_multi_word_double_weight() {
        let keywords = vec!["mac".to_string(), "menu bar apps".to_string()];
        // Both match: "mac" weight=1, "menu bar apps" weight=2, max_possible=3, matched=3
        let score = keyword_relevance("I love mac menu bar apps for productivity", &keywords, 40.0);
        assert!((score - 40.0).abs() < 0.01);
    }

    #[test]
    fn keyword_multi_word_only() {
        let keywords = vec!["mac".to_string(), "menu bar apps".to_string()];
        // Only "mac" matches: weight=1, max_possible=3, matched=1
        let score = keyword_relevance("My mac is slow", &keywords, 40.0);
        let expected = (1.0 / 3.0) * 40.0;
        assert!((score - expected).abs() < 0.01);
    }

    #[test]
    fn keyword_case_insensitive() {
        let keywords = vec!["RUST".to_string()];
        let score = keyword_relevance("rust is awesome", &keywords, 40.0);
        assert!((score - 40.0).abs() < 0.01);
    }

    // --- follower_score tests ---

    #[test]
    fn follower_zero() {
        assert_eq!(follower_score(0, 20.0), 0.0);
    }

    #[test]
    fn follower_100() {
        let score = follower_score(100, 20.0);
        // log10(100)/5.0 = 2/5 = 0.4, * 20 = 8.0
        assert!((score - 8.0).abs() < 0.1);
    }

    #[test]
    fn follower_1000() {
        let score = follower_score(1000, 20.0);
        // log10(1000)/5.0 = 3/5 = 0.6, * 20 = 12.0
        assert!((score - 12.0).abs() < 0.1);
    }

    #[test]
    fn follower_10000() {
        let score = follower_score(10000, 20.0);
        // log10(10000)/5.0 = 4/5 = 0.8, * 20 = 16.0
        assert!((score - 16.0).abs() < 0.1);
    }

    #[test]
    fn follower_100000() {
        let score = follower_score(100000, 20.0);
        // log10(100000)/5.0 = 5/5 = 1.0, * 20 = 20.0
        assert!((score - 20.0).abs() < 0.1);
    }

    #[test]
    fn follower_million_clamped() {
        let score = follower_score(1_000_000, 20.0);
        // log10(1M)/5.0 = 6/5 = 1.2, but clamped to 20.0
        assert!((score - 20.0).abs() < 0.01);
    }

    // --- recency_score tests ---

    #[test]
    fn recency_1_minute_ago() {
        let now = Utc::now();
        let created = (now - Duration::minutes(1)).to_rfc3339();
        let score = recency_score_at(&created, 15.0, now);
        // 1 min: within 0-5 bracket, should be 100%
        assert!((score - 15.0).abs() < 0.5);
    }

    #[test]
    fn recency_15_minutes_ago() {
        let now = Utc::now();
        let created = (now - Duration::minutes(15)).to_rfc3339();
        let score = recency_score_at(&created, 15.0, now);
        // 15 min: in 5-30 bracket, interpolated ~92%
        assert!(score > 12.0 && score < 15.0);
    }

    #[test]
    fn recency_45_minutes_ago() {
        let now = Utc::now();
        let created = (now - Duration::minutes(45)).to_rfc3339();
        let score = recency_score_at(&created, 15.0, now);
        // 45 min: in 30-60 bracket, interpolated ~65%
        assert!(score > 7.0 && score < 12.0);
    }

    #[test]
    fn recency_3_hours_ago() {
        let now = Utc::now();
        let created = (now - Duration::hours(3)).to_rfc3339();
        let score = recency_score_at(&created, 15.0, now);
        // 3 hours: in 1-6 bracket, interpolated ~38%
        assert!(score > 3.0 && score < 8.0);
    }

    #[test]
    fn recency_12_hours_ago() {
        let now = Utc::now();
        let created = (now - Duration::hours(12)).to_rfc3339();
        let score = recency_score_at(&created, 15.0, now);
        // 12 hours: beyond 6 hour bracket, should be 0
        assert!((score - 0.0).abs() < 0.01);
    }

    #[test]
    fn recency_invalid_timestamp() {
        let now = Utc::now();
        let score = recency_score_at("not-a-timestamp", 15.0, now);
        assert_eq!(score, 0.0);
    }

    // --- engagement_rate tests ---

    #[test]
    fn engagement_zero() {
        let score = engagement_rate(0, 0, 0, 1000, 25.0);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn engagement_average_1_5_percent() {
        // 15 likes on 1000 followers = 1.5%
        let score = engagement_rate(15, 0, 0, 1000, 25.0);
        // rate=0.015, score = (0.015/0.05)*25 = 7.5
        assert!((score - 7.5).abs() < 0.1);
    }

    #[test]
    fn engagement_high_5_percent() {
        // 50 likes on 1000 followers = 5%
        let score = engagement_rate(50, 0, 0, 1000, 25.0);
        assert!((score - 25.0).abs() < 0.1);
    }

    #[test]
    fn engagement_above_ceiling() {
        // 100 likes on 1000 followers = 10%, clamped to max
        let score = engagement_rate(100, 0, 0, 1000, 25.0);
        assert!((score - 25.0).abs() < 0.01);
    }

    #[test]
    fn engagement_zero_followers() {
        // Avoids division by zero, uses max(0,1) = 1
        let score = engagement_rate(10, 5, 2, 0, 25.0);
        // rate = 17/1 = 17.0, way above 5% ceiling
        assert!((score - 25.0).abs() < 0.01);
    }

    #[test]
    fn engagement_all_metrics() {
        // 10 likes + 5 retweets + 3 replies = 18 engagements on 1000 followers = 1.8%
        let score = engagement_rate(10, 5, 3, 1000, 25.0);
        // rate=0.018, score = (0.018/0.05)*25 = 9.0
        assert!((score - 9.0).abs() < 0.1);
    }
}
