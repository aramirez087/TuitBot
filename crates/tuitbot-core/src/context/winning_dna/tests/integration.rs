//! Async DB integration tests for winning_dna retrieval and context building.
//! Requires a real (in-memory) SQLite DB via `init_test_db()`.

use crate::context::winning_dna::{build_draft_context, retrieve_ancestors, RAG_MAX_CHARS};
use crate::storage::{analytics, watchtower};

const TEST_ACCOUNT: &str = "00000000-0000-0000-0000-000000000000";

#[tokio::test]
async fn retrieve_ancestors_empty_db() {
    let pool = crate::storage::init_test_db().await.expect("init db");
    let ancestors = retrieve_ancestors(&pool, TEST_ACCOUNT, &[], 5, 14.0)
        .await
        .expect("retrieve");
    assert!(ancestors.is_empty());
}

#[tokio::test]
async fn retrieve_ancestors_ranks_by_weight() {
    let pool = crate::storage::init_test_db().await.expect("init db");

    for (id, content, score, eng) in [
        ("tw1", "Low performer", 30.0, 0.3),
        ("tw2", "High performer", 90.0, 0.9),
    ] {
        sqlx::query(
            "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
             VALUES ('00000000-0000-0000-0000-000000000000', ?, ?, 'rust', 'sent', '2026-02-27T10:00:00Z')",
        )
        .bind(id)
        .bind(content)
        .execute(&pool)
        .await
        .expect("insert tweet");

        analytics::upsert_tweet_performance(&pool, id, 10, 5, 3, 500, score)
            .await
            .expect("upsert perf");
        analytics::update_tweet_engagement_score(&pool, id, eng)
            .await
            .expect("update score");
    }

    let ancestors = retrieve_ancestors(&pool, TEST_ACCOUNT, &[], 5, 14.0)
        .await
        .expect("retrieve");
    assert_eq!(ancestors.len(), 2);
    assert_eq!(
        ancestors[0].tweet_id, "tw2",
        "higher score should rank first"
    );
    assert!(ancestors[0].retrieval_weight > ancestors[1].retrieval_weight);
}

#[tokio::test]
async fn cold_start_falls_back_to_seeds() {
    let pool = crate::storage::init_test_db().await.expect("init db");

    let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    watchtower::upsert_content_node(
        &pool,
        source_id,
        "notes.md",
        "h1",
        Some("My Notes"),
        "Body",
        None,
        None,
    )
    .await
    .expect("upsert node");
    watchtower::insert_draft_seed_with_weight(
        &pool,
        1,
        "Hook about Rust ownership",
        Some("tip"),
        0.5,
    )
    .await
    .expect("insert seed");

    let ctx = build_draft_context(&pool, TEST_ACCOUNT, &[], 5, 14.0)
        .await
        .expect("build context");
    assert!(ctx.winning_ancestors.is_empty());
    assert_eq!(ctx.content_seeds.len(), 1);
    assert!(ctx.prompt_block.contains("Relevant ideas"));
}

#[tokio::test]
async fn build_draft_context_formats_prompt_with_ancestors() {
    let pool = crate::storage::init_test_db().await.expect("init db");

    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES ('00000000-0000-0000-0000-000000000000', 'tw1', 'Testing is key', 'rust', 'sent', '2026-02-27T10:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("insert tweet");

    analytics::upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert perf");
    analytics::update_tweet_engagement_score(&pool, "tw1", 0.85)
        .await
        .expect("update score");

    let ctx = build_draft_context(&pool, TEST_ACCOUNT, &[], 5, 14.0)
        .await
        .expect("build context");
    assert_eq!(ctx.winning_ancestors.len(), 1);
    assert!(ctx.prompt_block.contains("Winning patterns"));
    assert!(ctx.prompt_block.len() <= RAG_MAX_CHARS);
}

#[tokio::test]
async fn build_draft_context_empty_db_returns_empty_prompt() {
    let pool = crate::storage::init_test_db().await.expect("init db");
    let ctx = build_draft_context(&pool, TEST_ACCOUNT, &[], 5, 14.0)
        .await
        .expect("build context");
    assert!(ctx.winning_ancestors.is_empty());
    assert!(ctx.content_seeds.is_empty());
    assert!(ctx.prompt_block.is_empty());
}

#[tokio::test]
async fn retrieve_ancestors_account_isolation() {
    let pool = crate::storage::init_test_db().await.expect("init db");

    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES ('account-a', 'tw-a', 'Account A tweet', 'rust', 'sent', '2026-02-27T10:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("insert tweet");

    analytics::upsert_tweet_performance(&pool, "tw-a", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert perf");
    analytics::update_tweet_engagement_score(&pool, "tw-a", 0.9)
        .await
        .expect("update score");

    let ancestors = retrieve_ancestors(&pool, "account-b", &[], 5, 14.0)
        .await
        .expect("retrieve");
    assert!(
        ancestors.is_empty(),
        "account B should see no ancestors from account A"
    );

    let ancestors = retrieve_ancestors(&pool, "account-a", &[], 5, 14.0)
        .await
        .expect("retrieve");
    assert_eq!(ancestors.len(), 1);
}

#[tokio::test]
async fn build_draft_context_with_fragments() {
    let pool = crate::storage::init_test_db().await.expect("init db");

    let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    watchtower::upsert_content_node(
        &pool,
        source_id,
        "rust-tips.md",
        "h-frag",
        Some("Rust Tips"),
        "Ownership makes concurrency safe in Rust",
        None,
        None,
    )
    .await
    .expect("upsert node");
    watchtower::insert_chunk(
        &pool,
        TEST_ACCOUNT,
        1,
        "# Rust Tips",
        "Ownership makes concurrency safe in Rust",
        "hash-frag-1",
        0,
    )
    .await
    .expect("insert chunk");

    let ctx = build_draft_context(&pool, TEST_ACCOUNT, &["rust".to_string()], 5, 14.0)
        .await
        .expect("build context");

    assert!(ctx.prompt_block.contains("Relevant knowledge"));
    assert_eq!(ctx.vault_citations.len(), 1);
    assert_eq!(ctx.vault_citations[0].source_path, "rust-tips.md");
}

#[tokio::test]
async fn build_draft_context_mixed_ancestors_and_fragments() {
    let pool = crate::storage::init_test_db().await.expect("init db");

    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES (?, 'tw-mix', 'Testing patterns rock', 'testing', 'sent', '2026-02-27T10:00:00Z')",
    )
    .bind(TEST_ACCOUNT)
    .execute(&pool)
    .await
    .expect("insert tweet");
    analytics::upsert_tweet_performance(&pool, "tw-mix", 10, 5, 3, 500, 82.0)
        .await
        .expect("upsert perf");
    analytics::update_tweet_engagement_score(&pool, "tw-mix", 0.9)
        .await
        .expect("update score");

    let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    watchtower::upsert_content_node(
        &pool,
        source_id,
        "testing.md",
        "h-mix",
        Some("Testing Notes"),
        "Testing strategies for better code",
        None,
        None,
    )
    .await
    .expect("upsert node");
    watchtower::insert_chunk(
        &pool,
        TEST_ACCOUNT,
        1,
        "# Testing",
        "Testing strategies for better code",
        "hash-mix-1",
        0,
    )
    .await
    .expect("insert chunk");

    let ctx = build_draft_context(&pool, TEST_ACCOUNT, &["testing".to_string()], 5, 14.0)
        .await
        .expect("build context");

    assert!(!ctx.winning_ancestors.is_empty(), "should have ancestors");
    assert!(!ctx.vault_citations.is_empty(), "should have citations");
    assert!(ctx.prompt_block.contains("Winning patterns"));
    assert!(ctx.prompt_block.contains("Relevant knowledge"));
    assert!(ctx.prompt_block.len() <= RAG_MAX_CHARS);
}

#[tokio::test]
async fn fragment_citations_populated_correctly() {
    let pool = crate::storage::init_test_db().await.expect("init db");

    let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    watchtower::upsert_content_node(
        &pool,
        source_id,
        "notes/deep-work.md",
        "h-cite",
        Some("Deep Work"),
        "Focus on cognitively demanding tasks",
        None,
        None,
    )
    .await
    .expect("upsert node");
    watchtower::insert_chunk(
        &pool,
        TEST_ACCOUNT,
        1,
        "# Productivity > ## Deep Work",
        "Focus on cognitively demanding tasks without distraction",
        "hash-cite-1",
        0,
    )
    .await
    .expect("insert chunk");

    let ctx = build_draft_context(&pool, TEST_ACCOUNT, &["focus".to_string()], 5, 14.0)
        .await
        .expect("build context");

    assert_eq!(ctx.vault_citations.len(), 1);
    let cite = &ctx.vault_citations[0];
    assert_eq!(cite.source_path, "notes/deep-work.md");
    assert_eq!(cite.source_title.as_deref(), Some("Deep Work"));
    assert_eq!(cite.heading_path, "# Productivity > ## Deep Work");
    assert!(cite.snippet.contains("cognitively demanding"));
}
