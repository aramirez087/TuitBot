//! Tests for struct debug/clone behaviour, prompt formatting edge cases,
//! and constants. All deterministic — no DB required.

use crate::context::winning_dna::analysis::{
    combine_prompt_blocks, format_ancestors_prompt, format_ancestors_prompt_capped,
    format_seeds_prompt,
};
use crate::context::winning_dna::{
    ContentSeedContext, DraftContext, WinningAncestor, COLD_START_WEIGHT, MAX_ANCESTORS,
    MAX_ANCESTOR_CHARS, MAX_COLD_START_SEEDS, MIN_ENGAGEMENT_SCORE, RAG_MAX_CHARS,
    RECENCY_HALF_LIFE_DAYS,
};

// ============================================================================
// format_ancestors_prompt edge cases
// ============================================================================

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

// ============================================================================
// format_ancestors_prompt_capped
// ============================================================================

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

// ============================================================================
// format_seeds_prompt edge cases
// ============================================================================

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

// ============================================================================
// combine_prompt_blocks edge cases
// ============================================================================

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

// ============================================================================
// Struct debug and clone
// ============================================================================

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

// ============================================================================
// Constants
// ============================================================================

#[test]
fn constants_have_expected_values() {
    assert!((RECENCY_HALF_LIFE_DAYS - 14.0).abs() < 0.001);
    assert_eq!(MAX_ANCESTORS, 5);
    assert!((COLD_START_WEIGHT - 0.5).abs() < 0.001);
    assert!((MIN_ENGAGEMENT_SCORE - 0.1).abs() < 0.001);
    assert_eq!(RAG_MAX_CHARS, 4000);
    assert_eq!(MAX_ANCESTOR_CHARS, 800);
    assert_eq!(MAX_COLD_START_SEEDS, 5);
}
