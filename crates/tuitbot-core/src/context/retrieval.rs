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
    /// Graph edge type (e.g., "wikilink", "backlink", "shared_tag"). None for non-graph citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<String>,
    /// Graph edge label for provenance tracking. None for non-graph citations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_label: Option<String>,
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
            edge_type: c.edge_type.clone(),
            edge_label: c.edge_label.clone(),
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
// Selection identity resolution
// ============================================================================

/// Resolve a Ghostwriter selection payload to the best available indexed block identity.
///
/// Returns `(Option<node_id>, Option<chunk_id>)`. Both are `None` if the note
/// isn't indexed yet. Resolution is best-effort — the `selected_text` is always
/// the authoritative payload.
pub async fn resolve_selection_identity(
    pool: &DbPool,
    account_id: &str,
    file_path: &str,
    heading_context: Option<&str>,
) -> Result<(Option<i64>, Option<i64>), StorageError> {
    let node = watchtower::find_node_by_path_for(pool, account_id, file_path).await?;

    let node = match node {
        Some(n) => n,
        None => return Ok((None, None)),
    };

    let chunk =
        watchtower::find_best_chunk_by_heading_for(pool, account_id, node.id, heading_context)
            .await?;

    Ok((Some(node.id), chunk.map(|c| c.id)))
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
            edge_type: None,
            edge_label: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fragment(chunk_id: i64, text: &str, path: &str) -> FragmentContext {
        FragmentContext {
            chunk_text: text.to_string(),
            citation: VaultCitation {
                chunk_id,
                node_id: chunk_id * 10,
                heading_path: String::new(),
                source_path: path.to_string(),
                source_title: None,
                snippet: text.chars().take(50).collect(),
                retrieval_boost: 1.0,
                edge_type: None,
                edge_label: None,
            },
        }
    }

    fn sample_citation() -> VaultCitation {
        VaultCitation {
            chunk_id: 1,
            node_id: 10,
            heading_path: "# Guide > ## Setup".to_string(),
            source_path: "notes/guide.md".to_string(),
            source_title: Some("Installation Guide".to_string()),
            snippet: "Install with cargo install".to_string(),
            retrieval_boost: 1.0,
            edge_type: None,
            edge_label: None,
        }
    }

    fn sample_fragment() -> FragmentContext {
        FragmentContext {
            chunk_text: "Install the CLI with cargo install tuitbot".to_string(),
            citation: sample_citation(),
        }
    }

    #[test]
    fn format_fragments_prompt_empty() {
        let result = format_fragments_prompt(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn format_fragments_prompt_single() {
        let f = make_fragment(1, "Some interesting insight about Rust", "notes/rust.md");
        let result = format_fragments_prompt(&[f]);
        assert!(result.contains("Relevant knowledge from your notes:"));
        assert!(result.contains("(from: notes/rust.md)"));
        assert!(result.contains("Some interesting insight about Rust"));
        assert!(result.contains("Reference these insights"));
    }

    #[test]
    fn format_fragments_single_with_heading() {
        let frags = vec![sample_fragment()];
        let result = format_fragments_prompt(&frags);
        assert!(result.contains("Relevant knowledge"));
        assert!(result.contains("Installation Guide"));
        assert!(result.contains("# Guide > ## Setup"));
        assert!(result.contains("Reference these insights"));
    }

    #[test]
    fn format_fragments_prompt_truncates_at_limit() {
        let big_text = "A".repeat(300);
        let fragments: Vec<FragmentContext> = (0..20)
            .map(|i| make_fragment(i, &big_text, &format!("notes/{i}.md")))
            .collect();
        let result = format_fragments_prompt(&fragments);
        assert!(result.len() <= MAX_FRAGMENT_CHARS);
    }

    #[test]
    fn format_fragments_multiple_items_numbered() {
        let mut f1 = sample_fragment();
        f1.citation.source_title = Some("First".to_string());
        let mut f2 = sample_fragment();
        f2.citation.source_title = Some("Second".to_string());
        let result = format_fragments_prompt(&[f1, f2]);
        assert!(result.contains("1."));
        assert!(result.contains("2."));
    }

    #[test]
    fn build_citations_empty() {
        let result = build_citations(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn build_citations_preserves_fields() {
        let f = make_fragment(42, "chunk text here", "vault/note.md");
        let citations = build_citations(&[f]);
        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].chunk_id, 42);
        assert_eq!(citations[0].node_id, 420);
        assert_eq!(citations[0].source_path, "vault/note.md");
        assert_eq!(citations[0].retrieval_boost, 1.0);
    }

    #[test]
    fn build_citations_returns_all() {
        let frags = vec![sample_fragment(), sample_fragment()];
        let citations = build_citations(&frags);
        assert_eq!(citations.len(), 2);
    }

    #[test]
    fn citations_to_provenance_refs_maps_fields() {
        let citation = VaultCitation {
            chunk_id: 5,
            node_id: 50,
            heading_path: "# Title > ## Section".to_string(),
            source_path: "docs/guide.md".to_string(),
            source_title: Some("Guide".to_string()),
            snippet: "snippet text".to_string(),
            retrieval_boost: 1.5,
            edge_type: None,
            edge_label: None,
        };
        let refs = citations_to_provenance_refs(&[citation]);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].node_id, Some(50));
        assert_eq!(refs[0].chunk_id, Some(5));
        assert_eq!(refs[0].source_path.as_deref(), Some("docs/guide.md"));
        assert_eq!(
            refs[0].heading_path.as_deref(),
            Some("# Title > ## Section")
        );
        assert_eq!(refs[0].snippet.as_deref(), Some("snippet text"));
        assert!(refs[0].seed_id.is_none());
    }

    #[test]
    fn citations_to_chunks_json_empty() {
        let result = citations_to_chunks_json(&[]);
        assert_eq!(result, "[]");
    }

    #[test]
    fn citations_to_chunks_json_valid() {
        let citation = VaultCitation {
            chunk_id: 7,
            node_id: 70,
            heading_path: "# Intro".to_string(),
            source_path: "notes/intro.md".to_string(),
            source_title: None,
            snippet: "intro text".to_string(),
            retrieval_boost: 1.0,
            edge_type: None,
            edge_label: None,
        };
        let result = citations_to_chunks_json(&[citation]);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["chunk_id"], 7);
        assert_eq!(parsed[0]["node_id"], 70);
        assert_eq!(parsed[0]["source_path"], "notes/intro.md");
        assert_eq!(parsed[0]["heading_path"], "# Intro");
    }

    #[test]
    fn format_fragments_heading_path_empty() {
        let f = make_fragment(1, "some text", "path.md");
        let result = format_fragments_prompt(&[f]);
        assert!(!result.contains("[] "));
    }

    #[test]
    fn format_fragments_source_title_fallback() {
        let f = make_fragment(1, "content here", "vault/fallback.md");
        let result = format_fragments_prompt(&[f]);
        assert!(result.contains("vault/fallback.md"));
    }

    #[test]
    fn truncate_text_short_unchanged() {
        assert_eq!(truncate_text("hello", 10), "hello");
    }

    #[test]
    fn truncate_text_long_gets_ellipsis() {
        let result = truncate_text("hello world this is long", 10);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 13);
    }

    #[test]
    fn truncate_text_exact_boundary() {
        let result = truncate_text("hello", 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn truncate_text_empty_string() {
        assert_eq!(truncate_text("", 10), "");
    }

    #[test]
    fn truncate_text_zero_max() {
        let result = truncate_text("hello", 0);
        // max_len=0, sub(3) saturates to 0, so "..."
        assert_eq!(result, "...");
    }

    #[test]
    fn citations_to_provenance_refs_empty() {
        let refs = citations_to_provenance_refs(&[]);
        assert!(refs.is_empty());
    }

    #[test]
    fn citations_to_chunks_json_multiple() {
        let citations = vec![
            VaultCitation {
                chunk_id: 1,
                node_id: 10,
                heading_path: "# A".to_string(),
                source_path: "a.md".to_string(),
                source_title: None,
                snippet: "".to_string(),
                retrieval_boost: 1.0,
                edge_type: None,
                edge_label: None,
            },
            VaultCitation {
                chunk_id: 2,
                node_id: 20,
                heading_path: "# B".to_string(),
                source_path: "b.md".to_string(),
                source_title: Some("B".to_string()),
                snippet: "".to_string(),
                retrieval_boost: 2.0,
                edge_type: None,
                edge_label: None,
            },
        ];
        let json_str = citations_to_chunks_json(&citations);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["chunk_id"], 1);
        assert_eq!(parsed[1]["chunk_id"], 2);
    }

    #[test]
    fn format_fragments_with_source_title() {
        let f = FragmentContext {
            chunk_text: "CLI tool for managing bots".to_string(),
            citation: VaultCitation {
                chunk_id: 1,
                node_id: 10,
                heading_path: "".to_string(),
                source_path: "vault/cli.md".to_string(),
                source_title: Some("CLI Guide".to_string()),
                snippet: "CLI tool...".to_string(),
                retrieval_boost: 1.0,
                edge_type: None,
                edge_label: None,
            },
        };
        let result = format_fragments_prompt(&[f]);
        assert!(result.contains("CLI Guide"));
        assert!(!result.contains("vault/cli.md")); // title takes precedence
    }

    #[test]
    fn fragment_from_chunk_with_context_builds_correctly() {
        use crate::storage::watchtower::{ChunkWithNodeContext, ContentChunk};

        let cwc = ChunkWithNodeContext {
            chunk: ContentChunk {
                id: 42,
                account_id: "acct".to_string(),
                node_id: 100,
                heading_path: "# Title".to_string(),
                chunk_text: "Some chunk text for testing purposes".to_string(),
                chunk_hash: "hash".to_string(),
                chunk_index: 0,
                retrieval_boost: 1.5,
                status: "active".to_string(),
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
            },
            relative_path: "notes/test.md".to_string(),
            source_title: Some("Test Note".to_string()),
        };

        let frag = fragment_from_chunk_with_context(cwc);
        assert_eq!(frag.citation.chunk_id, 42);
        assert_eq!(frag.citation.node_id, 100);
        assert_eq!(frag.citation.source_path, "notes/test.md");
        assert_eq!(frag.citation.source_title, Some("Test Note".to_string()));
        assert_eq!(frag.citation.heading_path, "# Title");
        assert!((frag.citation.retrieval_boost - 1.5).abs() < 0.001);
        assert_eq!(frag.chunk_text, "Some chunk text for testing purposes");
    }

    #[test]
    fn vault_citation_clone() {
        let c = sample_citation();
        let c2 = c.clone();
        assert_eq!(c.chunk_id, c2.chunk_id);
        assert_eq!(c.heading_path, c2.heading_path);
    }

    #[test]
    fn fragment_context_clone() {
        let f = sample_fragment();
        let f2 = f.clone();
        assert_eq!(f.chunk_text, f2.chunk_text);
        assert_eq!(f.citation.chunk_id, f2.citation.chunk_id);
    }

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(MAX_FRAGMENT_CHARS, 1000);
        assert_eq!(MAX_FRAGMENTS, 5);
    }
}
