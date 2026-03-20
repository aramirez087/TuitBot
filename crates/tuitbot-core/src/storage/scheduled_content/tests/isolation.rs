//! QA fields, provenance, tag/revision helpers.

use super::super::*;
use crate::storage::init_test_db;

// ============================================================================
// Tags: list_draft_tags_for
// ============================================================================

#[tokio::test]
async fn list_draft_tags_returns_assigned_tags() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let draft_id = insert_draft_for(&pool, acct, "tweet", "Tagged", "manual")
        .await
        .expect("insert");
    let tag1 = create_tag_for(&pool, acct, "alpha", Some("#00ff00"))
        .await
        .expect("tag1");
    let tag2 = create_tag_for(&pool, acct, "beta", None)
        .await
        .expect("tag2");
    let _tag3 = create_tag_for(&pool, acct, "gamma", None)
        .await
        .expect("tag3");

    assign_tag_for(&pool, draft_id, tag1)
        .await
        .expect("assign1");
    assign_tag_for(&pool, draft_id, tag2)
        .await
        .expect("assign2");
    // tag3 not assigned

    let tags = list_draft_tags_for(&pool, acct, draft_id)
        .await
        .expect("list tags");
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].name, "alpha");
    assert_eq!(tags[1].name, "beta");
}

// ============================================================================
// Revisions: get_revision_for
// ============================================================================

#[tokio::test]
async fn get_revision_by_id() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let draft_id = insert_draft_for(&pool, acct, "tweet", "Current", "manual")
        .await
        .expect("insert");

    let rev_id = insert_revision_for(&pool, acct, draft_id, "Snapshot v1", "tweet", "manual")
        .await
        .expect("rev");

    let rev = get_revision_for(&pool, acct, draft_id, rev_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(rev.content, "Snapshot v1");
    assert_eq!(rev.trigger_kind, "manual");
    assert_eq!(rev.content_id, draft_id);
}

#[tokio::test]
async fn get_revision_wrong_content_id_returns_none() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let draft_id = insert_draft_for(&pool, acct, "tweet", "Content", "manual")
        .await
        .expect("insert");

    let rev_id = insert_revision_for(&pool, acct, draft_id, "Rev", "tweet", "manual")
        .await
        .expect("rev");

    // Query with wrong content_id
    let rev = get_revision_for(&pool, acct, 99999, rev_id)
        .await
        .expect("get");
    assert!(rev.is_none());
}

// ============================================================================
// Account-scoped `_for` variant isolation tests
// ============================================================================

#[tokio::test]
async fn insert_for_and_get_by_id_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-sched-a";
    let acct_b = "acct-sched-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = insert_for(
        &pool,
        acct_a,
        "tweet",
        "Content A",
        Some("2099-01-01T10:00:00Z"),
    )
    .await
    .expect("insert a");
    let id_b = insert_for(
        &pool,
        acct_b,
        "tweet",
        "Content B",
        Some("2099-01-01T10:00:00Z"),
    )
    .await
    .expect("insert b");

    // Each account can see its own item
    let item_a = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert_eq!(item_a.content, "Content A");

    let item_b = get_by_id_for(&pool, acct_b, id_b)
        .await
        .expect("get b")
        .expect("exists");
    assert_eq!(item_b.content, "Content B");

    // Cross-account access returns None
    let cross = get_by_id_for(&pool, acct_a, id_b).await.expect("cross");
    assert!(cross.is_none(), "acct_a should not see acct_b's item");

    let cross = get_by_id_for(&pool, acct_b, id_a).await.expect("cross");
    assert!(cross.is_none(), "acct_b should not see acct_a's item");
}

#[tokio::test]
async fn get_due_items_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-due-a";
    let acct_b = "acct-due-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    // Both accounts have past-due items
    insert_for(
        &pool,
        acct_a,
        "tweet",
        "Due A",
        Some("2020-01-01T09:00:00Z"),
    )
    .await
    .expect("insert a");
    insert_for(
        &pool,
        acct_b,
        "tweet",
        "Due B",
        Some("2020-01-01T09:00:00Z"),
    )
    .await
    .expect("insert b");

    let due_a = get_due_items_for(&pool, acct_a).await.expect("due a");
    assert_eq!(due_a.len(), 1);
    assert_eq!(due_a[0].content, "Due A");

    let due_b = get_due_items_for(&pool, acct_b).await.expect("due b");
    assert_eq!(due_b.len(), 1);
    assert_eq!(due_b[0].content, "Due B");
}

#[tokio::test]
async fn cancel_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-cancel-a";
    let acct_b = "acct-cancel-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = insert_for(
        &pool,
        acct_a,
        "tweet",
        "Cancel A",
        Some("2099-01-01T10:00:00Z"),
    )
    .await
    .expect("insert a");
    let id_b = insert_for(
        &pool,
        acct_b,
        "tweet",
        "Cancel B",
        Some("2099-01-01T10:00:00Z"),
    )
    .await
    .expect("insert b");

    // Cancel acct_a's item using acct_b's context should be a no-op
    cancel_for(&pool, acct_b, id_a).await.expect("cross cancel");
    let item_a = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert_eq!(
        item_a.status, "scheduled",
        "cross-account cancel should be no-op"
    );

    // Cancel with correct account
    cancel_for(&pool, acct_a, id_a).await.expect("cancel a");
    let item_a = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert_eq!(item_a.status, "cancelled");

    // acct_b's item should be unaffected
    let item_b = get_by_id_for(&pool, acct_b, id_b)
        .await
        .expect("get b")
        .expect("exists");
    assert_eq!(item_b.status, "scheduled");
}

#[tokio::test]
async fn update_content_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-upd-a";
    let acct_b = "acct-upd-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = insert_for(
        &pool,
        acct_a,
        "tweet",
        "Original A",
        Some("2099-01-01T10:00:00Z"),
    )
    .await
    .expect("insert a");

    // Update with wrong account should be no-op
    update_content_for(&pool, acct_b, id_a, "Hacked!", Some("2099-02-01T10:00:00Z"))
        .await
        .expect("cross update");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert_eq!(
        item.content, "Original A",
        "cross-account update should be no-op"
    );

    // Update with correct account
    update_content_for(
        &pool,
        acct_a,
        id_a,
        "Updated A",
        Some("2099-02-01T10:00:00Z"),
    )
    .await
    .expect("update a");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert_eq!(item.content, "Updated A");
}

#[tokio::test]
async fn list_drafts_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-drafts-a";
    let acct_b = "acct-drafts-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    insert_draft_for(&pool, acct_a, "tweet", "Draft A1", "manual")
        .await
        .expect("insert a1");
    insert_draft_for(&pool, acct_a, "tweet", "Draft A2", "manual")
        .await
        .expect("insert a2");
    insert_draft_for(&pool, acct_b, "tweet", "Draft B1", "manual")
        .await
        .expect("insert b1");

    let drafts_a = list_drafts_for(&pool, acct_a).await.expect("list a");
    assert_eq!(drafts_a.len(), 2);
    assert!(drafts_a.iter().all(|d| d.content.starts_with("Draft A")));

    let drafts_b = list_drafts_for(&pool, acct_b).await.expect("list b");
    assert_eq!(drafts_b.len(), 1);
    assert_eq!(drafts_b[0].content, "Draft B1");
}

#[tokio::test]
async fn schedule_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-sched-draft-a";
    let acct_b = "acct-sched-draft-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = insert_draft_for(&pool, acct_a, "tweet", "Schedule me", "manual")
        .await
        .expect("insert a");

    // Schedule with wrong account should be no-op
    schedule_draft_for(&pool, acct_b, id_a, "2099-12-31T10:00:00Z")
        .await
        .expect("cross schedule");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert_eq!(
        item.status, "draft",
        "cross-account schedule should be no-op"
    );

    // Schedule with correct account
    schedule_draft_for(&pool, acct_a, id_a, "2099-12-31T10:00:00Z")
        .await
        .expect("schedule a");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert_eq!(item.status, "scheduled");
    assert_eq!(item.scheduled_for.as_deref(), Some("2099-12-31T10:00:00Z"));
}

#[tokio::test]
async fn update_qa_fields_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-qa-a";
    let acct_b = "acct-qa-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = insert_for(
        &pool,
        acct_a,
        "tweet",
        "QA test A",
        Some("2099-01-01T10:00:00Z"),
    )
    .await
    .expect("insert a");

    // Update QA with wrong account should be no-op (row exists but account_id doesn't match)
    update_qa_fields_for(
        &pool,
        acct_b,
        id_a,
        r#"{"summary":"hacked"}"#,
        "[]",
        "[]",
        "[]",
        99.0,
    )
    .await
    .expect("cross qa update");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert!(
        (item.qa_score - 0.0).abs() < 0.01,
        "cross-account QA update should be no-op"
    );

    // Update QA with correct account
    update_qa_fields_for(
        &pool,
        acct_a,
        id_a,
        r#"{"summary":"good"}"#,
        r#"["flag1"]"#,
        r#"["soft1"]"#,
        r#"["rec1"]"#,
        85.0,
    )
    .await
    .expect("qa update a");
    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get a")
        .expect("exists");
    assert!((item.qa_score - 85.0).abs() < 0.01);
    assert_eq!(item.qa_hard_flags, r#"["flag1"]"#);
}

// ============================================================================
// Additional deep coverage tests
// ============================================================================

#[tokio::test]
async fn get_in_range_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-range-a";
    let acct_b = "acct-range-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    insert_for(
        &pool,
        acct_a,
        "tweet",
        "A in range",
        Some("2026-06-15T10:00:00Z"),
    )
    .await
    .expect("insert a");
    insert_for(
        &pool,
        acct_b,
        "tweet",
        "B in range",
        Some("2026-06-15T12:00:00Z"),
    )
    .await
    .expect("insert b");

    let items_a = get_in_range_for(
        &pool,
        acct_a,
        "2026-06-01T00:00:00Z",
        "2026-06-30T23:59:59Z",
    )
    .await
    .expect("range a");
    assert_eq!(items_a.len(), 1);
    assert_eq!(items_a[0].content, "A in range");

    let items_b = get_in_range_for(
        &pool,
        acct_b,
        "2026-06-01T00:00:00Z",
        "2026-06-30T23:59:59Z",
    )
    .await
    .expect("range b");
    assert_eq!(items_b.len(), 1);
    assert_eq!(items_b[0].content, "B in range");
}
