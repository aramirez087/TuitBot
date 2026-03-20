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

// --- ScoringEngine tests ---

#[test]
fn score_total_is_sum_of_signals() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string(), "cli".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet_at(&tweet, now);
    let expected_total = score.keyword_relevance
        + score.follower
        + score.recency
        + score.engagement
        + score.reply_count
        + score.content_type;
    assert!((score.total - expected_total).abs() < 0.01);
}

#[test]
fn score_total_clamped_to_100() {
    let config = ScoringConfig {
        threshold: 70,
        keyword_relevance_max: 80.0,
        follower_count_max: 80.0,
        recency_max: 80.0,
        engagement_rate_max: 80.0,
        reply_count_max: 80.0,
        content_type_max: 80.0,
    };
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet_at(&tweet, now);
    assert!(score.total <= 100.0);
}

#[test]
fn score_total_includes_new_signals() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let tweet = test_tweet(now);
    let score = engine.score_tweet_at(&tweet, now);
    assert!(score.reply_count > 0.0);
    assert!(score.content_type > 0.0);
}

#[test]
fn score_zero_reply_higher_than_many_replies() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();

    let mut tweet_few = test_tweet(now);
    tweet_few.replies = 0;
    let mut tweet_many = test_tweet(now);
    tweet_many.replies = 50;

    let score_few = engine.score_tweet_at(&tweet_few, now);
    let score_many = engine.score_tweet_at(&tweet_many, now);
    assert!(score_few.total > score_many.total);
}

#[test]
fn score_1k_follower_higher_than_100k() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();

    let mut tweet_1k = test_tweet(now);
    tweet_1k.author_followers = 1_000;
    let mut tweet_100k = test_tweet(now);
    tweet_100k.author_followers = 100_000;

    let score_1k = engine.score_tweet_at(&tweet_1k, now);
    let score_100k = engine.score_tweet_at(&tweet_100k, now);
    assert!(
        score_1k.follower > score_100k.follower,
        "1K ({:.1}) should beat 100K ({:.1})",
        score_1k.follower,
        score_100k.follower
    );
}

#[test]
fn score_quote_tweet_zero_content_type() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();

    let mut tweet = test_tweet(now);
    tweet.is_quote_tweet = true;

    let score = engine.score_tweet_at(&tweet, now);
    assert!((score.content_type - 0.0).abs() < 0.01);
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
    assert!(result.len() <= 23);
}

// --- format_breakdown tests ---

#[test]
fn format_breakdown_contains_verdict() {
    let config = default_scoring_config();
    let now = Utc::now();
    let tweet = test_tweet(now);
    let score = TweetScore {
        total: 75.0,
        keyword_relevance: 20.0,
        follower: 12.0,
        recency: 8.0,
        engagement: 10.0,
        reply_count: 15.0,
        content_type: 10.0,
        meets_threshold: true,
    };

    let output = score.format_breakdown(&config, &tweet, &["rust".to_string()]);
    assert!(output.contains("REPLY"));
    assert!(output.contains("75/100"));
    assert!(output.contains("@devuser"));
    assert!(output.contains("Reply count"));
    assert!(output.contains("Content type"));
}

#[test]
fn format_breakdown_skip_verdict() {
    let config = default_scoring_config();
    let now = Utc::now();
    let tweet = test_tweet(now);
    let score = TweetScore {
        total: 40.0,
        keyword_relevance: 10.0,
        follower: 8.0,
        recency: 5.0,
        engagement: 7.0,
        reply_count: 5.0,
        content_type: 5.0,
        meets_threshold: false,
    };

    let output = score.format_breakdown(&config, &tweet, &[]);
    assert!(output.contains("SKIP"));
    assert!(output.contains("40/100"));
}

// --- ScoringEngine accessors ---

#[test]
fn engine_keywords_accessor() {
    let keywords = vec!["rust".to_string(), "cli".to_string()];
    let engine = ScoringEngine::new(default_scoring_config(), keywords.clone());
    assert_eq!(engine.keywords(), &keywords);
}

#[test]
fn engine_config_accessor() {
    let config = default_scoring_config();
    let engine = ScoringEngine::new(config.clone(), vec![]);
    assert_eq!(engine.config().threshold, 60);
    assert!((engine.config().keyword_relevance_max - 25.0).abs() < 0.01);
}

#[test]
fn score_text_only_higher_than_media() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();

    let text_tweet = test_tweet(now);
    let mut media_tweet = test_tweet(now);
    media_tweet.has_media = true;

    let text_score = engine.score_tweet_at(&text_tweet, now);
    let media_score = engine.score_tweet_at(&media_tweet, now);
    assert!(text_score.total > media_score.total);
}

#[test]
fn score_recent_tweet_higher_than_old() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();

    let recent_tweet = test_tweet(now);
    let mut old_tweet = test_tweet(now);
    old_tweet.created_at = (now - Duration::hours(8)).to_rfc3339();

    let recent_score = engine.score_tweet_at(&recent_tweet, now);
    let old_score = engine.score_tweet_at(&old_tweet, now);
    assert!(recent_score.recency > old_score.recency);
}

// --- find_matched_keywords edge cases ---

#[test]
fn find_matched_case_insensitive() {
    let keywords = vec!["RUST".to_string()];
    let matched = find_matched_keywords("rust is great", &keywords);
    assert_eq!(matched, vec!["RUST".to_string()]);
}

#[test]
fn find_matched_empty_keywords() {
    let matched = find_matched_keywords("some text", &[]);
    assert!(matched.is_empty());
}

// --- format_follower_count edge cases ---

#[test]
fn format_followers_zero() {
    assert_eq!(format_follower_count(0), "0");
}

#[test]
fn format_followers_exact_1k() {
    assert_eq!(format_follower_count(1000), "1.0K");
}

#[test]
fn format_followers_exact_1m() {
    assert_eq!(format_follower_count(1_000_000), "1.0M");
}

// --- truncate_text edge cases ---

#[test]
fn truncate_exact_length() {
    assert_eq!(truncate_text("hello", 5), "hello");
}

#[test]
fn truncate_empty() {
    assert_eq!(truncate_text("", 10), "");
}

#[test]
fn truncate_one_char() {
    assert_eq!(truncate_text("x", 1), "x");
}

// --- Display impl tests ---

#[test]
fn display_impl() {
    let score = TweetScore {
        total: 75.0,
        keyword_relevance: 20.0,
        follower: 12.0,
        recency: 8.0,
        engagement: 10.0,
        reply_count: 15.0,
        content_type: 10.0,
        meets_threshold: true,
    };
    let display = format!("{score}");
    assert!(display.contains("75/100"));
    assert!(display.contains("REPLY"));
    assert!(display.contains("rep:"));
    assert!(display.contains("ct:"));
}

#[test]
fn display_impl_skip() {
    let score = TweetScore {
        total: 30.0,
        keyword_relevance: 5.0,
        follower: 5.0,
        recency: 5.0,
        engagement: 5.0,
        reply_count: 5.0,
        content_type: 5.0,
        meets_threshold: false,
    };
    let display = format!("{score}");
    assert!(display.contains("30/100"));
    assert!(display.contains("SKIP"));
}

#[test]
fn score_tweet_uses_current_time() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet(&tweet);
    assert!(score.total > 0.0);
}

#[test]
fn score_zero_followers_no_panic() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.author_followers = 0;

    let score = engine.score_tweet_at(&tweet, now);
    assert_eq!(score.follower, 0.0);
    assert!(score.total >= 0.0);
}

#[test]
fn score_empty_text_no_keyword_match() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.text = String::new();

    let score = engine.score_tweet_at(&tweet, now);
    assert_eq!(score.keyword_relevance, 0.0);
}

#[test]
fn format_breakdown_media_tweet() {
    let config = default_scoring_config();
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.has_media = true;

    let score = TweetScore {
        total: 50.0,
        keyword_relevance: 15.0,
        follower: 10.0,
        recency: 8.0,
        engagement: 7.0,
        reply_count: 10.0,
        content_type: 0.0,
        meets_threshold: false,
    };

    let output = score.format_breakdown(&config, &tweet, &[]);
    assert!(output.contains("media/quote"));
    assert!(output.contains("SKIP"));
}

#[test]
fn format_breakdown_no_keywords_matched() {
    let config = default_scoring_config();
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = TweetScore {
        total: 60.0,
        keyword_relevance: 0.0,
        follower: 15.0,
        recency: 10.0,
        engagement: 15.0,
        reply_count: 10.0,
        content_type: 10.0,
        meets_threshold: true,
    };

    let output = score.format_breakdown(&config, &tweet, &[]);
    assert!(output.contains("none"));
}

#[test]
fn find_matched_partial_text() {
    let keywords = vec!["rust programming".to_string()];
    let matched = find_matched_keywords("I love rust programming", &keywords);
    assert_eq!(matched.len(), 1);
}

#[test]
fn format_followers_999() {
    assert_eq!(format_follower_count(999), "999");
}

#[test]
fn format_followers_large_m() {
    assert_eq!(format_follower_count(5_500_000), "5.5M");
}

#[test]
fn format_age_zero_seconds() {
    let now = Utc::now();
    let created = now.to_rfc3339();
    let result = format_tweet_age_at(&created, now);
    assert_eq!(result, "0 seconds");
}

#[test]
fn format_age_boundary_59_minutes() {
    let now = Utc::now();
    let created = (now - Duration::minutes(59)).to_rfc3339();
    assert_eq!(format_tweet_age_at(&created, now), "59 minutes");
}

#[test]
fn format_age_boundary_60_minutes() {
    let now = Utc::now();
    let created = (now - Duration::minutes(60)).to_rfc3339();
    assert_eq!(format_tweet_age_at(&created, now), "1 hours");
}

#[test]
fn format_age_boundary_23_hours() {
    let now = Utc::now();
    let created = (now - Duration::hours(23)).to_rfc3339();
    assert_eq!(format_tweet_age_at(&created, now), "23 hours");
}

#[test]
fn format_age_boundary_24_hours() {
    let now = Utc::now();
    let created = (now - Duration::hours(24)).to_rfc3339();
    assert_eq!(format_tweet_age_at(&created, now), "1 days");
}

#[test]
fn format_tweet_age_uses_current_time() {
    let now = Utc::now();
    let created = (now - Duration::minutes(5)).to_rfc3339();
    let result = format_tweet_age(&created);
    assert!(result.contains("minutes") || result.contains("seconds"));
}

#[test]
fn tweet_score_clone() {
    let score = TweetScore {
        total: 50.0,
        keyword_relevance: 10.0,
        follower: 10.0,
        recency: 10.0,
        engagement: 10.0,
        reply_count: 5.0,
        content_type: 5.0,
        meets_threshold: false,
    };
    let cloned = score.clone();
    assert!((cloned.total - score.total).abs() < 0.01);
    assert_eq!(cloned.meets_threshold, score.meets_threshold);
}

#[test]
fn tweet_data_clone() {
    let now = Utc::now();
    let tweet = test_tweet(now);
    let cloned = tweet.clone();
    assert_eq!(cloned.text, tweet.text);
    assert_eq!(cloned.author_followers, tweet.author_followers);
}

#[test]
fn score_high_engagement_tweet() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string(), "cli".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.likes = 500;
    tweet.retweets = 100;
    tweet.replies = 50;

    let score = engine.score_tweet_at(&tweet, now);
    assert!(score.engagement > 0.0);
}

#[test]
fn format_breakdown_multiple_keywords() {
    let config = default_scoring_config();
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = TweetScore {
        total: 70.0,
        keyword_relevance: 20.0,
        follower: 10.0,
        recency: 10.0,
        engagement: 10.0,
        reply_count: 10.0,
        content_type: 10.0,
        meets_threshold: true,
    };

    let keywords = vec!["rust".to_string(), "cli".to_string()];
    let output = score.format_breakdown(&config, &tweet, &keywords);
    assert!(output.contains("rust, cli"));
}

#[test]
fn score_very_old_tweet_low_recency() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.created_at = (now - Duration::days(30)).to_rfc3339();

    let score = engine.score_tweet_at(&tweet, now);
    assert!(
        score.recency < 2.0,
        "30-day old tweet should have near-zero recency"
    );
}

#[test]
fn score_zero_engagement_tweet() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.likes = 0;
    tweet.retweets = 0;
    tweet.replies = 0;

    let score = engine.score_tweet_at(&tweet, now);
    assert!(score.engagement >= 0.0);
    assert!(score.total >= 0.0);
}

#[test]
fn score_massive_followers_low_follower_score() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.author_followers = 10_000_000;

    let score = engine.score_tweet_at(&tweet, now);
    assert!(score.follower < 5.0);
}

#[test]
fn score_optimal_followers_high_score() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.author_followers = 3000;

    let score = engine.score_tweet_at(&tweet, now);
    assert!(
        score.follower > 5.0,
        "3K followers should score well on bell curve"
    );
}

#[test]
fn score_multiple_keyword_matches() {
    let config = default_scoring_config();
    let keywords = vec![
        "rust".to_string(),
        "cli".to_string(),
        "tools".to_string(),
        "developers".to_string(),
    ];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet_at(&tweet, now);
    assert!(score.keyword_relevance > 0.0);
}

#[test]
fn score_no_keyword_matches() {
    let config = default_scoring_config();
    let keywords = vec!["python".to_string(), "java".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet_at(&tweet, now);
    assert_eq!(score.keyword_relevance, 0.0);
}

#[test]
fn score_has_media_zero_content_type() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.has_media = true;

    let score = engine.score_tweet_at(&tweet, now);
    assert!((score.content_type - 0.0).abs() < 0.01);
}

#[test]
fn score_text_only_max_content_type() {
    let config = default_scoring_config();
    let expected_max = config.content_type_max;
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet_at(&tweet, now);
    assert!((score.content_type - expected_max).abs() < 0.01);
}

#[test]
fn score_high_reply_count_low_reply_score() {
    let config = default_scoring_config();
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.replies = 100;

    let score = engine.score_tweet_at(&tweet, now);
    assert!(
        score.reply_count < 5.0,
        "100 replies should yield low reply score"
    );
}

#[test]
fn score_zero_replies_max_reply_score() {
    let config = default_scoring_config();
    let expected_max = config.reply_count_max;
    let keywords = vec!["rust".to_string()];
    let engine = ScoringEngine::new(config, keywords);
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.replies = 0;

    let score = engine.score_tweet_at(&tweet, now);
    assert!((score.reply_count - expected_max).abs() < 0.01);
}

#[test]
fn score_threshold_exactly_at_boundary() {
    let config = ScoringConfig {
        threshold: 0,
        ..default_scoring_config()
    };
    let engine = ScoringEngine::new(config, vec![]);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet_at(&tweet, now);
    assert!(score.meets_threshold);
}

#[test]
fn score_threshold_100_nearly_impossible() {
    let config = ScoringConfig {
        threshold: 100,
        ..default_scoring_config()
    };
    let engine = ScoringEngine::new(config, vec!["nonexistent".to_string()]);
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = engine.score_tweet_at(&tweet, now);
    assert!(!score.meets_threshold || score.total >= 100.0);
}

#[test]
fn find_matched_keywords_substring() {
    let keywords = vec!["rust".to_string()];
    let matched = find_matched_keywords("trustworthy tool", &keywords);
    assert_eq!(matched.len(), 1);
}

#[test]
fn find_matched_keywords_multiple_occurrences() {
    let keywords = vec!["rust".to_string()];
    let matched = find_matched_keywords("Rust is great. I love Rust!", &keywords);
    assert_eq!(matched.len(), 1);
}

#[test]
fn format_follower_count_boundary_values() {
    assert_eq!(format_follower_count(1), "1");
    assert_eq!(format_follower_count(999), "999");
    assert_eq!(format_follower_count(1000), "1.0K");
    assert_eq!(format_follower_count(999_999), "1000.0K");
    assert_eq!(format_follower_count(1_000_000), "1.0M");
    assert_eq!(format_follower_count(10_000_000), "10.0M");
}

#[test]
fn format_age_negative_duration_handled() {
    let now = Utc::now();
    let future = (now + Duration::hours(1)).to_rfc3339();
    let result = format_tweet_age_at(&future, now);
    assert_eq!(result, "0 seconds");
}

#[test]
fn truncate_text_boundary_at_max() {
    let text = "abcde";
    assert_eq!(truncate_text(text, 5), "abcde");
    assert_eq!(truncate_text(text, 4), "abcd...");
    assert_eq!(truncate_text(text, 6), "abcde");
}

#[test]
fn truncate_text_max_zero() {
    assert_eq!(truncate_text("hello", 0), "...");
}

#[test]
fn display_impl_all_zero_scores() {
    let score = TweetScore {
        total: 0.0,
        keyword_relevance: 0.0,
        follower: 0.0,
        recency: 0.0,
        engagement: 0.0,
        reply_count: 0.0,
        content_type: 0.0,
        meets_threshold: false,
    };
    let display = format!("{score}");
    assert!(display.contains("0/100"));
    assert!(display.contains("SKIP"));
}

#[test]
fn display_impl_max_scores() {
    let score = TweetScore {
        total: 100.0,
        keyword_relevance: 25.0,
        follower: 15.0,
        recency: 10.0,
        engagement: 15.0,
        reply_count: 15.0,
        content_type: 10.0,
        meets_threshold: true,
    };
    let display = format!("{score}");
    assert!(display.contains("100/100"));
    assert!(display.contains("REPLY"));
}

#[test]
fn tweet_data_debug_format() {
    let now = Utc::now();
    let tweet = test_tweet(now);
    let debug = format!("{:?}", tweet);
    assert!(debug.contains("TweetData"));
    assert!(debug.contains("devuser"));
}

#[test]
fn tweet_score_debug_format() {
    let score = TweetScore {
        total: 42.0,
        keyword_relevance: 10.0,
        follower: 8.0,
        recency: 5.0,
        engagement: 7.0,
        reply_count: 7.0,
        content_type: 5.0,
        meets_threshold: false,
    };
    let debug = format!("{:?}", score);
    assert!(debug.contains("TweetScore"));
    assert!(debug.contains("42"));
}

#[test]
fn format_breakdown_text_only_tweet() {
    let config = default_scoring_config();
    let now = Utc::now();
    let tweet = test_tweet(now);

    let score = TweetScore {
        total: 65.0,
        keyword_relevance: 20.0,
        follower: 12.0,
        recency: 8.0,
        engagement: 10.0,
        reply_count: 10.0,
        content_type: 5.0,
        meets_threshold: true,
    };

    let output = score.format_breakdown(&config, &tweet, &["rust".to_string()]);
    assert!(output.contains("text-only"));
    assert!(output.contains("REPLY"));
    assert!(output.contains("65/100"));
    assert!(output.contains("5.0K"));
}

#[test]
fn format_breakdown_zero_follower_tweet() {
    let config = default_scoring_config();
    let now = Utc::now();
    let mut tweet = test_tweet(now);
    tweet.author_followers = 0;

    let score = TweetScore {
        total: 30.0,
        keyword_relevance: 10.0,
        follower: 0.0,
        recency: 5.0,
        engagement: 5.0,
        reply_count: 5.0,
        content_type: 5.0,
        meets_threshold: false,
    };

    let output = score.format_breakdown(&config, &tweet, &[]);
    assert!(output.contains("0 followers"));
    assert!(output.contains("SKIP"));
}

#[test]
fn engine_config_threshold_accessible() {
    let config = ScoringConfig {
        threshold: 42,
        ..default_scoring_config()
    };
    let engine = ScoringEngine::new(config, vec![]);
    assert_eq!(engine.config().threshold, 42);
}

#[test]
fn engine_keywords_empty() {
    let engine = ScoringEngine::new(default_scoring_config(), vec![]);
    assert!(engine.keywords().is_empty());
}

#[test]
fn engine_keywords_many() {
    let keywords: Vec<String> = (0..50).map(|i| format!("keyword_{i}")).collect();
    let engine = ScoringEngine::new(default_scoring_config(), keywords.clone());
    assert_eq!(engine.keywords().len(), 50);
    assert_eq!(engine.keywords()[25], "keyword_25");
}
