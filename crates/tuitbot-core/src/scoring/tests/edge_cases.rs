//! Edge cases, display, and accessor tests.

use super::*;

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
