//! Integration tests for note graph: edges, tags, account isolation, and
//! re-ingest idempotency.

use super::*;
use crate::automation::watchtower::chunker::chunk_node;
use crate::storage::init_test_db;

const TEST_ACCOUNT: &str = "00000000-0000-0000-0000-000000000000";
const OTHER_ACCOUNT: &str = "11111111-1111-1111-1111-111111111111";

/// Helper: create a source + node with given path, title, body, and tags.
/// Returns (source_id, node_id).
async fn create_node(
    pool: &crate::storage::DbPool,
    account_id: &str,
    source_id: i64,
    path: &str,
    title: &str,
    body: &str,
    tags: Option<&str>,
) -> i64 {
    upsert_content_node_for(
        pool,
        account_id,
        source_id,
        path,
        &format!("hash-{}", path),
        Some(title),
        body,
        None,
        tags,
    )
    .await
    .expect("upsert node");

    let node = find_node_by_path_for(pool, account_id, path)
        .await
        .expect("find node")
        .expect("node exists");
    node.id
}

// ============================================================================
// Edge tests
// ============================================================================

#[tokio::test]
async fn insert_and_query_edges() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, TEST_ACCOUNT, src, "b.md", "B", "body", None).await;

    let edge = NewEdge {
        source_node_id: n1,
        target_node_id: n2,
        edge_type: "wikilink".to_string(),
        edge_label: Some("B".to_string()),
        source_chunk_id: None,
    };
    insert_edge(&pool, TEST_ACCOUNT, &edge)
        .await
        .expect("insert");

    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].source_node_id, n1);
    assert_eq!(edges[0].target_node_id, n2);
    assert_eq!(edges[0].edge_type, "wikilink");
}

#[tokio::test]
async fn delete_edges_for_source_idempotent() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, TEST_ACCOUNT, src, "b.md", "B", "body", None).await;

    let edge = NewEdge {
        source_node_id: n1,
        target_node_id: n2,
        edge_type: "wikilink".to_string(),
        edge_label: None,
        source_chunk_id: None,
    };
    insert_edge(&pool, TEST_ACCOUNT, &edge)
        .await
        .expect("insert");

    let deleted = delete_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("delete");
    assert_eq!(deleted, 1);

    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert!(edges.is_empty());

    // Second delete is a no-op.
    let deleted2 = delete_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("delete again");
    assert_eq!(deleted2, 0);
}

#[tokio::test]
async fn edges_account_isolated() {
    let pool = init_test_db().await.expect("init db");
    let src_a = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source a");
    let src_b = insert_source_context_for(&pool, OTHER_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source b");

    let n1 = create_node(&pool, TEST_ACCOUNT, src_a, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, TEST_ACCOUNT, src_a, "b.md", "B", "body", None).await;
    let n3 = create_node(&pool, OTHER_ACCOUNT, src_b, "a.md", "A", "body", None).await;
    let n4 = create_node(&pool, OTHER_ACCOUNT, src_b, "b.md", "B", "body", None).await;

    insert_edge(
        &pool,
        TEST_ACCOUNT,
        &NewEdge {
            source_node_id: n1,
            target_node_id: n2,
            edge_type: "wikilink".to_string(),
            edge_label: None,
            source_chunk_id: None,
        },
    )
    .await
    .expect("insert test");

    insert_edge(
        &pool,
        OTHER_ACCOUNT,
        &NewEdge {
            source_node_id: n3,
            target_node_id: n4,
            edge_type: "wikilink".to_string(),
            edge_label: None,
            source_chunk_id: None,
        },
    )
    .await
    .expect("insert other");

    let test_edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query test");
    assert_eq!(test_edges.len(), 1);

    let other_edges = get_edges_for_source(&pool, OTHER_ACCOUNT, n3)
        .await
        .expect("query other");
    assert_eq!(other_edges.len(), 1);

    // Cross-account query returns empty.
    let cross = get_edges_for_source(&pool, OTHER_ACCOUNT, n1)
        .await
        .expect("query cross");
    assert!(cross.is_empty());
}

#[tokio::test]
async fn duplicate_edge_ignored() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, TEST_ACCOUNT, src, "b.md", "B", "body", None).await;

    let edge = NewEdge {
        source_node_id: n1,
        target_node_id: n2,
        edge_type: "wikilink".to_string(),
        edge_label: Some("B".to_string()),
        source_chunk_id: None,
    };
    insert_edge(&pool, TEST_ACCOUNT, &edge)
        .await
        .expect("first");
    insert_edge(&pool, TEST_ACCOUNT, &edge)
        .await
        .expect("second should not error");

    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert_eq!(edges.len(), 1, "duplicate edge should be ignored");
}

#[tokio::test]
async fn cascade_deletes_edges() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, TEST_ACCOUNT, src, "b.md", "B", "body", None).await;

    insert_edge(
        &pool,
        TEST_ACCOUNT,
        &NewEdge {
            source_node_id: n1,
            target_node_id: n2,
            edge_type: "wikilink".to_string(),
            edge_label: None,
            source_chunk_id: None,
        },
    )
    .await
    .expect("insert");

    // Delete the source node — edges should cascade.
    sqlx::query("DELETE FROM content_nodes WHERE id = ?")
        .bind(n1)
        .execute(&pool)
        .await
        .expect("delete node");

    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert!(edges.is_empty(), "edges should cascade-delete with node");
}

// ============================================================================
// Tag tests
// ============================================================================

#[tokio::test]
async fn insert_and_query_tags() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;

    let tags = vec![
        NormalizedTag {
            tag_text: "rust".to_string(),
            source: TagSource::Frontmatter,
        },
        NormalizedTag {
            tag_text: "systems".to_string(),
            source: TagSource::Inline,
        },
    ];
    insert_tags(&pool, TEST_ACCOUNT, n1, &tags)
        .await
        .expect("insert");

    let stored = get_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert_eq!(stored.len(), 2);
    let tag_texts: Vec<&str> = stored.iter().map(|t| t.tag_text.as_str()).collect();
    assert!(tag_texts.contains(&"rust"));
    assert!(tag_texts.contains(&"systems"));
}

#[tokio::test]
async fn delete_tags_for_node_idempotent() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;

    let tags = vec![NormalizedTag {
        tag_text: "rust".to_string(),
        source: TagSource::Frontmatter,
    }];
    insert_tags(&pool, TEST_ACCOUNT, n1, &tags)
        .await
        .expect("insert");

    let deleted = delete_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("delete");
    assert_eq!(deleted, 1);

    let stored = get_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert!(stored.is_empty());

    // Second delete is a no-op.
    let deleted2 = delete_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("delete again");
    assert_eq!(deleted2, 0);
}

#[tokio::test]
async fn tags_account_isolated() {
    let pool = init_test_db().await.expect("init db");
    let src_a = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source a");
    let src_b = insert_source_context_for(&pool, OTHER_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source b");

    let n1 = create_node(&pool, TEST_ACCOUNT, src_a, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, OTHER_ACCOUNT, src_b, "a.md", "A", "body", None).await;

    insert_tags(
        &pool,
        TEST_ACCOUNT,
        n1,
        &[NormalizedTag {
            tag_text: "rust".to_string(),
            source: TagSource::Frontmatter,
        }],
    )
    .await
    .expect("insert test");

    insert_tags(
        &pool,
        OTHER_ACCOUNT,
        n2,
        &[NormalizedTag {
            tag_text: "go".to_string(),
            source: TagSource::Frontmatter,
        }],
    )
    .await
    .expect("insert other");

    let test_tags = get_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query test");
    assert_eq!(test_tags.len(), 1);
    assert_eq!(test_tags[0].tag_text, "rust");

    let other_tags = get_tags_for_node(&pool, OTHER_ACCOUNT, n2)
        .await
        .expect("query other");
    assert_eq!(other_tags.len(), 1);
    assert_eq!(other_tags[0].tag_text, "go");

    // Cross-account returns empty.
    let cross = get_tags_for_node(&pool, OTHER_ACCOUNT, n1)
        .await
        .expect("cross");
    assert!(cross.is_empty());
}

#[tokio::test]
async fn find_shared_tag_neighbors_basic() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, TEST_ACCOUNT, src, "b.md", "B", "body", None).await;

    let tag = NormalizedTag {
        tag_text: "rust".to_string(),
        source: TagSource::Frontmatter,
    };
    insert_tags(&pool, TEST_ACCOUNT, n1, &[tag.clone()])
        .await
        .expect("tags n1");
    insert_tags(&pool, TEST_ACCOUNT, n2, &[tag])
        .await
        .expect("tags n2");

    let neighbors = find_shared_tag_neighbors(&pool, TEST_ACCOUNT, n1, 10)
        .await
        .expect("neighbors");
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].0, n2);
    assert_eq!(neighbors[0].1, "rust");
}

#[tokio::test]
async fn find_shared_tag_neighbors_capped() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");

    let origin = create_node(
        &pool,
        TEST_ACCOUNT,
        src,
        "origin.md",
        "Origin",
        "body",
        None,
    )
    .await;
    insert_tags(
        &pool,
        TEST_ACCOUNT,
        origin,
        &[NormalizedTag {
            tag_text: "common".to_string(),
            source: TagSource::Frontmatter,
        }],
    )
    .await
    .expect("tag origin");

    // Create 15 neighbors sharing the same tag.
    for i in 0..15 {
        let path = format!("n{i}.md");
        let title = format!("N{i}");
        let nid = create_node(&pool, TEST_ACCOUNT, src, &path, &title, "body", None).await;
        insert_tags(
            &pool,
            TEST_ACCOUNT,
            nid,
            &[NormalizedTag {
                tag_text: "common".to_string(),
                source: TagSource::Frontmatter,
            }],
        )
        .await
        .expect("tag neighbor");
    }

    let neighbors = find_shared_tag_neighbors(&pool, TEST_ACCOUNT, origin, 10)
        .await
        .expect("neighbors");
    assert_eq!(neighbors.len(), 10, "should be capped at max_results=10");
}

// ============================================================================
// Re-ingest idempotency tests
// ============================================================================

#[tokio::test]
async fn rechunk_replaces_edges() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;
    let n2 = create_node(&pool, TEST_ACCOUNT, src, "b.md", "B", "body", None).await;
    let n3 = create_node(&pool, TEST_ACCOUNT, src, "c.md", "C", "body", None).await;

    // Insert edges: n1 -> n2
    insert_edges(
        &pool,
        TEST_ACCOUNT,
        &[NewEdge {
            source_node_id: n1,
            target_node_id: n2,
            edge_type: "wikilink".to_string(),
            edge_label: Some("B".to_string()),
            source_chunk_id: None,
        }],
    )
    .await
    .expect("initial edges");

    // Simulate re-chunk: delete and insert new edges n1 -> n3
    delete_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("delete");
    insert_edges(
        &pool,
        TEST_ACCOUNT,
        &[NewEdge {
            source_node_id: n1,
            target_node_id: n3,
            edge_type: "wikilink".to_string(),
            edge_label: Some("C".to_string()),
            source_chunk_id: None,
        }],
    )
    .await
    .expect("new edges");

    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].target_node_id, n3, "old edge replaced by new one");
}

#[tokio::test]
async fn rechunk_replaces_tags() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "a.md", "A", "body", None).await;

    insert_tags(
        &pool,
        TEST_ACCOUNT,
        n1,
        &[NormalizedTag {
            tag_text: "old-tag".to_string(),
            source: TagSource::Frontmatter,
        }],
    )
    .await
    .expect("old tags");

    // Simulate re-chunk: delete and insert new tags.
    delete_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("delete");
    insert_tags(
        &pool,
        TEST_ACCOUNT,
        n1,
        &[NormalizedTag {
            tag_text: "new-tag".to_string(),
            source: TagSource::Inline,
        }],
    )
    .await
    .expect("new tags");

    let tags = get_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("query");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].tag_text, "new-tag");
}

// ============================================================================
// End-to-end integration (chunk_node creates edges and tags)
// ============================================================================

#[tokio::test]
async fn chunk_node_creates_edges_and_tags() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");

    // Create two nodes: "Note A" links to "Note B" via wikilink.
    let n_b = create_node(
        &pool,
        TEST_ACCOUNT,
        src,
        "note-b.md",
        "Note B",
        "Content of B.",
        None,
    )
    .await;

    let body_a = "This note links to [[Note B]] and has #architecture tag.";
    let n_a = create_node(
        &pool,
        TEST_ACCOUNT,
        src,
        "note-a.md",
        "Note A",
        body_a,
        Some("rust,design"),
    )
    .await;

    // Chunk node A — should create edges and tags.
    chunk_node(&pool, TEST_ACCOUNT, n_a, body_a)
        .await
        .expect("chunk A");

    // Verify wikilink edge A -> B.
    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n_a)
        .await
        .expect("edges");
    let wikilinks: Vec<_> = edges.iter().filter(|e| e.edge_type == "wikilink").collect();
    assert_eq!(wikilinks.len(), 1);
    assert_eq!(wikilinks[0].target_node_id, n_b);

    // Verify backlink edge B -> A was created (source=B, target=A, type=backlink).
    // Query edges pointing TO n_a (backlinks from other nodes).
    let incoming = get_edges_for_target(&pool, TEST_ACCOUNT, n_a)
        .await
        .expect("incoming edges");
    let backlinks: Vec<_> = incoming
        .iter()
        .filter(|e| e.edge_type == "backlink")
        .collect();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].source_node_id, n_b);
    assert_eq!(backlinks[0].target_node_id, n_a);

    // Verify tags: frontmatter ("rust", "design") + inline ("architecture").
    let tags = get_tags_for_node(&pool, TEST_ACCOUNT, n_a)
        .await
        .expect("tags");
    let tag_texts: Vec<&str> = tags.iter().map(|t| t.tag_text.as_str()).collect();
    assert!(tag_texts.contains(&"rust"), "frontmatter tag 'rust'");
    assert!(tag_texts.contains(&"design"), "frontmatter tag 'design'");
    assert!(
        tag_texts.contains(&"architecture"),
        "inline tag 'architecture'"
    );
}

#[tokio::test]
async fn chunk_node_no_links_no_edges() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");

    let body = "Just some plain text without any links or tags.";
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "plain.md", "Plain", body, None).await;

    chunk_node(&pool, TEST_ACCOUNT, n1, body)
        .await
        .expect("chunk");

    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("edges");
    assert!(edges.is_empty(), "plain text should produce zero edges");

    let tags = get_tags_for_node(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("tags");
    assert!(tags.is_empty(), "plain text with no tags column");
}

#[tokio::test]
async fn chunk_node_unresolvable_link_no_edge() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("source");

    let body = "Links to [[Nonexistent Note]] which does not exist.";
    let n1 = create_node(&pool, TEST_ACCOUNT, src, "orphan.md", "Orphan", body, None).await;

    chunk_node(&pool, TEST_ACCOUNT, n1, body)
        .await
        .expect("chunk");

    let edges = get_edges_for_source(&pool, TEST_ACCOUNT, n1)
        .await
        .expect("edges");
    assert!(
        edges.is_empty(),
        "unresolvable link should produce no edges (fail-open)"
    );
}

// ============================================================================
// Migration verification
// ============================================================================

#[tokio::test]
async fn migration_creates_graph_tables() {
    let pool = init_test_db().await.expect("init db");

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' \
         AND name IN ('note_edges', 'note_tags') ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .expect("query tables");

    let names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
    assert!(
        names.contains(&"note_edges"),
        "note_edges table should exist"
    );
    assert!(names.contains(&"note_tags"), "note_tags table should exist");
}

#[tokio::test]
async fn migration_adds_provenance_columns() {
    let pool = init_test_db().await.expect("init db");

    let cols: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM pragma_table_info('vault_provenance_links')")
            .fetch_all(&pool)
            .await
            .expect("pragma");
    let col_names: Vec<&str> = cols.iter().map(|c| c.0.as_str()).collect();
    assert!(
        col_names.contains(&"edge_type"),
        "vault_provenance_links should have edge_type"
    );
    assert!(
        col_names.contains(&"edge_label"),
        "vault_provenance_links should have edge_label"
    );
}
