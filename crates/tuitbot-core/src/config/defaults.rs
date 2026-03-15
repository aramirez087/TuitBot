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
            template: None,
            rules: Vec::new(),
            rate_limits: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_config_defaults() {
        let config = AuthConfig::default();
        assert_eq!(config.mode, "manual");
        assert_eq!(config.callback_host, "127.0.0.1");
        assert_eq!(config.callback_port, 8080);
    }

    #[test]
    fn scoring_config_defaults() {
        let config = ScoringConfig::default();
        assert_eq!(config.threshold, 60);
        assert!((config.keyword_relevance_max - 25.0).abs() < f32::EPSILON);
        assert!((config.follower_count_max - 15.0).abs() < f32::EPSILON);
        assert!((config.recency_max - 10.0).abs() < f32::EPSILON);
        assert!((config.engagement_rate_max - 15.0).abs() < f32::EPSILON);
        assert!((config.reply_count_max - 15.0).abs() < f32::EPSILON);
        assert!((config.content_type_max - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scoring_config_weights_sum_to_90() {
        let config = ScoringConfig::default();
        let sum = config.keyword_relevance_max
            + config.follower_count_max
            + config.recency_max
            + config.engagement_rate_max
            + config.reply_count_max
            + config.content_type_max;
        assert!((sum - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn limits_config_defaults() {
        let config = LimitsConfig::default();
        assert_eq!(config.max_replies_per_day, 5);
        assert_eq!(config.max_tweets_per_day, 6);
        assert_eq!(config.max_threads_per_week, 1);
        assert_eq!(config.min_action_delay_seconds, 45);
        assert_eq!(config.max_action_delay_seconds, 180);
        assert_eq!(config.max_replies_per_author_per_day, 1);
        assert_eq!(config.banned_phrases.len(), 4);
        assert!(config.banned_phrases.contains(&"check out".to_string()));
        assert!(config
            .banned_phrases
            .contains(&"you should try".to_string()));
        assert!(config.banned_phrases.contains(&"I recommend".to_string()));
        assert!(config.banned_phrases.contains(&"link in bio".to_string()));
        assert!((config.product_mention_ratio - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn limits_config_min_delay_less_than_max() {
        let config = LimitsConfig::default();
        assert!(config.min_action_delay_seconds < config.max_action_delay_seconds);
    }

    #[test]
    fn intervals_config_defaults() {
        let config = IntervalsConfig::default();
        assert_eq!(config.mentions_check_seconds, 300);
        assert_eq!(config.discovery_search_seconds, 900);
        assert_eq!(config.content_post_window_seconds, 10800);
        assert_eq!(config.thread_interval_seconds, 604800);
    }

    #[test]
    fn intervals_config_reasonable_ranges() {
        let config = IntervalsConfig::default();
        // Mentions check: 5 minutes
        assert_eq!(config.mentions_check_seconds, 5 * 60);
        // Discovery search: 15 minutes
        assert_eq!(config.discovery_search_seconds, 15 * 60);
        // Content post window: 3 hours
        assert_eq!(config.content_post_window_seconds, 3 * 60 * 60);
        // Thread interval: 7 days
        assert_eq!(config.thread_interval_seconds, 7 * 24 * 60 * 60);
    }

    #[test]
    fn storage_config_defaults() {
        let config = StorageConfig::default();
        assert_eq!(config.db_path, "~/.tuitbot/tuitbot.db");
        assert_eq!(config.retention_days, 90);
    }

    #[test]
    fn mcp_policy_config_defaults() {
        let config = McpPolicyConfig::default();
        assert!(config.enforce_for_mutations);
        assert!(!config.dry_run_mutations);
        assert_eq!(config.max_mutations_per_hour, 20);
        assert_eq!(config.require_approval_for.len(), 4);
        assert!(config
            .require_approval_for
            .contains(&"post_tweet".to_string()));
        assert!(config
            .require_approval_for
            .contains(&"reply_to_tweet".to_string()));
        assert!(config
            .require_approval_for
            .contains(&"follow_user".to_string()));
        assert!(config
            .require_approval_for
            .contains(&"like_tweet".to_string()));
        assert!(config.blocked_tools.is_empty());
        assert!(config.template.is_none());
        assert!(config.rules.is_empty());
        assert!(config.rate_limits.is_empty());
    }
}
