//! Account-scoped _for isolation tests and provenance lifecycle coverage.

use super::super::*;
use crate::storage::init_test_db;
use crate::storage::provenance::{self, ProvenanceRef};

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

// ---------------------------------------------------------------------------
// Provenance lifecycle tests
// ---------------------------------------------------------------------------

fn sample_provenance_refs() -> Vec<ProvenanceRef> {
    vec![
        ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: None,
            source_path: Some("notes/rust-async.md".to_string()),
            heading_path: Some("# Async > ## Tokio".to_string()),
            snippet: Some("Tokio runtime patterns...".to_string()),
        },
        ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: None,
            source_path: Some("notes/testing.md".to_string()),
            heading_path: None,
            snippet: Some("Property-based testing...".to_string()),
        },
    ]
}

#[tokio::test]
async fn draft_created_with_provenance_has_links() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-prov-create";

    crate::storage::accounts::create_account(&pool, acct, "P")
        .await
        .expect("create");

    let refs = sample_provenance_refs();
    let id =
        insert_draft_with_provenance_for(&pool, acct, "tweet", "Provenance draft", "assist", &refs)
            .await
            .expect("insert with provenance");

    let links = provenance::get_links_for(&pool, acct, "scheduled_content", id)
        .await
        .expect("get links");

    assert_eq!(links.len(), 2);
    assert!(links[0].node_id.is_none());
    assert!(links[0].chunk_id.is_none());
    assert_eq!(links[0].source_path.as_deref(), Some("notes/rust-async.md"));
    assert_eq!(links[1].source_path.as_deref(), Some("notes/testing.md"));
    assert_eq!(links[1].source_path.as_deref(), Some("notes/testing.md"));
}

#[tokio::test]
async fn draft_duplication_preserves_provenance_when_copied() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-prov-dup";

    crate::storage::accounts::create_account(&pool, acct, "PD")
        .await
        .expect("create");

    let refs = sample_provenance_refs();
    let original_id = insert_draft_with_provenance_for(
        &pool,
        acct,
        "tweet",
        "Original with prov",
        "assist",
        &refs,
    )
    .await
    .expect("insert");

    let dup_id = duplicate_draft_for(&pool, acct, original_id)
        .await
        .expect("duplicate")
        .expect("dup exists");

    // Simulate server-level provenance copy (as the route handler does).
    provenance::copy_links_for(
        &pool,
        acct,
        "scheduled_content",
        original_id,
        "scheduled_content",
        dup_id,
    )
    .await
    .expect("copy links");

    let dup_links = provenance::get_links_for(&pool, acct, "scheduled_content", dup_id)
        .await
        .expect("get dup links");

    assert_eq!(dup_links.len(), 2);
    assert_eq!(
        dup_links[0].source_path.as_deref(),
        Some("notes/rust-async.md")
    );
    assert_eq!(
        dup_links[1].source_path.as_deref(),
        Some("notes/testing.md")
    );
    assert_eq!(dup_links[0].entity_id, dup_id);
}

#[tokio::test]
async fn provenance_survives_schedule_unschedule() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-prov-sched";

    crate::storage::accounts::create_account(&pool, acct, "PS")
        .await
        .expect("create");

    let refs = sample_provenance_refs();
    let id = insert_draft_with_provenance_for(&pool, acct, "tweet", "Schedule me", "assist", &refs)
        .await
        .expect("insert");

    // Schedule the draft
    schedule_draft_for(&pool, acct, id, "2026-08-01T10:00:00Z")
        .await
        .expect("schedule");

    // Check provenance still present after scheduling
    let links = provenance::get_links_for(&pool, acct, "scheduled_content", id)
        .await
        .expect("get links after schedule");
    assert_eq!(links.len(), 2, "provenance should survive scheduling");

    // Unschedule the draft
    let unscheduled = unschedule_draft_for(&pool, acct, id)
        .await
        .expect("unschedule");
    assert!(unscheduled);

    // Check provenance still present after unscheduling
    let links = provenance::get_links_for(&pool, acct, "scheduled_content", id)
        .await
        .expect("get links after unschedule");
    assert_eq!(links.len(), 2, "provenance should survive unscheduling");
}

#[tokio::test]
async fn provenance_survives_autosave() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-prov-autosave";

    crate::storage::accounts::create_account(&pool, acct, "PA")
        .await
        .expect("create");

    let refs = sample_provenance_refs();
    let id =
        insert_draft_with_provenance_for(&pool, acct, "tweet", "Before autosave", "assist", &refs)
            .await
            .expect("insert");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");

    // Autosave changes the content but should not affect provenance
    let result = autosave_draft_for(&pool, acct, id, "After autosave", "tweet", &item.updated_at)
        .await
        .expect("autosave");
    assert!(result.is_some());

    let links = provenance::get_links_for(&pool, acct, "scheduled_content", id)
        .await
        .expect("get links after autosave");
    assert_eq!(links.len(), 2, "provenance should survive autosave");
    assert_eq!(links[0].source_path.as_deref(), Some("notes/rust-async.md"));
}

#[tokio::test]
async fn provenance_approval_to_scheduled_bridge() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-prov-bridge";

    crate::storage::accounts::create_account(&pool, acct, "PB")
        .await
        .expect("create");

    let refs = sample_provenance_refs();

    // Insert provenance on an approval_queue entity
    let aq_id = 999_i64;
    provenance::insert_links_for(&pool, acct, "approval_queue", aq_id, &refs)
        .await
        .expect("insert approval provenance");

    // Bridge to scheduled_content (simulate what the handler does)
    let sc_id = insert_for(
        &pool,
        acct,
        "tweet",
        "Approved content",
        Some("2026-08-01T10:00:00Z"),
    )
    .await
    .expect("insert sc");

    provenance::copy_links_for(
        &pool,
        acct,
        "approval_queue",
        aq_id,
        "scheduled_content",
        sc_id,
    )
    .await
    .expect("copy links");

    // Verify the scheduled_content now has provenance
    let sc_links = provenance::get_links_for(&pool, acct, "scheduled_content", sc_id)
        .await
        .expect("get sc links");
    assert_eq!(sc_links.len(), 2);
    assert_eq!(sc_links[0].entity_type, "scheduled_content");
    assert_eq!(sc_links[0].entity_id, sc_id);
    assert_eq!(
        sc_links[0].source_path.as_deref(),
        Some("notes/rust-async.md")
    );

    // Original approval_queue provenance should still exist
    let aq_links = provenance::get_links_for(&pool, acct, "approval_queue", aq_id)
        .await
        .expect("get aq links");
    assert_eq!(aq_links.len(), 2);
}
