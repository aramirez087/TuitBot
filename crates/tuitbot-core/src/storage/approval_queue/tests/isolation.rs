//! Account isolation tests — `_for` QA/override/batch/post/expire variants (write-side ops).

use super::super::*;
use crate::storage::init_test_db;

#[tokio::test]
async fn set_and_clear_qa_override_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "override-acct";
    crate::storage::accounts::create_account(&pool, acct, "Override")
        .await
        .expect("create");

    let id = enqueue_for(
        &pool, acct, "tweet", "", "", "Draft", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    set_qa_override_for(&pool, acct, id, "admin", "Validated manually")
        .await
        .expect("set override");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(item.qa_override_by.as_deref(), Some("admin"));
    assert_eq!(item.qa_override_note.as_deref(), Some("Validated manually"));
    assert!(item.qa_override_at.is_some());

    clear_qa_override_for(&pool, acct, id)
        .await
        .expect("clear override");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert!(item.qa_override_by.is_none());
    assert!(item.qa_override_note.is_none());
    assert!(item.qa_override_at.is_none());
}

#[tokio::test]
async fn mark_failed_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "fail-acct";
    crate::storage::accounts::create_account(&pool, acct, "Fail")
        .await
        .expect("create");

    let id = enqueue_for(
        &pool, acct, "tweet", "", "", "Hello", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    sqlx::query("UPDATE approval_queue SET status = 'approved' WHERE id = ? AND account_id = ?")
        .bind(id)
        .bind(acct)
        .execute(&pool)
        .await
        .expect("raw approve");

    mark_failed_for(&pool, acct, id, "API timeout")
        .await
        .expect("mark failed");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(item.status, "failed");
    assert!(item
        .review_notes
        .as_deref()
        .unwrap()
        .contains("API timeout"));

    let stats = get_stats_for(&pool, acct).await.expect("stats");
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.approved, 0);
}

#[tokio::test]
async fn mark_posted_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "posted-acct";
    crate::storage::accounts::create_account(&pool, acct, "Posted")
        .await
        .expect("create");

    let id = enqueue_for(
        &pool, acct, "tweet", "", "", "Hello", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    sqlx::query("UPDATE approval_queue SET status = 'approved' WHERE id = ? AND account_id = ?")
        .bind(id)
        .bind(acct)
        .execute(&pool)
        .await
        .expect("raw approve");

    mark_posted_for(&pool, acct, id, "tweet_id_123")
        .await
        .expect("mark posted");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(item.status, "posted");

    let next = get_next_approved_for(&pool, acct).await.expect("next");
    assert!(next.is_none());
}

#[tokio::test]
async fn expire_old_items_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "expire-acct";
    crate::storage::accounts::create_account(&pool, acct, "Expire")
        .await
        .expect("create");

    enqueue_for(
        &pool, acct, "tweet", "", "", "Fresh", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    let expired = expire_old_items_for(&pool, acct, 24).await.expect("expire");
    assert_eq!(expired, 0);

    let pending = get_pending_for(&pool, acct).await.expect("pending");
    assert_eq!(pending.len(), 1);
}

#[tokio::test]
async fn get_next_approved_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct_a = "next-acct-a";
    let acct_b = "next-acct-b";
    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = enqueue_for(
        &pool, acct_a, "tweet", "", "", "From A", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue a");
    let id_b = enqueue_for(
        &pool, acct_b, "tweet", "", "", "From B", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue b");

    update_status_for(&pool, acct_a, id_a, "approved")
        .await
        .expect("approve a");
    update_status_for(&pool, acct_b, id_b, "approved")
        .await
        .expect("approve b");

    let next_a = get_next_approved_for(&pool, acct_a)
        .await
        .expect("next a")
        .expect("found");
    assert_eq!(next_a.generated_content, "From A");

    let next_b = get_next_approved_for(&pool, acct_b)
        .await
        .expect("next b")
        .expect("found");
    assert_eq!(next_b.generated_content, "From B");
}

#[tokio::test]
async fn batch_approve_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "batch-acct";
    crate::storage::accounts::create_account(&pool, acct, "Batch")
        .await
        .expect("create");

    for i in 0..4 {
        enqueue_for(
            &pool,
            acct,
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
    let ids = batch_approve_for(&pool, acct, 2, &review)
        .await
        .expect("batch");
    assert_eq!(ids.len(), 2);

    let pending = get_pending_for(&pool, acct).await.expect("pending");
    assert_eq!(pending.len(), 2);

    let default_pending = get_pending(&pool).await.expect("default");
    assert!(default_pending.is_empty());
}

#[tokio::test]
async fn update_media_paths_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "media-acct";
    crate::storage::accounts::create_account(&pool, acct, "Media")
        .await
        .expect("create");

    let id = enqueue_for(
        &pool, acct, "tweet", "", "", "Hello", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    update_media_paths_for(&pool, acct, id, r#"["img.png"]"#)
        .await
        .expect("update");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(item.media_paths, r#"["img.png"]"#);
}

#[tokio::test]
async fn update_content_and_approve_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "edit-approve-acct";
    crate::storage::accounts::create_account(&pool, acct, "EditApprove")
        .await
        .expect("create");

    let id = enqueue_for(
        &pool, acct, "tweet", "", "", "Draft", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    update_content_and_approve_for(&pool, acct, id, "Final version")
        .await
        .expect("update and approve");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(item.status, "approved");
    assert_eq!(item.generated_content, "Final version");
}
