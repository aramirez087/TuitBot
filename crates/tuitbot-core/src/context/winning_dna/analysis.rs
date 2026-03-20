//! Classification, retrieval, prompt formatting, and helper functions.
//!
//! Contains:
//! - Deterministic archetype classifiers (no DB).
//! - DB-backed retrieval functions for winning ancestors and cold-start seeds.
//! - Three-tier context builder.
//! - Prompt formatting utilities.

use crate::context::retrieval;
use crate::context::winning_dna::{
    ContentSeedContext, DraftContext, WinningAncestor, MAX_ANCESTOR_CHARS, MAX_COLD_START_SEEDS,
    MIN_ENGAGEMENT_SCORE, RAG_MAX_CHARS,
};
use crate::error::StorageError;
use crate::storage::analytics;
use crate::storage::watchtower;
use crate::storage::DbPool;

use super::scoring::compute_retrieval_weight;

// ============================================================================
// Classification (deterministic, rule-based)
// ============================================================================

/// Classify reply text into a reply archetype by keyword/pattern matching.
///
/// These heuristics are conservative — the default is `agree_and_expand`.
/// Accuracy doesn't need to be perfect since archetypes are used for
/// retrieval weighting, not display.
pub fn classify_reply_archetype(content: &str) -> String {
    let lower = content.to_lowercase();

    // Question signals
    if lower.ends_with('?')
        || lower.starts_with("what ")
        || lower.starts_with("how ")
        || lower.starts_with("why ")
        || lower.starts_with("have you")
        || lower.starts_with("do you")
    {
        return "ask_question".to_string();
    }

    // Experience sharing signals
    if lower.contains("i've found")
        || lower.contains("i've noticed")
        || lower.contains("i've experienced")
        || lower.contains("in my experience")
        || lower.contains("i recently")
        || lower.contains("when i was")
    {
        return "share_experience".to_string();
    }

    // Data/evidence signals
    if lower.contains("data shows")
        || lower.contains("stats show")
        || lower.contains("study shows")
        || lower.contains("research shows")
        || lower.contains("according to")
        || lower.contains("% of")
    {
        return "add_data".to_string();
    }

    // Disagreement signals
    if (lower.contains("actually") || lower.contains("however") || lower.contains("but "))
        && (lower.contains("i think") || lower.contains("i'd argue") || lower.contains("not sure"))
    {
        return "respectful_disagree".to_string();
    }

    // Default
    "agree_and_expand".to_string()
}

/// Classify tweet text into a tweet format by keyword/pattern matching.
///
/// Heuristic-based; defaults to `storytelling`.
pub fn classify_tweet_format(content: &str) -> String {
    let lower = content.to_lowercase();

    // Numbered list detection
    if lower.contains("1.") && lower.contains("2.") {
        return "list".to_string();
    }

    // Contrarian / most-people patterns
    if lower.contains("most people think") || lower.contains("everyone says") {
        return "most_people_think_x".to_string();
    }
    if lower.contains("actually,") && lower.contains("but") {
        return "contrarian_take".to_string();
    }

    // Before/after
    if lower.contains("before:") || lower.contains("after:") || lower.contains("before →") {
        return "before_after".to_string();
    }

    // Question
    if lower.ends_with('?') {
        return "question".to_string();
    }

    // Tip (short actionable)
    if (lower.starts_with("tip:") || lower.starts_with("pro tip:") || lower.contains("→"))
        && content.len() < 200
    {
        return "tip".to_string();
    }

    // Default
    "storytelling".to_string()
}

// ============================================================================
// Internal helpers
// ============================================================================

pub(super) fn compute_days_since(posted_at: &str, now: &chrono::DateTime<chrono::Utc>) -> f64 {
    chrono::DateTime::parse_from_rfc3339(posted_at)
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(posted_at, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(posted_at, "%Y-%m-%dT%H:%M:%SZ")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .map(|dt| (*now - dt.to_utc()).num_hours() as f64 / 24.0)
        .unwrap_or(30.0) // default to 30 days if unparseable
}

pub(super) fn classify_for_row(content_type: &str, content: &str) -> String {
    match content_type {
        "reply" => classify_reply_archetype(content),
        _ => classify_tweet_format(content),
    }
}

// ============================================================================
// Retrieval
// ============================================================================

/// Retrieve high-performing ancestors for use as draft context.
///
/// Queries the DB for scored ancestors matching topic keywords, computes
/// retrieval weights with recency decay, and returns the top K.
pub async fn retrieve_ancestors(
    pool: &DbPool,
    account_id: &str,
    topic_keywords: &[String],
    max_results: u32,
    half_life_days: f64,
) -> Result<Vec<WinningAncestor>, StorageError> {
    let rows =
        analytics::get_scored_ancestors(pool, account_id, topic_keywords, MIN_ENGAGEMENT_SCORE, 50)
            .await?;

    let now = chrono::Utc::now();

    let mut ancestors: Vec<WinningAncestor> = rows
        .into_iter()
        .filter_map(|row| {
            let engagement = row.engagement_score?;
            let days_since = compute_days_since(&row.posted_at, &now);
            let weight = compute_retrieval_weight(engagement, days_since, half_life_days);

            let archetype = row
                .archetype_vibe
                .clone()
                .unwrap_or_else(|| classify_for_row(&row.content_type, &row.content_preview));

            Some(WinningAncestor {
                tweet_id: row.id,
                content_preview: row.content_preview,
                content_type: row.content_type,
                archetype_vibe: archetype,
                engagement_score: engagement,
                retrieval_weight: weight,
                posted_at: row.posted_at,
            })
        })
        .collect();

    ancestors.sort_by(|a, b| {
        b.retrieval_weight
            .partial_cmp(&a.retrieval_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ancestors.truncate(max_results as usize);
    Ok(ancestors)
}

/// Retrieve cold-start seeds when no performance data and no chunks exist.
pub async fn retrieve_cold_start_seeds(
    pool: &DbPool,
    account_id: &str,
    max_results: u32,
) -> Result<Vec<ContentSeedContext>, StorageError> {
    let rows = watchtower::get_seeds_for_context_for(pool, account_id, max_results).await?;

    Ok(rows
        .into_iter()
        .map(|r| ContentSeedContext {
            seed_text: r.seed_text,
            source_title: r.source_title,
            archetype_suggestion: r.archetype_suggestion,
            engagement_weight: r.engagement_weight,
        })
        .collect())
}

/// Build the complete draft context using a three-tier model:
///
/// 1. **Winning ancestors** — behavioral patterns from high-performing content
/// 2. **Vault fragments** — knowledge from the user's notes via chunk retrieval
/// 3. **Content seeds** — LLM-extracted hooks as last-resort fallback
///
/// Ancestors and fragments combine when both are available. Seeds are used
/// only when no fragments exist (chunking hasn't run or vault is empty).
///
/// The `prompt_block` is capped at `RAG_MAX_CHARS` characters.
pub async fn build_draft_context(
    pool: &DbPool,
    account_id: &str,
    topic_keywords: &[String],
    max_ancestors: u32,
    half_life_days: f64,
) -> Result<DraftContext, StorageError> {
    build_draft_context_with_selection(
        pool,
        account_id,
        topic_keywords,
        max_ancestors,
        half_life_days,
        None,
    )
    .await
}

/// Build draft context with optional selected note IDs for biased retrieval.
///
/// When `selected_node_ids` is provided, chunks from those notes are
/// retrieved first, then remaining slots are filled with keyword matches.
pub async fn build_draft_context_with_selection(
    pool: &DbPool,
    account_id: &str,
    topic_keywords: &[String],
    max_ancestors: u32,
    half_life_days: f64,
    selected_node_ids: Option<&[i64]>,
) -> Result<DraftContext, StorageError> {
    // Tier 1: Winning ancestors (always attempted)
    let ancestors = retrieve_ancestors(
        pool,
        account_id,
        topic_keywords,
        max_ancestors,
        half_life_days,
    )
    .await?;

    // Tier 2: Vault fragments via keyword search (always attempted)
    let fragments = retrieval::retrieve_vault_fragments(
        pool,
        account_id,
        topic_keywords,
        selected_node_ids,
        retrieval::MAX_FRAGMENTS,
    )
    .await?;

    let has_ancestors = !ancestors.is_empty();
    let has_fragments = !fragments.is_empty();

    // Combined: ancestors + fragments
    if has_ancestors && has_fragments {
        let vault_citations = retrieval::build_citations(&fragments);
        let ancestor_block = format_ancestors_prompt_capped(&ancestors, MAX_ANCESTOR_CHARS);
        let fragment_block = retrieval::format_fragments_prompt(&fragments);
        let prompt_block = combine_prompt_blocks(&ancestor_block, &fragment_block);
        return Ok(DraftContext {
            winning_ancestors: ancestors,
            content_seeds: vec![],
            vault_citations,
            prompt_block,
        });
    }

    // Ancestors only
    if has_ancestors {
        let prompt_block = format_ancestors_prompt(&ancestors);
        return Ok(DraftContext {
            winning_ancestors: ancestors,
            content_seeds: vec![],
            vault_citations: vec![],
            prompt_block,
        });
    }

    // Fragments only
    if has_fragments {
        let vault_citations = retrieval::build_citations(&fragments);
        let prompt_block = retrieval::format_fragments_prompt(&fragments);
        return Ok(DraftContext {
            winning_ancestors: vec![],
            content_seeds: vec![],
            vault_citations,
            prompt_block,
        });
    }

    // Tier 3: Cold-start fallback — content seeds
    let seeds = retrieve_cold_start_seeds(pool, account_id, MAX_COLD_START_SEEDS).await?;
    let prompt_block = format_seeds_prompt(&seeds);

    Ok(DraftContext {
        winning_ancestors: vec![],
        content_seeds: seeds,
        vault_citations: vec![],
        prompt_block,
    })
}

// ============================================================================
// Prompt formatting
// ============================================================================

pub(super) fn format_ancestors_prompt(ancestors: &[WinningAncestor]) -> String {
    if ancestors.is_empty() {
        return String::new();
    }

    let mut block = String::from("\nWinning patterns from your best-performing content:\n");

    for (i, a) in ancestors.iter().enumerate() {
        let entry = format!(
            "{}. [{}] ({}): \"{}\"\n",
            i + 1,
            a.archetype_vibe,
            a.content_type,
            a.content_preview,
        );
        if block.len() + entry.len() > RAG_MAX_CHARS {
            break;
        }
        block.push_str(&entry);
    }

    block.push_str("Use these patterns as inspiration but don't copy them directly.\n");

    if block.len() > RAG_MAX_CHARS {
        block.truncate(RAG_MAX_CHARS);
    }
    block
}

/// Format ancestors with a custom character cap (used when combining with fragments).
pub(super) fn format_ancestors_prompt_capped(
    ancestors: &[WinningAncestor],
    max_chars: usize,
) -> String {
    if ancestors.is_empty() {
        return String::new();
    }

    let mut block = String::from("\nWinning patterns from your best-performing content:\n");

    for (i, a) in ancestors.iter().enumerate() {
        let entry = format!(
            "{}. [{}] ({}): \"{}\"\n",
            i + 1,
            a.archetype_vibe,
            a.content_type,
            a.content_preview,
        );
        if block.len() + entry.len() > max_chars {
            break;
        }
        block.push_str(&entry);
    }

    block.push_str("Use these patterns as inspiration but don't copy them directly.\n");

    if block.len() > max_chars {
        block.truncate(max_chars);
    }
    block
}

/// Combine ancestor and fragment prompt blocks, respecting `RAG_MAX_CHARS`.
pub(super) fn combine_prompt_blocks(ancestor_block: &str, fragment_block: &str) -> String {
    let combined = format!("{ancestor_block}{fragment_block}");
    if combined.len() > RAG_MAX_CHARS {
        truncate_at_char_boundary(&combined, RAG_MAX_CHARS)
    } else {
        combined
    }
}

/// Truncate a string at the given byte position, backing up to a char boundary.
pub(super) fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

pub(super) fn format_seeds_prompt(seeds: &[ContentSeedContext]) -> String {
    if seeds.is_empty() {
        return String::new();
    }

    let mut block = String::from("\nRelevant ideas from your notes:\n");

    for (i, s) in seeds.iter().enumerate() {
        let title_part = s
            .source_title
            .as_deref()
            .map(|t| format!(" (from: {t})"))
            .unwrap_or_default();
        let entry = format!("{}. \"{}\"{}\n", i + 1, s.seed_text, title_part);
        if block.len() + entry.len() > RAG_MAX_CHARS {
            break;
        }
        block.push_str(&entry);
    }

    block.push_str("Draw on these ideas to make your response more informed.\n");

    if block.len() > RAG_MAX_CHARS {
        block.truncate(RAG_MAX_CHARS);
    }
    block
}
