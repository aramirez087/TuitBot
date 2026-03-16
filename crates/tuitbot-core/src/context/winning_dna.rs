//! Winning DNA classification, scoring, and retrieval.
//!
//! Classifies historical tweet output into archetypes, computes
//! engagement-weighted success scores, and retrieves ranked ancestors
//! for use as RAG context in new draft generation.

use crate::context::retrieval::{self, VaultCitation};
use crate::error::StorageError;
use crate::storage::analytics;
use crate::storage::watchtower;
use crate::storage::DbPool;

// ============================================================================
// Thresholds (documented in rag-ranking.md)
// ============================================================================

/// Exponential decay half-life for retrieval weight (days).
/// Content success patterns change; older hits contribute less.
pub const RECENCY_HALF_LIFE_DAYS: f64 = 14.0;

/// Maximum number of winning ancestors to include in a draft context.
pub const MAX_ANCESTORS: u32 = 5;

/// Default engagement weight for unscored content (cold-start baseline).
pub const COLD_START_WEIGHT: f64 = 0.5;

/// Minimum engagement score to include an ancestor in retrieval.
/// Filters out bottom ~10% performers.
pub const MIN_ENGAGEMENT_SCORE: f64 = 0.1;

/// Maximum character count for the formatted RAG prompt block.
/// Conservative estimate at ~500 tokens (4 chars/token).
pub const RAG_MAX_CHARS: usize = 2000;

/// Maximum character count for the ancestor prompt section when combined with fragments.
pub const MAX_ANCESTOR_CHARS: usize = 800;

/// Maximum number of cold-start seeds to retrieve as fallback.
pub const MAX_COLD_START_SEEDS: u32 = 5;

// ============================================================================
// Structs
// ============================================================================

/// A historically successful tweet classified with engagement data.
#[derive(Debug, Clone)]
pub struct WinningAncestor {
    /// Tweet or reply ID.
    pub tweet_id: String,
    /// Truncated content preview (up to 120 chars).
    pub content_preview: String,
    /// "reply" or "tweet".
    pub content_type: String,
    /// Classified archetype/format name.
    pub archetype_vibe: String,
    /// Normalized engagement score (0.0-1.0).
    pub engagement_score: f64,
    /// Engagement score weighted by recency decay.
    pub retrieval_weight: f64,
    /// When the content was posted (ISO-8601).
    pub posted_at: String,
}

/// Context block ready for injection into LLM prompts.
#[derive(Debug, Clone)]
pub struct DraftContext {
    /// High-performing historical content for reference.
    pub winning_ancestors: Vec<WinningAncestor>,
    /// Content seeds from ingested notes (cold-start fallback).
    pub content_seeds: Vec<ContentSeedContext>,
    /// Structured citations for vault fragments used in the prompt.
    pub vault_citations: Vec<VaultCitation>,
    /// Formatted text block for LLM prompt injection.
    pub prompt_block: String,
}

/// A content seed from an ingested note.
#[derive(Debug, Clone)]
pub struct ContentSeedContext {
    /// The seed hook text.
    pub seed_text: String,
    /// Title from the originating note.
    pub source_title: Option<String>,
    /// Suggested archetype for the seed.
    pub archetype_suggestion: Option<String>,
    /// Engagement weight for ranking.
    pub engagement_weight: f64,
}

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
// Scoring
// ============================================================================

/// Normalize a raw performance_score to 0.0-1.0 range.
///
/// Returns `COLD_START_WEIGHT` (0.5) if max_score is zero (cold-start).
pub fn compute_engagement_score(performance_score: f64, max_score: f64) -> f64 {
    if max_score <= 0.0 {
        return COLD_START_WEIGHT;
    }
    (performance_score / max_score).clamp(0.0, 1.0)
}

/// Compute retrieval weight with exponential recency decay.
///
/// Formula: `engagement_score * exp(-0.693 * days_since / half_life)`
///
/// At `days_since = half_life`, weight ≈ engagement_score / 2.
/// At `days_since = 4 * half_life` (56 days), weight ≈ engagement_score * 0.0625.
pub fn compute_retrieval_weight(engagement_score: f64, days_since: f64, half_life: f64) -> f64 {
    if half_life <= 0.0 {
        return engagement_score;
    }
    engagement_score * (-0.693 * days_since / half_life).exp()
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

fn format_ancestors_prompt(ancestors: &[WinningAncestor]) -> String {
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
fn format_ancestors_prompt_capped(ancestors: &[WinningAncestor], max_chars: usize) -> String {
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
fn combine_prompt_blocks(ancestor_block: &str, fragment_block: &str) -> String {
    let combined = format!("{ancestor_block}{fragment_block}");
    if combined.len() > RAG_MAX_CHARS {
        truncate_at_char_boundary(&combined, RAG_MAX_CHARS)
    } else {
        combined
    }
}

/// Truncate a string at the given byte position, backing up to a char boundary.
fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

fn format_seeds_prompt(seeds: &[ContentSeedContext]) -> String {
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

// ============================================================================
// Helpers
// ============================================================================

fn compute_days_since(posted_at: &str, now: &chrono::DateTime<chrono::Utc>) -> f64 {
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

fn classify_for_row(content_type: &str, content: &str) -> String {
    match content_type {
        "reply" => classify_reply_archetype(content),
        _ => classify_tweet_format(content),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Classification tests (deterministic, no DB) ---

    #[test]
    fn classify_reply_question() {
        assert_eq!(
            classify_reply_archetype("What tools do you use for this?"),
            "ask_question"
        );
        assert_eq!(
            classify_reply_archetype("How did you solve this problem?"),
            "ask_question"
        );
    }

    #[test]
    fn classify_reply_experience() {
        assert_eq!(
            classify_reply_archetype("I've noticed the same pattern in my projects"),
            "share_experience"
        );
        assert_eq!(
            classify_reply_archetype("In my experience, this approach works well"),
            "share_experience"
        );
    }

    #[test]
    fn classify_reply_data() {
        assert_eq!(
            classify_reply_archetype("Research shows that 80% of teams benefit from this"),
            "add_data"
        );
        assert_eq!(
            classify_reply_archetype("According to the latest study, this is effective"),
            "add_data"
        );
    }

    #[test]
    fn classify_reply_disagree() {
        assert_eq!(
            classify_reply_archetype("However, I think the tradeoffs are different here"),
            "respectful_disagree"
        );
        assert_eq!(
            classify_reply_archetype("But actually, I'd argue the opposite is true"),
            "respectful_disagree"
        );
    }

    #[test]
    fn classify_reply_default() {
        assert_eq!(
            classify_reply_archetype("Totally agree, and this also applies to team dynamics"),
            "agree_and_expand"
        );
    }

    #[test]
    fn classify_tweet_list() {
        assert_eq!(
            classify_tweet_format("Top tips:\n1. Write tests\n2. Use CI\n3. Review code"),
            "list"
        );
    }

    #[test]
    fn classify_tweet_contrarian() {
        assert_eq!(
            classify_tweet_format("Most people think testing is optional. It's not."),
            "most_people_think_x"
        );
    }

    #[test]
    fn classify_tweet_question() {
        assert_eq!(
            classify_tweet_format("What's the one tool you can't live without?"),
            "question"
        );
    }

    #[test]
    fn classify_tweet_tip() {
        assert_eq!(
            classify_tweet_format("Pro tip: use → to chain operations in Rust"),
            "tip"
        );
    }

    #[test]
    fn classify_tweet_default() {
        assert_eq!(
            classify_tweet_format("Last week I shipped a feature that changed everything"),
            "storytelling"
        );
    }

    // --- Scoring tests (deterministic, no DB) ---

    #[test]
    fn engagement_score_normalization() {
        assert!((compute_engagement_score(50.0, 100.0) - 0.5).abs() < 0.001);
        assert!((compute_engagement_score(100.0, 100.0) - 1.0).abs() < 0.001);
        assert!((compute_engagement_score(0.0, 100.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn engagement_score_zero_max() {
        // Cold-start: max_score=0 returns baseline
        assert!((compute_engagement_score(50.0, 0.0) - COLD_START_WEIGHT).abs() < 0.001);
    }

    #[test]
    fn engagement_score_clamps() {
        assert!((compute_engagement_score(200.0, 100.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn retrieval_weight_no_decay() {
        // days_since=0 → weight = engagement_score
        let w = compute_retrieval_weight(0.8, 0.0, 14.0);
        assert!((w - 0.8).abs() < 0.001);
    }

    #[test]
    fn retrieval_weight_half_life() {
        // At exactly half_life, weight ≈ engagement_score / 2
        let w = compute_retrieval_weight(1.0, 14.0, 14.0);
        assert!((w - 0.5).abs() < 0.05, "got {w}");
    }

    #[test]
    fn retrieval_weight_old_content() {
        // At 4x half_life (56 days), weight should be very low
        let w = compute_retrieval_weight(1.0, 56.0, 14.0);
        assert!(w < 0.1, "got {w}");
    }

    #[test]
    fn retrieval_weight_zero_half_life() {
        // Zero half-life → no decay
        let w = compute_retrieval_weight(0.8, 100.0, 0.0);
        assert!((w - 0.8).abs() < 0.001);
    }

    #[test]
    fn retrieval_weight_negative_half_life() {
        let w = compute_retrieval_weight(0.5, 10.0, -1.0);
        assert!((w - 0.5).abs() < 0.001);
    }

    #[test]
    fn engagement_score_negative_max() {
        // Negative max_score treated as cold-start
        assert!((compute_engagement_score(50.0, -10.0) - COLD_START_WEIGHT).abs() < 0.001);
    }

    // --- classify_for_row ---

    #[test]
    fn classify_for_row_reply() {
        assert_eq!(
            classify_for_row("reply", "How did you do that?"),
            "ask_question"
        );
    }

    #[test]
    fn classify_for_row_tweet() {
        assert_eq!(
            classify_for_row("tweet", "What's your favorite tool?"),
            "question"
        );
    }

    #[test]
    fn classify_for_row_unknown_type_defaults_to_tweet() {
        let result = classify_for_row("other", "Some content");
        assert_eq!(result, "storytelling");
    }

    // --- compute_days_since ---

    #[test]
    fn compute_days_since_recent() {
        let now = chrono::Utc::now();
        let posted = now.to_rfc3339();
        let days = compute_days_since(&posted, &now);
        assert!(days < 0.1, "should be ~0 days, got {days}");
    }

    #[test]
    fn compute_days_since_invalid_date() {
        let now = chrono::Utc::now();
        let days = compute_days_since("not-a-date", &now);
        assert!((days - 30.0).abs() < 0.1, "should default to 30 days");
    }

    #[test]
    fn compute_days_since_sqlite_format() {
        let now = chrono::Utc::now();
        let one_day_ago = now - chrono::Duration::hours(24);
        let posted = one_day_ago.format("%Y-%m-%d %H:%M:%S").to_string();
        let days = compute_days_since(&posted, &now);
        assert!((days - 1.0).abs() < 0.1, "should be ~1 day, got {days}");
    }

    // --- truncate_at_char_boundary ---

    #[test]
    fn truncate_at_char_boundary_short() {
        assert_eq!(truncate_at_char_boundary("hello", 100), "hello");
    }

    #[test]
    fn truncate_at_char_boundary_exact() {
        assert_eq!(truncate_at_char_boundary("hello", 5), "hello");
    }

    #[test]
    fn truncate_at_char_boundary_truncates() {
        let result = truncate_at_char_boundary("hello world", 5);
        assert_eq!(result, "hello");
    }

    // --- combine_prompt_blocks ---

    #[test]
    fn combine_prompt_blocks_under_limit() {
        let a = "short ancestor";
        let f = "short fragment";
        let result = combine_prompt_blocks(a, f);
        assert_eq!(result, "short ancestorshort fragment");
    }

    // --- format_seeds_prompt ---

    #[test]
    fn format_seeds_with_title() {
        let seeds = vec![ContentSeedContext {
            seed_text: "Test seed".into(),
            source_title: Some("My Notes".into()),
            archetype_suggestion: None,
            engagement_weight: 0.5,
        }];
        let block = format_seeds_prompt(&seeds);
        assert!(block.contains("(from: My Notes)"));
    }

    #[test]
    fn format_seeds_without_title() {
        let seeds = vec![ContentSeedContext {
            seed_text: "Test seed".into(),
            source_title: None,
            archetype_suggestion: None,
            engagement_weight: 0.5,
        }];
        let block = format_seeds_prompt(&seeds);
        assert!(!block.contains("(from:"));
    }

    // --- Prompt formatting tests ---

    #[test]
    fn format_ancestors_empty() {
        assert!(format_ancestors_prompt(&[]).is_empty());
    }

    #[test]
    fn format_ancestors_contains_header() {
        let ancestors = vec![WinningAncestor {
            tweet_id: "tw1".into(),
            content_preview: "Great testing advice".into(),
            content_type: "tweet".into(),
            archetype_vibe: "tip".into(),
            engagement_score: 0.9,
            retrieval_weight: 0.85,
            posted_at: "2026-02-27T10:00:00Z".into(),
        }];
        let block = format_ancestors_prompt(&ancestors);
        assert!(block.contains("Winning patterns"));
        assert!(block.contains("[tip]"));
        assert!(block.contains("Great testing advice"));
    }

    #[test]
    fn format_ancestors_caps_length() {
        let ancestors: Vec<WinningAncestor> = (0..100)
            .map(|i| WinningAncestor {
                tweet_id: format!("tw{i}"),
                content_preview: "A".repeat(120),
                content_type: "tweet".into(),
                archetype_vibe: "storytelling".into(),
                engagement_score: 0.9,
                retrieval_weight: 0.85,
                posted_at: "2026-02-27T10:00:00Z".into(),
            })
            .collect();
        let block = format_ancestors_prompt(&ancestors);
        assert!(block.len() <= RAG_MAX_CHARS);
    }

    #[test]
    fn format_seeds_empty() {
        assert!(format_seeds_prompt(&[]).is_empty());
    }

    #[test]
    fn format_seeds_contains_header() {
        let seeds = vec![ContentSeedContext {
            seed_text: "Rust ownership makes concurrency safe".into(),
            source_title: Some("Rust Notes".into()),
            archetype_suggestion: Some("tip".into()),
            engagement_weight: 0.5,
        }];
        let block = format_seeds_prompt(&seeds);
        assert!(block.contains("Relevant ideas"));
        assert!(block.contains("Rust ownership"));
        assert!(block.contains("(from: Rust Notes)"));
    }

    // --- DB integration tests ---

    const TEST_ACCOUNT: &str = "00000000-0000-0000-0000-000000000000";

    #[tokio::test]
    async fn retrieve_ancestors_empty_db() {
        let pool = crate::storage::init_test_db().await.expect("init db");
        let ancestors = retrieve_ancestors(&pool, TEST_ACCOUNT, &[], 5, 14.0)
            .await
            .expect("retrieve");
        assert!(ancestors.is_empty());
    }

    #[tokio::test]
    async fn retrieve_ancestors_ranks_by_weight() {
        let pool = crate::storage::init_test_db().await.expect("init db");

        // Insert two tweets with different engagement scores
        for (id, content, score, eng) in [
            ("tw1", "Low performer", 30.0, 0.3),
            ("tw2", "High performer", 90.0, 0.9),
        ] {
            sqlx::query(
                "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
                 VALUES ('00000000-0000-0000-0000-000000000000', ?, ?, 'rust', 'sent', '2026-02-27T10:00:00Z')",
            )
            .bind(id)
            .bind(content)
            .execute(&pool)
            .await
            .expect("insert tweet");

            analytics::upsert_tweet_performance(&pool, id, 10, 5, 3, 500, score)
                .await
                .expect("upsert perf");
            analytics::update_tweet_engagement_score(&pool, id, eng)
                .await
                .expect("update score");
        }

        let ancestors = retrieve_ancestors(&pool, TEST_ACCOUNT, &[], 5, 14.0)
            .await
            .expect("retrieve");
        assert_eq!(ancestors.len(), 2);
        assert_eq!(
            ancestors[0].tweet_id, "tw2",
            "higher score should rank first"
        );
        assert!(ancestors[0].retrieval_weight > ancestors[1].retrieval_weight);
    }

    #[tokio::test]
    async fn cold_start_falls_back_to_seeds() {
        let pool = crate::storage::init_test_db().await.expect("init db");

        // No performance data, but add content seeds
        let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
            .await
            .expect("insert source");
        watchtower::upsert_content_node(
            &pool,
            source_id,
            "notes.md",
            "h1",
            Some("My Notes"),
            "Body",
            None,
            None,
        )
        .await
        .expect("upsert node");
        watchtower::insert_draft_seed_with_weight(
            &pool,
            1,
            "Hook about Rust ownership",
            Some("tip"),
            0.5,
        )
        .await
        .expect("insert seed");

        let ctx = build_draft_context(&pool, TEST_ACCOUNT, &[], 5, 14.0)
            .await
            .expect("build context");
        assert!(ctx.winning_ancestors.is_empty());
        assert_eq!(ctx.content_seeds.len(), 1);
        assert!(ctx.prompt_block.contains("Relevant ideas"));
    }

    #[tokio::test]
    async fn build_draft_context_formats_prompt_with_ancestors() {
        let pool = crate::storage::init_test_db().await.expect("init db");

        sqlx::query(
            "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
             VALUES ('00000000-0000-0000-0000-000000000000', 'tw1', 'Testing is key', 'rust', 'sent', '2026-02-27T10:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert tweet");

        analytics::upsert_tweet_performance(&pool, "tw1", 10, 5, 3, 500, 82.0)
            .await
            .expect("upsert perf");
        analytics::update_tweet_engagement_score(&pool, "tw1", 0.85)
            .await
            .expect("update score");

        let ctx = build_draft_context(&pool, TEST_ACCOUNT, &[], 5, 14.0)
            .await
            .expect("build context");
        assert_eq!(ctx.winning_ancestors.len(), 1);
        assert!(ctx.prompt_block.contains("Winning patterns"));
        assert!(ctx.prompt_block.len() <= RAG_MAX_CHARS);
    }

    #[tokio::test]
    async fn build_draft_context_empty_db_returns_empty_prompt() {
        let pool = crate::storage::init_test_db().await.expect("init db");
        let ctx = build_draft_context(&pool, TEST_ACCOUNT, &[], 5, 14.0)
            .await
            .expect("build context");
        assert!(ctx.winning_ancestors.is_empty());
        assert!(ctx.content_seeds.is_empty());
        assert!(ctx.prompt_block.is_empty());
    }

    #[tokio::test]
    async fn retrieve_ancestors_account_isolation() {
        let pool = crate::storage::init_test_db().await.expect("init db");

        // Insert tweet for account A
        sqlx::query(
            "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
             VALUES ('account-a', 'tw-a', 'Account A tweet', 'rust', 'sent', '2026-02-27T10:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert tweet");

        analytics::upsert_tweet_performance(&pool, "tw-a", 10, 5, 3, 500, 82.0)
            .await
            .expect("upsert perf");
        analytics::update_tweet_engagement_score(&pool, "tw-a", 0.9)
            .await
            .expect("update score");

        // Query as account B → should get nothing
        let ancestors = retrieve_ancestors(&pool, "account-b", &[], 5, 14.0)
            .await
            .expect("retrieve");
        assert!(
            ancestors.is_empty(),
            "account B should see no ancestors from account A"
        );

        // Query as account A → should get 1
        let ancestors = retrieve_ancestors(&pool, "account-a", &[], 5, 14.0)
            .await
            .expect("retrieve");
        assert_eq!(ancestors.len(), 1);
    }

    #[tokio::test]
    async fn build_draft_context_with_fragments() {
        let pool = crate::storage::init_test_db().await.expect("init db");

        // Insert source + node + chunk
        let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
            .await
            .expect("insert source");
        watchtower::upsert_content_node(
            &pool,
            source_id,
            "rust-tips.md",
            "h-frag",
            Some("Rust Tips"),
            "Ownership makes concurrency safe in Rust",
            None,
            None,
        )
        .await
        .expect("upsert node");

        // Insert a chunk directly
        watchtower::insert_chunk(
            &pool,
            TEST_ACCOUNT,
            1,
            "# Rust Tips",
            "Ownership makes concurrency safe in Rust",
            "hash-frag-1",
            0,
        )
        .await
        .expect("insert chunk");

        let ctx = build_draft_context(&pool, TEST_ACCOUNT, &["rust".to_string()], 5, 14.0)
            .await
            .expect("build context");

        assert!(ctx.prompt_block.contains("Relevant knowledge"));
        assert_eq!(ctx.vault_citations.len(), 1);
        assert_eq!(ctx.vault_citations[0].source_path, "rust-tips.md");
    }

    #[tokio::test]
    async fn build_draft_context_mixed_ancestors_and_fragments() {
        let pool = crate::storage::init_test_db().await.expect("init db");

        // Seed ancestors
        sqlx::query(
            "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
             VALUES (?, 'tw-mix', 'Testing patterns rock', 'testing', 'sent', '2026-02-27T10:00:00Z')",
        )
        .bind(TEST_ACCOUNT)
        .execute(&pool)
        .await
        .expect("insert tweet");
        analytics::upsert_tweet_performance(&pool, "tw-mix", 10, 5, 3, 500, 82.0)
            .await
            .expect("upsert perf");
        analytics::update_tweet_engagement_score(&pool, "tw-mix", 0.9)
            .await
            .expect("update score");

        // Seed vault chunks
        let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
            .await
            .expect("insert source");
        watchtower::upsert_content_node(
            &pool,
            source_id,
            "testing.md",
            "h-mix",
            Some("Testing Notes"),
            "Testing strategies for better code",
            None,
            None,
        )
        .await
        .expect("upsert node");
        watchtower::insert_chunk(
            &pool,
            TEST_ACCOUNT,
            1,
            "# Testing",
            "Testing strategies for better code",
            "hash-mix-1",
            0,
        )
        .await
        .expect("insert chunk");

        let ctx = build_draft_context(&pool, TEST_ACCOUNT, &["testing".to_string()], 5, 14.0)
            .await
            .expect("build context");

        assert!(!ctx.winning_ancestors.is_empty(), "should have ancestors");
        assert!(!ctx.vault_citations.is_empty(), "should have citations");
        assert!(ctx.prompt_block.contains("Winning patterns"));
        assert!(ctx.prompt_block.contains("Relevant knowledge"));
        assert!(ctx.prompt_block.len() <= RAG_MAX_CHARS);
    }

    // =========================================================================
    // Additional edge case tests for coverage push
    // =========================================================================

    // --- classify_reply_archetype edge cases ---

    #[test]
    fn classify_reply_why_question() {
        assert_eq!(
            classify_reply_archetype("Why do you think this approach works?"),
            "ask_question"
        );
    }

    #[test]
    fn classify_reply_have_you_question() {
        assert_eq!(
            classify_reply_archetype("Have you tried using this pattern?"),
            "ask_question"
        );
    }

    #[test]
    fn classify_reply_do_you_question() {
        assert_eq!(
            classify_reply_archetype("Do you think this scales well?"),
            "ask_question"
        );
    }

    #[test]
    fn classify_reply_ends_with_question_mark() {
        assert_eq!(
            classify_reply_archetype("Is this the right approach to use here?"),
            "ask_question"
        );
    }

    #[test]
    fn classify_reply_i_recently_experience() {
        assert_eq!(
            classify_reply_archetype("I recently deployed this in production and it worked"),
            "share_experience"
        );
    }

    #[test]
    fn classify_reply_when_i_was_experience() {
        assert_eq!(
            classify_reply_archetype("When I was building my last project, this helped a lot"),
            "share_experience"
        );
    }

    #[test]
    fn classify_reply_ive_experienced() {
        assert_eq!(
            classify_reply_archetype("I've experienced the same issue with my deployments"),
            "share_experience"
        );
    }

    #[test]
    fn classify_reply_ive_found() {
        assert_eq!(
            classify_reply_archetype("I've found that testing early prevents most regressions"),
            "share_experience"
        );
    }

    #[test]
    fn classify_reply_data_shows() {
        assert_eq!(
            classify_reply_archetype("Data shows that teams shipping daily have fewer bugs"),
            "add_data"
        );
    }

    #[test]
    fn classify_reply_stats_show() {
        assert_eq!(
            classify_reply_archetype("Stats show a 40% improvement in deployment frequency"),
            "add_data"
        );
    }

    #[test]
    fn classify_reply_study_shows() {
        assert_eq!(
            classify_reply_archetype("A recent study shows that TDD improves code quality"),
            "add_data"
        );
    }

    #[test]
    fn classify_reply_percent_of() {
        assert_eq!(
            classify_reply_archetype("About 73% of teams use some form of CI now"),
            "add_data"
        );
    }

    #[test]
    fn classify_reply_actually_i_think_disagree() {
        assert_eq!(
            classify_reply_archetype("Actually, I think there are better alternatives"),
            "respectful_disagree"
        );
    }

    #[test]
    fn classify_reply_but_not_sure_disagree() {
        assert_eq!(
            classify_reply_archetype("But I'm not sure that holds in all cases"),
            "respectful_disagree"
        );
    }

    #[test]
    fn classify_reply_however_id_argue() {
        assert_eq!(
            classify_reply_archetype("However, I'd argue that simplicity wins here"),
            "respectful_disagree"
        );
    }

    #[test]
    fn classify_reply_empty_string_defaults() {
        assert_eq!(classify_reply_archetype(""), "agree_and_expand");
    }

    #[test]
    fn classify_reply_generic_agreement() {
        assert_eq!(
            classify_reply_archetype("Great point, I think that applies to many teams"),
            "agree_and_expand"
        );
    }

    #[test]
    fn classify_reply_case_insensitive_question() {
        assert_eq!(
            classify_reply_archetype("WHAT tools do you use for deployment?"),
            "ask_question"
        );
    }

    #[test]
    fn classify_reply_case_insensitive_experience() {
        assert_eq!(
            classify_reply_archetype("IN MY EXPERIENCE, this approach works well"),
            "share_experience"
        );
    }

    // --- classify_tweet_format edge cases ---

    #[test]
    fn classify_tweet_everyone_says_pattern() {
        assert_eq!(
            classify_tweet_format("Everyone says you need a degree. They're wrong."),
            "most_people_think_x"
        );
    }

    #[test]
    fn classify_tweet_contrarian_actually_but() {
        assert_eq!(
            classify_tweet_format("Actually, the common wisdom is wrong but there's a better way"),
            "contrarian_take"
        );
    }

    #[test]
    fn classify_tweet_before_after() {
        assert_eq!(
            classify_tweet_format("Before: 200ms response time. After: 20ms."),
            "before_after"
        );
    }

    #[test]
    fn classify_tweet_before_arrow() {
        assert_eq!(
            classify_tweet_format("Before → After transformation in our pipeline"),
            "before_after"
        );
    }

    #[test]
    fn classify_tweet_tip_short() {
        assert_eq!(
            classify_tweet_format("Tip: always validate input before processing"),
            "tip"
        );
    }

    #[test]
    fn classify_tweet_pro_tip_with_arrow() {
        assert_eq!(
            classify_tweet_format("Pro tip: use → for chaining operations"),
            "tip"
        );
    }

    #[test]
    fn classify_tweet_tip_too_long_defaults() {
        let long_tip = format!("Tip: {}", "a".repeat(200));
        assert_eq!(classify_tweet_format(&long_tip), "storytelling");
    }

    #[test]
    fn classify_tweet_empty_string() {
        assert_eq!(classify_tweet_format(""), "storytelling");
    }

    #[test]
    fn classify_tweet_numbered_list_with_three_items() {
        assert_eq!(
            classify_tweet_format("1. Write tests 2. Run CI 3. Deploy"),
            "list"
        );
    }

    // --- compute_engagement_score edge cases ---

    #[test]
    fn engagement_score_mid_range() {
        let score = compute_engagement_score(25.0, 100.0);
        assert!((score - 0.25).abs() < 0.001);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn engagement_score_very_small_max() {
        let score = compute_engagement_score(0.001, 0.001);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn engagement_score_zero_performance() {
        let score = compute_engagement_score(0.0, 50.0);
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn engagement_score_negative_performance() {
        let score = compute_engagement_score(-10.0, 100.0);
        assert!((score - 0.0).abs() < 0.001);
    }

    // --- compute_retrieval_weight edge cases ---

    #[test]
    fn retrieval_weight_two_half_lives() {
        let w = compute_retrieval_weight(1.0, 28.0, 14.0);
        assert!((w - 0.25).abs() < 0.05, "got {w}");
    }

    #[test]
    fn retrieval_weight_three_half_lives() {
        let w = compute_retrieval_weight(1.0, 42.0, 14.0);
        assert!((w - 0.125).abs() < 0.05, "got {w}");
    }

    #[test]
    fn retrieval_weight_with_partial_engagement() {
        let w = compute_retrieval_weight(0.5, 14.0, 14.0);
        assert!((w - 0.25).abs() < 0.05, "got {w}");
    }

    #[test]
    fn retrieval_weight_zero_engagement() {
        let w = compute_retrieval_weight(0.0, 14.0, 14.0);
        assert!((w - 0.0).abs() < 0.001);
    }

    #[test]
    fn retrieval_weight_negative_days() {
        let w = compute_retrieval_weight(1.0, -5.0, 14.0);
        assert!(w > 1.0, "negative days should increase weight: got {w}");
    }

    #[test]
    fn retrieval_weight_very_large_days() {
        let w = compute_retrieval_weight(1.0, 365.0, 14.0);
        assert!(
            w < 0.001,
            "very old content should have near-zero weight: got {w}"
        );
    }

    #[test]
    fn retrieval_weight_very_small_half_life() {
        let w = compute_retrieval_weight(1.0, 1.0, 0.001);
        assert!(w < 0.001, "tiny half-life should decay quickly: got {w}");
    }

    // --- compute_days_since edge cases ---

    #[test]
    fn compute_days_since_iso_z_format() {
        let now = chrono::Utc::now();
        let one_week_ago = now - chrono::Duration::days(7);
        let posted = one_week_ago.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let days = compute_days_since(&posted, &now);
        assert!((days - 7.0).abs() < 0.2, "should be ~7 days, got {days}");
    }

    #[test]
    fn compute_days_since_rfc3339_with_offset() {
        let now = chrono::Utc::now();
        let posted = (now - chrono::Duration::hours(48)).to_rfc3339();
        let days = compute_days_since(&posted, &now);
        assert!((days - 2.0).abs() < 0.1, "should be ~2 days, got {days}");
    }

    #[test]
    fn compute_days_since_future_date() {
        let now = chrono::Utc::now();
        let future = (now + chrono::Duration::hours(24)).to_rfc3339();
        let days = compute_days_since(&future, &now);
        assert!(
            days < 0.0,
            "future date should give negative days: got {days}"
        );
    }

    #[test]
    fn compute_days_since_empty_string() {
        let now = chrono::Utc::now();
        let days = compute_days_since("", &now);
        assert!(
            (days - 30.0).abs() < 0.1,
            "empty string should default to 30 days"
        );
    }

    #[test]
    fn compute_days_since_partial_date() {
        let now = chrono::Utc::now();
        let days = compute_days_since("2026-01-01", &now);
        assert!(
            (days - 30.0).abs() < 0.1,
            "unparseable partial date should default to 30"
        );
    }

    // --- classify_for_row edge cases ---

    #[test]
    fn classify_for_row_reply_experience() {
        assert_eq!(
            classify_for_row("reply", "I've found this to be true in practice"),
            "share_experience"
        );
    }

    #[test]
    fn classify_for_row_tweet_list() {
        assert_eq!(
            classify_for_row("tweet", "1. First item 2. Second item"),
            "list"
        );
    }

    #[test]
    fn classify_for_row_empty_type() {
        let result = classify_for_row("", "Some content here");
        assert_eq!(result, "storytelling");
    }

    // --- truncate_at_char_boundary edge cases ---

    #[test]
    fn truncate_at_char_boundary_empty_string() {
        assert_eq!(truncate_at_char_boundary("", 10), "");
    }

    #[test]
    fn truncate_at_char_boundary_zero_max() {
        assert_eq!(truncate_at_char_boundary("hello", 0), "");
    }

    #[test]
    fn truncate_at_char_boundary_multibyte_char() {
        let s = "hello \u{1F600} world";
        let result = truncate_at_char_boundary(s, 7);
        assert!(result.len() <= 7);
        assert!(result.is_char_boundary(result.len()));
    }

    #[test]
    fn truncate_at_char_boundary_unicode() {
        let s = "\u{00E9}\u{00E9}\u{00E9}\u{00E9}\u{00E9}";
        let result = truncate_at_char_boundary(s, 4);
        assert!(result.len() <= 4);
        assert!(result.is_char_boundary(result.len()));
    }

    // --- format_ancestors_prompt edge cases ---

    #[test]
    fn format_ancestors_single_ancestor() {
        let ancestors = vec![WinningAncestor {
            tweet_id: "tw1".into(),
            content_preview: "Test content".into(),
            content_type: "reply".into(),
            archetype_vibe: "ask_question".into(),
            engagement_score: 0.7,
            retrieval_weight: 0.6,
            posted_at: "2026-03-01T10:00:00Z".into(),
        }];
        let block = format_ancestors_prompt(&ancestors);
        assert!(block.contains("Winning patterns"));
        assert!(block.contains("[ask_question]"));
        assert!(block.contains("(reply)"));
        assert!(block.contains("Test content"));
        assert!(block.contains("inspiration"));
    }

    #[test]
    fn format_ancestors_multiple_types() {
        let ancestors = vec![
            WinningAncestor {
                tweet_id: "tw1".into(),
                content_preview: "A reply".into(),
                content_type: "reply".into(),
                archetype_vibe: "agree_and_expand".into(),
                engagement_score: 0.9,
                retrieval_weight: 0.85,
                posted_at: "2026-03-01T10:00:00Z".into(),
            },
            WinningAncestor {
                tweet_id: "tw2".into(),
                content_preview: "A tweet".into(),
                content_type: "tweet".into(),
                archetype_vibe: "tip".into(),
                engagement_score: 0.8,
                retrieval_weight: 0.7,
                posted_at: "2026-03-01T10:00:00Z".into(),
            },
        ];
        let block = format_ancestors_prompt(&ancestors);
        assert!(block.contains("1."));
        assert!(block.contains("2."));
        assert!(block.contains("[agree_and_expand]"));
        assert!(block.contains("[tip]"));
    }

    // --- format_ancestors_prompt_capped ---

    #[test]
    fn format_ancestors_capped_small_limit() {
        let ancestors = vec![
            WinningAncestor {
                tweet_id: "tw1".into(),
                content_preview: "A".repeat(100),
                content_type: "tweet".into(),
                archetype_vibe: "storytelling".into(),
                engagement_score: 0.9,
                retrieval_weight: 0.85,
                posted_at: "2026-03-01T10:00:00Z".into(),
            },
            WinningAncestor {
                tweet_id: "tw2".into(),
                content_preview: "B".repeat(100),
                content_type: "tweet".into(),
                archetype_vibe: "tip".into(),
                engagement_score: 0.8,
                retrieval_weight: 0.7,
                posted_at: "2026-03-01T10:00:00Z".into(),
            },
        ];
        let block = format_ancestors_prompt_capped(&ancestors, 200);
        assert!(block.len() <= 200);
    }

    #[test]
    fn format_ancestors_capped_empty() {
        let block = format_ancestors_prompt_capped(&[], 500);
        assert!(block.is_empty());
    }

    // --- format_seeds_prompt edge cases ---

    #[test]
    fn format_seeds_multiple_with_archetype() {
        let seeds = vec![
            ContentSeedContext {
                seed_text: "First seed about Rust".into(),
                source_title: Some("Rust Notes".into()),
                archetype_suggestion: Some("tip".into()),
                engagement_weight: 0.8,
            },
            ContentSeedContext {
                seed_text: "Second seed about testing".into(),
                source_title: None,
                archetype_suggestion: None,
                engagement_weight: 0.5,
            },
        ];
        let block = format_seeds_prompt(&seeds);
        assert!(block.contains("1."));
        assert!(block.contains("2."));
        assert!(block.contains("First seed about Rust"));
        assert!(block.contains("(from: Rust Notes)"));
        assert!(block.contains("Second seed about testing"));
        assert!(!block.contains("(from: )"));
        assert!(block.contains("Draw on these ideas"));
    }

    #[test]
    fn format_seeds_caps_at_rag_max_chars() {
        let seeds: Vec<ContentSeedContext> = (0..100)
            .map(|i| ContentSeedContext {
                seed_text: format!("Seed {i}: {}", "x".repeat(100)),
                source_title: Some(format!("Note {i}")),
                archetype_suggestion: None,
                engagement_weight: 0.5,
            })
            .collect();
        let block = format_seeds_prompt(&seeds);
        assert!(block.len() <= RAG_MAX_CHARS);
    }

    // --- combine_prompt_blocks edge cases ---

    #[test]
    fn combine_prompt_blocks_over_limit_truncates() {
        let a = "a".repeat(RAG_MAX_CHARS);
        let f = "b".repeat(100);
        let result = combine_prompt_blocks(&a, &f);
        assert!(result.len() <= RAG_MAX_CHARS);
    }

    #[test]
    fn combine_prompt_blocks_exact_limit() {
        let total = RAG_MAX_CHARS;
        let a = "a".repeat(total / 2);
        let f = "b".repeat(total - a.len());
        let result = combine_prompt_blocks(&a, &f);
        assert!(result.len() <= RAG_MAX_CHARS);
    }

    #[test]
    fn combine_prompt_blocks_empty_strings() {
        let result = combine_prompt_blocks("", "");
        assert!(result.is_empty());
    }

    #[test]
    fn combine_prompt_blocks_one_empty() {
        let result = combine_prompt_blocks("ancestors", "");
        assert_eq!(result, "ancestors");
        let result2 = combine_prompt_blocks("", "fragments");
        assert_eq!(result2, "fragments");
    }

    // --- WinningAncestor struct tests ---

    #[test]
    fn winning_ancestor_debug_and_clone() {
        let ancestor = WinningAncestor {
            tweet_id: "tw1".into(),
            content_preview: "Test".into(),
            content_type: "tweet".into(),
            archetype_vibe: "tip".into(),
            engagement_score: 0.9,
            retrieval_weight: 0.85,
            posted_at: "2026-03-01T10:00:00Z".into(),
        };
        let cloned = ancestor.clone();
        assert_eq!(cloned.tweet_id, "tw1");
        assert_eq!(cloned.content_preview, "Test");
        assert_eq!(cloned.content_type, "tweet");
        assert_eq!(cloned.archetype_vibe, "tip");
        assert!((cloned.engagement_score - 0.9).abs() < 0.001);
        assert!((cloned.retrieval_weight - 0.85).abs() < 0.001);
        let debug = format!("{ancestor:?}");
        assert!(debug.contains("tw1"));
        assert!(debug.contains("tip"));
    }

    #[test]
    fn draft_context_debug_and_clone() {
        let ctx = DraftContext {
            winning_ancestors: vec![],
            content_seeds: vec![],
            vault_citations: vec![],
            prompt_block: "test prompt".into(),
        };
        let cloned = ctx.clone();
        assert!(cloned.winning_ancestors.is_empty());
        assert!(cloned.content_seeds.is_empty());
        assert!(cloned.vault_citations.is_empty());
        assert_eq!(cloned.prompt_block, "test prompt");
        let debug = format!("{ctx:?}");
        assert!(debug.contains("test prompt"));
    }

    #[test]
    fn content_seed_context_debug_and_clone() {
        let seed = ContentSeedContext {
            seed_text: "Hook text".into(),
            source_title: Some("Notes".into()),
            archetype_suggestion: Some("tip".into()),
            engagement_weight: 0.75,
        };
        let cloned = seed.clone();
        assert_eq!(cloned.seed_text, "Hook text");
        assert_eq!(cloned.source_title.as_deref(), Some("Notes"));
        assert_eq!(cloned.archetype_suggestion.as_deref(), Some("tip"));
        assert!((cloned.engagement_weight - 0.75).abs() < 0.001);
        let debug = format!("{seed:?}");
        assert!(debug.contains("Hook text"));
    }

    // --- Constants tests ---

    #[test]
    fn constants_have_expected_values() {
        assert!((RECENCY_HALF_LIFE_DAYS - 14.0).abs() < 0.001);
        assert_eq!(MAX_ANCESTORS, 5);
        assert!((COLD_START_WEIGHT - 0.5).abs() < 0.001);
        assert!((MIN_ENGAGEMENT_SCORE - 0.1).abs() < 0.001);
        assert_eq!(RAG_MAX_CHARS, 2000);
        assert_eq!(MAX_ANCESTOR_CHARS, 800);
        assert_eq!(MAX_COLD_START_SEEDS, 5);
    }

    #[tokio::test]
    async fn fragment_citations_populated_correctly() {
        let pool = crate::storage::init_test_db().await.expect("init db");

        let source_id = watchtower::insert_source_context(&pool, "local_fs", "{}")
            .await
            .expect("insert source");
        watchtower::upsert_content_node(
            &pool,
            source_id,
            "notes/deep-work.md",
            "h-cite",
            Some("Deep Work"),
            "Focus on cognitively demanding tasks",
            None,
            None,
        )
        .await
        .expect("upsert node");
        watchtower::insert_chunk(
            &pool,
            TEST_ACCOUNT,
            1,
            "# Productivity > ## Deep Work",
            "Focus on cognitively demanding tasks without distraction",
            "hash-cite-1",
            0,
        )
        .await
        .expect("insert chunk");

        let ctx = build_draft_context(&pool, TEST_ACCOUNT, &["focus".to_string()], 5, 14.0)
            .await
            .expect("build context");

        assert_eq!(ctx.vault_citations.len(), 1);
        let cite = &ctx.vault_citations[0];
        assert_eq!(cite.source_path, "notes/deep-work.md");
        assert_eq!(cite.source_title.as_deref(), Some("Deep Work"));
        assert_eq!(cite.heading_path, "# Productivity > ## Deep Work");
        assert!(cite.snippet.contains("cognitively demanding"));
    }
}
