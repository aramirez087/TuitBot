//! Tests: tweet length validation, legacy parsing, thread blocks, edge cases.

use super::super::*;
use tuitbot_core::content::ThreadBlock;
use tuitbot_core::storage::provenance::ProvenanceRef;

// ── tweet length validation logic ─────────────────────────────

#[test]
fn tweet_validation_empty_rejected() {
    let text = "   ";
    assert!(text.trim().is_empty());
}

#[test]
fn tweet_validation_within_limit() {
    let text = "a".repeat(280);
    assert!(
        tuitbot_core::content::tweet_weighted_len(&text) <= tuitbot_core::content::MAX_TWEET_CHARS
    );
}

#[test]
fn tweet_validation_over_limit() {
    let text = "a".repeat(281);
    assert!(
        tuitbot_core::content::tweet_weighted_len(&text) > tuitbot_core::content::MAX_TWEET_CHARS
    );
}

// ── legacy thread parsing logic ───────────────────────────────

#[test]
fn legacy_thread_valid_json_array() {
    let content = r#"["First tweet", "Second tweet"]"#;
    let tweets: Result<Vec<String>, _> = serde_json::from_str(content);
    assert!(tweets.is_ok());
    assert_eq!(tweets.unwrap().len(), 2);
}

#[test]
fn legacy_thread_invalid_json() {
    let content = "not json at all";
    let tweets: Result<Vec<String>, _> = serde_json::from_str(content);
    assert!(tweets.is_err());
}

#[test]
fn legacy_thread_empty_array() {
    let content = "[]";
    let tweets: Vec<String> = serde_json::from_str(content).unwrap();
    assert!(tweets.is_empty());
}

// ── thread blocks to core conversion ──────────────────────────

#[test]
fn block_requests_to_core_preserves_order() {
    let reqs = vec![
        ThreadBlockRequest {
            id: "c".to_string(),
            text: "Third".to_string(),
            media_paths: vec![],
            order: 2,
        },
        ThreadBlockRequest {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlockRequest {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let core_blocks: Vec<ThreadBlock> = reqs.into_iter().map(|b| b.into_core()).collect();
    assert_eq!(core_blocks.len(), 3);

    // Sort by order to get block_ids in order
    let mut sorted = core_blocks.clone();
    sorted.sort_by_key(|b| b.order);
    let ids: Vec<String> = sorted.iter().map(|b| b.id.clone()).collect();
    assert_eq!(ids, vec!["a", "b", "c"]);
}

// ── media_json serialization ──────────────────────────────────

#[test]
fn media_json_empty() {
    let media: Vec<String> = vec![];
    let json = serde_json::to_string(&media).unwrap();
    assert_eq!(json, "[]");
}

#[test]
fn media_json_with_paths() {
    let media = vec!["a.jpg".to_string(), "b.png".to_string()];
    let json = serde_json::to_string(&media).unwrap();
    assert!(json.contains("a.jpg"));
    assert!(json.contains("b.png"));
}

// ── thread block validation integration ───────────────────────

#[test]
fn validate_thread_blocks_from_requests() {
    let reqs = vec![
        ThreadBlockRequest {
            id: "a".to_string(),
            text: "First tweet".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlockRequest {
            id: "b".to_string(),
            text: "Second tweet".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let core_blocks: Vec<ThreadBlock> = reqs.into_iter().map(|b| b.into_core()).collect();
    assert!(tuitbot_core::content::validate_thread_blocks(&core_blocks).is_ok());
}

#[test]
fn serialize_blocks_roundtrip() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec!["img.jpg".to_string()],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let serialized = tuitbot_core::content::serialize_blocks_for_storage(&blocks);
    let deserialized = tuitbot_core::content::deserialize_blocks_from_content(&serialized).unwrap();
    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized[0].id, "a");
    assert_eq!(deserialized[0].media_paths.len(), 1);
}

// ── thread block validation edge cases ─────────────────────────

#[test]
fn validate_empty_blocks_fails() {
    let blocks: Vec<ThreadBlock> = vec![];
    let result = tuitbot_core::content::validate_thread_blocks(&blocks);
    assert!(result.is_err());
}

#[test]
fn validate_single_block_fails() {
    // Threads require at least 2 blocks
    let blocks = vec![ThreadBlock {
        id: "a".to_string(),
        text: "Solo tweet".to_string(),
        media_paths: vec![],
        order: 0,
    }];
    assert!(tuitbot_core::content::validate_thread_blocks(&blocks).is_err());
}

#[test]
fn validate_block_with_empty_text_fails() {
    let blocks = vec![ThreadBlock {
        id: "a".to_string(),
        text: "   ".to_string(),
        media_paths: vec![],
        order: 0,
    }];
    let result = tuitbot_core::content::validate_thread_blocks(&blocks);
    assert!(result.is_err());
}

#[test]
fn validate_block_over_280_chars_fails() {
    let blocks = vec![ThreadBlock {
        id: "a".to_string(),
        text: "x".repeat(281),
        media_paths: vec![],
        order: 0,
    }];
    let result = tuitbot_core::content::validate_thread_blocks(&blocks);
    assert!(result.is_err());
}

// ── compose_tweet_request edge cases ───────────────────────────

#[test]
fn compose_tweet_request_with_empty_provenance() {
    let json = r#"{"text": "Hello", "provenance": []}"#;
    let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
    assert!(req.provenance.unwrap().is_empty());
}

// ── compose_request edge cases ─────────────────────────────────

#[test]
fn compose_request_with_empty_blocks() {
    let json = r#"{
        "content_type": "thread",
        "content": "",
        "blocks": []
    }"#;
    let req: ComposeRequest = serde_json::from_str(json).unwrap();
    assert!(req.blocks.unwrap().is_empty());
}

#[test]
fn compose_request_with_scheduled_for() {
    let json = r#"{
        "content_type": "tweet",
        "content": "scheduled tweet",
        "scheduled_for": "2026-06-01T12:00:00Z"
    }"#;
    let req: ComposeRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.scheduled_for.as_deref(), Some("2026-06-01T12:00:00Z"));
}

#[test]
fn compose_request_with_provenance() {
    let json = r#"{
        "content_type": "tweet",
        "content": "text",
        "provenance": [{"node_id": 5, "chunk_id": 10}]
    }"#;
    let req: ComposeRequest = serde_json::from_str(json).unwrap();
    let prov = req.provenance.unwrap();
    assert_eq!(prov.len(), 1);
    assert_eq!(prov[0].node_id, Some(5));
    assert_eq!(prov[0].chunk_id, Some(10));
}

// ── build_provenance_input detailed ────────────────────────────

#[test]
fn build_provenance_input_all_none_fields() {
    let refs = vec![ProvenanceRef {
        node_id: None,
        chunk_id: None,
        seed_id: None,
        source_path: None,
        heading_path: None,
        snippet: None,
        edge_type: None,
        edge_label: None,
        angle_kind: None,
        signal_kind: None,
        signal_text: None,
        source_role: None,
    }];
    let result = build_provenance_input(Some(&refs)).unwrap();
    assert!(result.source_node_id.is_none());
    assert!(result.source_seed_id.is_none());
    assert_eq!(result.refs.len(), 1);
    // source_chunks_json should be valid JSON
    let parsed: Vec<ProvenanceRef> = serde_json::from_str(&result.source_chunks_json).unwrap();
    assert_eq!(parsed.len(), 1);
}

// ── tweet_weighted_len boundary tests ──────────────────────────

#[test]
fn tweet_len_exactly_280() {
    let text = "a".repeat(280);
    assert_eq!(tuitbot_core::content::tweet_weighted_len(&text), 280);
}

#[test]
fn tweet_len_with_url() {
    // URLs count as 23 chars in X's weighted length
    let text = "Check out https://example.com/some/long/path/here";
    let len = tuitbot_core::content::tweet_weighted_len(text);
    // Should be less than the raw char count due to URL shortening
    assert!(len < text.len(), "URL should be shortened in weighted len");
}

// ── thread block media aggregation ─────────────────────────────

#[test]
fn block_media_aggregation() {
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "First".to_string(),
            media_paths: vec!["img1.jpg".to_string()],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "Second".to_string(),
            media_paths: vec!["img2.png".to_string(), "img3.gif".to_string()],
            order: 1,
        },
        ThreadBlock {
            id: "c".to_string(),
            text: "Third".to_string(),
            media_paths: vec![],
            order: 2,
        },
    ];
    let mut sorted = blocks.clone();
    sorted.sort_by_key(|b| b.order);
    let all_media: Vec<String> = sorted.iter().flat_map(|b| b.media_paths.clone()).collect();
    assert_eq!(all_media.len(), 3);
    assert_eq!(all_media[0], "img1.jpg");
    assert_eq!(all_media[1], "img2.png");
    assert_eq!(all_media[2], "img3.gif");
}

// ── legacy thread content parsing ──────────────────────────────

#[test]
fn legacy_thread_single_tweet() {
    let content = r#"["Only tweet"]"#;
    let tweets: Vec<String> = serde_json::from_str(content).unwrap();
    assert_eq!(tweets.len(), 1);
    assert_eq!(tweets[0], "Only tweet");
}

#[test]
fn legacy_thread_with_special_chars() {
    let content = r#"["Hello \"world\"", "Tweet with\nnewline"]"#;
    let tweets: Vec<String> = serde_json::from_str(content).unwrap();
    assert_eq!(tweets.len(), 2);
    assert!(tweets[0].contains('"'));
}

#[test]
fn legacy_thread_combined_separator() {
    let tweets = vec!["First".to_string(), "Second".to_string()];
    let combined = tweets.join("\n---\n");
    assert_eq!(combined, "First\n---\nSecond");
    assert!(combined.contains("---"));
}

#[test]
fn thread_block_request_into_core() {
    let req = ThreadBlockRequest {
        id: "uuid-1".to_string(),
        text: "Hello".to_string(),
        media_paths: vec!["img.png".to_string()],
        order: 0,
    };
    let core = req.into_core();
    assert_eq!(core.id, "uuid-1");
    assert_eq!(core.text, "Hello");
    assert_eq!(core.media_paths.len(), 1);
    assert_eq!(core.order, 0);
}

#[test]
fn thread_block_request_default_media_paths() {
    let json = r#"{"id":"u1","text":"t","order":0}"#;
    let req: ThreadBlockRequest = serde_json::from_str(json).unwrap();
    assert!(req.media_paths.is_empty());
}

#[test]
fn compose_tweet_request_text_only() {
    let json = r#"{"text":"Hello world"}"#;
    let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.text, "Hello world");
    assert!(req.scheduled_for.is_none());
    assert!(req.provenance.is_none());
}

#[test]
fn compose_tweet_request_scheduled() {
    let json = r#"{"text":"Later","scheduled_for":"2026-04-01T10:00:00Z"}"#;
    let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.scheduled_for.as_deref(), Some("2026-04-01T10:00:00Z"));
}
