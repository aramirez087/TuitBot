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

/// Compute reply count score — fewer existing replies = higher score.
///
/// Targets underserved conversations where a reply is more likely to be seen.
/// - 0 replies = max_score (100%)
/// - 5 replies = 50% of max_score
/// - 20+ replies = 0% (conversation already crowded)
pub fn reply_count_score(reply_count: u64, max_score: f32) -> f32 {
    if reply_count >= 20 {
        return 0.0;
    }
    // Linear decay: score = max_score * (1 - count/20)
    let fraction = 1.0 - (reply_count as f64 / 20.0);
    (fraction as f32 * max_score).clamp(0.0, max_score)
}

/// Compute targeted follower score using a bell curve.
///
/// Peaks at ~1K followers, drops off for very small (<100) and
/// very large (>10K) accounts. This targets the mid-range "emerging
/// voices" who are most likely to engage back.
///
/// - <100 followers: ramp up from 0 to 50%
/// - 100-1K: ramp up from 50% to 100%
/// - 1K-10K: 100% (sweet spot)
/// - 10K-100K: decay from 100% to 25%
/// - 100K+: 25% (still some value for visibility)
pub fn targeted_follower_score(follower_count: u64, max_score: f32) -> f32 {
    if follower_count == 0 {
        return 0.0;
    }

    let fraction = if follower_count < 100 {
        // Ramp from 0% to 50%
        follower_count as f64 / 200.0
    } else if follower_count < 1_000 {
        // Ramp from 50% to 100%
        0.5 + (follower_count as f64 - 100.0) / 1_800.0
    } else if follower_count <= 10_000 {
        // Sweet spot: 100%
        1.0
    } else if follower_count <= 100_000 {
        // Decay from 100% to 25%
        let t = (follower_count as f64 - 10_000.0) / 90_000.0;
        1.0 - t * 0.75
    } else {
        // Floor at 25%
        0.25
    };

    (fraction as f32 * max_score).clamp(0.0, max_score)
}

/// Compute content type score.
///
/// Text-only original tweets score highest. Media, quotes, and retweets
/// score 0 because they are harder to reply to meaningfully.
///
/// - `has_media` = false, `is_quote_tweet` = false → max_score
/// - otherwise → 0
pub fn content_type_score(has_media: bool, is_quote_tweet: bool, max_score: f32) -> f32 {
    if has_media || is_quote_tweet {
        0.0
    } else {
        max_score
    }
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
