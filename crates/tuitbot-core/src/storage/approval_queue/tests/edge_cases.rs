//! Account isolation tests — `_for` variants and multi-account edge cases.

use super::super::*;
use crate::storage::init_test_db;

#[tokio::test]
async fn enqueue_for_multi_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let account_a = "account-aaa";
    let account_b = "account-bbb";

    crate::storage::accounts::create_account(&pool, account_a, "Account A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, account_b, "Account B")
        .await
        .expect("create b");

    enqueue_for(
        &pool, account_a, "tweet", "", "", "From A", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue a");

    enqueue_for(
        &pool, account_b, "reply", "t1", "@u", "From B", "Rust", "", 50.0, "[]",
    )
    .await
    .expect("enqueue b");

    let pending_a = get_pending_for(&pool, account_a).await.expect("pending a");
    assert_eq!(pending_a.len(), 1);
    assert_eq!(pending_a[0].generated_content, "From A");

    let pending_b = get_pending_for(&pool, account_b).await.expect("pending b");
    assert_eq!(pending_b.len(), 1);
    assert_eq!(pending_b[0].generated_content, "From B");

    let pending_default = get_pending(&pool).await.expect("pending default");
    assert!(pending_default.is_empty());
}

#[tokio::test]
async fn stats_for_multi_account() {
    let pool = init_test_db().await.expect("init db");

    let account = "stats-account";
    crate::storage::accounts::create_account(&pool, account, "Stats")
        .await
        .expect("create");

    let id1 = enqueue_for(
        &pool, account, "tweet", "", "", "A", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");
    enqueue_for(
        &pool, account, "tweet", "", "", "B", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    update_status_for(&pool, account, id1, "approved")
        .await
        .expect("approve");

    let stats = get_stats_for(&pool, account).await.expect("stats");
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.approved, 1);
    assert_eq!(stats.rejected, 0);
    assert_eq!(stats.failed, 0);

    let default_stats = get_stats(&pool).await.expect("default stats");
    assert_eq!(default_stats.pending, 0);
}

#[tokio::test]
async fn pending_count_for_account() {
    let pool = init_test_db().await.expect("init db");

    let account = "count-account";
    crate::storage::accounts::create_account(&pool, account, "Count")
        .await
        .expect("create");

    assert_eq!(pending_count_for(&pool, account).await.expect("count"), 0);

    enqueue_for(
        &pool, account, "tweet", "", "", "X", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    assert_eq!(pending_count_for(&pool, account).await.expect("count"), 1);
    assert_eq!(pending_count(&pool).await.expect("count"), 0);
}

#[tokio::test]
async fn enqueue_with_context_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct_a = "ctx-acct-a";
    let acct_b = "ctx-acct-b";
    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = enqueue_with_context_for(
        &pool,
        acct_a,
        "reply",
        "t1",
        "@author",
        "Content A",
        "Tech",
        "Helpful",
        80.0,
        "[]",
        Some("score_gate"),
        Some(r#"["risk_a"]"#),
        Some("2099-06-01T10:00:00Z"),
    )
    .await
    .expect("enqueue a");

    let id_b = enqueue_with_context_for(
        &pool,
        acct_b,
        "tweet",
        "",
        "",
        "Content B",
        "General",
        "",
        50.0,
        "[]",
        None,
        None,
        None,
    )
    .await
    .expect("enqueue b");

    let item_a = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("found");
    assert_eq!(item_a.reason.as_deref(), Some("score_gate"));
    assert_eq!(
        item_a.scheduled_for.as_deref(),
        Some("2099-06-01T10:00:00Z")
    );

    let cross = get_by_id_for(&pool, acct_a, id_b).await.expect("cross");
    assert!(cross.is_none(), "acct_a should not see acct_b's item");
}

#[tokio::test]
async fn update_status_with_review_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "review-acct";
    crate::storage::accounts::create_account(&pool, acct, "Review")
        .await
        .expect("create");

    let id = enqueue_for(
        &pool, acct, "tweet", "", "", "Hello", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    let review = ReviewAction {
        actor: Some("reviewer_x".to_string()),
        notes: Some("LGTM".to_string()),
    };
    update_status_with_review_for(&pool, acct, id, "approved", &review)
        .await
        .expect("review");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(item.status, "approved");
    assert_eq!(item.reviewed_by.as_deref(), Some("reviewer_x"));
    assert_eq!(item.review_notes.as_deref(), Some("LGTM"));
}

#[tokio::test]
async fn get_by_statuses_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct_a = "stat-acct-a";
    let acct_b = "stat-acct-b";
    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = enqueue_for(
        &pool, acct_a, "tweet", "", "", "A1", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue a");
    enqueue_for(
        &pool, acct_a, "reply", "t1", "@u", "A2", "Rust", "", 50.0, "[]",
    )
    .await
    .expect("enqueue a2");
    enqueue_for(
        &pool, acct_b, "tweet", "", "", "B1", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue b");

    update_status_for(&pool, acct_a, id_a, "approved")
        .await
        .expect("approve a");

    let items = get_by_statuses_for(&pool, acct_a, &["pending", "approved"], None)
        .await
        .expect("get a");
    assert_eq!(items.len(), 2);

    let replies = get_by_statuses_for(&pool, acct_a, &["pending"], Some("reply"))
        .await
        .expect("replies a");
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].action_type, "reply");

    let items_b = get_by_statuses_for(&pool, acct_b, &["pending", "approved"], None)
        .await
        .expect("get b");
    assert_eq!(items_b.len(), 1);
    assert_eq!(items_b[0].generated_content, "B1");
}

#[tokio::test]
async fn update_content_for_approval_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct_a = "upd-content-a";
    let acct_b = "upd-content-b";
    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = enqueue_for(
        &pool, acct_a, "tweet", "", "", "Original", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue a");

    // Cross-account update should be a no-op (different account_id).
    super::super::update_content_for(&pool, acct_b, id_a, "Hacked!")
        .await
        .expect("cross update");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("found");
    assert_eq!(item.generated_content, "Original");

    super::super::update_content_for(&pool, acct_a, id_a, "Edited")
        .await
        .expect("update a");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("found");
    assert_eq!(item.generated_content, "Edited");
    assert_eq!(item.status, "pending");
}

#[tokio::test]
async fn update_qa_fields_for_approval_account_isolation() {
    let pool = init_test_db().await.expect("init db");

    let acct = "qa-fields-acct";
    crate::storage::accounts::create_account(&pool, acct, "QA")
        .await
        .expect("create");

    let id = enqueue_for(
        &pool, acct, "tweet", "", "", "Draft", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    super::super::update_qa_fields_for(
        &pool,
        acct,
        id,
        r#"{"score":{"overall":72.0}}"#,
        r#"[{"code":"lang_mismatch"}]"#,
        r#"[{"code":"short"}]"#,
        r#"["Expand content"]"#,
        72.0,
        true,
    )
    .await
    .expect("update qa");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(item.qa_score, 72.0);
    assert!(item.qa_requires_override);
    assert_eq!(item.qa_hard_flags, r#"[{"code":"lang_mismatch"}]"#);
}

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
