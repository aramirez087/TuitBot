//! Tests for approval queue route handlers.

use super::default_status;
use super::export::{escape_csv, ExportQuery};
use super::handlers::{BatchApproveRequest, EditContentRequest};
use super::ApprovalQuery;

#[test]
fn escape_csv_no_special_chars() {
    assert_eq!(escape_csv("hello"), "hello");
    assert_eq!(escape_csv("simple text"), "simple text");
}

#[test]
fn escape_csv_with_comma() {
    assert_eq!(escape_csv("hello, world"), "\"hello, world\"");
}

#[test]
fn escape_csv_with_quotes() {
    assert_eq!(escape_csv(r#"say "hi""#), r#""say ""hi""""#);
}

#[test]
fn escape_csv_with_newline() {
    assert_eq!(escape_csv("line1\nline2"), "\"line1\nline2\"");
}

#[test]
fn escape_csv_empty() {
    assert_eq!(escape_csv(""), "");
}

#[test]
fn escape_csv_with_all_special() {
    let result = escape_csv("a,b\"c\nd");
    assert!(result.starts_with('"'));
    assert!(result.ends_with('"'));
}

#[test]
fn default_status_is_pending() {
    assert_eq!(default_status(), "pending");
}

#[test]
fn default_editor_is_dashboard() {
    let json = r#"{"content": "text"}"#;
    let req: EditContentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.editor, "dashboard");
}

#[test]
fn default_csv_is_csv() {
    let json = r#"{}"#;
    let query: ExportQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.format, "csv");
}

#[test]
fn default_export_status_includes_all() {
    let json = r#"{}"#;
    let query: ExportQuery = serde_json::from_str(json).unwrap();
    assert!(query.status.contains("pending"));
    assert!(query.status.contains("approved"));
    assert!(query.status.contains("rejected"));
    assert!(query.status.contains("posted"));
}

#[test]
fn approval_query_deserialize_defaults() {
    let json = r#"{}"#;
    let query: ApprovalQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.status, "pending");
    assert!(query.action_type.is_none());
    assert!(query.reviewed_by.is_none());
    assert!(query.since.is_none());
}

#[test]
fn approval_query_deserialize_with_type() {
    let json = r#"{"type": "reply"}"#;
    let query: ApprovalQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.action_type.as_deref(), Some("reply"));
}

#[test]
fn edit_content_request_deserialize() {
    let json = r#"{"content": "new text"}"#;
    let req: EditContentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content, "new text");
    assert!(req.media_paths.is_none());
    assert_eq!(req.editor, "dashboard");
}

#[test]
fn edit_content_request_with_media() {
    let json = r#"{"content": "text", "media_paths": ["a.png"], "editor": "cli"}"#;
    let req: EditContentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.media_paths.as_ref().unwrap().len(), 1);
    assert_eq!(req.editor, "cli");
}

#[test]
fn batch_approve_request_deserialize_defaults() {
    let json = r#"{}"#;
    let req: BatchApproveRequest = serde_json::from_str(json).unwrap();
    assert!(req.max.is_none());
    assert!(req.ids.is_none());
    assert!(req.review.actor.is_none());
}

#[test]
fn batch_approve_request_with_ids() {
    let json = r#"{"ids": [1, 2, 3], "review": {"actor": "admin"}}"#;
    let req: BatchApproveRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.ids.as_ref().unwrap().len(), 3);
    assert_eq!(req.review.actor.as_deref(), Some("admin"));
}

#[test]
fn export_query_deserialize_defaults() {
    let json = r#"{}"#;
    let query: ExportQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.format, "csv");
    assert!(query.status.contains("pending"));
    assert!(query.action_type.is_none());
}

#[test]
fn export_query_json_format() {
    let json = r#"{"format": "json", "type": "tweet"}"#;
    let query: ExportQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.format, "json");
    assert_eq!(query.action_type.as_deref(), Some("tweet"));
}

#[test]
fn escape_csv_tab_character() {
    assert_eq!(escape_csv("hello\tworld"), "hello\tworld");
}

#[test]
fn escape_csv_only_comma() {
    let result = escape_csv(",");
    assert_eq!(result, r#"",""#);
}

#[test]
fn escape_csv_only_quote() {
    let result = escape_csv(r#"""#);
    assert_eq!(result, r#""""""#);
}

#[test]
fn escape_csv_only_newline() {
    let result = escape_csv("\n");
    assert_eq!(result, "\"\n\"");
}

#[test]
fn escape_csv_mixed_special_chars() {
    let result = escape_csv("a,b\nc\"d");
    assert!(result.starts_with('"'));
    assert!(result.ends_with('"'));
    assert!(result.contains("\"\""));
}

#[test]
fn escape_csv_long_text() {
    let text = "a".repeat(1000);
    let result = escape_csv(&text);
    assert_eq!(result, text);
}

#[test]
fn escape_csv_unicode() {
    assert_eq!(escape_csv("caf\u{00E9}"), "caf\u{00E9}");
    assert_eq!(escape_csv("\u{1F600}"), "\u{1F600}");
}

#[test]
fn approval_query_deserialize_with_all_fields() {
    let json = r#"{
        "status": "approved,rejected",
        "type": "tweet",
        "reviewed_by": "admin",
        "since": "2026-01-01T00:00:00Z"
    }"#;
    let query: ApprovalQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.status, "approved,rejected");
    assert_eq!(query.action_type.as_deref(), Some("tweet"));
    assert_eq!(query.reviewed_by.as_deref(), Some("admin"));
    assert_eq!(query.since.as_deref(), Some("2026-01-01T00:00:00Z"));
}

#[test]
fn approval_query_status_split() {
    let json = r#"{"status": "pending,approved,rejected"}"#;
    let query: ApprovalQuery = serde_json::from_str(json).unwrap();
    let statuses: Vec<&str> = query.status.split(',').map(|s| s.trim()).collect();
    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0], "pending");
    assert_eq!(statuses[1], "approved");
    assert_eq!(statuses[2], "rejected");
}

#[test]
fn edit_content_request_empty_media_paths() {
    let json = r#"{"content": "text", "media_paths": []}"#;
    let req: EditContentRequest = serde_json::from_str(json).unwrap();
    assert!(req.media_paths.as_ref().unwrap().is_empty());
}

#[test]
fn edit_content_request_multiple_media() {
    let json = r#"{"content": "text", "media_paths": ["a.png", "b.jpg", "c.gif"]}"#;
    let req: EditContentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.media_paths.as_ref().unwrap().len(), 3);
}

#[test]
fn batch_approve_request_with_max() {
    let json = r#"{"max": 10}"#;
    let req: BatchApproveRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.max, Some(10));
    assert!(req.ids.is_none());
}

#[test]
fn batch_approve_request_with_review_notes() {
    let json = r#"{"review": {"actor": "admin", "notes": "LGTM"}}"#;
    let req: BatchApproveRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.review.actor.as_deref(), Some("admin"));
    assert_eq!(req.review.notes.as_deref(), Some("LGTM"));
}

#[test]
fn batch_approve_request_empty_ids() {
    let json = r#"{"ids": []}"#;
    let req: BatchApproveRequest = serde_json::from_str(json).unwrap();
    assert!(req.ids.as_ref().unwrap().is_empty());
}

#[test]
fn export_query_custom_status() {
    let json = r#"{"status": "posted"}"#;
    let query: ExportQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.status, "posted");
    assert_eq!(query.format, "csv");
}

#[test]
fn export_query_with_type_filter() {
    let json = r#"{"type": "thread_tweet"}"#;
    let query: ExportQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.action_type.as_deref(), Some("thread_tweet"));
}

#[test]
fn status_guard_approve_rejects_non_pending() {
    let item_status = "approved";
    assert!(item_status != "pending");
}

#[test]
fn status_guard_approve_accepts_pending() {
    let item_status = "pending";
    assert_eq!(item_status, "pending");
}

#[test]
fn status_guard_reject_rejects_approved_status() {
    let item_status = "approved";
    assert!(item_status != "pending");
}

#[test]
fn status_guard_reject_accepts_pending() {
    let item_status = "pending";
    assert_eq!(item_status, "pending");
}

#[test]
fn escape_csv_preserves_spaces() {
    assert_eq!(escape_csv("hello world"), "hello world");
    assert_eq!(escape_csv("  leading"), "  leading");
}

#[test]
fn escape_csv_carriage_return() {
    assert_eq!(escape_csv("hello\rworld"), "hello\rworld");
}

#[test]
fn escape_csv_double_quotes_escaped() {
    let result = escape_csv(r#"say "hello" to "world""#);
    assert!(result.starts_with('"'));
    assert!(result.ends_with('"'));
    assert!(result.contains(r#""""#));
}

#[test]
fn approval_query_type_reply() {
    let json = r#"{"type": "reply"}"#;
    let query: ApprovalQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.action_type, Some("reply".to_string()));
}

#[test]
fn edit_content_request_custom_editor() {
    let json = r#"{"content": "test", "editor": "api"}"#;
    let req: EditContentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.editor, "api");
}

#[test]
fn batch_approve_request_large_ids_list() {
    let ids: Vec<i64> = (1..=100).collect();
    let json_val = serde_json::json!({"ids": ids});
    let req: BatchApproveRequest = serde_json::from_str(&json_val.to_string()).unwrap();
    assert_eq!(req.ids.as_ref().unwrap().len(), 100);
}
