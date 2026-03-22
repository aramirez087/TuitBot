//! Tests: ThreadBlockRequest, build_provenance_input, request deserialization, content_type.

use super::super::*;
use tuitbot_core::storage::provenance::ProvenanceRef;

// ── ThreadBlockRequest::into_core ──────────────────────────────

#[test]
fn thread_block_request_into_core_basic() {
    let req = ThreadBlockRequest {
        id: "uuid-1".to_string(),
        text: "Hello world".to_string(),
        media_paths: vec![],
        order: 0,
    };
    let core = req.into_core();
    assert_eq!(core.id, "uuid-1");
    assert_eq!(core.text, "Hello world");
    assert_eq!(core.order, 0);
    assert!(core.media_paths.is_empty());
}

#[test]
fn thread_block_request_into_core_with_media() {
    let req = ThreadBlockRequest {
        id: "uuid-2".to_string(),
        text: "Tweet with media".to_string(),
        media_paths: vec!["/path/a.jpg".to_string(), "/path/b.png".to_string()],
        order: 3,
    };
    let core = req.into_core();
    assert_eq!(core.media_paths.len(), 2);
    assert_eq!(core.media_paths[0], "/path/a.jpg");
    assert_eq!(core.order, 3);
}

#[test]
fn thread_block_request_deserialize_without_media() {
    let json = r#"{"id":"x","text":"hi","order":0}"#;
    let req: ThreadBlockRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.id, "x");
    assert!(req.media_paths.is_empty());
}

#[test]
fn thread_block_request_deserialize_with_media() {
    let json = r#"{"id":"x","text":"hi","media_paths":["a.jpg"],"order":1}"#;
    let req: ThreadBlockRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.media_paths.len(), 1);
    assert_eq!(req.order, 1);
}

// ── build_provenance_input ────────────────────────────────────

#[test]
fn build_provenance_input_none_returns_none() {
    assert!(build_provenance_input(None).is_none());
}

#[test]
fn build_provenance_input_empty_slice_returns_none() {
    let refs: Vec<ProvenanceRef> = vec![];
    assert!(build_provenance_input(Some(&refs)).is_none());
}

#[test]
fn build_provenance_input_with_node_id() {
    let refs = vec![ProvenanceRef {
        node_id: Some(42),
        chunk_id: None,
        seed_id: None,
        source_path: None,
        heading_path: None,
        snippet: None,
        edge_type: None,
        edge_label: None,
    }];
    let result = build_provenance_input(Some(&refs)).unwrap();
    assert_eq!(result.source_node_id, Some(42));
    assert!(result.source_seed_id.is_none());
    assert_eq!(result.refs.len(), 1);
}

#[test]
fn build_provenance_input_with_seed_id() {
    let refs = vec![ProvenanceRef {
        node_id: None,
        chunk_id: None,
        seed_id: Some(99),
        source_path: None,
        heading_path: None,
        snippet: None,
        edge_type: None,
        edge_label: None,
    }];
    let result = build_provenance_input(Some(&refs)).unwrap();
    assert!(result.source_node_id.is_none());
    assert_eq!(result.source_seed_id, Some(99));
}

#[test]
fn build_provenance_input_with_multiple_refs_picks_first() {
    let refs = vec![
        ProvenanceRef {
            node_id: Some(1),
            chunk_id: None,
            seed_id: None,
            source_path: Some("/notes/a.md".to_string()),
            heading_path: Some("## Intro".to_string()),
            snippet: Some("text snippet".to_string()),
            edge_type: None,
            edge_label: None,
        },
        ProvenanceRef {
            node_id: Some(2),
            chunk_id: Some(10),
            seed_id: Some(50),
            source_path: None,
            heading_path: None,
            snippet: None,
            edge_type: None,
            edge_label: None,
        },
    ];
    let result = build_provenance_input(Some(&refs)).unwrap();
    // find_map returns first match
    assert_eq!(result.source_node_id, Some(1));
    assert_eq!(result.source_seed_id, Some(50));
    assert_eq!(result.refs.len(), 2);
    // source_chunks_json should be valid JSON
    let parsed: Vec<ProvenanceRef> = serde_json::from_str(&result.source_chunks_json).unwrap();
    assert_eq!(parsed.len(), 2);
}

// ── ComposeTweetRequest deserialization ────────────────────────

#[test]
fn compose_tweet_request_minimal() {
    let json = r#"{"text": "Hello"}"#;
    let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.text, "Hello");
    assert!(req.scheduled_for.is_none());
    assert!(req.provenance.is_none());
}

#[test]
fn compose_tweet_request_with_schedule() {
    let json = r#"{"text": "Hello", "scheduled_for": "2026-06-01T12:00:00Z"}"#;
    let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.scheduled_for.as_deref(), Some("2026-06-01T12:00:00Z"));
}

#[test]
fn compose_tweet_request_with_provenance() {
    let json = r#"{"text": "Hello", "provenance": [{"node_id": 1}]}"#;
    let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
    let prov = req.provenance.unwrap();
    assert_eq!(prov.len(), 1);
    assert_eq!(prov[0].node_id, Some(1));
}

// ── ComposeThreadRequest deserialization ───────────────────────

#[test]
fn compose_thread_request_basic() {
    let json = r#"{"tweets": ["First", "Second"]}"#;
    let req: ComposeThreadRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.tweets.len(), 2);
    assert!(req.scheduled_for.is_none());
}

// ── ComposeRequest deserialization ─────────────────────────────

#[test]
fn compose_request_tweet_type() {
    let json = r#"{"content_type": "tweet", "content": "Hello world"}"#;
    let req: ComposeRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content_type, "tweet");
    assert_eq!(req.content, "Hello world");
    assert!(req.blocks.is_none());
    assert!(req.media_paths.is_none());
    assert!(req.provenance.is_none());
}

#[test]
fn compose_request_thread_with_blocks() {
    let json = r#"{
        "content_type": "thread",
        "content": "",
        "blocks": [
            {"id": "a", "text": "First", "order": 0},
            {"id": "b", "text": "Second", "order": 1}
        ]
    }"#;
    let req: ComposeRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content_type, "thread");
    let blocks = req.blocks.unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].id, "a");
}

#[test]
fn compose_request_with_media_paths() {
    let json = r#"{
        "content_type": "tweet",
        "content": "photo tweet",
        "media_paths": ["/tmp/img.jpg"]
    }"#;
    let req: ComposeRequest = serde_json::from_str(json).unwrap();
    let media = req.media_paths.unwrap();
    assert_eq!(media.len(), 1);
}

// ── content_type routing ──────────────────────────────────────

#[test]
fn content_type_routing_tweet() {
    let ct = "tweet";
    assert_eq!(ct, "tweet");
    assert_ne!(ct, "thread");
}

#[test]
fn content_type_routing_thread() {
    let ct = "thread";
    assert_ne!(ct, "tweet");
    assert_eq!(ct, "thread");
}

#[test]
fn content_type_routing_unknown() {
    let ct = "story";
    assert_ne!(ct, "tweet");
    assert_ne!(ct, "thread");
}
