//! Shared RAG context resolution for assist and discovery routes.

use tuitbot_core::context::graph_expansion::{self, GraphNeighbor, GraphState};
use tuitbot_core::context::winning_dna;
use tuitbot_core::storage::vault_selections;

use crate::state::AppState;

/// Resolve optional RAG context from the vault for reply/compose handlers.
///
/// Loads the business profile's keyword set, queries winning ancestors and
/// content seeds via `build_draft_context()`, and returns the full
/// `DraftContext` including vault citations. Returns `None` (fail-open) on
/// any error or when no relevant context exists.
pub(crate) async fn resolve_composer_rag_context(
    state: &AppState,
    account_id: &str,
    selected_node_ids: Option<&[i64]>,
) -> Option<winning_dna::DraftContext> {
    let config = match state.load_effective_config(account_id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("composer RAG: failed to load config: {e}");
            return None;
        }
    };

    let keywords = config.business.draft_context_keywords();
    // When the user explicitly selected vault notes, always proceed even
    // without business keywords — the selected node IDs drive retrieval.
    let has_selection = selected_node_ids.is_some_and(|ids| !ids.is_empty());
    if keywords.is_empty() && !has_selection {
        return None;
    }

    let draft_context = match winning_dna::build_draft_context_with_selection(
        &state.db,
        account_id,
        &keywords,
        winning_dna::MAX_ANCESTORS,
        winning_dna::RECENCY_HALF_LIFE_DAYS,
        selected_node_ids,
    )
    .await
    {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::warn!("composer RAG: failed to build draft context: {e}");
            return None;
        }
    };

    if draft_context.prompt_block.is_empty() {
        None
    } else {
        Some(draft_context)
    }
}

/// Resolve RAG context from a Ghostwriter selection session_id.
///
/// Fetches the selection, then delegates to `resolve_composer_rag_context`
/// using the resolved node ID when available. Also returns any raw
/// `selected_text` for direct injection as additional context.
///
/// Returns `None` (fail-open) if the selection is expired, not found,
/// or context resolution fails.
pub(crate) async fn resolve_selection_rag_context(
    state: &AppState,
    account_id: &str,
    session_id: &str,
) -> Option<SelectionRagContext> {
    let selection =
        match vault_selections::get_selection_by_session(&state.db, account_id, session_id).await {
            Ok(Some(sel)) => sel,
            Ok(None) => {
                tracing::debug!(session_id, "selection not found or expired");
                return None;
            }
            Err(e) => {
                tracing::warn!(session_id, "failed to fetch selection: {e}");
                return None;
            }
        };

    let node_ids = selection.resolved_node_id.map(|id| vec![id]);
    let draft_context = resolve_composer_rag_context(state, account_id, node_ids.as_deref()).await;

    Some(SelectionRagContext {
        draft_context,
        selected_text: if selection.selected_text.is_empty() {
            None
        } else {
            Some(selection.selected_text)
        },
    })
}

/// Resolve graph neighbor suggestions for a note node.
///
/// Returns neighbors with a graph state indicator. Fail-open: any error
/// yields `FallbackActive` with empty neighbors.
pub(crate) async fn resolve_graph_suggestions(
    state: &AppState,
    account_id: &str,
    node_id: i64,
    max_neighbors: u32,
) -> GraphSuggestionResult {
    match graph_expansion::expand_graph_neighbors(&state.db, account_id, node_id, max_neighbors)
        .await
    {
        Ok(neighbors) if neighbors.is_empty() => GraphSuggestionResult {
            neighbors: Vec::new(),
            graph_state: GraphState::NoRelatedNotes,
        },
        Ok(neighbors) => GraphSuggestionResult {
            graph_state: GraphState::Available,
            neighbors,
        },
        Err(e) => {
            tracing::warn!(node_id, "graph expansion failed: {e}");
            GraphSuggestionResult {
                neighbors: Vec::new(),
                graph_state: GraphState::FallbackActive,
            }
        }
    }
}

/// Result of graph neighbor resolution.
pub(crate) struct GraphSuggestionResult {
    pub neighbors: Vec<GraphNeighbor>,
    pub graph_state: GraphState,
}

/// RAG context resolved from a Ghostwriter selection.
pub(crate) struct SelectionRagContext {
    /// Standard RAG context from node resolution (may be None if node wasn't indexed).
    pub draft_context: Option<winning_dna::DraftContext>,
    /// Raw selected text from Obsidian (for direct injection when no indexed node).
    pub selected_text: Option<String>,
}
