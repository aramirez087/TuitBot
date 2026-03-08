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
