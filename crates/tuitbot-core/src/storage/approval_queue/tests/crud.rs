//! CRUD tests — basic enqueue, get, update, and status operations.

use super::super::*;
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
async fn get_next_approved_returns_oldest() {
    let pool = init_test_db().await.expect("init db");

    let next = get_next_approved(&pool).await.expect("next");
    assert!(next.is_none());

    let id1 = enqueue(&pool, "tweet", "", "", "First", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    let id2 = enqueue(&pool, "tweet", "", "", "Second", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_status(&pool, id1, "approved")
        .await
        .expect("approve");
    update_status(&pool, id2, "approved")
        .await
        .expect("approve");

    let next = get_next_approved(&pool)
        .await
        .expect("next")
        .expect("found");
    assert_eq!(next.id, id1, "should return the first approved item");
}

#[tokio::test]
async fn mark_posted_sets_status_and_tweet_id() {
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

    mark_posted(&pool, id, "posted_tweet_999")
        .await
        .expect("mark posted");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.status, "posted");

    let next = get_next_approved(&pool).await.expect("next");
    assert!(next.is_none());
}

#[tokio::test]
async fn update_media_paths_works() {
    let pool = init_test_db().await.expect("init db");

    let id = enqueue(&pool, "tweet", "", "", "Hello", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    update_media_paths(&pool, id, r#"["/tmp/img.png"]"#)
        .await
        .expect("update media");

    let item = get_by_id(&pool, id).await.expect("get").expect("found");
    assert_eq!(item.media_paths, r#"["/tmp/img.png"]"#);
}

#[tokio::test]
async fn get_filtered_by_action_type() {
    let pool = init_test_db().await.expect("init db");

    enqueue(&pool, "tweet", "", "", "T1", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    enqueue(&pool, "reply", "t1", "@u", "R1", "Rust", "", 50.0, "[]")
        .await
        .expect("enqueue");
    enqueue(&pool, "tweet", "", "", "T2", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let items = get_filtered(&pool, &["pending"], Some("tweet"), None, None)
        .await
        .expect("filtered");
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|i| i.action_type == "tweet"));
}

#[tokio::test]
async fn get_filtered_by_reviewer() {
    let pool = init_test_db().await.expect("init db");

    let id1 = enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");
    let id2 = enqueue(&pool, "tweet", "", "", "B", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let review_alice = ReviewAction {
        actor: Some("alice".to_string()),
        notes: None,
    };
    let review_bob = ReviewAction {
        actor: Some("bob".to_string()),
        notes: None,
    };
    update_status_with_review(&pool, id1, "approved", &review_alice)
        .await
        .expect("approve");
    update_status_with_review(&pool, id2, "approved", &review_bob)
        .await
        .expect("approve");

    let items = get_filtered(&pool, &["approved"], None, Some("alice"), None)
        .await
        .expect("filtered");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].generated_content, "A");
}

#[tokio::test]
async fn get_filtered_empty_statuses_returns_empty() {
    let pool = init_test_db().await.expect("init db");

    enqueue(&pool, "tweet", "", "", "A", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let items = get_filtered(&pool, &[], None, None, None)
        .await
        .expect("filtered");
    assert!(items.is_empty());
}

#[tokio::test]
async fn get_filtered_for_with_since_parameter() {
    let pool = init_test_db().await.expect("init db");

    enqueue(&pool, "tweet", "", "", "Ancient", "General", "", 0.0, "[]")
        .await
        .expect("enqueue");

    let items = get_filtered(
        &pool,
        &["pending"],
        None,
        None,
        Some("2099-01-01T00:00:00Z"),
    )
    .await
    .expect("filtered");
    assert!(items.is_empty());

    let items = get_filtered(
        &pool,
        &["pending"],
        None,
        None,
        Some("2000-01-01T00:00:00Z"),
    )
    .await
    .expect("filtered");
    assert_eq!(items.len(), 1);
}
