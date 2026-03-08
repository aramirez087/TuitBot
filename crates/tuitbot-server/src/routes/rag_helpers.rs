//! Shared RAG context resolution for assist and discovery routes.

use tuitbot_core::context::winning_dna;

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
    if keywords.is_empty() {
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
