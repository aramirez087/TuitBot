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
