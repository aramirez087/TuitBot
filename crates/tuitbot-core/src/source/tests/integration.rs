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
    let ctx = build_draft_context(&pool, &["strategy".into()], 5, 14.0)
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
    let ctx = build_draft_context(&pool, &["remote".into()], 5, 14.0)
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
    let ctx = build_draft_context(&pool, &[], 5, 14.0).await.unwrap();
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
