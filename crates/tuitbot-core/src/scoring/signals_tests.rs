//! Tests for scoring signal functions.

use super::signals::*;
use chrono::{Duration, Utc};

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

// --- reply_count_score tests ---

#[test]
fn reply_count_zero_replies_max_score() {
    let score = reply_count_score(0, 15.0);
    assert!((score - 15.0).abs() < 0.01);
}

#[test]
fn reply_count_5_replies_half() {
    let score = reply_count_score(5, 15.0);
    // (1 - 5/20) * 15 = 0.75 * 15 = 11.25
    assert!((score - 11.25).abs() < 0.01);
}

#[test]
fn reply_count_20_replies_zero() {
    let score = reply_count_score(20, 15.0);
    assert!((score - 0.0).abs() < 0.01);
}

#[test]
fn reply_count_50_replies_still_zero() {
    let score = reply_count_score(50, 15.0);
    assert!((score - 0.0).abs() < 0.01);
}

// --- targeted_follower_score tests ---

#[test]
fn targeted_follower_zero() {
    assert_eq!(targeted_follower_score(0, 15.0), 0.0);
}

#[test]
fn targeted_follower_50_low() {
    let score = targeted_follower_score(50, 15.0);
    // 50/200 * 15 = 3.75
    assert!(score > 0.0 && score < 7.5);
}

#[test]
fn targeted_follower_1000_sweet_spot() {
    let score = targeted_follower_score(1000, 15.0);
    assert!((score - 15.0).abs() < 0.1);
}

#[test]
fn targeted_follower_5000_still_sweet_spot() {
    let score = targeted_follower_score(5000, 15.0);
    assert!((score - 15.0).abs() < 0.01);
}

#[test]
fn targeted_follower_100k_drops() {
    let score_1k = targeted_follower_score(1000, 15.0);
    let score_100k = targeted_follower_score(100_000, 15.0);
    assert!(score_1k > score_100k);
}

#[test]
fn targeted_follower_500k_floor() {
    let score = targeted_follower_score(500_000, 15.0);
    // Floor at 25% = 3.75
    assert!((score - 3.75).abs() < 0.01);
}

// --- content_type_score tests ---

#[test]
fn content_type_text_only_max() {
    assert!((content_type_score(false, false, 10.0) - 10.0).abs() < 0.01);
}

#[test]
fn content_type_with_media_zero() {
    assert!((content_type_score(true, false, 10.0) - 0.0).abs() < 0.01);
}

#[test]
fn content_type_quote_tweet_zero() {
    assert!((content_type_score(false, true, 10.0) - 0.0).abs() < 0.01);
}

#[test]
fn content_type_media_and_quote_zero() {
    assert!((content_type_score(true, true, 10.0) - 0.0).abs() < 0.01);
}

// -----------------------------------------------------------------------
// Additional signals coverage tests
// -----------------------------------------------------------------------

#[test]
fn keyword_relevance_clamped() {
    let keywords = vec!["rust".to_string()];
    let score = keyword_relevance("rust", &keywords, 25.0);
    assert!(score <= 25.0);
}

#[test]
fn keyword_relevance_all_multi_word() {
    let keywords = vec!["rust programming".to_string(), "cli tools".to_string()];
    // Both match, both weight 2. Total weight 4/4 = max
    let score = keyword_relevance("rust programming and cli tools are great", &keywords, 30.0);
    assert!((score - 30.0).abs() < 0.01);
}

#[test]
fn keyword_relevance_partial_multi_word() {
    let keywords = vec![
        "rust programming".to_string(),
        "python scripting".to_string(),
    ];
    // Only "rust programming" matches (weight 2), total max = 4
    let score = keyword_relevance("rust programming is great", &keywords, 40.0);
    assert!((score - 20.0).abs() < 0.01); // 2/4 * 40 = 20
}

#[test]
fn follower_score_1() {
    let score = follower_score(1, 20.0);
    // log10(1) = 0 => score = 0
    assert!((score - 0.0).abs() < 0.01);
}

#[test]
fn follower_score_10() {
    let score = follower_score(10, 20.0);
    // log10(10)/5 = 0.2 * 20 = 4.0
    assert!((score - 4.0).abs() < 0.1);
}

#[test]
fn recency_exactly_5_minutes() {
    let now = Utc::now();
    let created = (now - Duration::minutes(5)).to_rfc3339();
    let score = recency_score_at(&created, 15.0, now);
    // At 5 min boundary: still in 0-5 bracket = 100%
    assert!((score - 15.0).abs() < 0.5);
}

#[test]
fn recency_exactly_30_minutes() {
    let now = Utc::now();
    let created = (now - Duration::minutes(30)).to_rfc3339();
    let score = recency_score_at(&created, 15.0, now);
    // At 30 min: 5-30 bracket, t=1.0 => 80%
    let expected = 0.8 * 15.0;
    assert!((score - expected).abs() < 0.5);
}

#[test]
fn recency_exactly_60_minutes() {
    let now = Utc::now();
    let created = (now - Duration::minutes(60)).to_rfc3339();
    let score = recency_score_at(&created, 15.0, now);
    // At 60 min: 30-60 bracket, t=1.0 => 50%
    let expected = 0.5 * 15.0;
    assert!((score - expected).abs() < 0.5);
}

#[test]
fn recency_exactly_6_hours() {
    let now = Utc::now();
    let created = (now - Duration::hours(6)).to_rfc3339();
    let score = recency_score_at(&created, 15.0, now);
    // At 360 min: 1-6 hour bracket, t=1.0 => 25%
    let expected = 0.25 * 15.0;
    assert!((score - expected).abs() < 0.5);
}

#[test]
fn recency_score_convenience_wrapper() {
    let now = Utc::now();
    let created = (now - Duration::minutes(2)).to_rfc3339();
    let score = recency_score(&created, 10.0);
    // Very recent, should be near max
    assert!(score > 8.0);
}

#[test]
fn reply_count_10_replies() {
    let score = reply_count_score(10, 15.0);
    // (1 - 10/20) * 15 = 0.5 * 15 = 7.5
    assert!((score - 7.5).abs() < 0.01);
}

#[test]
fn reply_count_19_replies() {
    let score = reply_count_score(19, 15.0);
    // (1 - 19/20) * 15 = 0.05 * 15 = 0.75
    assert!((score - 0.75).abs() < 0.01);
}

#[test]
fn reply_count_1_reply() {
    let score = reply_count_score(1, 15.0);
    // (1 - 1/20) * 15 = 0.95 * 15 = 14.25
    assert!((score - 14.25).abs() < 0.01);
}

#[test]
fn targeted_follower_100() {
    let score = targeted_follower_score(100, 15.0);
    // 100 followers: 0.5 * 15 = 7.5
    assert!((score - 7.5).abs() < 0.1);
}

#[test]
fn targeted_follower_500() {
    let score = targeted_follower_score(500, 15.0);
    // 500 followers: 0.5 + (500-100)/1800 = 0.5 + 0.222 = 0.722
    let expected = 0.722 * 15.0;
    assert!((score - expected as f32).abs() < 0.2);
}

#[test]
fn targeted_follower_10000() {
    let score = targeted_follower_score(10_000, 15.0);
    // Sweet spot: 100%
    assert!((score - 15.0).abs() < 0.01);
}

#[test]
fn targeted_follower_50000() {
    let score = targeted_follower_score(50_000, 15.0);
    // 50K: decay zone. t = (50000-10000)/90000 = 0.444
    // fraction = 1.0 - 0.444*0.75 = 0.667
    let expected = 0.667 * 15.0;
    assert!((score - expected as f32).abs() < 0.2);
}

#[test]
fn targeted_follower_1_million() {
    let score = targeted_follower_score(1_000_000, 15.0);
    // > 100K: floor at 25%
    assert!((score - 3.75).abs() < 0.01);
}

#[test]
fn content_type_zero_max_score() {
    assert!((content_type_score(false, false, 0.0) - 0.0).abs() < 0.01);
}

#[test]
fn engagement_mixed_metrics() {
    // 20 likes + 10 RTs + 5 replies = 35 on 1000 followers = 3.5%
    let score = engagement_rate(20, 10, 5, 1000, 25.0);
    // rate=0.035, score = (0.035/0.05)*25 = 17.5
    assert!((score - 17.5).abs() < 0.1);
}

#[test]
fn engagement_1_like_1_follower() {
    // edge case: 1 engagement on 1 follower = 100%
    let score = engagement_rate(1, 0, 0, 1, 25.0);
    // rate=1.0, capped at 1.0 => max
    assert!((score - 25.0).abs() < 0.01);
}
