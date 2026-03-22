//! Fragment extraction and indexing for ingested content nodes.
//!
//! Splits markdown notes into heading-delimited fragments so retrieval can
//! cite specific sections instead of whole notes. Plain-text files (or notes
//! without headings) produce a single root fragment.

use std::sync::OnceLock;

use regex::Regex;
use sha2::{Digest, Sha256};

use crate::error::StorageError;
use crate::storage::watchtower as store;
use crate::storage::DbPool;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors specific to the chunking pipeline.
#[derive(Debug, thiserror::Error)]
pub enum ChunkerError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}

// ---------------------------------------------------------------------------
// Fragment type
// ---------------------------------------------------------------------------

/// A parsed fragment from a note body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    /// Slash-delimited heading hierarchy, e.g. `"## Section/### Sub"`.
    /// Empty string for text before any heading (root fragment).
    pub heading_path: String,
    /// Body text under this heading (trimmed of leading/trailing blank lines).
    pub text: String,
    /// 0-based position within the note.
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Heading regex
// ---------------------------------------------------------------------------

fn heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(#{1,6})\s+(.+)$").expect("heading regex"))
}

// ---------------------------------------------------------------------------
// Fragment extraction
// ---------------------------------------------------------------------------

/// Extract fragments from a markdown or plain-text body.
///
/// The caller is responsible for stripping front-matter before calling this.
/// If the body contains no headings, a single root fragment is returned.
/// Empty or whitespace-only fragments are skipped.
pub fn extract_fragments(body: &str) -> Vec<Fragment> {
    let mut fragments: Vec<Fragment> = Vec::new();
    // heading_stack: Vec<(level, heading_text_with_hashes)>
    let mut heading_stack: Vec<(usize, String)> = Vec::new();
    let mut current_text = String::new();
    let mut in_code_block = false;

    for line in body.lines() {
        // Track fenced code blocks to avoid treating `# ` inside them as headings.
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            current_text.push_str(line);
            current_text.push('\n');
            continue;
        }

        if in_code_block {
            current_text.push_str(line);
            current_text.push('\n');
            continue;
        }

        if let Some(caps) = heading_re().captures(line) {
            // Flush accumulated text as a fragment.
            flush_fragment(&heading_stack, &mut current_text, &mut fragments);

            let level = caps[1].len();
            let heading_text = caps[2].trim().to_string();
            let heading_label = format!("{} {heading_text}", &caps[1]);

            // Pop headings at the same level or deeper.
            while heading_stack.last().is_some_and(|(lvl, _)| *lvl >= level) {
                heading_stack.pop();
            }
            heading_stack.push((level, heading_label));
        } else {
            current_text.push_str(line);
            current_text.push('\n');
        }
    }

    // Flush final accumulated text.
    flush_fragment(&heading_stack, &mut current_text, &mut fragments);

    // Re-index after filtering.
    for (i, frag) in fragments.iter_mut().enumerate() {
        frag.index = i;
    }

    fragments
}

/// Flush accumulated text into a fragment if non-empty.
fn flush_fragment(
    heading_stack: &[(usize, String)],
    current_text: &mut String,
    fragments: &mut Vec<Fragment>,
) {
    let trimmed = current_text.trim();
    if !trimmed.is_empty() {
        let heading_path = heading_stack
            .iter()
            .map(|(_, h)| h.as_str())
            .collect::<Vec<_>>()
            .join("/");

        fragments.push(Fragment {
            heading_path,
            text: trimmed.to_string(),
            index: 0, // Will be re-indexed later.
        });
    }
    current_text.clear();
}

// ---------------------------------------------------------------------------
// Chunk a content node
// ---------------------------------------------------------------------------

/// Extract fragments from a content node and persist them as chunks.
///
/// 1. Extracts fragments from `node.body_text`
/// 2. Marks existing chunks stale (idempotent on first chunk)
/// 3. Upserts new chunks (deduplicates by hash)
/// 4. Marks the node as `chunked`
///
/// Returns the IDs of all active chunks for the node.
pub async fn chunk_node(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    body_text: &str,
) -> Result<Vec<i64>, ChunkerError> {
    let fragments = extract_fragments(body_text);

    // Mark existing chunks stale before upserting new ones.
    store::mark_chunks_stale(pool, account_id, node_id).await?;

    // Build NewChunk structs with SHA-256 hashes.
    let new_chunks: Vec<store::NewChunk> = fragments
        .iter()
        .map(|f| {
            let mut hasher = Sha256::new();
            hasher.update(f.text.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            store::NewChunk {
                heading_path: f.heading_path.clone(),
                chunk_text: f.text.clone(),
                chunk_hash: hash,
                chunk_index: f.index as i64,
            }
        })
        .collect();

    let ids = store::upsert_chunks_for_node(pool, account_id, node_id, &new_chunks).await?;

    // Extract links/tags and persist graph edges (fail-open).
    super::graph_ingest::extract_and_persist_graph(pool, account_id, node_id, body_text).await;

    // Transition node status: pending → chunked.
    store::mark_node_chunked(pool, account_id, node_id).await?;

    Ok(ids)
}

/// Process all pending content nodes for an account: extract and persist fragments.
///
/// Returns the total number of nodes chunked.
pub async fn chunk_pending_nodes(pool: &DbPool, account_id: &str, limit: u32) -> u32 {
    let nodes = match store::get_pending_content_nodes_for(pool, account_id, limit).await {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to get pending nodes for chunking");
            return 0;
        }
    };

    let mut chunked = 0u32;
    for node in &nodes {
        match chunk_node(pool, account_id, node.id, &node.body_text).await {
            Ok(_ids) => {
                chunked += 1;
                tracing::debug!(
                    node_id = node.id,
                    path = %node.relative_path,
                    "Chunked content node"
                );
            }
            Err(e) => {
                tracing::warn!(
                    node_id = node.id,
                    path = %node.relative_path,
                    error = %e,
                    "Failed to chunk node"
                );
            }
        }
    }

    chunked
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Simple markdown document ────────────────────────────────────

    #[test]
    fn chunk_simple_markdown() {
        let body = "First paragraph.\n\nSecond paragraph.\n";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 1);
        assert_eq!(frags[0].heading_path, "");
        assert!(frags[0].text.contains("First paragraph"));
        assert!(frags[0].text.contains("Second paragraph"));
    }

    // ── Splits on headings ──────────────────────────────────────────

    #[test]
    fn chunk_splits_on_headings() {
        let body = "\
## Introduction

Welcome to the guide.

## Getting Started

Install the CLI tool.

## Advanced Usage

Use the `--verbose` flag.
";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 3);
        assert_eq!(frags[0].heading_path, "## Introduction");
        assert!(frags[0].text.contains("Welcome"));
        assert_eq!(frags[1].heading_path, "## Getting Started");
        assert!(frags[1].text.contains("Install"));
        assert_eq!(frags[2].heading_path, "## Advanced Usage");
        assert!(frags[2].text.contains("--verbose"));
    }

    // ── Empty input ─────────────────────────────────────────────────

    #[test]
    fn chunk_empty_input_returns_empty() {
        assert!(extract_fragments("").is_empty());
    }

    #[test]
    fn chunk_whitespace_only_returns_empty() {
        assert!(extract_fragments("   \n\n\t\n  ").is_empty());
    }

    // ── Long text without headings stays as single fragment ─────────

    #[test]
    fn chunk_long_text_single_fragment() {
        let long = "word ".repeat(500);
        let frags = extract_fragments(&long);
        assert_eq!(frags.len(), 1);
        assert_eq!(frags[0].heading_path, "");
        assert_eq!(frags[0].index, 0);
    }

    // ── Heading hierarchy preserved ─────────────────────────────────

    #[test]
    fn chunk_heading_hierarchy_preserved() {
        let body = "\
# Top Level

Intro text.

## Section One

Content one.

### Subsection A

Deep content A.

### Subsection B

Deep content B.

## Section Two

Content two.
";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 5);
        assert_eq!(frags[0].heading_path, "# Top Level");
        assert_eq!(frags[1].heading_path, "# Top Level/## Section One");
        assert_eq!(
            frags[2].heading_path,
            "# Top Level/## Section One/### Subsection A"
        );
        assert_eq!(
            frags[3].heading_path,
            "# Top Level/## Section One/### Subsection B"
        );
        assert_eq!(frags[4].heading_path, "# Top Level/## Section Two");
    }

    // ── Root text before any heading ────────────────────────────────

    #[test]
    fn chunk_root_text_before_heading() {
        let body = "Preamble text.\n\n## Heading\n\nBody.\n";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 2);
        assert_eq!(frags[0].heading_path, "");
        assert!(frags[0].text.contains("Preamble"));
        assert_eq!(frags[1].heading_path, "## Heading");
    }

    // ── Code blocks do not create headings ──────────────────────────

    #[test]
    fn chunk_code_block_headings_ignored() {
        let body = "\
## Real Heading

Some text.

```markdown
# This is inside a code block
## Also inside
```

More text after code.
";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 1);
        assert_eq!(frags[0].heading_path, "## Real Heading");
        assert!(frags[0].text.contains("# This is inside a code block"));
        assert!(frags[0].text.contains("More text after code"));
    }

    // ── Consecutive headings with no body skip empty fragments ──────

    #[test]
    fn chunk_consecutive_headings_skip_empty() {
        let body = "## A\n## B\n## C\nFinal content.\n";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 1);
        assert_eq!(frags[0].heading_path, "## C");
        assert!(frags[0].text.contains("Final content"));
    }

    // ── Index values are sequential ─────────────────────────────────

    #[test]
    fn chunk_indices_sequential() {
        let body = "Intro.\n\n## A\n\nA text.\n\n## B\n\nB text.\n\n## C\n\nC text.\n";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 4);
        for (i, frag) in frags.iter().enumerate() {
            assert_eq!(frag.index, i, "fragment {i} should have index {i}");
        }
    }

    // ── H6 heading level works ──────────────────────────────────────

    #[test]
    fn chunk_h6_heading() {
        let body = "###### Deep Heading\n\nDeep content.\n";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 1);
        assert_eq!(frags[0].heading_path, "###### Deep Heading");
    }

    // ── Heading level reset pops stack correctly ────────────────────

    #[test]
    fn chunk_heading_level_reset() {
        let body = "\
### Level 3

Content at 3.

## Level 2

Content at 2.
";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 2);
        assert_eq!(frags[0].heading_path, "### Level 3");
        assert_eq!(frags[1].heading_path, "## Level 2");
    }

    // ── Fragment text is trimmed ────────────────────────────────────

    #[test]
    fn chunk_fragment_text_trimmed() {
        let body = "## Heading\n\n\n   Some text.   \n\n\n";
        let frags = extract_fragments(body);
        assert_eq!(frags.len(), 1);
        assert_eq!(frags[0].text, "Some text.");
    }
}
