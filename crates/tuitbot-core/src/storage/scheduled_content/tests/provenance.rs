//! Account-scoped _for isolation tests.

use super::super::*;
use crate::storage::init_test_db;

#[tokio::test]
async fn get_in_range_includes_unscheduled_by_created_at() {
    let pool = init_test_db().await.expect("init db");

    // Insert without scheduled_for — should match by created_at
    let id = insert(&pool, "tweet", "No schedule time", None)
        .await
        .expect("insert");
    let item = get_by_id(&pool, id).await.expect("get").expect("exists");

    // created_at is auto-set to datetime('now'), so use a wide range
    let items = get_in_range(&pool, "2020-01-01T00:00:00Z", "2099-12-31T23:59:59Z")
        .await
        .expect("range");
    assert!(
        items.iter().any(|i| i.id == id),
        "unscheduled item should be found by created_at"
    );

    // Verify the item's scheduled_for is None
    assert!(item.scheduled_for.is_none());
}

#[tokio::test]
async fn reschedule_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-resched-a";
    let acct_b = "acct-resched-b";

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
        "Reschedule me",
        Some("2026-07-01T10:00:00Z"),
    )
    .await
    .expect("insert a");

    // Reschedule with wrong account should return false
    let changed = reschedule_draft_for(&pool, acct_b, id_a, "2026-08-01T10:00:00Z")
        .await
        .expect("reschedule wrong acct");
    assert!(!changed, "cross-account reschedule should fail");

    // Reschedule with correct account
    let changed = reschedule_draft_for(&pool, acct_a, id_a, "2026-08-01T10:00:00Z")
        .await
        .expect("reschedule correct acct");
    assert!(changed);

    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.scheduled_for.as_deref(), Some("2026-08-01T10:00:00Z"));
}

#[tokio::test]
async fn unschedule_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-unsched-a";
    let acct_b = "acct-unsched-b";

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
        "Will be unscheduled",
        Some("2026-07-01T10:00:00Z"),
    )
    .await
    .expect("insert a");

    // Unschedule with wrong account should return false
    let changed = unschedule_draft_for(&pool, acct_b, id_a)
        .await
        .expect("unsched wrong");
    assert!(!changed);

    // Unschedule with correct account
    let changed = unschedule_draft_for(&pool, acct_a, id_a)
        .await
        .expect("unsched correct");
    assert!(changed);

    let item = get_by_id_for(&pool, acct_a, id_a)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "draft");
    assert!(item.scheduled_for.is_none());
}

#[tokio::test]
async fn archive_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-archive-a";
    let acct_b = "acct-archive-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id_a = insert_draft_for(&pool, acct_a, "tweet", "Archive test", "manual")
        .await
        .expect("insert a");

    // Archive with wrong account should return false
    let archived = archive_draft_for(&pool, acct_b, id_a)
        .await
        .expect("archive wrong");
    assert!(!archived);

    // Archive with correct account
    let archived = archive_draft_for(&pool, acct_a, id_a)
        .await
        .expect("archive correct");
    assert!(archived);

    // List archived drafts
    let archived_list = list_archived_drafts_for(&pool, acct_a).await.expect("list");
    assert_eq!(archived_list.len(), 1);
    assert_eq!(archived_list[0].content, "Archive test");

    // Restore with wrong account
    let restored = restore_draft_for(&pool, acct_b, id_a)
        .await
        .expect("restore wrong");
    assert!(!restored);

    // Restore with correct account
    let restored = restore_draft_for(&pool, acct_a, id_a)
        .await
        .expect("restore correct");
    assert!(restored);
}

#[tokio::test]
async fn duplicate_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-dup";

    crate::storage::accounts::create_account(&pool, acct, "Dup")
        .await
        .expect("create");

    let id = insert_draft_for(&pool, acct, "tweet", "Original", "manual")
        .await
        .expect("insert");

    // Duplicate from wrong account
    let dup = duplicate_draft_for(&pool, "wrong-acct", id)
        .await
        .expect("dup wrong");
    assert!(dup.is_none());

    // Duplicate from correct account
    let dup = duplicate_draft_for(&pool, acct, id)
        .await
        .expect("dup correct");
    assert!(dup.is_some());

    let dup_item = get_by_id_for(&pool, acct, dup.unwrap())
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(dup_item.content, "Original");
    assert_eq!(dup_item.title.as_deref(), Some("(copy)"));
}

#[tokio::test]
async fn update_draft_meta_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-meta";

    crate::storage::accounts::create_account(&pool, acct, "Meta")
        .await
        .expect("create");

    let id = insert_draft_for(&pool, acct, "tweet", "Meta test", "manual")
        .await
        .expect("insert");

    // Update meta from wrong account
    let changed = update_draft_meta_for(&pool, "wrong", id, Some("Title"), Some("Notes"))
        .await
        .expect("meta wrong");
    assert!(!changed);

    // Update meta from correct account
    let changed = update_draft_meta_for(&pool, acct, id, Some("My Title"), Some("My Notes"))
        .await
        .expect("meta correct");
    assert!(changed);

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.title.as_deref(), Some("My Title"));
    assert_eq!(item.notes.as_deref(), Some("My Notes"));
}

#[tokio::test]
async fn autosave_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-autosave";

    crate::storage::accounts::create_account(&pool, acct, "AS")
        .await
        .expect("create");

    let id = insert_draft_for(&pool, acct, "tweet", "Before save", "manual")
        .await
        .expect("insert");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");

    // Autosave with correct timestamp
    let result = autosave_draft_for(&pool, acct, id, "After save", "tweet", &item.updated_at)
        .await
        .expect("autosave");
    assert!(result.is_some(), "autosave should succeed");

    let updated = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(updated.content, "After save");

    // Autosave with a definitely-wrong timestamp should fail
    let stale_result = autosave_draft_for(
        &pool,
        acct,
        id,
        "Stale attempt",
        "tweet",
        "1970-01-01 00:00:00", // definitely stale timestamp
    )
    .await
    .expect("autosave stale");
    assert!(stale_result.is_none(), "stale autosave should return None");
}
