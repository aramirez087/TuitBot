//! Vault fragment retrieval and citation engine.
//!
//! Retrieves account-scoped content chunks from the vault, builds structured
//! citation records, and formats fragment text for LLM prompt injection.

use std::collections::HashSet;

use crate::error::StorageError;
use crate::storage::provenance::ProvenanceRef;
use crate::storage::watchtower::{self, ChunkWithNodeContext};
use crate::storage::DbPool;

/// Maximum character budget for the vault fragment prompt section.
pub const MAX_FRAGMENT_CHARS: usize = 1000;

/// Maximum number of fragments to include in context.
pub const MAX_FRAGMENTS: u32 = 5;

/// Maximum snippet length in citation records (characters).
const CITATION_SNIPPET_LEN: usize = 120;

// ============================================================================
// Structs
// ============================================================================

/// A structured citation linking a prompt fragment back to its vault source.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultCitation {
    /// ID of the content chunk.
    pub chunk_id: i64,
    /// ID of the parent content node.
    pub node_id: i64,
    /// Heading hierarchy path (e.g., "# Title > ## Section").
    pub heading_path: String,
    /// Relative file path of the source note.
    pub source_path: String,
    /// Title of the source note (if available).
    pub source_title: Option<String>,
    /// Short excerpt from the chunk text.
    pub snippet: String,
    /// Retrieval boost score.
    pub retrieval_boost: f64,
}

/// Intermediate result pairing chunk text with citation metadata.
#[derive(Debug, Clone)]
pub struct FragmentContext {
    /// The full chunk text for prompt inclusion.
    pub chunk_text: String,
    /// Citation metadata for this fragment.
    pub citation: VaultCitation,
}

// ============================================================================
// Retrieval
// ============================================================================

/// Retrieve vault fragments matching keywords, with optional selected-note bias.
///
/// When `selected_node_ids` is provided, chunks from those notes are retrieved
/// first, then remaining slots are filled with keyword-matched results (deduplicated).
pub async fn retrieve_vault_fragments(
    pool: &DbPool,
    account_id: &str,
    keywords: &[String],
    selected_node_ids: Option<&[i64]>,
    max_results: u32,
) -> Result<Vec<FragmentContext>, StorageError> {
    let mut results: Vec<FragmentContext> = Vec::new();
    let mut seen_ids: HashSet<i64> = HashSet::new();

    // Step 1: If selected nodes provided, fetch their chunks first.
    if let Some(node_ids) = selected_node_ids {
        if !node_ids.is_empty() {
            let biased = watchtower::get_chunks_for_nodes_with_context(
                pool,
                account_id,
                node_ids,
                max_results,
            )
            .await?;

            for cwc in biased {
                if seen_ids.insert(cwc.chunk.id) {
                    results.push(fragment_from_chunk_with_context(cwc));
                }
                if results.len() >= max_results as usize {
                    break;
                }
            }
        }
    }

    // Step 2: Fill remaining slots with keyword search results.
    if results.len() < max_results as usize && !keywords.is_empty() {
        let remaining = max_results - results.len() as u32;
        let kw_refs: Vec<&str> = keywords.iter().map(|s| s.as_str()).collect();
        let keyword_results =
            watchtower::search_chunks_with_context(pool, account_id, &kw_refs, remaining + 5)
                .await?;

        for cwc in keyword_results {
            if seen_ids.insert(cwc.chunk.id) {
                results.push(fragment_from_chunk_with_context(cwc));
            }
            if results.len() >= max_results as usize {
                break;
            }
        }
    }

    Ok(results)
}

// ============================================================================
// Formatting
// ============================================================================

/// Format fragment text as a prompt section with inline citations.
///
/// Output is capped at `MAX_FRAGMENT_CHARS`.
pub fn format_fragments_prompt(fragments: &[FragmentContext]) -> String {
    if fragments.is_empty() {
        return String::new();
    }

    let mut block = String::from("\nRelevant knowledge from your notes:\n");

    for (i, f) in fragments.iter().enumerate() {
        let title = f
            .citation
            .source_title
            .as_deref()
            .unwrap_or(&f.citation.source_path);
        let heading = if f.citation.heading_path.is_empty() {
            String::new()
        } else {
            format!("[{}] ", f.citation.heading_path)
        };
        let preview = truncate_text(&f.chunk_text, 200);
        let entry = format!("{}. {}(from: {}): \"{}\"\n", i + 1, heading, title, preview);

        if block.len() + entry.len() > MAX_FRAGMENT_CHARS {
            break;
        }
        block.push_str(&entry);
    }

    block.push_str("Reference these insights to ground your response in your own expertise.\n");

    if block.len() > MAX_FRAGMENT_CHARS {
        block.truncate(MAX_FRAGMENT_CHARS);
    }
    block
}

// ============================================================================
// Citation builders
// ============================================================================

/// Extract `VaultCitation` records from fragment contexts.
pub fn build_citations(fragments: &[FragmentContext]) -> Vec<VaultCitation> {
    fragments.iter().map(|f| f.citation.clone()).collect()
}

// ============================================================================
// Provenance converters
// ============================================================================

/// Convert `VaultCitation` records to `ProvenanceRef` for persistence.
pub fn citations_to_provenance_refs(citations: &[VaultCitation]) -> Vec<ProvenanceRef> {
    citations
        .iter()
        .map(|c| ProvenanceRef {
            node_id: Some(c.node_id),
            chunk_id: Some(c.chunk_id),
            seed_id: None,
            source_path: Some(c.source_path.clone()),
            heading_path: Some(c.heading_path.clone()),
            snippet: Some(c.snippet.clone()),
        })
        .collect()
}

/// Serialize citations as a JSON array for the legacy `source_chunks_json` column.
pub fn citations_to_chunks_json(citations: &[VaultCitation]) -> String {
    let entries: Vec<serde_json::Value> = citations
        .iter()
        .map(|c| {
            serde_json::json!({
                "chunk_id": c.chunk_id,
                "node_id": c.node_id,
                "source_path": c.source_path,
                "heading_path": c.heading_path,
            })
        })
        .collect();
    serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
}

// ============================================================================
// Helpers
// ============================================================================

fn fragment_from_chunk_with_context(cwc: ChunkWithNodeContext) -> FragmentContext {
    let snippet = truncate_text(&cwc.chunk.chunk_text, CITATION_SNIPPET_LEN);
    FragmentContext {
        chunk_text: cwc.chunk.chunk_text.clone(),
        citation: VaultCitation {
            chunk_id: cwc.chunk.id,
            node_id: cwc.chunk.node_id,
            heading_path: cwc.chunk.heading_path.clone(),
            source_path: cwc.relative_path,
            source_title: cwc.source_title,
            snippet,
            retrieval_boost: cwc.chunk.retrieval_boost,
        },
    }
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let mut end = max_len.saturating_sub(3);
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}
