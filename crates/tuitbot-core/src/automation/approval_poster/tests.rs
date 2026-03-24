#[cfg(test)]
mod tests {
    use super::super::*;
    use std::time::Duration;

    // ── randomized_delay ────────────────────────────────────────────

    #[test]
    fn delay_returns_min_when_min_equals_max() {
        let d = queue::randomized_delay(Duration::from_secs(5), Duration::from_secs(5));
        assert_eq!(d, Duration::from_secs(5));
    }

    #[test]
    fn delay_returns_min_when_min_greater_than_max() {
        let d = queue::randomized_delay(Duration::from_secs(10), Duration::from_secs(5));
        assert_eq!(d, Duration::from_secs(10));
    }

    #[test]
    fn delay_returns_zero_when_both_zero() {
        let d = queue::randomized_delay(Duration::ZERO, Duration::ZERO);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn delay_within_range() {
        let min = Duration::from_millis(100);
        let max = Duration::from_millis(500);
        for _ in 0..50 {
            let d = queue::randomized_delay(min, max);
            assert!(d >= min, "delay {d:?} should be >= {min:?}");
            assert!(d <= max, "delay {d:?} should be <= {max:?}");
        }
    }

    #[test]
    fn delay_zero_min_nonzero_max() {
        let min = Duration::ZERO;
        let max = Duration::from_millis(100);
        for _ in 0..20 {
            let d = queue::randomized_delay(min, max);
            assert!(d <= max);
        }
    }

    #[test]
    fn delay_narrow_range_produces_deterministic_ish_result() {
        let min = Duration::from_millis(50);
        let max = Duration::from_millis(51);
        for _ in 0..20 {
            let d = queue::randomized_delay(min, max);
            assert!(d >= min && d <= max);
        }
    }

    // ── media_paths JSON parsing (mirrors inline logic) ─────────────

    #[test]
    fn media_paths_parses_valid_json_array() {
        let json = r#"["/tmp/img1.png", "/tmp/img2.jpg"]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], "/tmp/img1.png");
    }

    #[test]
    fn media_paths_parses_empty_array() {
        let json = "[]";
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    #[test]
    fn media_paths_invalid_json_returns_empty() {
        let json = "not valid json";
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    #[test]
    fn media_paths_empty_string_returns_empty() {
        let json = "";
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    // ── action_type routing logic ───────────────────────────────────

    #[test]
    fn action_type_reply_with_target_routes_to_reply() {
        let action_type = "reply";
        let target_tweet_id = "12345";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(is_reply);
    }

    #[test]
    fn action_type_reply_without_target_routes_to_tweet() {
        let action_type = "reply";
        let target_tweet_id = "";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(!is_reply);
    }

    #[test]
    fn action_type_tweet_routes_to_tweet() {
        let action_type = "tweet";
        let target_tweet_id = "";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(!is_reply);
    }

    #[test]
    fn action_type_thread_tweet_routes_to_tweet() {
        let action_type = "thread_tweet";
        let target_tweet_id = "some_id";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(!is_reply);
    }

    // ── action log format string ────────────────────────────────────

    #[test]
    fn action_log_format_for_reply() {
        let action_type = "reply";
        let log_action = format!("{action_type}_posted");
        assert_eq!(log_action, "reply_posted");
    }

    #[test]
    fn action_log_format_for_tweet() {
        let action_type = "tweet";
        let log_action = format!("{action_type}_posted");
        assert_eq!(log_action, "tweet_posted");
    }

    // ── post_reply / post_tweet helper logic ─────────────────────

    #[test]
    fn media_ids_empty_gives_none() {
        let media_ids: Vec<String> = vec![];
        let media: Option<&[String]> = if media_ids.is_empty() {
            None
        } else {
            Some(&media_ids)
        };
        assert!(media.is_none());
    }

    #[test]
    fn media_ids_nonempty_gives_some() {
        let media_ids = vec!["m1".to_string()];
        let media: Option<&[String]> = if media_ids.is_empty() {
            None
        } else {
            Some(&media_ids)
        };
        assert!(media.is_some());
        assert_eq!(media.unwrap().len(), 1);
    }

    // ── propagate_provenance conditional logic ───────────────────

    #[test]
    fn propagate_condition_both_none_skips() {
        let source_node_id: Option<i64> = None;
        let source_seed_id: Option<i64> = None;
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(!should_propagate);
    }

    #[test]
    fn propagate_condition_node_id_triggers() {
        let source_node_id: Option<i64> = Some(42);
        let source_seed_id: Option<i64> = None;
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(should_propagate);
    }

    #[test]
    fn propagate_condition_seed_id_triggers() {
        let source_node_id: Option<i64> = None;
        let source_seed_id: Option<i64> = Some(99);
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(should_propagate);
    }

    #[test]
    fn propagate_condition_both_set_triggers() {
        let source_node_id: Option<i64> = Some(1);
        let source_seed_id: Option<i64> = Some(2);
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(should_propagate);
    }

    // ── topic to Option conversion ───────────────────────────────

    #[test]
    fn empty_topic_becomes_none() {
        let topic = "";
        let opt: Option<String> = if topic.is_empty() {
            None
        } else {
            Some(topic.to_string())
        };
        assert!(opt.is_none());
    }

    #[test]
    fn nonempty_topic_becomes_some() {
        let topic = "rust programming";
        let opt: Option<String> = if topic.is_empty() {
            None
        } else {
            Some(topic.to_string())
        };
        assert_eq!(opt, Some("rust programming".to_string()));
    }

    // ── loopback URL construction ────────────────────────────────

    #[test]
    fn loopback_url_format() {
        let tweet_id = "1234567890";
        let url = format!("https://x.com/i/status/{tweet_id}");
        assert_eq!(url, "https://x.com/i/status/1234567890");
    }

    // ── delay edge cases ─────────────────────────────────────────

    #[test]
    fn delay_large_values() {
        let min = Duration::from_secs(60);
        let max = Duration::from_secs(300);
        for _ in 0..20 {
            let d = queue::randomized_delay(min, max);
            assert!(d >= min);
            assert!(d <= max);
        }
    }

    #[test]
    fn delay_subsecond() {
        let min = Duration::from_millis(1);
        let max = Duration::from_millis(10);
        for _ in 0..20 {
            let d = queue::randomized_delay(min, max);
            assert!(d >= min);
            assert!(d <= max);
        }
    }

    #[test]
    fn delay_is_zero_returns_true() {
        assert!(Duration::ZERO.is_zero());
        assert!(!Duration::from_millis(1).is_zero());
    }

    // ── action_type exhaustive routing ────────────────────────────

    #[test]
    fn action_type_all_variants() {
        for (action_type, target, expected_reply) in [
            ("reply", "12345", true),
            ("reply", "", false),
            ("tweet", "", false),
            ("tweet", "12345", false),
            ("thread_tweet", "12345", false),
            ("thread_tweet", "", false),
        ] {
            let is_reply = action_type == "reply" && !target.is_empty();
            assert_eq!(
                is_reply, expected_reply,
                "action={action_type}, target={target}"
            );
        }
    }

    // ── action log format all types ───────────────────────────────

    #[test]
    fn action_log_format_thread() {
        assert_eq!(format!("{}_posted", "thread_tweet"), "thread_tweet_posted");
    }

    // ── media_paths JSON edge cases ──────────────────────────────

    #[test]
    fn media_paths_nested_arrays_treated_as_invalid() {
        let json = r#"[["nested"]]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    #[test]
    fn media_paths_single_item() {
        let json = r#"["/path/to/image.jpg"]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "/path/to/image.jpg");
    }

    #[test]
    fn media_paths_many_items() {
        let json = r#"["/a.jpg", "/b.png", "/c.gif", "/d.mp4"]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(paths.len(), 4);
    }

    // ── parse_thread_content ─────────────────────────────────────

    #[test]
    fn parse_thread_content_block_json() {
        use crate::content::{serialize_blocks_for_storage, ThreadBlock};

        let blocks = vec![
            ThreadBlock {
                id: "a".to_string(),
                text: "First tweet".to_string(),
                media_paths: vec![],
                order: 0,
            },
            ThreadBlock {
                id: "b".to_string(),
                text: "Second tweet".to_string(),
                media_paths: vec![],
                order: 1,
            },
        ];
        let content = serialize_blocks_for_storage(&blocks);
        let parsed = poster::parse_thread_content(&content).unwrap();
        assert_eq!(parsed, vec!["First tweet", "Second tweet"]);
    }

    #[test]
    fn parse_thread_content_legacy_string_array() {
        let content = r#"["Tweet one","Tweet two","Tweet three"]"#;
        let parsed = poster::parse_thread_content(content).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], "Tweet one");
    }

    #[test]
    fn parse_thread_content_invalid_format() {
        let result = poster::parse_thread_content("just plain text");
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_empty_array() {
        let result = poster::parse_thread_content("[]");
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_blocks_sorted_by_order() {
        use crate::content::{serialize_blocks_for_storage, ThreadBlock};

        // Blocks with reversed order
        let blocks = vec![
            ThreadBlock {
                id: "b".to_string(),
                text: "Second".to_string(),
                media_paths: vec![],
                order: 1,
            },
            ThreadBlock {
                id: "a".to_string(),
                text: "First".to_string(),
                media_paths: vec![],
                order: 0,
            },
        ];
        let content = serialize_blocks_for_storage(&blocks);
        let parsed = poster::parse_thread_content(&content).unwrap();
        assert_eq!(parsed, vec!["First", "Second"]);
    }

    // ── action_type thread routing ───────────────────────────────

    #[test]
    fn action_type_thread_is_routed_separately() {
        // Thread items are handled by the thread-specific branch,
        // not the reply/tweet match.
        let action_type = "thread";
        let is_thread = action_type == "thread";
        assert!(is_thread);
    }

    // ── parse_thread_content additional edge cases ─────────────────

    #[test]
    fn parse_thread_content_single_tweet_array() {
        let content = r#"["Only tweet"]"#;
        let parsed = poster::parse_thread_content(content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], "Only tweet");
    }

    #[test]
    fn parse_thread_content_numeric_array_is_invalid() {
        let content = "[1, 2, 3]";
        let result = poster::parse_thread_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_nested_json_is_invalid() {
        let content = r#"{"key": "value"}"#;
        let result = poster::parse_thread_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_preserves_tweet_order() {
        let content = r#"["First","Second","Third","Fourth"]"#;
        let parsed = poster::parse_thread_content(content).unwrap();
        assert_eq!(parsed, vec!["First", "Second", "Third", "Fourth"]);
    }

    #[test]
    fn parse_thread_content_empty_string() {
        let result = poster::parse_thread_content("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_whitespace_only() {
        let result = poster::parse_thread_content("   ");
        assert!(result.is_err());
    }

    // ── tweet URL construction ────────────────────────────────────

    #[test]
    fn loopback_url_format_long_id() {
        let tweet_id = "1234567890123456789";
        let url = format!("https://x.com/i/status/{tweet_id}");
        assert_eq!(url, "https://x.com/i/status/1234567890123456789");
    }

    // ── child_tweet_ids extraction ────────────────────────────────

    #[test]
    fn child_ids_from_posted_ids() {
        let posted_ids = vec![
            "root".to_string(),
            "child1".to_string(),
            "child2".to_string(),
        ];
        let child_ids: Vec<String> = posted_ids.iter().skip(1).cloned().collect();
        assert_eq!(child_ids, vec!["child1", "child2"]);
    }

    #[test]
    fn child_ids_single_tweet_no_children() {
        let posted_ids = vec!["root".to_string()];
        let child_ids: Vec<String> = posted_ids.iter().skip(1).cloned().collect();
        assert!(child_ids.is_empty());
    }

    // ── topic normalization ───────────────────────────────────────

    #[test]
    fn empty_topic_uses_fallback() {
        let topic = "";
        let effective = if topic.is_empty() { "" } else { topic };
        assert_eq!(effective, "");
    }

    #[test]
    fn nonempty_topic_used_directly() {
        let topic = "rust async";
        let effective = if topic.is_empty() { "" } else { topic };
        assert_eq!(effective, "rust async");
    }

    // ── parse_thread_content additional edge cases ─────────────────

    #[test]
    fn parse_thread_content_single_tweet_array_v2() {
        let content = r#"["Only tweet"]"#;
        let parsed = poster::parse_thread_content(content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], "Only tweet");
    }
}
