//! Graph ingestion: extract links/tags from note bodies and persist edges.
//!
//! Called from `chunk_node()` after fragment extraction. Errors are logged
//! but do not fail the chunking pipeline (fail-open).

use super::link_extractor::{extract_links, normalize_tag, LinkType};
use crate::storage::watchtower as store;
use crate::storage::DbPool;

/// Extract links and tags from a note body, persist edges and normalized tags.
///
/// Fail-open: errors are logged at warn/debug level but do not propagate.
/// This keeps the existing chunking pipeline backward-compatible for notes
/// without links or with unresolvable targets.
pub async fn extract_and_persist_graph(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    body_text: &str,
) {
    let raw_links = extract_links(body_text);

    // 1. Delete stale edges and tags for idempotency.
    if let Err(e) = store::delete_edges_for_source(pool, account_id, node_id).await {
        tracing::warn!(node_id, error = %e, "Failed to delete stale edges");
        return;
    }
    if let Err(e) = store::delete_tags_for_node(pool, account_id, node_id).await {
        tracing::warn!(node_id, error = %e, "Failed to delete stale tags");
        return;
    }

    // 2. Collect inline tags from extracted links.
    let mut tags: Vec<store::NormalizedTag> = Vec::new();
    for link in &raw_links {
        if link.link_type == LinkType::InlineTag {
            let normalized = normalize_tag(&link.target_raw);
            if !normalized.is_empty() {
                tags.push(store::NormalizedTag {
                    tag_text: normalized,
                    source: store::TagSource::Inline,
                });
            }
        }
    }

    // 3. Parse frontmatter tags from the node's existing `tags` column.
    if let Ok(Some(node)) = store::get_content_node_for(pool, account_id, node_id).await {
        if let Some(ref tag_str) = node.tags {
            for t in tag_str.split(',') {
                let normalized = normalize_tag(t);
                if !normalized.is_empty() {
                    tags.push(store::NormalizedTag {
                        tag_text: normalized,
                        source: store::TagSource::Frontmatter,
                    });
                }
            }
        }
    }

    // 4. Persist tags.
    if let Err(e) = store::insert_tags(pool, account_id, node_id, &tags).await {
        tracing::warn!(node_id, error = %e, "Failed to insert tags");
    }

    // 5. Resolve links to edges.
    let mut edges: Vec<store::NewEdge> = Vec::new();
    for link in &raw_links {
        match link.link_type {
            LinkType::Wikilink => {
                if let Some(target) = resolve_wikilink(pool, account_id, &link.target_raw).await {
                    if target == node_id {
                        continue; // skip self-links
                    }
                    let label = link
                        .display_text
                        .clone()
                        .unwrap_or_else(|| link.target_raw.clone());
                    edges.push(store::NewEdge {
                        source_node_id: node_id,
                        target_node_id: target,
                        edge_type: "wikilink".to_string(),
                        edge_label: Some(label.clone()),
                        source_chunk_id: None,
                    });
                    edges.push(store::NewEdge {
                        source_node_id: target,
                        target_node_id: node_id,
                        edge_type: "backlink".to_string(),
                        edge_label: Some(label),
                        source_chunk_id: None,
                    });
                } else {
                    tracing::debug!(
                        node_id,
                        target = %link.target_raw,
                        "Unresolvable wikilink target, skipping"
                    );
                }
            }
            LinkType::MarkdownLink => {
                if let Some(target) =
                    resolve_md_link(pool, account_id, node_id, &link.target_raw).await
                {
                    if target == node_id {
                        continue;
                    }
                    let label = link.display_text.clone();
                    edges.push(store::NewEdge {
                        source_node_id: node_id,
                        target_node_id: target,
                        edge_type: "markdown_link".to_string(),
                        edge_label: label.clone(),
                        source_chunk_id: None,
                    });
                    edges.push(store::NewEdge {
                        source_node_id: target,
                        target_node_id: node_id,
                        edge_type: "backlink".to_string(),
                        edge_label: label,
                        source_chunk_id: None,
                    });
                } else {
                    tracing::debug!(
                        node_id,
                        target = %link.target_raw,
                        "Unresolvable markdown link target, skipping"
                    );
                }
            }
            LinkType::InlineTag => {
                // Tags are handled separately — shared-tag edges below.
            }
        }
    }

    // 6. Create shared-tag edges.
    match store::find_shared_tag_neighbors(pool, account_id, node_id, 10).await {
        Ok(neighbors) => {
            for (target_id, tag_text) in neighbors {
                edges.push(store::NewEdge {
                    source_node_id: node_id,
                    target_node_id: target_id,
                    edge_type: "shared_tag".to_string(),
                    edge_label: Some(tag_text),
                    source_chunk_id: None,
                });
            }
        }
        Err(e) => {
            tracing::warn!(node_id, error = %e, "Failed to find shared-tag neighbors");
        }
    }

    // 7. Persist edges.
    if let Err(e) = store::insert_edges(pool, account_id, &edges).await {
        tracing::warn!(node_id, error = %e, "Failed to insert edges");
    }
}

// ---------------------------------------------------------------------------
// Link resolution helpers
// ---------------------------------------------------------------------------

/// Resolve a wikilink target to a content_nodes ID.
///
/// Strategy: case-insensitive title match, then path ending match.
async fn resolve_wikilink(pool: &DbPool, account_id: &str, target: &str) -> Option<i64> {
    // 1. Case-insensitive exact title match.
    let by_title: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM content_nodes \
         WHERE account_id = ? AND LOWER(title) = LOWER(?) \
         ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(target)
    .fetch_optional(pool)
    .await
    .ok()?;

    if let Some((id,)) = by_title {
        return Some(id);
    }

    // 2. Path ending match: relative_path ends with "target.md" or "/target.md".
    let pattern_suffix = format!("{}.md", target);
    let pattern_dir = format!("/{}.md", target);
    let by_path: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM content_nodes \
         WHERE account_id = ? \
           AND (relative_path = ? OR relative_path LIKE '%' || ?) \
         ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(&pattern_suffix)
    .bind(&pattern_dir)
    .fetch_optional(pool)
    .await
    .ok()?;

    by_path.map(|(id,)| id)
}

/// Resolve a markdown link path relative to the source note's directory.
async fn resolve_md_link(
    pool: &DbPool,
    account_id: &str,
    source_node_id: i64,
    raw_path: &str,
) -> Option<i64> {
    // Get source node's relative_path to determine its directory.
    let source = store::get_content_node_for(pool, account_id, source_node_id)
        .await
        .ok()??;

    let source_dir = std::path::Path::new(&source.relative_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new(""));

    // Normalize: strip leading './', build relative to source dir.
    let cleaned = raw_path.strip_prefix("./").unwrap_or(raw_path);
    let joined = source_dir.join(cleaned);

    // Normalize path components (resolve `..`).
    let normalized = normalize_path_components(&joined);

    // Convert back to slash-delimited string for DB matching.
    let slash_path = normalized
        .iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");

    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM content_nodes \
         WHERE account_id = ? AND relative_path = ? \
         ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(&slash_path)
    .fetch_optional(pool)
    .await
    .ok()?;

    row.map(|(id,)| id)
}

/// Normalize path components, resolving `..` without touching the filesystem.
fn normalize_path_components(path: &std::path::Path) -> Vec<std::ffi::OsString> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(c) => {
                components.push(c.to_os_string());
            }
            _ => {}
        }
    }
    components
}
