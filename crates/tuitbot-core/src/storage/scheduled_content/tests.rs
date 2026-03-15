use super::*;
use crate::storage::init_test_db;

// ============================================================================
// Existing tests (preserved from pre-module-split)
// ============================================================================

#[tokio::test]
async fn insert_and_retrieve() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "Hello world!", Some("2026-02-24T09:15:00Z"))
        .await
        .expect("insert");
    assert!(id > 0);

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.content_type, "tweet");
    assert_eq!(item.content, "Hello world!");
    assert_eq!(item.scheduled_for.as_deref(), Some("2026-02-24T09:15:00Z"));
    assert_eq!(item.status, "scheduled");
    assert!(item.posted_tweet_id.is_none());
    // New fields should be None on legacy inserts
    assert!(item.title.is_none());
    assert!(item.notes.is_none());
    assert!(item.archived_at.is_none());
}

#[tokio::test]
async fn insert_without_scheduled_time() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "No time set", None)
        .await
        .expect("insert");
    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert!(item.scheduled_for.is_none());
}

#[tokio::test]
async fn get_in_range_filters() {
    let pool = init_test_db().await.expect("init db");

    insert(&pool, "tweet", "In range", Some("2026-02-24T09:00:00Z"))
        .await
        .expect("insert");
    insert(&pool, "tweet", "Out of range", Some("2026-03-01T09:00:00Z"))
        .await
        .expect("insert");

    let items = get_in_range(&pool, "2026-02-23T00:00:00Z", "2026-02-25T00:00:00Z")
        .await
        .expect("range");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].content, "In range");
}

#[tokio::test]
async fn get_due_items_returns_past_scheduled() {
    let pool = init_test_db().await.expect("init db");

    insert(&pool, "tweet", "Past tweet", Some("2020-01-01T09:00:00Z"))
        .await
        .expect("insert");
    insert(&pool, "tweet", "Future tweet", Some("2099-01-01T09:00:00Z"))
        .await
        .expect("insert");
    insert(&pool, "tweet", "No schedule", None)
        .await
        .expect("insert");

    let due = get_due_items(&pool).await.expect("due");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].content, "Past tweet");
}

#[tokio::test]
async fn update_status_marks_posted() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "Will post", Some("2026-02-24T09:00:00Z"))
        .await
        .expect("insert");

    update_status(&pool, id, "posted", Some("x_tweet_123"))
        .await
        .expect("update");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.status, "posted");
    assert_eq!(item.posted_tweet_id.as_deref(), Some("x_tweet_123"));
}

#[tokio::test]
async fn cancel_sets_cancelled_status() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "Will cancel", Some("2026-02-24T09:00:00Z"))
        .await
        .expect("insert");

    cancel(&pool, id).await.expect("cancel");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.status, "cancelled");
}

#[tokio::test]
async fn cancel_only_affects_scheduled_items() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "Posted item", Some("2026-02-24T09:00:00Z"))
        .await
        .expect("insert");

    update_status(&pool, id, "posted", Some("x_123"))
        .await
        .expect("update");

    cancel(&pool, id).await.expect("cancel");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.status, "posted");
}

#[tokio::test]
async fn update_content_changes_text_and_time() {
    let pool = init_test_db().await.expect("init db");

    let id = insert(&pool, "tweet", "Original", Some("2026-02-24T09:00:00Z"))
        .await
        .expect("insert");

    update_content(&pool, id, "Updated text", Some("2026-02-25T12:00:00Z"))
        .await
        .expect("update");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.content, "Updated text");
    assert_eq!(item.scheduled_for.as_deref(), Some("2026-02-25T12:00:00Z"));
}

#[tokio::test]
async fn get_nonexistent_returns_none() {
    let pool = init_test_db().await.expect("init db");
    let item = get_by_id(&pool, 999).await.expect("get");
    assert!(item.is_none());
}

#[tokio::test]
async fn insert_thread_content() {
    let pool = init_test_db().await.expect("init db");

    let thread_content =
        serde_json::to_string(&vec!["First tweet", "Second tweet", "Third tweet"]).expect("json");
    let id = insert(
        &pool,
        "thread",
        &thread_content,
        Some("2026-02-24T10:00:00Z"),
    )
    .await
    .expect("insert");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");
    assert_eq!(item.content_type, "thread");

    let tweets: Vec<String> = serde_json::from_str(&item.content).expect("parse");
    assert_eq!(tweets.len(), 3);
}

// ============================================================================
// Draft Studio: archive and restore
// ============================================================================

#[tokio::test]
async fn archive_and_restore_draft() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Archivable draft", "manual")
        .await
        .expect("insert");

    // Draft appears in list
    let drafts = list_drafts_for(&pool, acct).await.expect("list");
    assert!(drafts.iter().any(|d| d.id == id));

    // Archive it
    let changed = archive_draft_for(&pool, acct, id).await.expect("archive");
    assert!(changed);

    // No longer in active draft list
    let drafts = list_drafts_for(&pool, acct).await.expect("list");
    assert!(!drafts.iter().any(|d| d.id == id));

    // But still fetchable by ID (with archived_at set)
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert!(item.archived_at.is_some());

    // Restore it
    let changed = restore_draft_for(&pool, acct, id).await.expect("restore");
    assert!(changed);

    // Back in active list
    let drafts = list_drafts_for(&pool, acct).await.expect("list");
    assert!(drafts.iter().any(|d| d.id == id));

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert!(item.archived_at.is_none());
}

#[tokio::test]
async fn archive_already_archived_is_noop() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Draft", "manual")
        .await
        .expect("insert");

    let first = archive_draft_for(&pool, acct, id).await.expect("archive");
    assert!(first);

    let second = archive_draft_for(&pool, acct, id)
        .await
        .expect("archive again");
    assert!(!second); // no rows affected
}

// ============================================================================
// Draft Studio: duplicate
// ============================================================================

#[tokio::test]
async fn duplicate_draft_copies_content() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Original content", "manual")
        .await
        .expect("insert");

    // Set a title on the original
    update_draft_meta_for(&pool, acct, id, Some("My Draft"), None)
        .await
        .expect("meta");

    let new_id = duplicate_draft_for(&pool, acct, id)
        .await
        .expect("duplicate")
        .expect("should return id");

    assert_ne!(id, new_id);

    let copy = get_by_id_for(&pool, acct, new_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(copy.content, "Original content");
    assert_eq!(copy.content_type, "tweet");
    assert_eq!(copy.status, "draft");
    assert_eq!(copy.title.as_deref(), Some("My Draft (copy)"));
    assert!(copy.archived_at.is_none());
    assert!(copy.scheduled_for.is_none());
}

#[tokio::test]
async fn duplicate_nonexistent_returns_none() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let result = duplicate_draft_for(&pool, acct, 999)
        .await
        .expect("duplicate");
    assert!(result.is_none());
}

// ============================================================================
// Draft Studio: metadata
// ============================================================================

#[tokio::test]
async fn update_draft_meta_sets_title_and_notes() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Some content", "manual")
        .await
        .expect("insert");

    let changed = update_draft_meta_for(&pool, acct, id, Some("Title"), Some("My notes"))
        .await
        .expect("meta");
    assert!(changed);

    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.title.as_deref(), Some("Title"));
    assert_eq!(item.notes.as_deref(), Some("My notes"));
}

// ============================================================================
// Draft Studio: revisions
// ============================================================================

#[tokio::test]
async fn insert_and_list_revisions() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Current text", "manual")
        .await
        .expect("insert");

    insert_revision_for(&pool, acct, id, "Version 1", "tweet", "manual")
        .await
        .expect("rev1");
    insert_revision_for(&pool, acct, id, "Version 2", "tweet", "ai_rewrite")
        .await
        .expect("rev2");

    let revs = list_revisions_for(&pool, acct, id).await.expect("list");
    assert_eq!(revs.len(), 2);
    // Newest first
    assert_eq!(revs[0].content, "Version 2");
    assert_eq!(revs[0].trigger_kind, "ai_rewrite");
    assert_eq!(revs[1].content, "Version 1");
    assert_eq!(revs[1].trigger_kind, "manual");
}

// ============================================================================
// Draft Studio: activity
// ============================================================================

#[tokio::test]
async fn insert_and_list_activity() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Draft", "manual")
        .await
        .expect("insert");

    insert_activity_for(&pool, acct, id, "created", None)
        .await
        .expect("act1");
    insert_activity_for(&pool, acct, id, "edited", Some("{\"chars\":42}"))
        .await
        .expect("act2");

    let acts = list_activity_for(&pool, acct, id).await.expect("list");
    assert_eq!(acts.len(), 2);
    assert_eq!(acts[0].action, "edited");
    assert_eq!(acts[0].detail.as_deref(), Some("{\"chars\":42}"));
    assert_eq!(acts[1].action, "created");
    assert!(acts[1].detail.is_none());
}

// ============================================================================
// Draft Studio: tags
// ============================================================================

#[tokio::test]
async fn create_and_list_tags() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id1 = create_tag_for(&pool, acct, "marketing", Some("#ff0000"))
        .await
        .expect("tag1");
    let id2 = create_tag_for(&pool, acct, "announcement", None)
        .await
        .expect("tag2");
    assert_ne!(id1, id2);

    let tags = list_tags_for(&pool, acct).await.expect("list");
    assert_eq!(tags.len(), 2);
    // Ordered by name
    assert_eq!(tags[0].name, "announcement");
    assert_eq!(tags[1].name, "marketing");
    assert_eq!(tags[1].color.as_deref(), Some("#ff0000"));
}

#[tokio::test]
async fn assign_and_unassign_tag() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let draft_id = insert_draft_for(&pool, acct, "tweet", "Tagged draft", "manual")
        .await
        .expect("insert");
    let tag_id = create_tag_for(&pool, acct, "important", None)
        .await
        .expect("tag");

    // Assign
    assign_tag_for(&pool, draft_id, tag_id)
        .await
        .expect("assign");

    // Verify via raw query
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM content_tag_assignments WHERE content_id = ? AND tag_id = ?",
    )
    .bind(draft_id)
    .bind(tag_id)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(count.0, 1);

    // Assign again (no-op via INSERT OR IGNORE)
    assign_tag_for(&pool, draft_id, tag_id)
        .await
        .expect("re-assign");

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM content_tag_assignments WHERE content_id = ? AND tag_id = ?",
    )
    .bind(draft_id)
    .bind(tag_id)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(count.0, 1); // still 1

    // Unassign
    let removed = unassign_tag_for(&pool, draft_id, tag_id)
        .await
        .expect("unassign");
    assert!(removed);

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM content_tag_assignments WHERE content_id = ? AND tag_id = ?",
    )
    .bind(draft_id)
    .bind(tag_id)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(count.0, 0);
}

// ============================================================================
// Reschedule
// ============================================================================

#[tokio::test]
async fn reschedule_draft_updates_time() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft_for(&pool, acct, "tweet", "Reschedule me", "manual")
        .await
        .expect("insert");

    // Schedule the draft first
    schedule_draft_for(&pool, acct, id, "2099-12-31T10:00:00Z")
        .await
        .expect("schedule");

    // Verify it's scheduled
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "scheduled");
    assert_eq!(item.scheduled_for.as_deref(), Some("2099-12-31T10:00:00Z"));

    // Reschedule to a new time
    let updated = reschedule_draft_for(&pool, acct, id, "2099-12-31T15:00:00Z")
        .await
        .expect("reschedule");
    assert!(updated);

    // Verify the new time
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "scheduled");
    assert_eq!(item.scheduled_for.as_deref(), Some("2099-12-31T15:00:00Z"));
}

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
