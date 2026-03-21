//! Workflow tests — QA fields, overrides, batch ops, edit history, error guards.

use super::super::*;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::init_test_db;

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

    let next = get_next_approved(&pool).await.expect("next");
    assert!(next.is_none());

    let stats = get_stats(&pool).await.expect("stats");
    assert_eq!(stats.failed, 1);
}

#[tokio::test]
async fn mark_failed_for_only_affects_approved() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    // Item is pending, not approved — mark_failed should be a no-op.
    mark_failed(&pool, id, "Error message")
        .await
        .expect("mark failed");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(
        item.status, "pending",
        "pending item should not be affected"
    );
}

#[tokio::test]
async fn expire_old_items_expires_nothing_when_recent() {
    let pool = init_test_db().await.expect("init db");

    enqueue(&pool, "tweet", "", "", "Fresh", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let expired = expire_old_items(&pool, 24).await.expect("expire");
    assert_eq!(expired, 0);

    let pending = get_pending(&pool).await.expect("pending");
    assert_eq!(pending.len(), 1);
}

#[tokio::test]
async fn update_content_and_approve_already_reviewed() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Draft", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id, "rejected").await.expect("reject");

    let err = update_content_and_approve(&pool, id, "New content")
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
async fn batch_approve_empty_queue() {
    let pool = init_test_db().await.expect("init db");

    let review = ReviewAction {
        actor: Some("user".to_string()),
        notes: None,
    };
    let ids = batch_approve(&pool, 10, &review).await.expect("batch");
    assert!(ids.is_empty());
}

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

    let prov = super::super::ProvenanceInput {
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

#[tokio::test]
async fn enqueue_with_provenance_for_no_provenance() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue_with_provenance_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        "tweet",
        "",
        "",
        "No prov",
        "General",
        "",
        0.0,
        "[]",
        None,
        None,
        None,
        None,
    )
    .await
    .expect("enqueue");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.source_chunks_json, "[]");
    assert!(item.scheduled_for.is_none());
}
