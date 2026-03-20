//! Format helper and accessor tests.

use super::*;

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
