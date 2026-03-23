//! E2E integration tests: full pipeline validation across content source providers.

/// E2E: Local folder ingest -> front-matter parsing -> dedup -> seed generation -> draft context.
#[tokio::test]
async fn e2e_local_folder_ingest_to_seed_pipeline() {
    use std::sync::Arc;

    use crate::automation::watchtower::ingest_file;
    use crate::context::winning_dna::build_draft_context;
    use crate::error::LlmError;
    use crate::llm::{GenerationParams, LlmProvider, LlmResponse, TokenUsage};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    struct MockLlm;
    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        fn name(&self) -> &str {
            "mock"
        }
        async fn complete(
            &self,
            _system: &str,
            _user_message: &str,
            _params: &GenerationParams,
        ) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                text: "HOOK: Product strategy is underrated\nFORMAT: tip".to_string(),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
            })
        }
        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    // Step 1: Create source files.
    let md_with_fm = "---\ntitle: Product Strategy\ntags: [strategy, growth]\n---\nOur key differentiator is authentic engagement.\n";
    let md_plain = "# Quick Idea\nSometimes simple wins.\n";
    std::fs::write(dir.path().join("strategy.md"), md_with_fm).unwrap();
    std::fs::write(dir.path().join("idea.md"), md_plain).unwrap();

    // Step 2: Register source and ingest files.
    let src_id = store::insert_source_context(&pool, "local_fs", r#"{"path":"test"}"#)
        .await
        .unwrap();

    ingest_file(&pool, src_id, dir.path(), "strategy.md", false)
        .await
        .unwrap();
    ingest_file(&pool, src_id, dir.path(), "idea.md", false)
        .await
        .unwrap();

    // Step 3: Verify content nodes were created with correct status.
    let nodes = store::get_nodes_for_source(&pool, src_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 2);
    assert!(nodes.iter().all(|n| n.status == "pending"));

    // Verify front-matter was parsed.
    let strategy_node = nodes
        .iter()
        .find(|n| n.relative_path == "strategy.md")
        .unwrap();
    assert_eq!(strategy_node.title.as_deref(), Some("Product Strategy"));
    assert_eq!(strategy_node.tags.as_deref(), Some("strategy,growth"));

    // Step 4: Verify dedup — re-ingest same content -> Skipped.
    let r = ingest_file(&pool, src_id, dir.path(), "strategy.md", false)
        .await
        .unwrap();
    assert_eq!(r, store::UpsertResult::Skipped);

    // Step 5: Verify pending nodes are returned for seed generation.
    let pending = store::get_pending_content_nodes(&pool, 10).await.unwrap();
    assert_eq!(pending.len(), 2);

    // Step 6: Run seed generation with mock LLM.
    let worker = crate::automation::seed_worker::SeedWorker::new(pool.clone(), Arc::new(MockLlm));
    for node in &pending {
        worker.process_node_for_test(node).await.unwrap();
        store::mark_node_processed(&pool, node.id).await.unwrap();
    }

    // Step 7: Verify seeds exist.
    let seeds = store::get_seeds_for_context(&pool, 10).await.unwrap();
    assert_eq!(seeds.len(), 2); // 1 seed per node from our mock
    assert!(seeds
        .iter()
        .any(|s| s.seed_text.contains("Product strategy")));

    // Step 8: Build draft context — should use cold-start path (no ancestors).
    let ctx = build_draft_context(
        &pool,
        crate::storage::accounts::DEFAULT_ACCOUNT_ID,
        &["strategy".into()],
        5,
        14.0,
    )
    .await
    .unwrap();
    assert!(ctx.winning_ancestors.is_empty());
    assert_eq!(ctx.content_seeds.len(), 2);
    assert!(!ctx.prompt_block.is_empty());
}

/// E2E: Simulated Google Drive ingest -> dedup -> update -> seed generation -> draft context.
#[tokio::test]
async fn e2e_google_drive_ingest_to_seed_pipeline() {
    use std::sync::Arc;

    use crate::automation::watchtower::ingest_content;
    use crate::context::winning_dna::build_draft_context;
    use crate::error::LlmError;
    use crate::llm::{GenerationParams, LlmProvider, LlmResponse, TokenUsage};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    struct MockLlm;
    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        fn name(&self) -> &str {
            "mock"
        }
        async fn complete(
            &self,
            _s: &str,
            _u: &str,
            _p: &GenerationParams,
        ) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                text: "HOOK: Remote content insight\nFORMAT: contrarian_take".to_string(),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
            })
        }
        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    let pool = init_test_db().await.expect("init db");

    // Step 1: Register a Google Drive source.
    let src_id =
        store::ensure_google_drive_source(&pool, "folder_abc", r#"{"folder_id":"folder_abc"}"#)
            .await
            .unwrap();

    // Step 2: Ingest via direct content with gdrive:// provider ID.
    let body = "---\ntitle: Remote Notes\ntags: [remote, gdrive]\n---\nContent from Drive.\n";
    let r1 = ingest_content(&pool, src_id, "gdrive://fileA/notes.md", body, false)
        .await
        .unwrap();
    assert_eq!(r1, store::UpsertResult::Inserted);

    // Step 3: Verify content node with gdrive:// relative_path.
    let nodes = store::get_nodes_for_source(&pool, src_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].relative_path, "gdrive://fileA/notes.md");
    assert_eq!(nodes[0].status, "pending");
    assert_eq!(nodes[0].title.as_deref(), Some("Remote Notes"));

    // Step 4: Same content -> Skipped.
    let r2 = ingest_content(&pool, src_id, "gdrive://fileA/notes.md", body, false)
        .await
        .unwrap();
    assert_eq!(r2, store::UpsertResult::Skipped);

    // Step 5: Updated content -> Updated, status resets to pending.
    let updated = "---\ntitle: Remote Notes v2\n---\nUpdated Drive content.\n";
    let r3 = ingest_content(&pool, src_id, "gdrive://fileA/notes.md", updated, false)
        .await
        .unwrap();
    assert_eq!(r3, store::UpsertResult::Updated);

    let nodes = store::get_nodes_for_source(&pool, src_id, Some("pending"))
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].title.as_deref(), Some("Remote Notes v2"));

    // Step 6: Run seed generation.
    let worker = crate::automation::seed_worker::SeedWorker::new(pool.clone(), Arc::new(MockLlm));
    worker.process_node_for_test(&nodes[0]).await.unwrap();
    store::mark_node_processed(&pool, nodes[0].id)
        .await
        .unwrap();

    // Step 7: Build draft context -> cold-start seeds appear.
    let ctx = build_draft_context(
        &pool,
        crate::storage::accounts::DEFAULT_ACCOUNT_ID,
        &["remote".into()],
        5,
        14.0,
    )
    .await
    .unwrap();
    assert!(ctx.winning_ancestors.is_empty());
    assert_eq!(ctx.content_seeds.len(), 1);
    assert!(ctx.content_seeds[0]
        .seed_text
        .contains("Remote content insight"));
}

/// E2E: Mixed local + Drive sources both feed draft context.
#[tokio::test]
async fn e2e_mixed_sources_feed_draft_context() {
    use std::sync::Arc;

    use crate::automation::watchtower::{ingest_content, ingest_file};
    use crate::context::winning_dna::build_draft_context;
    use crate::error::LlmError;
    use crate::llm::{GenerationParams, LlmProvider, LlmResponse, TokenUsage};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    struct MockLlm;
    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        fn name(&self) -> &str {
            "mock"
        }
        async fn complete(
            &self,
            _s: &str,
            user_msg: &str,
            _p: &GenerationParams,
        ) -> Result<LlmResponse, LlmError> {
            // Return different hooks based on content to verify source attribution.
            let hook = if user_msg.contains("local vault") {
                "Local vault insight"
            } else {
                "Drive folder insight"
            };
            Ok(LlmResponse {
                text: format!("HOOK: {hook}\nFORMAT: tip"),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
            })
        }
        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    // Create local source.
    std::fs::write(dir.path().join("note.md"), "Content from local vault.\n").unwrap();
    let local_id = store::insert_source_context(&pool, "local_fs", r#"{"path":"test"}"#)
        .await
        .unwrap();
    ingest_file(&pool, local_id, dir.path(), "note.md", false)
        .await
        .unwrap();

    // Create Drive source.
    let drive_id =
        store::ensure_google_drive_source(&pool, "drv_123", r#"{"folder_id":"drv_123"}"#)
            .await
            .unwrap();
    ingest_content(
        &pool,
        drive_id,
        "gdrive://fileX/doc.md",
        "Content from Drive folder.\n",
        false,
    )
    .await
    .unwrap();

    // Generate seeds from both sources.
    let worker = crate::automation::seed_worker::SeedWorker::new(pool.clone(), Arc::new(MockLlm));
    let pending = store::get_pending_content_nodes(&pool, 10).await.unwrap();
    assert_eq!(pending.len(), 2);

    for node in &pending {
        worker.process_node_for_test(node).await.unwrap();
        store::mark_node_processed(&pool, node.id).await.unwrap();
    }

    // Build context — seeds from both sources appear.
    let ctx = build_draft_context(
        &pool,
        crate::storage::accounts::DEFAULT_ACCOUNT_ID,
        &[],
        5,
        14.0,
    )
    .await
    .unwrap();
    assert_eq!(ctx.content_seeds.len(), 2);

    let texts: Vec<&str> = ctx
        .content_seeds
        .iter()
        .map(|s| s.seed_text.as_str())
        .collect();
    assert!(texts.iter().any(|t| t.contains("Local vault")));
    assert!(texts.iter().any(|t| t.contains("Drive folder")));
}

/// E2E: Inline node ingest creates manual source and supports dedup.
#[tokio::test]
async fn e2e_inline_node_ingest_creates_manual_source() {
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    // Step 1: Ensure manual source.
    let src_id = store::ensure_manual_source(&pool).await.unwrap();

    // Step 2: Upsert inline nodes (simulating POST /api/ingest).
    let hash1 = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"# My Idea\nContent here.\n");
        format!("{:x}", h.finalize())
    };
    let r1 = store::upsert_content_node(
        &pool,
        src_id,
        "idea.md",
        &hash1,
        Some("My Idea"),
        "# My Idea\nContent here.\n",
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(r1, store::UpsertResult::Inserted);

    let hash2 = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"# Second\nMore content.\n");
        format!("{:x}", h.finalize())
    };
    let r2 = store::upsert_content_node(
        &pool,
        src_id,
        "second.md",
        &hash2,
        Some("Second"),
        "# Second\nMore content.\n",
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(r2, store::UpsertResult::Inserted);

    // Step 3: Same content -> Skipped.
    let r3 = store::upsert_content_node(
        &pool,
        src_id,
        "idea.md",
        &hash1,
        Some("My Idea"),
        "# My Idea\nContent here.\n",
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(r3, store::UpsertResult::Skipped);

    // Step 4: Verify manual source context exists.
    let ctx = store::get_source_context(&pool, src_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ctx.source_type, "manual");

    // Step 5: Verify nodes are linked to the manual source.
    let nodes = store::get_nodes_for_source(&pool, src_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 2);
}

/// E2E: Loopback writes metadata to source file, idempotently, and re-ingest detects change.
#[tokio::test]
async fn e2e_loopback_writes_metadata_and_reingest_detects_change() {
    use crate::automation::watchtower::{ingest_file, loopback};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    // Step 1: Write a source file with front-matter.
    let original = "---\ntitle: My Launch Plan\ntags: [launch]\n---\nDetailed launch strategy.\n";
    std::fs::write(dir.path().join("launch.md"), original).unwrap();

    // Step 2: Ingest the file.
    let src_id = store::insert_source_context(&pool, "local_fs", r#"{"path":"test"}"#)
        .await
        .unwrap();
    let r1 = ingest_file(&pool, src_id, dir.path(), "launch.md", false)
        .await
        .unwrap();
    assert_eq!(r1, store::UpsertResult::Inserted);

    // Step 3: Write loopback metadata.
    let entry = loopback::LoopBackEntry {
        tweet_id: "tweet_999".to_string(),
        url: "https://x.com/user/status/tweet_999".to_string(),
        published_at: "2026-02-28T10:00:00Z".to_string(),
        content_type: "tweet".to_string(),
        status: None,
        thread_url: None,
        child_tweet_ids: None,
    };
    let written =
        loopback::write_metadata_to_file(dir.path().join("launch.md").as_path(), &entry).unwrap();
    assert!(written);

    // Step 4: Verify original front-matter preserved.
    let content = std::fs::read_to_string(dir.path().join("launch.md")).unwrap();
    assert!(content.contains("My Launch Plan") || content.contains("launch"));
    assert!(content.contains("tweet_999"));
    assert!(content.contains("Detailed launch strategy."));

    // Step 5: Idempotent — same tweet_id -> no modification.
    let written2 =
        loopback::write_metadata_to_file(dir.path().join("launch.md").as_path(), &entry).unwrap();
    assert!(!written2);

    // Step 6: Re-ingest -> content hash changed (front-matter updated) -> Updated.
    let r2 = ingest_file(&pool, src_id, dir.path(), "launch.md", false)
        .await
        .unwrap();
    assert_eq!(r2, store::UpsertResult::Updated);

    // Verify the node's status reset to 'pending' after update.
    let nodes = store::get_nodes_for_source(&pool, src_id, Some("pending"))
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].relative_path, "launch.md");
}

// ---------------------------------------------------------------------------
// E2E: Connection-based Drive ingest
// ---------------------------------------------------------------------------

/// E2E: Connection broken gracefully marks source as error, preserves existing nodes.
#[tokio::test]
async fn e2e_connection_revoked_degrades_gracefully() {
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    // Pre-populate a Drive source with existing ingested content.
    let src_id = store::ensure_google_drive_source(
        &pool,
        "folder_revoke",
        r#"{"folder_id":"folder_revoke"}"#,
    )
    .await
    .unwrap();

    // Ingest a content node (simulating a previous successful poll).
    let r = crate::automation::watchtower::ingest_content(
        &pool,
        src_id,
        "gdrive://fileX/existing.md",
        "# Existing Content\nThis was ingested before revocation.\n",
        false,
    )
    .await
    .unwrap();
    assert_eq!(r, store::UpsertResult::Inserted);

    // Simulate a ConnectionBroken error being handled by the Watchtower.
    // (In production, this happens in poll_remote_sources.)
    let _ = store::update_source_status(
        &pool,
        src_id,
        "error",
        Some("token revoked: Token has been revoked"),
    )
    .await;

    // Verify the source is now in error state.
    let ctx = store::get_source_context(&pool, src_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ctx.status, "error");
    assert!(ctx.error_message.as_deref().unwrap().contains("revoked"));

    // Verify existing content nodes are preserved.
    let nodes = store::get_nodes_for_source(&pool, src_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].relative_path, "gdrive://fileX/existing.md");
}

/// E2E: Restart recovery -- cursor is preserved across provider rebuild.
#[tokio::test]
async fn e2e_restart_recovery_cursor_survives() {
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    // Register a Drive source and set a sync cursor.
    let src_id = store::ensure_google_drive_source(
        &pool,
        "folder_restart",
        r#"{"folder_id":"folder_restart"}"#,
    )
    .await
    .unwrap();

    let cursor = "2026-02-28T12:00:00Z";
    store::update_sync_cursor(&pool, src_id, cursor)
        .await
        .unwrap();

    // Simulate restart: look up the source context again.
    let ctx = store::get_source_context(&pool, src_id)
        .await
        .unwrap()
        .unwrap();

    // The cursor should survive the "restart" (it's in the DB).
    assert_eq!(ctx.sync_cursor.as_deref(), Some(cursor));
    assert_eq!(ctx.status, "active");
}

// ---------------------------------------------------------------------------
// E2E: Fragment extraction and indexing
// ---------------------------------------------------------------------------

/// E2E: Ingest markdown with headings → verify fragments created with correct heading_path.
#[tokio::test]
async fn e2e_fragment_extraction_from_markdown() {
    use crate::automation::watchtower::{chunker, ingest_file};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let content = "\
---
title: Growth Strategy
tags: [growth, product]
---

Some intro text before any heading.

## Market Analysis

The market is shifting toward...

### Competitor Landscape

Our main competitors are...

## Product Roadmap

Next quarter we plan to...
";
    std::fs::write(dir.path().join("strategy.md"), content).unwrap();

    let src_id = store::insert_source_context(&pool, "local_fs", r#"{"path":"test"}"#)
        .await
        .unwrap();

    ingest_file(&pool, src_id, dir.path(), "strategy.md", false)
        .await
        .unwrap();

    let nodes = store::get_nodes_for_source(&pool, src_id, Some("pending"))
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];

    let ids = chunker::chunk_node(&pool, &node.account_id, node.id, &node.body_text)
        .await
        .unwrap();
    assert_eq!(ids.len(), 4);

    let chunks = store::get_chunks_for_node(&pool, &node.account_id, node.id)
        .await
        .unwrap();
    assert_eq!(chunks.len(), 4);

    // Verify heading paths.
    assert_eq!(chunks[0].heading_path, "");
    assert!(chunks[0].chunk_text.contains("intro text"));

    assert_eq!(chunks[1].heading_path, "## Market Analysis");
    assert!(chunks[1].chunk_text.contains("market is shifting"));

    assert_eq!(
        chunks[2].heading_path,
        "## Market Analysis/### Competitor Landscape"
    );
    assert!(chunks[2].chunk_text.contains("main competitors"));

    assert_eq!(chunks[3].heading_path, "## Product Roadmap");
    assert!(chunks[3].chunk_text.contains("Next quarter"));

    // Verify ordering.
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.chunk_index, i as i64);
    }

    // Verify node transitioned to chunked.
    let chunked = store::get_nodes_for_source(&pool, src_id, Some("chunked"))
        .await
        .unwrap();
    assert_eq!(chunked.len(), 1);
}

/// E2E: Re-ingest changed content → old chunks become stale, unchanged preserved.
#[tokio::test]
async fn e2e_fragment_update_on_content_change() {
    use crate::automation::watchtower::{chunker, ingest_content};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    let src_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    // V1: two sections.
    let v1 = "## Ideas\n\nBuild something great.\n\n## Plan\n\nShip it fast.\n";
    ingest_content(&pool, src_id, "note.md", v1, false)
        .await
        .unwrap();

    let nodes = store::get_nodes_for_source(&pool, src_id, Some("pending"))
        .await
        .unwrap();
    let node = &nodes[0];

    let ids_v1 = chunker::chunk_node(&pool, &node.account_id, node.id, &node.body_text)
        .await
        .unwrap();
    assert_eq!(ids_v1.len(), 2);

    // V2: keep Ideas, change Plan.
    let v2 = "## Ideas\n\nBuild something great.\n\n## Plan\n\nShip it carefully.\n";
    ingest_content(&pool, src_id, "note.md", v2, false)
        .await
        .unwrap();

    let nodes_v2 = store::get_nodes_for_source(&pool, src_id, Some("pending"))
        .await
        .unwrap();
    let node_v2 = &nodes_v2[0];

    let ids_v2 = chunker::chunk_node(&pool, &node_v2.account_id, node_v2.id, &node_v2.body_text)
        .await
        .unwrap();
    assert_eq!(ids_v2.len(), 2);

    // Ideas chunk unchanged → same ID.
    assert_eq!(ids_v1[0], ids_v2[0]);
    // Plan chunk changed → new ID.
    assert_ne!(ids_v1[1], ids_v2[1]);
}

/// E2E: Plain text file (no headings) → single root fragment.
#[tokio::test]
async fn e2e_plain_text_fallback_fragment() {
    use crate::automation::watchtower::{chunker, ingest_file};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    std::fs::write(
        dir.path().join("notes.txt"),
        "Just a plain text file.\nNo headings here.\n",
    )
    .unwrap();

    let src_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    ingest_file(&pool, src_id, dir.path(), "notes.txt", false)
        .await
        .unwrap();

    let nodes = store::get_nodes_for_source(&pool, src_id, Some("pending"))
        .await
        .unwrap();
    let node = &nodes[0];

    let ids = chunker::chunk_node(&pool, &node.account_id, node.id, &node.body_text)
        .await
        .unwrap();
    assert_eq!(ids.len(), 1);

    let chunks = store::get_chunks_for_node(&pool, &node.account_id, node.id)
        .await
        .unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].heading_path, "");
    assert!(chunks[0].chunk_text.contains("plain text file"));
}

/// E2E: Chunks from different sources are per-node and account-scoped.
#[tokio::test]
async fn e2e_mixed_source_fragment_isolation() {
    use crate::automation::watchtower::{chunker, ingest_content, ingest_file};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    // Local source.
    std::fs::write(
        dir.path().join("local.md"),
        "## Local Section\n\nLocal content.\n",
    )
    .unwrap();
    let local_id = store::insert_source_context(&pool, "local_fs", r#"{"path":"test"}"#)
        .await
        .unwrap();
    ingest_file(&pool, local_id, dir.path(), "local.md", false)
        .await
        .unwrap();

    // Drive source.
    let drive_id =
        store::ensure_google_drive_source(&pool, "drv_frag", r#"{"folder_id":"drv_frag"}"#)
            .await
            .unwrap();
    ingest_content(
        &pool,
        drive_id,
        "gdrive://fileA/remote.md",
        "## Remote Section\n\nRemote content.\n",
        false,
    )
    .await
    .unwrap();

    // Chunk both.
    let all_nodes = store::get_pending_content_nodes(&pool, 10).await.unwrap();
    assert_eq!(all_nodes.len(), 2);

    for node in &all_nodes {
        chunker::chunk_node(&pool, &node.account_id, node.id, &node.body_text)
            .await
            .unwrap();
    }

    // Verify each node has its own chunk, not mixed.
    for node in &all_nodes {
        let chunks = store::get_chunks_for_node(&pool, &node.account_id, node.id)
            .await
            .unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].node_id, node.id);
    }
}

// ---------------------------------------------------------------------------
// E2E: Provenance-driven loopback
// ---------------------------------------------------------------------------

/// E2E: Provenance-driven loopback writes metadata to source note and is idempotent.
#[tokio::test]
async fn e2e_provenance_driven_loopback_writes_to_source_note() {
    use crate::automation::watchtower::{ingest_file, loopback};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    // Step 1: Create source with a real path in config_json.
    let config = serde_json::json!({ "path": dir.path().to_str().unwrap() }).to_string();
    let src_id = store::insert_source_context(&pool, "local_fs", &config)
        .await
        .unwrap();

    // Step 2: Write and ingest a markdown file.
    std::fs::write(
        dir.path().join("ideas.md"),
        "---\ntitle: Big Ideas\n---\nSome great ideas here.\n",
    )
    .unwrap();
    ingest_file(&pool, src_id, dir.path(), "ideas.md", false)
        .await
        .unwrap();

    // Get the node_id.
    let nodes = store::get_nodes_for_source(&pool, src_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    let node_id = nodes[0].id;

    // Step 3: Execute loopback.
    let result = loopback::execute_loopback(
        &pool,
        node_id,
        "tweet_lb_001",
        "https://x.com/i/status/tweet_lb_001",
        "tweet",
    )
    .await;
    assert_eq!(result, loopback::LoopBackResult::Written);

    // Step 4: Verify the file has tuitbot metadata.
    let content = std::fs::read_to_string(dir.path().join("ideas.md")).unwrap();
    assert!(content.contains("tweet_lb_001"));
    assert!(content.contains("Big Ideas") || content.contains("title"));
    assert!(content.contains("Some great ideas here."));

    // Step 5: Idempotent — second call returns AlreadyPresent.
    let result2 = loopback::execute_loopback(
        &pool,
        node_id,
        "tweet_lb_001",
        "https://x.com/i/status/tweet_lb_001",
        "tweet",
    )
    .await;
    assert_eq!(result2, loopback::LoopBackResult::AlreadyPresent);

    // Step 6: Re-ingest detects hash change.
    let r = ingest_file(&pool, src_id, dir.path(), "ideas.md", false)
        .await
        .unwrap();
    assert_eq!(r, store::UpsertResult::Updated);
}

/// E2E: Loopback skips non-local (google_drive) sources gracefully.
#[tokio::test]
async fn e2e_loopback_skips_non_local_sources() {
    use crate::automation::watchtower::{ingest_content, loopback};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    // Create a Google Drive source.
    let src_id = store::ensure_google_drive_source(&pool, "drv_lb", r#"{"folder_id":"drv_lb"}"#)
        .await
        .unwrap();

    // Ingest content into it.
    ingest_content(
        &pool,
        src_id,
        "gdrive://fileY/doc.md",
        "Remote content.\n",
        false,
    )
    .await
    .unwrap();

    let nodes = store::get_nodes_for_source(&pool, src_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    let node_id = nodes[0].id;

    // Loopback should return SourceNotWritable.
    let result = loopback::execute_loopback(
        &pool,
        node_id,
        "tweet_skip",
        "https://x.com/i/status/tweet_skip",
        "tweet",
    )
    .await;
    assert!(matches!(
        result,
        loopback::LoopBackResult::SourceNotWritable(_)
    ));
}

/// E2E: Loopback writes to multiple notes from same posting event.
#[tokio::test]
async fn e2e_loopback_multiple_nodes_from_same_post() {
    use crate::automation::watchtower::{ingest_file, loopback};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let config = serde_json::json!({ "path": dir.path().to_str().unwrap() }).to_string();
    let src_id = store::insert_source_context(&pool, "local_fs", &config)
        .await
        .unwrap();

    // Create two notes.
    std::fs::write(dir.path().join("note_a.md"), "Content from note A.\n").unwrap();
    std::fs::write(dir.path().join("note_b.md"), "Content from note B.\n").unwrap();

    ingest_file(&pool, src_id, dir.path(), "note_a.md", false)
        .await
        .unwrap();
    ingest_file(&pool, src_id, dir.path(), "note_b.md", false)
        .await
        .unwrap();

    let nodes = store::get_nodes_for_source(&pool, src_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 2);

    // Execute loopback for both nodes with the same tweet_id.
    for node in &nodes {
        let result = loopback::execute_loopback(
            &pool,
            node.id,
            "tweet_multi",
            "https://x.com/i/status/tweet_multi",
            "tweet",
        )
        .await;
        assert_eq!(result, loopback::LoopBackResult::Written);
    }

    // Verify both files got metadata.
    let a = std::fs::read_to_string(dir.path().join("note_a.md")).unwrap();
    let b = std::fs::read_to_string(dir.path().join("note_b.md")).unwrap();
    assert!(a.contains("tweet_multi"));
    assert!(b.contains("tweet_multi"));
    assert!(a.contains("Content from note A."));
    assert!(b.contains("Content from note B."));
}

/// E2E: File with only front-matter and no body → no fragments created.
#[tokio::test]
async fn e2e_empty_body_no_fragments() {
    use crate::automation::watchtower::{chunker, ingest_content};
    use crate::storage::init_test_db;
    use crate::storage::watchtower as store;

    let pool = init_test_db().await.expect("init db");

    let src_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    // Front-matter only, empty body.
    let content = "---\ntitle: Empty Note\ntags: [empty]\n---\n";
    ingest_content(&pool, src_id, "empty.md", content, false)
        .await
        .unwrap();

    let nodes = store::get_nodes_for_source(&pool, src_id, Some("pending"))
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];

    let ids = chunker::chunk_node(&pool, &node.account_id, node.id, &node.body_text)
        .await
        .unwrap();
    // No body text → no fragments.
    assert!(ids.is_empty());
}
