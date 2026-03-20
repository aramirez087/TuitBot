//! Additional deep coverage tests.

use super::super::*;
use crate::storage::init_test_db;

#[tokio::test]
async fn activity_and_revisions_for_account_scoped() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-act-rev";

    crate::storage::accounts::create_account(&pool, acct, "AR")
        .await
        .expect("create");

    let draft_id = insert_draft_for(&pool, acct, "tweet", "Draft v1", "manual")
        .await
        .expect("insert");

    // Insert activity
    let act_id = insert_activity_for(&pool, acct, draft_id, "created", Some("Initial draft"))
        .await
        .expect("insert activity");
    assert!(act_id > 0);

    insert_activity_for(&pool, acct, draft_id, "edited", None)
        .await
        .expect("insert activity 2");

    let activities = list_activity_for(&pool, acct, draft_id)
        .await
        .expect("list activity");
    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].action, "edited"); // newest first

    // Insert revisions
    let rev_id = insert_revision_for(&pool, acct, draft_id, "Draft v1", "tweet", "created")
        .await
        .expect("insert revision");
    insert_revision_for(&pool, acct, draft_id, "Draft v2", "tweet", "autosave")
        .await
        .expect("insert revision 2");

    let revisions = list_revisions_for(&pool, acct, draft_id)
        .await
        .expect("list revisions");
    assert_eq!(revisions.len(), 2);
    assert_eq!(revisions[0].content, "Draft v2"); // newest first

    // Get revision by ID
    let rev = get_revision_for(&pool, acct, draft_id, rev_id)
        .await
        .expect("get revision")
        .expect("exists");
    assert_eq!(rev.content, "Draft v1");
    assert_eq!(rev.trigger_kind, "created");

    // Get revision from wrong content_id
    let no_rev = get_revision_for(&pool, acct, 9999, rev_id)
        .await
        .expect("get revision wrong");
    assert!(no_rev.is_none());
}

#[tokio::test]
async fn tags_for_account_scoped() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-tags";

    crate::storage::accounts::create_account(&pool, acct, "Tags")
        .await
        .expect("create");

    // Create tags
    let tag1 = create_tag_for(&pool, acct, "urgent", Some("#ff0000"))
        .await
        .expect("create tag 1");
    let tag2 = create_tag_for(&pool, acct, "later", None)
        .await
        .expect("create tag 2");

    let tags = list_tags_for(&pool, acct).await.expect("list tags");
    assert_eq!(tags.len(), 2);

    // Create a draft and assign tags
    let draft_id = insert_draft_for(&pool, acct, "tweet", "Tagged draft", "manual")
        .await
        .expect("insert draft");

    assign_tag_for(&pool, draft_id, tag1)
        .await
        .expect("assign 1");
    assign_tag_for(&pool, draft_id, tag2)
        .await
        .expect("assign 2");

    // Double-assign is a no-op
    assign_tag_for(&pool, draft_id, tag1)
        .await
        .expect("assign 1 again");

    let draft_tags = list_draft_tags_for(&pool, acct, draft_id)
        .await
        .expect("list draft tags");
    assert_eq!(draft_tags.len(), 2);

    // Unassign
    let removed = unassign_tag_for(&pool, draft_id, tag1)
        .await
        .expect("unassign");
    assert!(removed);

    let draft_tags = list_draft_tags_for(&pool, acct, draft_id)
        .await
        .expect("list draft tags after unassign");
    assert_eq!(draft_tags.len(), 1);
    assert_eq!(draft_tags[0].name, "later");

    // Unassign already-removed tag returns false
    let removed = unassign_tag_for(&pool, draft_id, tag1)
        .await
        .expect("unassign again");
    assert!(!removed);
}

#[tokio::test]
async fn update_status_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct_a = "acct-status-a";
    let acct_b = "acct-status-b";

    crate::storage::accounts::create_account(&pool, acct_a, "A")
        .await
        .expect("create a");
    crate::storage::accounts::create_account(&pool, acct_b, "B")
        .await
        .expect("create b");

    let id = insert_for(
        &pool,
        acct_a,
        "tweet",
        "Status test",
        Some("2026-01-01T10:00:00Z"),
    )
    .await
    .expect("insert");

    // Update from wrong account — should not change
    update_status_for(&pool, acct_b, id, "posted", Some("tweet-123"))
        .await
        .expect("update wrong acct");
    let item = get_by_id_for(&pool, acct_a, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "scheduled");

    // Update from correct account
    update_status_for(&pool, acct_a, id, "posted", Some("tweet-456"))
        .await
        .expect("update correct acct");
    let item = get_by_id_for(&pool, acct_a, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "posted");
    assert_eq!(item.posted_tweet_id.as_deref(), Some("tweet-456"));
}

#[tokio::test]
async fn delete_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-deldraft";

    crate::storage::accounts::create_account(&pool, acct, "Del")
        .await
        .expect("create");

    let id = insert_draft_for(&pool, acct, "tweet", "Delete me", "manual")
        .await
        .expect("insert");

    // Delete from wrong account — should be no-op
    delete_draft_for(&pool, "wrong-acct", id)
        .await
        .expect("del wrong");
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "draft");

    // Delete from correct account
    delete_draft_for(&pool, acct, id)
        .await
        .expect("del correct");
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.status, "cancelled");
}

#[tokio::test]
async fn update_draft_for_account_isolation() {
    let pool = init_test_db().await.expect("init db");
    let acct = "acct-upddraft";

    crate::storage::accounts::create_account(&pool, acct, "Upd")
        .await
        .expect("create");

    let id = insert_draft_for(&pool, acct, "tweet", "Before", "manual")
        .await
        .expect("insert");

    // Update from wrong account — should be no-op
    update_draft_for(&pool, "wrong-acct", id, "Hacked")
        .await
        .expect("update wrong");
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.content, "Before");

    // Update from correct account
    update_draft_for(&pool, acct, id, "After")
        .await
        .expect("update correct");
    let item = get_by_id_for(&pool, acct, id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(item.content, "After");
}

#[tokio::test]
async fn serialize_json_string_in_scheduled_content() {
    let pool = init_test_db().await.expect("init db");
    let id = insert(&pool, "tweet", "JSON test", Some("2026-01-01T10:00:00Z"))
        .await
        .expect("insert");

    update_qa_fields(
        &pool,
        id,
        r#"{"summary":"good"}"#,
        r#"["flag1","flag2"]"#,
        r#"["soft1"]"#,
        r#"["rec1"]"#,
        92.5,
    )
    .await
    .expect("update qa");

    let item = get_by_id(&pool, id).await.expect("get").expect("exists");

    // Serialize to JSON to verify serialize_json_string works
    let json = serde_json::to_value(&item).expect("serialize");
    assert!(json["qa_report"].is_object());
    assert!(json["qa_hard_flags"].is_array());
    assert_eq!(json["qa_hard_flags"].as_array().unwrap().len(), 2);
    assert!(json["qa_soft_flags"].is_array());
    assert!(json["qa_recommendations"].is_array());
    assert!((json["qa_score"].as_f64().unwrap() - 92.5).abs() < 0.01);
}

#[tokio::test]
async fn duplicate_draft_with_title_appends_copy() {
    let pool = init_test_db().await.expect("init db");
    let acct = "00000000-0000-0000-0000-000000000000";

    let id = insert_draft(&pool, "tweet", "My content", "manual")
        .await
        .expect("insert");

    // Set a title
    update_draft_meta_for(&pool, acct, id, Some("My Great Tweet"), None)
        .await
        .expect("set title");

    let dup_id = duplicate_draft_for(&pool, acct, id)
        .await
        .expect("duplicate")
        .expect("should return id");

    let dup = get_by_id(&pool, dup_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(dup.title.as_deref(), Some("My Great Tweet (copy)"));
    assert_eq!(dup.content, "My content");
    assert_eq!(dup.status, "draft");
}
