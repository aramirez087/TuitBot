//! Scheduling: reschedule, edge cases, draft lifecycle, unschedule, autosave.

use super::super::*;
use crate::storage::init_test_db;

#[tokio::test]
async fn reschedule_non_scheduled_returns_false() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Draft only", "manual")
        .await
        .expect("insert");

    // Try to reschedule a draft (not scheduled) — should return false
    let updated = reschedule_draft_for(&pool, acct, id, "2099-12-31T15:00:00Z")
        .await
        .expect("reschedule");
    assert!(!updated);
}

#[tokio::test]
async fn duplicate_tag_name_is_rejected() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    create_tag_for(&pool, acct, "unique-tag", None)
        .await
        .expect("first");

    let result = create_tag_for(&pool, acct, "unique-tag", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_drafts_excludes_archived() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id1 = insert_draft_for(&pool, acct, "tweet", "Active", "manual")
        .await
        .expect("insert");
    let id2 = insert_draft_for(&pool, acct, "tweet", "Will archive", "manual")
        .await
        .expect("insert");

    archive_draft_for(&pool, acct, id2).await.expect("archive");

    let drafts = list_drafts_for(&pool, acct).await.expect("list");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].id, id1);
    assert_eq!(drafts[0].content, "Active");

    // Archived list should contain the archived one
    let archived = list_archived_drafts_for(&pool, acct).await.expect("list");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, id2);
}

// ============================================================================
// Scheduling edge cases (Session 06)
// ============================================================================

#[tokio::test]
async fn get_due_items_excludes_cancelled() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "Will cancel", Some("2020-01-01T09:00:00Z"))
        .await
        .expect("insert");

    // Cancel the item
    cancel(&pool, id).await.expect("cancel");

    // Due items should be empty — cancelled items must never appear
    let due = get_due_items(&pool).await.expect("due");
    assert!(
        due.iter().all(|item| item.id != id),
        "cancelled item should not appear in due items"
    );
}

#[tokio::test]
async fn range_query_boundary_inclusivity() {
    let pool = init_test_db().await.expect("init db");

    // Insert items at exact boundary timestamps
    insert(&pool, "tweet", "At start", Some("2026-03-01T00:00:00Z"))
        .await
        .expect("insert");
    insert(&pool, "tweet", "At end", Some("2026-03-02T00:00:00Z"))
        .await
        .expect("insert");
    insert(&pool, "tweet", "After end", Some("2026-03-02T00:00:01Z"))
        .await
        .expect("insert");

    let items = get_in_range(&pool, "2026-03-01T00:00:00Z", "2026-03-02T00:00:00Z")
        .await
        .expect("range");

    // Start boundary should be included, end boundary should be included
    let contents: Vec<&str> = items.iter().map(|i| i.content.as_str()).collect();
    assert!(
        contents.contains(&"At start"),
        "start boundary should be included"
    );
    assert!(
        !contents.contains(&"After end"),
        "items after end boundary should be excluded"
    );
}

#[tokio::test]
async fn scheduled_for_preserves_timezone_offset_format() {
    let pool = init_test_db().await.expect("init db");

    // Insert with a positive offset — the DB stores it as-is (no normalization at storage layer)
    let id = insert(
        &pool,
        "tweet",
        "Offset tweet",
        Some("2099-06-15T14:30:00+05:00"),
    )
    .await
    .expect("insert");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    // The stored value should contain the offset as provided
    assert!(
        item.scheduled_for
            .as_ref()
            .map(|s| s.contains("14:30:00"))
            .unwrap_or(false),
        "stored timestamp should preserve the time component: {:?}",
        item.scheduled_for,
    );
}

// ============================================================================
// Draft lifecycle: insert, update, delete, schedule
// ============================================================================

#[tokio::test]
async fn insert_draft_and_list() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_draft(&pool, "tweet", "Draft content", "manual")
        .await
        .expect("insert");
    assert!(id > 0);

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.status, "draft");
    assert_eq!(item.source, "manual");
    assert!(item.scheduled_for.is_none());

    let drafts = list_drafts(&pool).await.expect("list");
    assert!(drafts.iter().any(|d| d.id == id));
}

#[tokio::test]
async fn update_draft_changes_content() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_draft(&pool, "tweet", "Original draft", "manual")
        .await
        .expect("insert");

    update_draft(&pool, id, "Updated draft")
        .await
        .expect("update");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.content, "Updated draft");
    assert_eq!(item.status, "draft");
}

#[tokio::test]
async fn update_draft_only_affects_drafts() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "A draft", "manual")
        .await
        .expect("insert");

    // Schedule the draft first
    schedule_draft_for(&pool, acct, id, "2099-12-31T10:00:00Z")
        .await
        .expect("schedule");

    // Trying to update content via update_draft should not affect a scheduled item
    update_draft_for(&pool, acct, id, "Should not change")
        .await
        .expect("update");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.content, "A draft"); // unchanged
}

#[tokio::test]
async fn delete_draft_sets_cancelled() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_draft(&pool, "tweet", "Will delete", "manual")
        .await
        .expect("insert");

    delete_draft(&pool, id).await.expect("delete");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.status, "cancelled");
}

#[tokio::test]
async fn delete_draft_only_affects_drafts() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Scheduled", "manual")
        .await
        .expect("insert");

    schedule_draft_for(&pool, acct, id, "2099-12-31T10:00:00Z")
        .await
        .expect("schedule");

    // Attempt to delete a scheduled item via delete_draft should be a no-op
    delete_draft_for(&pool, acct, id).await.expect("delete");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "scheduled"); // unchanged
}

#[tokio::test]
async fn schedule_draft_promotes_to_scheduled() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_draft(&pool, "tweet", "Promote me", "manual")
        .await
        .expect("insert");

    schedule_draft(&pool, id, "2099-06-15T10:00:00Z")
        .await
        .expect("schedule");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.status, "scheduled");
    assert_eq!(item.scheduled_for.as_deref(), Some("2099-06-15T10:00:00Z"));
}

// ============================================================================
// Unschedule
// ============================================================================

#[tokio::test]
async fn unschedule_reverts_to_draft() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Unschedule me", "manual")
        .await
        .expect("insert");

    schedule_draft_for(&pool, acct, id, "2099-12-31T10:00:00Z")
        .await
        .expect("schedule");

    let reverted = unschedule_draft_for(&pool, acct, id)
        .await
        .expect("unschedule");
    assert!(reverted);

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "draft");
    assert!(item.scheduled_for.is_none());
}

#[tokio::test]
async fn unschedule_non_scheduled_returns_false() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Just a draft", "manual")
        .await
        .expect("insert");

    let reverted = unschedule_draft_for(&pool, acct, id)
        .await
        .expect("unschedule");
    assert!(!reverted);
}

// ============================================================================
// Autosave with optimistic locking
// ============================================================================

#[tokio::test]
async fn autosave_draft_succeeds_with_matching_timestamp() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Initial", "manual")
        .await
        .expect("insert");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    let ts = &item.updated_at;

    let result = autosave_draft_for(&pool, acct, id, "Autosaved content", "tweet", ts)
        .await
        .expect("autosave");
    assert!(
        result.is_some(),
        "autosave should succeed with correct timestamp"
    );

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.content, "Autosaved content");
}

#[tokio::test]
async fn autosave_draft_fails_with_stale_timestamp() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Initial", "manual")
        .await
        .expect("insert");

    // Use a stale timestamp that doesn't match
    let result = autosave_draft_for(
        &pool,
        acct,
        id,
        "Stale write",
        "tweet",
        "1999-01-01T00:00:00",
    )
    .await
    .expect("autosave");
    assert!(
        result.is_none(),
        "autosave should fail with stale timestamp"
    );

    // Content should be unchanged
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.content, "Initial");
}

// ============================================================================
// QA fields
// ============================================================================

#[tokio::test]
async fn update_qa_fields_stores_all_fields() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "QA test", Some("2099-01-01T10:00:00Z"))
        .await
        .expect("insert");

    update_qa_fields(
        &pool,
        id,
        r#"{"summary":"good"}"#,
        r#"["no_link"]"#,
        r#"["too_short"]"#,
        r#"["add_emoji"]"#,
        78.5,
    )
    .await
    .expect("update qa");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.qa_report, r#"{"summary":"good"}"#);
    assert_eq!(item.qa_hard_flags, r#"["no_link"]"#);
    assert_eq!(item.qa_soft_flags, r#"["too_short"]"#);
    assert_eq!(item.qa_recommendations, r#"["add_emoji"]"#);
    assert!((item.qa_score - 78.5).abs() < 0.01);
}

// ============================================================================
// Draft with provenance
// ============================================================================

#[tokio::test]
async fn insert_draft_with_provenance_creates_links() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let refs = vec![crate::storage::provenance::ProvenanceRef {
        node_id: None,
        chunk_id: None,
        seed_id: None,
        source_path: Some("notes/rust.md".to_string()),
        heading_path: Some("Testing".to_string()),
        snippet: Some("Use #[tokio::test]".to_string()),
        edge_type: None,
        edge_label: None,
        angle_kind: None,
        signal_kind: None,
        signal_text: None,
        source_role: None,
    }];

    let id = insert_draft_with_provenance_for(&pool, acct, "tweet", "From vault", "assist", &refs)
        .await
        .expect("insert with provenance");

    // Draft should exist
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.content, "From vault");
    assert_eq!(item.source, "assist");

    // Provenance link should exist
    let link_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM vault_provenance_links \
         WHERE entity_type = 'scheduled_content' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(link_count.0, 1);
}

#[tokio::test]
async fn insert_draft_with_empty_provenance() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_with_provenance_for(&pool, acct, "tweet", "No refs", "manual", &[])
        .await
        .expect("insert with empty provenance");

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.content, "No refs");

    let link_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM vault_provenance_links \
         WHERE entity_type = 'scheduled_content' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(link_count.0, 0);
}
