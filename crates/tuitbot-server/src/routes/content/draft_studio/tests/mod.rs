//! Tests for draft studio.

use super::*;

// --- content_preview tests ---

#[test]
fn content_preview_short_text() {
    let preview = content_preview("Hello world", "tweet");
    assert_eq!(preview, "Hello world");
}

#[test]
fn content_preview_truncates_long_text() {
    let long = "a".repeat(100);
    let preview = content_preview(&long, "tweet");
    assert_eq!(preview.len(), 60);
    assert!(preview.ends_with("..."));
}

#[test]
fn content_preview_exactly_60_chars() {
    let text = "a".repeat(60);
    let preview = content_preview(&text, "tweet");
    assert_eq!(preview.len(), 60);
    assert!(!preview.ends_with("..."));
}

#[test]
fn content_preview_thread_with_blocks() {
    let json = r#"{"blocks":[{"text":"First tweet here"}]}"#;
    let preview = content_preview(json, "thread");
    assert_eq!(preview, "First tweet here");
}

#[test]
fn content_preview_thread_with_legacy_array() {
    let json = r#"["First tweet","Second tweet"]"#;
    let preview = content_preview(json, "thread");
    assert_eq!(preview, "First tweet");
}

#[test]
fn content_preview_thread_invalid_json_falls_back() {
    let preview = content_preview("not json", "thread");
    assert_eq!(preview, "not json");
}

#[test]
fn content_preview_trims_whitespace() {
    let preview = content_preview("  hello  ", "tweet");
    assert_eq!(preview, "hello");
}

// --- extract_first_block_text tests ---

#[test]
fn extract_first_block_text_valid_blocks() {
    let json = r#"{"blocks":[{"text":"Block one"},{"text":"Block two"}]}"#;
    assert_eq!(extract_first_block_text(json), "Block one");
}

#[test]
fn extract_first_block_text_empty_blocks() {
    let json = r#"{"blocks":[]}"#;
    assert_eq!(extract_first_block_text(json), json);
}

#[test]
fn extract_first_block_text_no_blocks_key() {
    let json = r#"{"other": "value"}"#;
    assert_eq!(extract_first_block_text(json), json);
}

#[test]
fn extract_first_block_text_legacy_array() {
    let json = r#"["Tweet 1","Tweet 2"]"#;
    assert_eq!(extract_first_block_text(json), "Tweet 1");
}

// --- default functions ---

#[test]
fn default_tweet_returns_tweet() {
    assert_eq!(default_tweet(), "tweet");
}

#[test]
fn default_blank_content_returns_space() {
    assert_eq!(default_blank_content(), " ");
}

#[test]
fn default_manual_returns_manual() {
    assert_eq!(default_manual(), "manual");
}

#[test]
fn default_manual_trigger_returns_manual() {
    assert_eq!(default_manual_trigger(), "manual");
}

// --- request deserialization ---

#[test]
fn draft_list_query_all_none() {
    let json = "{}";
    let q: DraftListQuery = serde_json::from_str(json).expect("deser");
    assert!(q.status.is_none());
    assert!(q.tag.is_none());
    assert!(q.search.is_none());
    assert!(q.archived.is_none());
}

#[test]
fn draft_list_query_with_filters() {
    let json = r#"{"status":"draft","tag":1,"search":"hello","archived":true}"#;
    let q: DraftListQuery = serde_json::from_str(json).expect("deser");
    assert_eq!(q.status.as_deref(), Some("draft"));
    assert_eq!(q.tag, Some(1));
    assert_eq!(q.search.as_deref(), Some("hello"));
    assert_eq!(q.archived, Some(true));
}

#[test]
fn create_studio_draft_body_defaults() {
    let json = "{}";
    let body: CreateStudioDraftBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.content_type, "tweet");
    assert_eq!(body.content, " ");
    assert_eq!(body.source, "manual");
    assert!(body.title.is_none());
}

#[test]
fn autosave_patch_body_deserializes() {
    let json =
        r#"{"content":"updated","content_type":"tweet","updated_at":"2026-03-15T10:00:00Z"}"#;
    let body: AutosavePatchBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.content, "updated");
    assert_eq!(body.content_type, "tweet");
}

#[test]
fn meta_patch_body_optional_fields() {
    let json = r#"{"title":"My Title"}"#;
    let body: MetaPatchBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.title.as_deref(), Some("My Title"));
    assert!(body.notes.is_none());
}

#[test]
fn schedule_body_deserializes() {
    let json = r#"{"scheduled_for":"2026-03-15T12:00:00Z"}"#;
    let body: ScheduleBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.scheduled_for, "2026-03-15T12:00:00Z");
}

#[test]
fn reschedule_body_deserializes() {
    let json = r#"{"scheduled_for":"2026-03-16T14:00:00Z"}"#;
    let body: RescheduleBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.scheduled_for, "2026-03-16T14:00:00Z");
}

#[test]
fn create_revision_body_default_trigger() {
    let json = "{}";
    let body: CreateRevisionBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.trigger_kind, "manual");
}

// =========================================================================
// Additional edge case tests for coverage push
// =========================================================================

// --- content_preview edge cases ---

#[test]
fn content_preview_empty_string() {
    let preview = content_preview("", "tweet");
    assert!(preview.is_empty());
}

#[test]
fn content_preview_exactly_57_chars() {
    let text = "a".repeat(57);
    let preview = content_preview(&text, "tweet");
    assert_eq!(preview.len(), 57);
    assert!(!preview.ends_with("..."));
}

#[test]
fn content_preview_exactly_61_chars() {
    let text = "a".repeat(61);
    let preview = content_preview(&text, "tweet");
    assert_eq!(preview.len(), 60);
    assert!(preview.ends_with("..."));
}

#[test]
fn content_preview_whitespace_only() {
    let preview = content_preview("   \n\t  ", "tweet");
    assert!(preview.is_empty());
}

#[test]
fn content_preview_thread_empty_blocks() {
    let json = r#"{"blocks":[]}"#;
    let preview = content_preview(json, "thread");
    assert_eq!(preview, json);
}

#[test]
fn content_preview_thread_block_no_text_key() {
    let json = r#"{"blocks":[{"content":"no text key"}]}"#;
    let preview = content_preview(json, "thread");
    assert_eq!(preview, json);
}

#[test]
fn content_preview_thread_nested_blocks_long() {
    let long_text = "x".repeat(100);
    let json = format!(r#"{{"blocks":[{{"text":"{long_text}"}}]}}"#);
    let preview = content_preview(&json, "thread");
    assert_eq!(preview.len(), 60);
    assert!(preview.ends_with("..."));
}

#[test]
fn content_preview_thread_legacy_array_single() {
    let json = r#"["Only one"]"#;
    let preview = content_preview(json, "thread");
    assert_eq!(preview, "Only one");
}

#[test]
fn content_preview_thread_legacy_array_empty() {
    let json = r#"[]"#;
    let preview = content_preview(json, "thread");
    assert_eq!(preview, json);
}

#[test]
fn content_preview_tweet_with_newlines() {
    let preview = content_preview("Line 1\nLine 2\nLine 3", "tweet");
    assert!(preview.contains('\n'));
}

// --- extract_first_block_text edge cases ---

#[test]
fn extract_first_block_text_number_value() {
    let json = r#"{"blocks":[{"text":123}]}"#;
    assert_eq!(extract_first_block_text(json), json);
}

#[test]
fn extract_first_block_text_null_blocks() {
    let json = r#"{"blocks":null}"#;
    assert_eq!(extract_first_block_text(json), json);
}

#[test]
fn extract_first_block_text_nested_array_non_string() {
    let json = r#"[123, "text"]"#;
    // First element is not a string, so falls back to full content
    assert_eq!(extract_first_block_text(json), json);
}

#[test]
fn extract_first_block_text_string_value() {
    let json = r#""just a string""#;
    assert_eq!(extract_first_block_text(json), json);
}

#[test]
fn extract_first_block_text_many_blocks() {
    let json = r#"{"blocks":[{"text":"First"},{"text":"Second"},{"text":"Third"}]}"#;
    assert_eq!(extract_first_block_text(json), "First");
}

// --- DraftSummary serialization ---

#[test]
fn draft_summary_serializes() {
    let summary = DraftSummary {
        id: 42,
        title: Some("My Draft".into()),
        content_type: "tweet".into(),
        content_preview: "Preview text".into(),
        status: "draft".into(),
        scheduled_for: None,
        archived_at: None,
        updated_at: "2026-03-15T10:00:00Z".into(),
        created_at: "2026-03-14T10:00:00Z".into(),
        source: "manual".into(),
    };
    let json = serde_json::to_string(&summary).expect("serialize");
    assert!(json.contains("42"));
    assert!(json.contains("My Draft"));
    assert!(json.contains("Preview text"));
    assert!(json.contains("draft"));
    assert!(json.contains("manual"));
}

#[test]
fn draft_summary_serializes_with_schedule() {
    let summary = DraftSummary {
        id: 1,
        title: None,
        content_type: "tweet".into(),
        content_preview: "Test".into(),
        status: "scheduled".into(),
        scheduled_for: Some("2026-04-01T12:00:00Z".into()),
        archived_at: None,
        updated_at: "2026-03-15T10:00:00Z".into(),
        created_at: "2026-03-14T10:00:00Z".into(),
        source: "assist".into(),
    };
    let json = serde_json::to_string(&summary).expect("serialize");
    assert!(json.contains("2026-04-01"));
    assert!(json.contains("scheduled"));
    assert!(json.contains("assist"));
}

// --- Request body edge cases ---

#[test]
fn create_studio_draft_body_with_all_fields() {
    let json = r#"{"content_type":"thread","content":"thread content","source":"assist","title":"My Title"}"#;
    let body: CreateStudioDraftBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.content_type, "thread");
    assert_eq!(body.content, "thread content");
    assert_eq!(body.source, "assist");
    assert_eq!(body.title.as_deref(), Some("My Title"));
}

#[test]
fn create_studio_draft_body_partial_defaults() {
    let json = r#"{"content":"Hello"}"#;
    let body: CreateStudioDraftBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.content_type, "tweet");
    assert_eq!(body.content, "Hello");
    assert_eq!(body.source, "manual");
    assert!(body.title.is_none());
}

#[test]
fn autosave_patch_body_full_deserialize() {
    let json =
        r#"{"content":"new content","content_type":"thread","updated_at":"2026-03-15T12:00:00Z"}"#;
    let body: AutosavePatchBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.content, "new content");
    assert_eq!(body.content_type, "thread");
    assert_eq!(body.updated_at, "2026-03-15T12:00:00Z");
}

#[test]
fn meta_patch_body_all_none() {
    let json = r#"{}"#;
    let body: MetaPatchBody = serde_json::from_str(json).expect("deser");
    assert!(body.title.is_none());
    assert!(body.notes.is_none());
}

#[test]
fn meta_patch_body_both_fields() {
    let json = r#"{"title":"My Title","notes":"Some notes here"}"#;
    let body: MetaPatchBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.title.as_deref(), Some("My Title"));
    assert_eq!(body.notes.as_deref(), Some("Some notes here"));
}

#[test]
fn draft_list_query_partial_filters() {
    let json = r#"{"status":"scheduled","archived":false}"#;
    let q: DraftListQuery = serde_json::from_str(json).expect("deser");
    assert_eq!(q.status.as_deref(), Some("scheduled"));
    assert!(q.tag.is_none());
    assert!(q.search.is_none());
    assert_eq!(q.archived, Some(false));
}

#[test]
fn create_revision_body_custom_trigger() {
    let json = r#"{"trigger_kind":"autosave"}"#;
    let body: CreateRevisionBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.trigger_kind, "autosave");
}

#[test]
fn reschedule_body_iso_format() {
    let json = r#"{"scheduled_for":"2026-12-31T23:59:59Z"}"#;
    let body: RescheduleBody = serde_json::from_str(json).expect("deser");
    assert_eq!(body.scheduled_for, "2026-12-31T23:59:59Z");
}

#[test]
fn schedule_body_with_offset() {
    let json = r#"{"scheduled_for":"2026-03-15T12:00:00-05:00"}"#;
    let body: ScheduleBody = serde_json::from_str(json).expect("deser");
    assert!(body.scheduled_for.contains("-05:00"));
}
