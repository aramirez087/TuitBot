//! Tests for the approval queue storage module.

use super::*;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::init_test_db;

#[tokio::test]
async fn enqueue_and_get_pending() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(
        &pool,
        "reply",
        "tweet123",
        "@testuser",
        "Great point about Rust!",
        "Rust",
        "AgreeAndExpand",
        85.0,
        "[]",
    )
    .await
    .expect("enqueue");

    assert!(id > 0);

    let pending = get_pending(&pool).await.expect("get pending");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].action_type, "reply");
    assert_eq!(pending[0].target_tweet_id, "tweet123");
    assert_eq!(pending[0].generated_content, "Great point about Rust!");
    assert!(pending[0].reviewed_by.is_none());
    assert!(pending[0].reason.is_none());
    assert_eq!(pending[0].detected_risks, "[]");
    assert_eq!(pending[0].qa_report, "{}");
    assert_eq!(pending[0].qa_hard_flags, "[]");
    assert_eq!(pending[0].qa_soft_flags, "[]");
    assert_eq!(pending[0].qa_recommendations, "[]");
    assert_eq!(pending[0].qa_score, 0.0);
    assert!(!pending[0].qa_requires_override);
    assert!(pending[0].qa_override_by.is_none());
    assert!(pending[0].qa_override_note.is_none());
    assert!(pending[0].qa_override_at.is_none());
}

#[tokio::test]
async fn pending_count_works() {
    let pool = init_test_db().await.expect("init db");

    assert_eq!(pending_count(&pool).await.expect("count"), 0);

    enqueue(
        &pool,
        "tweet",
        "",
        "",
        "Hello world",
        "General",
        "",
        0.0,
        "[]",
    )
    .await
    .expect("enqueue");
    enqueue(&pool, "reply", "t1", "@u", "Nice!", "Rust", "", 50.0, "[]")
        .await
        .expect("enqueue");

    assert_eq!(pending_count(&pool).await.expect("count"), 2);
}

#[tokio::test]
async fn update_status_marks_approved() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id, "approved").await.expect("update");

    let pending = get_pending(&pool).await.expect("get pending");
    assert!(pending.is_empty());

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.status, "approved");
}

#[tokio::test]
async fn update_status_marks_rejected() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id, "rejected").await.expect("update");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.status, "rejected");
}

#[tokio::test]
async fn update_content_and_approve_works() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Draft", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_content_and_approve(&pool, id, "Final version")
        .await
        .expect("update");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.status, "approved");
    assert_eq!(item.generated_content, "Final version");
}

#[tokio::test]
async fn get_by_id_not_found() {
    let pool = init_test_db().await.expect("init db");
    let item = get_by_id(&pool, 99999).await.expect("get");
    assert!(item.is_none());
}

#[tokio::test]
async fn pending_ordered_by_creation_time() {
    let pool = init_test_db().await.expect("init db");

    enqueue(&pool, "tweet", "", "", "First", "A", "", 0.0, "[]")
        .await
        .expect("enqueue");
    enqueue(&pool, "tweet", "", "", "Second", "B", "", 0.0, "[]")
        .await
        .expect("enqueue");
    enqueue(&pool, "tweet", "", "", "Third", "C", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let pending = get_pending(&pool).await.expect("get pending");
    assert_eq!(pending.len(), 3);
    assert_eq!(pending[0].generated_content, "First");
    assert_eq!(pending[1].generated_content, "Second");
    assert_eq!(pending[2].generated_content, "Third");
}

#[tokio::test]
async fn get_stats_counts_correctly() {
    let pool = init_test_db().await.expect("init db");

    let stats = get_stats(&pool).await.expect("stats");
    assert_eq!(stats.pending, 0);
    assert_eq!(stats.approved, 0);
    assert_eq!(stats.rejected, 0);

    let id1 = enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    enqueue(&pool, "tweet", "", "", "B", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    let id3 = enqueue(&pool, "reply", "t1", "@u", "C", "Rust", "", 50.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id1, "approved").await.expect("update");
    update_status(&pool, id3, "rejected").await.expect("update");

    let stats = get_stats(&pool).await.expect("stats");
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.approved, 1);
    assert_eq!(stats.rejected, 1);
}

#[tokio::test]
async fn get_by_statuses_filters_correctly() {
    let pool = init_test_db().await.expect("init db");

    let id1 = enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    enqueue(&pool, "tweet", "", "", "B", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    let id3 = enqueue(&pool, "reply", "t1", "@u", "C", "Rust", "", 50.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id1, "approved").await.expect("update");
    update_status(&pool, id3, "rejected").await.expect("update");

    let items = get_by_statuses(&pool, &["pending"], None)
        .await
        .expect("query");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].generated_content, "B");

    let items = get_by_statuses(&pool, &["pending", "approved"], None)
        .await
        .expect("query");
    assert_eq!(items.len(), 2);

    let items = get_by_statuses(&pool, &["pending", "approved", "rejected"], None)
        .await
        .expect("query");
    assert_eq!(items.len(), 3);
}

#[tokio::test]
async fn get_by_statuses_with_action_type() {
    let pool = init_test_db().await.expect("init db");

    enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    enqueue(&pool, "reply", "t1", "@u", "B", "Rust", "", 50.0, "[]")
        .await
        .expect("enqueue");

    let items = get_by_statuses(&pool, &["pending"], Some("reply"))
        .await
        .expect("query");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].action_type, "reply");

    let items = get_by_statuses(&pool, &["pending"], Some("tweet"))
        .await
        .expect("query");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].action_type, "tweet");
}

#[tokio::test]
async fn get_by_statuses_empty_returns_empty() {
    let pool = init_test_db().await.expect("init db");

    enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let items = get_by_statuses(&pool, &[], None).await.expect("query");
    assert!(items.is_empty());
}

#[tokio::test]
async fn update_content_preserves_status() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Original", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_content(&pool, id, "Edited version")
        .await
        .expect("update");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.generated_content, "Edited version");
    assert_eq!(item.status, "pending");
}

#[tokio::test]
async fn update_status_with_review_stores_metadata() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let review = ReviewAction {
        actor: Some("dashboard_user".to_string()),
        notes: Some("Looks good!".to_string()),
    };
    update_status_with_review(&pool, id, "approved", &review)
        .await
        .expect("update");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.status, "approved");
    assert_eq!(item.reviewed_by.as_deref(), Some("dashboard_user"));
    assert_eq!(item.review_notes.as_deref(), Some("Looks good!"));
}

#[tokio::test]
async fn enqueue_with_context_stores_reason_and_risks() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue_with_context(
        &pool,
        "reply",
        "tweet456",
        "@author",
        "Reply content",
        "Tech",
        "Helpful",
        75.0,
        "[]",
        Some("policy_gate"),
        Some(r#"["policy_rule:no_after_hours"]"#),
    )
    .await
    .expect("enqueue");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.reason.as_deref(), Some("policy_gate"));
    assert_eq!(item.detected_risks, r#"["policy_rule:no_after_hours"]"#);
    assert_eq!(item.qa_report, "{}");
    assert_eq!(item.qa_hard_flags, "[]");
    assert_eq!(item.qa_soft_flags, "[]");
    assert_eq!(item.qa_recommendations, "[]");
    assert_eq!(item.qa_score, 0.0);
    assert!(!item.qa_requires_override);
}

#[tokio::test]
async fn update_qa_fields_and_override_roundtrip() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Draft", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_qa_fields(
        &pool,
        id,
        r#"{"score":{"overall":55.0}}"#,
        r#"[{"code":"language_mismatch"}]"#,
        r#"[{"code":"length_near_limit"}]"#,
        r#"["Regenerate in Spanish"]"#,
        55.0,
        true,
    )
    .await
    .expect("update qa");

    set_qa_override(&pool, id, "reviewer_1", "Manually validated context")
        .await
        .expect("override");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.qa_score, 55.0);
    assert!(item.qa_requires_override);
    assert_eq!(item.qa_override_by.as_deref(), Some("reviewer_1"));
    assert_eq!(
        item.qa_override_note.as_deref(),
        Some("Manually validated context")
    );
    assert!(item.qa_override_at.is_some());

    clear_qa_override(&pool, id).await.expect("clear override");
    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert!(item.qa_override_by.is_none());
    assert!(item.qa_override_note.is_none());
    assert!(item.qa_override_at.is_none());
}

#[tokio::test]
async fn batch_approve_respects_max() {
    let pool = init_test_db().await.expect("init db");

    for i in 0..5 {
        enqueue(
            &pool,
            "tweet",
            "",
            "",
            &format!("Item {i}"),
            "General",
            "",
            0.0,
            "[]",
        )
        .await
        .expect("enqueue");
    }

    let review = ReviewAction {
        actor: Some("batch_user".to_string()),
        notes: None,
    };
    let ids = batch_approve(&pool, 3, &review).await.expect("batch");
    assert_eq!(ids.len(), 3);

    let pending = get_pending(&pool).await.expect("pending");
    assert_eq!(pending.len(), 2);

    // Verify review metadata was set
    let approved_item = get_by_id(&pool, ids[0]).await.expect("get").expect("found");
    assert_eq!(approved_item.reviewed_by.as_deref(), Some("batch_user"));
}

#[tokio::test]
async fn edit_history_roundtrip() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Original", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    record_edit(
        &pool,
        id,
        "dashboard",
        "generated_content",
        "Original",
        "Edited",
    )
    .await
    .expect("record");

    let history = get_edit_history(&pool, id).await.expect("history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].editor, "dashboard");
    assert_eq!(history[0].field, "generated_content");
    assert_eq!(history[0].old_value, "Original");
    assert_eq!(history[0].new_value, "Edited");
}

#[tokio::test]
async fn re_review_approved_item_returns_error() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id, "approved").await.expect("approve");

    let err = update_status(&pool, id, "rejected")
        .await
        .expect_err("should fail");
    match err {
        crate::error::StorageError::AlreadyReviewed {
            id: err_id,
            current_status,
        } => {
            assert_eq!(err_id, id);
            assert_eq!(current_status, "approved");
        }
        other => panic!("expected AlreadyReviewed, got: {other:?}"),
    }
}

#[tokio::test]
async fn re_review_rejected_item_returns_error() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id, "rejected").await.expect("reject");

    let err = update_status(&pool, id, "approved")
        .await
        .expect_err("should fail");
    match err {
        crate::error::StorageError::AlreadyReviewed {
            id: err_id,
            current_status,
        } => {
            assert_eq!(err_id, id);
            assert_eq!(current_status, "rejected");
        }
        other => panic!("expected AlreadyReviewed, got: {other:?}"),
    }
}

#[tokio::test]
async fn mark_failed_sets_status() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    // Bypass pending guard with raw SQL to set status to approved.
    sqlx::query("UPDATE approval_queue SET status = 'approved' WHERE id = ? AND account_id = ?")
        .bind(id)
        .bind(DEFAULT_ACCOUNT_ID)
        .execute(&pool)
        .await
        .expect("raw approve");

    mark_failed(&pool, id, "Posting failed: auth expired")
        .await
        .expect("mark failed");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.status, "failed");
    assert!(item
        .review_notes
        .as_deref()
        .unwrap()
        .contains("auth expired"));

    // Should not appear in next approved.
    let next = get_next_approved(&pool).await.expect("next");
    assert!(next.is_none());

    // Stats should reflect failed count.
    let stats = get_stats(&pool).await.expect("stats");
    assert_eq!(stats.failed, 1);
}

// ── Scheduled intent tests ──────────────────────────────────────────

#[tokio::test]
async fn enqueue_with_scheduled_for_preserves_timestamp() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue_with_context_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "tweet",
        "",
        "",
        "Hello scheduled world",
        "General",
        "",
        0.0,
        "[]",
        None,
        None,
        Some("2026-03-15T14:00:00Z"),
    )
    .await
    .expect("enqueue");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.scheduled_for.as_deref(), Some("2026-03-15T14:00:00Z"));
    assert_eq!(item.status, "pending");
}

#[tokio::test]
async fn enqueue_without_scheduled_for_is_null() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(
        &pool,
        "tweet",
        "",
        "",
        "No schedule",
        "General",
        "",
        0.0,
        "[]",
    )
    .await
    .expect("enqueue");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert!(item.scheduled_for.is_none());
}

#[tokio::test]
async fn scheduled_status_counted_in_stats() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue_with_context_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "tweet",
        "",
        "",
        "Scheduled item",
        "",
        "",
        0.0,
        "[]",
        None,
        None,
        Some("2026-03-20T10:00:00Z"),
    )
    .await
    .expect("enqueue");

    // Simulate approval → scheduled transition.
    sqlx::query("UPDATE approval_queue SET status = 'scheduled' WHERE id = ? AND account_id = ?")
        .bind(id)
        .bind(DEFAULT_ACCOUNT_ID)
        .execute(&pool)
        .await
        .expect("set scheduled");

    let stats = get_stats(&pool).await.expect("stats");
    assert_eq!(stats.scheduled, 1);
    assert_eq!(stats.pending, 0);
}

#[tokio::test]
async fn scheduled_items_excluded_from_next_approved() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(
        &pool,
        "tweet",
        "",
        "",
        "Sched item",
        "General",
        "",
        0.0,
        "[]",
    )
    .await
    .expect("enqueue");

    // Set to "scheduled" (not "approved") — should not appear in next_approved.
    sqlx::query("UPDATE approval_queue SET status = 'scheduled' WHERE id = ? AND account_id = ?")
        .bind(id)
        .bind(DEFAULT_ACCOUNT_ID)
        .execute(&pool)
        .await
        .expect("set scheduled");

    let next = get_next_approved(&pool).await.expect("next");
    assert!(
        next.is_none(),
        "scheduled items must not be picked up by the posting engine"
    );
}

#[tokio::test]
async fn enqueue_with_provenance_and_scheduled_for() {
    let pool = init_test_db().await.expect("init db");

    let prov = super::ProvenanceInput {
        source_node_id: None,
        source_seed_id: None,
        source_chunks_json: r#"[{"type":"manual"}]"#.to_string(),
        refs: vec![],
    };

    let id = enqueue_with_provenance_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "thread",
        "",
        "",
        "Thread content",
        "Tech",
        "",
        0.0,
        "[]",
        None,
        None,
        Some(&prov),
        Some("2026-04-01T09:00:00Z"),
    )
    .await
    .expect("enqueue");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.scheduled_for.as_deref(), Some("2026-04-01T09:00:00Z"));
    assert_eq!(item.source_chunks_json, r#"[{"type":"manual"}]"#);
}
