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
