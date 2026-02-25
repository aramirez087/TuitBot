//! Default values for all configuration sections.
//!
//! These defaults match the values specified in the CLI interface contract.
//! Users only need to supply credentials and business profile.

use super::{
    AuthConfig, IntervalsConfig, LimitsConfig, McpPolicyConfig, ScoringConfig, StorageConfig,
};

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            mode: "manual".to_string(),
            callback_host: "127.0.0.1".to_string(),
            callback_port: 8080,
        }
    }
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            threshold: 60,
            keyword_relevance_max: 25.0,
            follower_count_max: 15.0,
            recency_max: 10.0,
            engagement_rate_max: 15.0,
            reply_count_max: 15.0,
            content_type_max: 10.0,
        }
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_replies_per_day: 5,
            max_tweets_per_day: 6,
            max_threads_per_week: 1,
            min_action_delay_seconds: 45,
            max_action_delay_seconds: 180,
            max_replies_per_author_per_day: 1,
            banned_phrases: vec![
                "check out".to_string(),
                "you should try".to_string(),
                "I recommend".to_string(),
                "link in bio".to_string(),
            ],
            product_mention_ratio: 0.2,
        }
    }
}

impl Default for IntervalsConfig {
    fn default() -> Self {
        Self {
            mentions_check_seconds: 300,
            discovery_search_seconds: 900,
            content_post_window_seconds: 10800,
            thread_interval_seconds: 604800,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: "~/.tuitbot/tuitbot.db".to_string(),
            retention_days: 90,
        }
    }
}

impl Default for McpPolicyConfig {
    fn default() -> Self {
        Self {
            enforce_for_mutations: true,
            require_approval_for: vec![
                "post_tweet".to_string(),
                "reply_to_tweet".to_string(),
                "follow_user".to_string(),
                "like_tweet".to_string(),
            ],
            blocked_tools: Vec::new(),
            dry_run_mutations: false,
            max_mutations_per_hour: 20,
        }
    }
}
