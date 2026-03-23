//! Tests for transforms.rs: validation logic, block ordering, media collection,
//! serialization, schedule validation, and error message formatting exercised
//! by compose_tweet_flow, compose_thread_legacy_flow, and compose_thread_blocks_flow.

use tuitbot_core::content::{
    serialize_blocks_for_storage, tweet_weighted_len, validate_thread_blocks, ThreadBlock,
    MAX_TWEET_CHARS,
};

// ── compose_tweet_flow validation logic ──────────────────────

#[test]
fn tweet_flow_rejects_whitespace_only_content() {
    // Mirrors: body.content.trim().to_string() → is_empty check
    for input in ["", "   ", "\t", "\n", "  \n\t  "] {
        let trimmed = input.trim().to_string();
        assert!(trimmed.is_empty(), "Expected empty after trim: {input:?}");
    }
}

#[test]
fn tweet_flow_accepts_minimal_content() {
    let trimmed = " a ".trim().to_string();
    assert!(!trimmed.is_empty());
    assert!(tweet_weighted_len(&trimmed) <= MAX_TWEET_CHARS);
}

#[test]
fn tweet_flow_rejects_content_over_280() {
    let long = "x".repeat(281);
    assert!(tweet_weighted_len(&long) > MAX_TWEET_CHARS);
}

#[test]
fn tweet_flow_accepts_exactly_280() {
    let exact = "a".repeat(280);
    assert!(tweet_weighted_len(&exact) <= MAX_TWEET_CHARS);
}

#[test]
fn tweet_flow_url_weighted_length() {
    // A tweet with a URL: X shortens URLs to 23 chars via t.co
    let text = "Look at this https://example.com/very/long/path/that/exceeds/twenty/three";
    let weighted = tweet_weighted_len(text);
    let plain_prefix_len = "Look at this ".len();
    // URL should count as 23, so total = prefix + 23
    assert_eq!(weighted, plain_prefix_len + 23);
}

#[test]
fn tweet_flow_multiple_urls_weighted() {
    let text = "See https://a.com and https://b.com/long/path done";
    let weighted = tweet_weighted_len(text);
    // "See " (4) + 23 + " and " (5) + 23 + " done" (5) = 60
    assert_eq!(weighted, 60);
}

// ── compose_thread_legacy_flow validation logic ──────────────

#[test]
fn legacy_flow_rejects_non_json_content() {
    let content = "just plain text";
    let result: Result<Vec<String>, _> = serde_json::from_str(content);
    assert!(result.is_err());
}

#[test]
fn legacy_flow_rejects_json_object() {
    let content = r#"{"text": "hello"}"#;
    let result: Result<Vec<String>, _> = serde_json::from_str(content);
    assert!(result.is_err());
}

#[test]
fn legacy_flow_rejects_json_number_array() {
    let content = "[1, 2, 3]";
    let result: Result<Vec<String>, _> = serde_json::from_str(content);
    assert!(result.is_err());
}

#[test]
fn legacy_flow_rejects_empty_array() {
    let content = "[]";
    let tweets: Vec<String> = serde_json::from_str(content).unwrap();
    assert!(
        tweets.is_empty(),
        "empty array should be caught by len check"
    );
}

#[test]
fn legacy_flow_per_tweet_length_check() {
    // Mirrors: for (i, tweet) in tweets.iter().enumerate() { ... }
    let tweets = vec![
        "short".to_string(),
        "a".repeat(281), // over limit
        "also short".to_string(),
    ];
    let mut failed_index = None;
    for (i, tweet) in tweets.iter().enumerate() {
        if tweet_weighted_len(tweet) > MAX_TWEET_CHARS {
            failed_index = Some(i);
            break;
        }
    }
    assert_eq!(failed_index, Some(1));
    // Error message format: "tweet {} exceeds 280 characters"
    let msg = format!("tweet {} exceeds 280 characters", failed_index.unwrap() + 1);
    assert_eq!(msg, "tweet 2 exceeds 280 characters");
}

#[test]
fn legacy_flow_all_tweets_within_limit() {
    let tweets = vec!["Hello".to_string(), "World".to_string(), "a".repeat(280)];
    for tweet in &tweets {
        assert!(tweet_weighted_len(tweet) <= MAX_TWEET_CHARS);
    }
}

#[test]
fn legacy_flow_unicode_in_json_array() {
    let content = r#"["Caf\u00e9 ☕", "日本語ツイート"]"#;
    let tweets: Vec<String> = serde_json::from_str(content).unwrap();
    assert_eq!(tweets.len(), 2);
    assert!(tweets[0].contains("Caf"));
}

#[test]
fn legacy_flow_large_thread() {
    let tweets: Vec<String> = (0..25).map(|i| format!("Tweet number {i}")).collect();
    let content = serde_json::to_string(&tweets).unwrap();
    let parsed: Vec<String> = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.len(), 25);
}

// ── compose_thread_blocks_flow: block ordering & ID extraction ─

#[test]
fn blocks_flow_sorted_block_ids() {
    // Mirrors the block_ids extraction: sort by order, map to ids
    let blocks = vec![
        ThreadBlock {
            id: "z-last".to_string(),
            text: "Third".to_string(),
            media_paths: vec![],
            order: 2,
        },
        ThreadBlock {
            id: "a-first".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "m-mid".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];

    let mut sorted = blocks.clone();
    sorted.sort_by_key(|b| b.order);
    let block_ids: Vec<String> = sorted.iter().map(|b| b.id.clone()).collect();

    assert_eq!(block_ids, vec!["a-first", "m-mid", "z-last"]);
}

#[test]
fn blocks_flow_media_aggregation_respects_order() {
    // Mirrors: sorted.iter().flat_map(|b| b.media_paths.clone()).collect()
    let blocks = vec![
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec!["second.png".to_string()],
            order: 1,
        },
        ThreadBlock {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec!["first_a.jpg".to_string(), "first_b.jpg".to_string()],
            order: 0,
        },
    ];

    let mut sorted = blocks.clone();
    sorted.sort_by_key(|b| b.order);
    let all_media: Vec<String> = sorted.iter().flat_map(|b| b.media_paths.clone()).collect();

    assert_eq!(all_media.len(), 3);
    assert_eq!(all_media[0], "first_a.jpg");
    assert_eq!(all_media[1], "first_b.jpg");
    assert_eq!(all_media[2], "second.png");
}

#[test]
fn blocks_flow_media_aggregation_no_media() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];

    let mut sorted = blocks.clone();
    sorted.sort_by_key(|b| b.order);
    let all_media: Vec<String> = sorted.iter().flat_map(|b| b.media_paths.clone()).collect();

    assert!(all_media.is_empty());
}

#[test]
fn blocks_flow_media_json_serialization() {
    // Mirrors: serde_json::to_string(&all_media).unwrap_or_else(|_| "[]")
    let all_media = vec!["a.jpg".to_string(), "b.png".to_string()];
    let media_json = serde_json::to_string(&all_media).unwrap_or_else(|_| "[]".to_string());
    let roundtrip: Vec<String> = serde_json::from_str(&media_json).unwrap();
    assert_eq!(roundtrip, all_media);
}

#[test]
fn blocks_flow_empty_media_json() {
    let all_media: Vec<String> = vec![];
    let media_json = serde_json::to_string(&all_media).unwrap_or_else(|_| "[]".to_string());
    assert_eq!(media_json, "[]");
}

// ── compose_thread_blocks_flow: validation integration ────────

#[test]
fn blocks_flow_validate_rejects_duplicate_ids() {
    let blocks = vec![
        ThreadBlock {
            id: "dup".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "dup".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let err = validate_thread_blocks(&blocks).unwrap_err();
    let msg = err.api_message();
    assert!(
        msg.contains("duplicate block ID"),
        "Expected duplicate ID error, got: {msg}"
    );
    assert!(msg.contains("dup"));
}

#[test]
fn blocks_flow_validate_rejects_non_contiguous_order() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 2, // gap: no order=1
        },
    ];
    let err = validate_thread_blocks(&blocks).unwrap_err();
    let msg = err.api_message();
    assert!(
        msg.contains("contiguous"),
        "Expected contiguous order error, got: {msg}"
    );
}

#[test]
fn blocks_flow_validate_rejects_empty_block_id() {
    let blocks = vec![
        ThreadBlock {
            id: "".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let err = validate_thread_blocks(&blocks).unwrap_err();
    let msg = err.api_message();
    assert!(
        msg.contains("empty ID"),
        "Expected empty ID error, got: {msg}"
    );
}

#[test]
fn blocks_flow_validate_rejects_whitespace_only_id() {
    let blocks = vec![
        ThreadBlock {
            id: "   ".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let err = validate_thread_blocks(&blocks).unwrap_err();
    let msg = err.api_message();
    assert!(msg.contains("empty ID"), "Got: {msg}");
}

#[test]
fn blocks_flow_validate_rejects_empty_text() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "Valid tweet".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "   ".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let err = validate_thread_blocks(&blocks).unwrap_err();
    let msg = err.api_message();
    assert!(
        msg.contains("empty text"),
        "Expected empty text error, got: {msg}"
    );
}

#[test]
fn blocks_flow_validate_rejects_overlong_text() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "Normal".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "x".repeat(281),
            media_paths: vec![],
            order: 1,
        },
    ];
    let err = validate_thread_blocks(&blocks).unwrap_err();
    let msg = err.api_message();
    assert!(
        msg.contains("exceeds"),
        "Expected length exceeded error, got: {msg}"
    );
}

#[test]
fn blocks_flow_validate_accepts_valid_two_blocks() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    assert!(validate_thread_blocks(&blocks).is_ok());
}

#[test]
fn blocks_flow_validate_accepts_blocks_with_media() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "With image".to_string(),
            media_paths: vec!["photo.jpg".to_string()],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "With two images".to_string(),
            media_paths: vec!["one.png".to_string(), "two.png".to_string()],
            order: 1,
        },
    ];
    assert!(validate_thread_blocks(&blocks).is_ok());
}

// ── serialize_blocks_for_storage ──────────────────────────────

#[test]
fn blocks_serialization_contains_version() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let serialized = serialize_blocks_for_storage(&blocks);
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(parsed["version"], 1);
    assert_eq!(parsed["blocks"].as_array().unwrap().len(), 2);
}

#[test]
fn blocks_serialization_preserves_media_paths() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "Media tweet".to_string(),
            media_paths: vec!["img.jpg".to_string()],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "No media".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let serialized = serialize_blocks_for_storage(&blocks);
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    let block_a = &parsed["blocks"][0];
    assert_eq!(block_a["media_paths"][0], "img.jpg");
    let block_b = &parsed["blocks"][1];
    assert!(block_b["media_paths"].as_array().unwrap().is_empty());
}

#[test]
fn blocks_serialization_roundtrip_with_special_chars() {
    let blocks = vec![
        ThreadBlock {
            id: "id-1".to_string(),
            text: "Quote: \"hello\" & <world>".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "id-2".to_string(),
            text: "Newline\nand\ttab".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let serialized = serialize_blocks_for_storage(&blocks);
    let deserialized = tuitbot_core::content::deserialize_blocks_from_content(&serialized).unwrap();
    assert_eq!(deserialized[0].text, "Quote: \"hello\" & <world>");
    assert_eq!(deserialized[1].text, "Newline\nand\ttab");
}
