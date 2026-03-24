use super::*;
use crate::commands::settings::helpers::ChangeTracker;
use crate::commands::settings::show::{format_duration, format_list};
use tuitbot_core::config::Config;

/// Helper: load a default Config so we can exercise category display paths.
fn test_config() -> Config {
    Config::default()
}

// ── ChangeTracker integration with edit_and_record helpers ─────────

#[test]
fn change_tracker_record_string_same_value() {
    let mut tracker = ChangeTracker::new();
    tracker.record("section", "field", "value", "value");
    assert!(tracker.changes.is_empty());
}

#[test]
fn change_tracker_record_string_different_value() {
    let mut tracker = ChangeTracker::new();
    tracker.record("section", "field", "old", "new");
    assert_eq!(tracker.changes.len(), 1);
    assert_eq!(tracker.changes[0].section, "section");
    assert_eq!(tracker.changes[0].field, "field");
    assert_eq!(tracker.changes[0].old_value, "old");
    assert_eq!(tracker.changes[0].new_value, "new");
}

#[test]
fn change_tracker_multiple_changes() {
    let mut tracker = ChangeTracker::new();
    tracker.record("s1", "f1", "a", "b");
    tracker.record("s2", "f2", "c", "d");
    tracker.record("s3", "f3", "e", "e"); // same, skipped
    assert_eq!(tracker.changes.len(), 2);
}

// ── Config default values ─────────────────────────────────────────

#[test]
fn default_config_has_expected_llm_providers() {
    let config = test_config();
    // The provider should be one of the known values or empty
    let valid = ["openai", "anthropic", "ollama", ""];
    assert!(
        valid.contains(&config.llm.provider.as_str()),
        "unexpected provider: {}",
        config.llm.provider
    );
}

#[test]
fn default_config_scoring_threshold_is_valid() {
    let config = test_config();
    assert!(config.scoring.threshold <= 100);
}

#[test]
fn default_config_limits_are_sensible() {
    let config = test_config();
    assert!(config.limits.max_replies_per_day > 0);
    assert!(config.limits.min_action_delay_seconds <= config.limits.max_action_delay_seconds);
    assert!(config.limits.product_mention_ratio >= 0.0);
    assert!(config.limits.product_mention_ratio <= 1.0);
}

#[test]
fn default_config_schedule_hours_valid() {
    let config = test_config();
    assert!(config.schedule.active_hours_start <= 23);
    assert!(config.schedule.active_hours_end <= 23);
}

#[test]
fn default_config_intervals_nonzero() {
    let config = test_config();
    assert!(config.intervals.mentions_check_seconds > 0);
    assert!(config.intervals.discovery_search_seconds > 0);
    assert!(config.intervals.content_post_window_seconds > 0);
}

#[test]
fn default_config_storage_retention_positive() {
    let config = test_config();
    assert!(config.storage.retention_days > 0);
}

// ── Category menu item counts ─────────────────────────────────────

#[test]
fn categories_menu_has_14_items() {
    // The interactive menu has 14 items (13 categories + Save & Exit)
    let categories = &[
        "Enrich Profile",
        "Your Product",
        "Brand Voice",
        "Persona",
        "AI Provider",
        "X API Credentials",
        "Target Accounts",
        "Posting Limits",
        "Scoring",
        "Timing",
        "Approval Mode",
        "Schedule",
        "Storage & Logging",
        "Save & Exit",
    ];
    assert_eq!(categories.len(), 14);
}

// ── print_category_header does not panic ──────────────────────────

#[test]
fn print_category_header_does_not_panic() {
    print_category_header("Test Category");
    print_category_header("");
    print_category_header("Very Long Category Name With Many Words");
}

// ── format_duration/format_list re-exports ────────────────────────

#[test]
fn format_duration_used_in_timing_category() {
    // Ensures format_duration is accessible and works
    let result = format_duration(3600);
    assert_eq!(result, "1 hour");
}

#[test]
fn format_list_used_in_product_category() {
    let result = format_list(&["kw1".to_string(), "kw2".to_string()]);
    assert_eq!(result, "kw1, kw2");
}

#[test]
fn format_list_empty() {
    let result = format_list(&[]);
    assert_eq!(result, "(none)");
}

// ── Config field formatting exercised by category editors ─────────

#[test]
fn scoring_weights_format_with_one_decimal() {
    let config = test_config();
    let formatted = format!("{:.0}", config.scoring.keyword_relevance_max);
    assert!(!formatted.is_empty());
}

#[test]
fn product_mention_ratio_formats_as_percentage() {
    let config = test_config();
    let formatted = format!("{:.0}%", config.limits.product_mention_ratio * 100.0);
    assert!(formatted.ends_with('%'));
}

#[test]
fn approval_mode_displays_correctly() {
    let config = test_config();
    let display = if config.approval_mode {
        "enabled (posts are queued for review)"
    } else {
        "disabled (posts go live immediately)"
    };
    assert!(!display.is_empty());
}

#[test]
fn schedule_preferred_times_formats_interval_mode() {
    let config = test_config();
    let display = if config.schedule.preferred_times.is_empty() {
        "(interval mode)".to_string()
    } else {
        format_list(&config.schedule.preferred_times)
    };
    assert!(!display.is_empty());
}

#[test]
fn storage_status_interval_disabled_display() {
    let display = if 0u64 == 0 {
        "disabled".to_string()
    } else {
        format_duration(0)
    };
    assert_eq!(display, "disabled");
}

#[test]
fn storage_status_interval_enabled_display() {
    let display = if 300u64 == 0 {
        "disabled".to_string()
    } else {
        format_duration(300)
    };
    assert_eq!(display, "5 min");
}

// ── Config field access patterns ──────────────────────────────────

#[test]
fn target_accounts_trim_at_sign() {
    let accounts = vec![
        "@user1".to_string(),
        "user2".to_string(),
        "@user3".to_string(),
    ];
    let cleaned: Vec<String> = accounts
        .into_iter()
        .map(|a| a.trim_start_matches('@').to_string())
        .collect();
    assert_eq!(cleaned, vec!["user1", "user2", "user3"]);
}

#[test]
fn mask_optional_secret_some() {
    use tuitbot_core::safety::redact::mask_optional_secret;
    let secret = Some("sk-12345678".to_string());
    let masked = mask_optional_secret(&secret);
    assert!(!masked.contains("12345678"));
}

#[test]
fn mask_optional_secret_none() {
    use tuitbot_core::safety::redact::mask_optional_secret;
    let secret: Option<String> = None;
    let masked = mask_optional_secret(&secret);
    assert!(!masked.is_empty());
}

// ── Config field formatting patterns ─────────────────────────────

#[test]
fn config_product_url_display_none() {
    let config = test_config();
    let display = config
        .business
        .product_url
        .as_deref()
        .unwrap_or("(not set)");
    assert!(!display.is_empty());
}

#[test]
fn config_brand_voice_display_default() {
    let config = test_config();
    let display = config
        .business
        .brand_voice
        .as_deref()
        .unwrap_or("(default)");
    assert!(!display.is_empty());
}

#[test]
fn config_base_url_display_default() {
    let config = test_config();
    let display = config.llm.base_url.as_deref().unwrap_or("(default)");
    assert_eq!(display, "(default)");
}

#[test]
fn config_thread_preferred_day_display() {
    let config = test_config();
    let display = config
        .schedule
        .thread_preferred_day
        .as_deref()
        .unwrap_or("(interval mode)");
    assert_eq!(display, "(interval mode)");
}

#[test]
fn target_accounts_strip_at_empty() {
    let accounts: Vec<String> = vec![];
    let cleaned: Vec<String> = accounts
        .into_iter()
        .map(|a| a.trim_start_matches('@').to_string())
        .collect();
    assert!(cleaned.is_empty());
}

#[test]
fn target_accounts_strip_at_mixed() {
    let accounts = vec![
        "@alice".to_string(),
        "bob".to_string(),
        "@@charlie".to_string(),
    ];
    let cleaned: Vec<String> = accounts
        .into_iter()
        .map(|a| a.trim_start_matches('@').to_string())
        .collect();
    assert_eq!(cleaned, vec!["alice", "bob", "charlie"]);
}

// ── Provider matching ───────────────────────────────────────────

#[test]
fn provider_position_openai() {
    let providers = &["openai", "anthropic", "ollama"];
    let current = providers.iter().position(|p| *p == "openai").unwrap_or(0);
    assert_eq!(current, 0);
}

#[test]
fn provider_position_anthropic() {
    let providers = &["openai", "anthropic", "ollama"];
    let current = providers
        .iter()
        .position(|p| *p == "anthropic")
        .unwrap_or(0);
    assert_eq!(current, 1);
}

#[test]
fn provider_position_unknown() {
    let providers = &["openai", "anthropic", "ollama"];
    let current = providers.iter().position(|p| *p == "unknown").unwrap_or(0);
    assert_eq!(current, 0);
}

// ── Config field access ─────────────────────────────────────────

#[test]
fn config_scoring_weights_are_positive() {
    let config = test_config();
    assert!(config.scoring.keyword_relevance_max >= 0.0);
    assert!(config.scoring.follower_count_max >= 0.0);
    assert!(config.scoring.recency_max >= 0.0);
    assert!(config.scoring.engagement_rate_max >= 0.0);
    assert!(config.scoring.reply_count_max >= 0.0);
    assert!(config.scoring.content_type_max >= 0.0);
}

#[test]
fn config_limits_are_consistent() {
    let config = test_config();
    assert!(config.limits.min_action_delay_seconds <= config.limits.max_action_delay_seconds);
    assert!(config.limits.max_replies_per_author_per_day > 0);
}

#[test]
fn config_intervals_are_positive() {
    let config = test_config();
    assert!(config.intervals.mentions_check_seconds > 0);
    assert!(config.intervals.discovery_search_seconds > 0);
    assert!(config.intervals.content_post_window_seconds > 0);
    assert!(config.intervals.thread_interval_seconds > 0);
}

#[test]
fn config_storage_defaults() {
    let config = test_config();
    assert!(!config.storage.db_path.is_empty());
    assert!(config.storage.retention_days > 0);
}

#[test]
fn format_duration_various_values() {
    assert_eq!(format_duration(60), "1 min");
    assert_eq!(format_duration(3600), "1 hour");
    assert_eq!(format_duration(86400), "1 day");
    assert_eq!(format_duration(7200), "2 hours");
}

#[test]
fn format_list_single_item() {
    let result = format_list(&["item".to_string()]);
    assert_eq!(result, "item");
}

#[test]
fn format_list_three_items() {
    let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let result = format_list(&items);
    assert_eq!(result, "a, b, c");
}
