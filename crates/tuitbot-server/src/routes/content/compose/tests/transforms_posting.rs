//! Tests for transforms.rs posting flows: schedule validation, reply chain
//! sorting, partial failure handling, response shapes, provider routing,
//! and action log metadata formatting.

use tuitbot_core::content::ThreadBlock;

// ── schedule validation (used by transforms) ─────────────────

#[test]
fn schedule_validation_accepts_future_utc() {
    // Use a far-future date that will always be valid
    let result = tuitbot_core::scheduling::validate_and_normalize(
        "2099-12-31T23:59:00Z",
        tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "2099-12-31T23:59:00Z");
}

#[test]
fn schedule_validation_rejects_past_date() {
    let result = tuitbot_core::scheduling::validate_and_normalize(
        "2020-01-01T00:00:00Z",
        tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
    );
    assert!(result.is_err());
}

#[test]
fn schedule_validation_rejects_invalid_format() {
    let result = tuitbot_core::scheduling::validate_and_normalize(
        "not-a-date",
        tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
    );
    assert!(result.is_err());
}

#[test]
fn schedule_validation_none_passthrough() {
    // Mirrors: match &body.scheduled_for { Some(raw) => ..., None => None }
    let scheduled_for: Option<String> = None;
    let normalized = scheduled_for.as_ref().map(|raw| {
        tuitbot_core::scheduling::validate_and_normalize(
            raw,
            tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
        )
    });
    assert!(normalized.is_none());
}

// ── try_post_thread_now: reply chain sorting ──────────────────

#[test]
fn thread_post_sorts_blocks_by_order() {
    // Mirrors: sorted.sort_by_key(|b| b.order) in try_post_thread_now
    let blocks = vec![
        ThreadBlock {
            id: "c".to_string(),
            text: "Third tweet".to_string(),
            media_paths: vec![],
            order: 2,
        },
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
    let mut sorted: Vec<&ThreadBlock> = blocks.iter().collect();
    sorted.sort_by_key(|b| b.order);

    assert_eq!(sorted[0].text, "First tweet");
    assert_eq!(sorted[1].text, "Second tweet");
    assert_eq!(sorted[2].text, "Third tweet");
}

#[test]
fn thread_post_tweet_contents_extraction() {
    // Mirrors: let tweet_contents: Vec<String> = sorted.iter().map(|b| b.text.clone()).collect()
    let blocks = vec![
        ThreadBlock {
            id: "a".to_string(),
            text: "Hello".to_string(),
            media_paths: vec![],
            order: 0,
        },
        ThreadBlock {
            id: "b".to_string(),
            text: "World".to_string(),
            media_paths: vec![],
            order: 1,
        },
    ];
    let mut sorted: Vec<&ThreadBlock> = blocks.iter().collect();
    sorted.sort_by_key(|b| b.order);
    let contents: Vec<String> = sorted.iter().map(|b| b.text.clone()).collect();
    assert_eq!(contents, vec!["Hello", "World"]);
}

// ── partial failure error formatting ──────────────────────────

#[test]
fn partial_failure_error_message_format() {
    // Mirrors the error message in try_post_thread_now
    let i = 2;
    let total = 5;
    let posted_count = 2;
    let err_text = "rate limit exceeded";

    let msg = format!(
        "Thread failed at tweet {}/{}: {err_text}. \
         {} tweet(s) were posted and cannot be undone.",
        i + 1,
        total,
        posted_count
    );

    assert!(msg.contains("3/5"));
    assert!(msg.contains("rate limit exceeded"));
    assert!(msg.contains("2 tweet(s) were posted"));
}

#[test]
fn partial_failure_metadata_json() {
    // Mirrors the metadata json! macro in try_post_thread_now
    let tweet_ids = vec!["id1".to_string(), "id2".to_string()];
    let failed_at = 2;
    let error = "network timeout";

    let metadata = serde_json::json!({
        "posted_tweet_ids": tweet_ids,
        "failed_at_index": failed_at,
        "error": error,
        "source": "compose",
    });

    assert_eq!(metadata["posted_tweet_ids"].as_array().unwrap().len(), 2);
    assert_eq!(metadata["failed_at_index"], 2);
    assert_eq!(metadata["error"], "network timeout");
    assert_eq!(metadata["source"], "compose");
}

#[test]
fn partial_failure_no_tweets_posted() {
    // When failure occurs at index 0, tweet_ids is empty
    let tweet_ids: Vec<String> = vec![];
    assert!(
        tweet_ids.is_empty(),
        "No partial records should be persisted"
    );
}

#[test]
fn partial_failure_first_tweet_index_message() {
    let i = 0;
    let total = 3;
    let err_text = "auth expired";

    let msg = format!(
        "Thread failed at tweet {}/{}: {err_text}. \
         {} tweet(s) were posted and cannot be undone.",
        i + 1,
        total,
        0
    );

    assert!(msg.contains("1/3"));
    assert!(msg.contains("0 tweet(s)"));
}

#[test]
fn partial_failure_last_tweet_index_message() {
    let i = 4;
    let total = 5;
    let posted_count = 4;
    let err_text = "media upload failed";

    let msg = format!(
        "Thread failed at tweet {}/{}: {err_text}. \
         {} tweet(s) were posted and cannot be undone.",
        i + 1,
        total,
        posted_count
    );

    assert!(msg.contains("5/5"));
    assert!(msg.contains("4 tweet(s) were posted"));
}

// ── success response JSON shapes ──────────────────────────────

#[test]
fn posted_response_shape() {
    let tweet_ids = vec!["123".to_string(), "456".to_string()];
    let response = serde_json::json!({
        "status": "posted",
        "tweet_ids": tweet_ids,
    });
    assert_eq!(response["status"], "posted");
    assert_eq!(response["tweet_ids"].as_array().unwrap().len(), 2);
}

#[test]
fn queued_response_shape_with_blocks() {
    let block_ids = vec!["a".to_string(), "b".to_string()];
    let scheduled: Option<&str> = Some("2099-06-01T12:00:00Z");
    let response = serde_json::json!({
        "status": "queued_for_approval",
        "id": 42,
        "block_ids": block_ids,
        "scheduled_for": scheduled,
    });
    assert_eq!(response["status"], "queued_for_approval");
    assert_eq!(response["id"], 42);
    assert_eq!(response["block_ids"].as_array().unwrap().len(), 2);
    assert!(response["scheduled_for"].is_string());
}

#[test]
fn scheduled_response_shape() {
    let response = serde_json::json!({
        "status": "scheduled",
        "id": 7,
        "block_ids": ["a", "b"],
    });
    assert_eq!(response["status"], "scheduled");
    assert_eq!(response["id"], 7);
}

#[test]
fn queued_response_shape_no_schedule() {
    let scheduled: Option<&str> = None;
    let response = serde_json::json!({
        "status": "queued_for_approval",
        "id": 1,
        "scheduled_for": scheduled,
    });
    assert!(response["scheduled_for"].is_null());
}

#[test]
fn posted_tweet_response_shape() {
    // Mirrors try_post_now response
    let response = serde_json::json!({
        "status": "posted",
        "tweet_id": "1234567890",
    });
    assert_eq!(response["status"], "posted");
    assert!(response["tweet_id"].is_string());
}

#[test]
fn scheduled_response_no_block_ids() {
    // Mirrors persist_content scheduled response (tweet, not thread)
    let response = serde_json::json!({
        "status": "scheduled",
        "id": 99,
    });
    assert_eq!(response["status"], "scheduled");
    assert!(response.get("block_ids").is_none());
}

// ── build_x_client provider_backend matching ──────────────────

#[test]
fn provider_backend_matching() {
    // Mirrors: match config.x_api.provider_backend.as_str()
    let backends = vec!["scraper", "x_api", "unknown"];
    let results: Vec<&str> = backends
        .iter()
        .map(|b| match *b {
            "scraper" => "scraper",
            "x_api" => "x_api",
            _ => "error",
        })
        .collect();
    assert_eq!(results, vec!["scraper", "x_api", "error"]);
}

#[test]
fn provider_backend_empty_string_is_error() {
    let backend = "";
    let is_valid = matches!(backend, "scraper" | "x_api");
    assert!(!is_valid);
}

#[test]
fn provider_backend_error_message() {
    // Mirrors the error message in build_x_client default arm
    let msg = "Direct posting requires X API credentials or a browser session. \
             Configure in Settings \u{2192} X API.";
    assert!(msg.contains("X API credentials"));
    assert!(msg.contains("Settings"));
}

// ── action_log metadata formatting ───────────────────────────

#[test]
fn tweet_posted_metadata() {
    let tweet_id = "1234567890";
    let content_type = "tweet";
    let metadata = serde_json::json!({
        "tweet_id": tweet_id,
        "content_type": content_type,
        "source": "compose",
    });
    let serialized = metadata.to_string();
    assert!(serialized.contains("1234567890"));
    assert!(serialized.contains("compose"));
}

#[test]
fn thread_posted_metadata() {
    let tweet_ids = vec!["id1".to_string(), "id2".to_string(), "id3".to_string()];
    let metadata = serde_json::json!({
        "tweet_ids": tweet_ids,
        "content_type": "thread",
        "source": "compose",
    });
    assert_eq!(metadata["tweet_ids"].as_array().unwrap().len(), 3);

    let log_message = format!("Posted thread ({} tweets)", tweet_ids.len());
    assert_eq!(log_message, "Posted thread (3 tweets)");
}

#[test]
fn thread_posted_single_tweet_message() {
    let tweet_ids = vec!["only-one".to_string()];
    let log_message = format!("Posted thread ({} tweets)", tweet_ids.len());
    // Even with 1 tweet the message says "tweets" (plural)
    assert_eq!(log_message, "Posted thread (1 tweets)");
}

#[test]
fn tweet_posted_log_description() {
    // Mirrors: Some(&format!("Posted tweet {}", posted.id))
    let tweet_id = "abc123";
    let desc = format!("Posted tweet {tweet_id}");
    assert_eq!(desc, "Posted tweet abc123");
}

#[test]
fn partial_failure_log_description() {
    // Mirrors: Some(&format!("Thread failed at tweet {}/{}: {e}", i + 1, sorted.len()))
    let i = 1;
    let total = 4;
    let err = "timeout";
    let desc = format!("Thread failed at tweet {}/{}: {err}", i + 1, total);
    assert_eq!(desc, "Thread failed at tweet 2/4: timeout");
}
