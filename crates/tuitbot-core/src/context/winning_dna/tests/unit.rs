//! Core unit tests: classification, scoring, helpers, and prompt formatting.
//! All deterministic — no DB required.

use crate::context::winning_dna::analysis::{
    classify_for_row, classify_reply_archetype, classify_tweet_format, combine_prompt_blocks,
    compute_days_since, format_ancestors_prompt, format_seeds_prompt, truncate_at_char_boundary,
};
use crate::context::winning_dna::scoring::{compute_engagement_score, compute_retrieval_weight};
use crate::context::winning_dna::{
    ContentSeedContext, WinningAncestor, COLD_START_WEIGHT, RAG_MAX_CHARS,
};

// ============================================================================
// Classification tests (deterministic, no DB)
// ============================================================================

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

// ============================================================================
// Scoring tests (deterministic, no DB)
// ============================================================================

#[test]
fn engagement_score_normalization() {
    assert!((compute_engagement_score(50.0, 100.0) - 0.5).abs() < 0.001);
    assert!((compute_engagement_score(100.0, 100.0) - 1.0).abs() < 0.001);
    assert!((compute_engagement_score(0.0, 100.0) - 0.0).abs() < 0.001);
}

#[test]
fn engagement_score_zero_max() {
    assert!((compute_engagement_score(50.0, 0.0) - COLD_START_WEIGHT).abs() < 0.001);
}

#[test]
fn engagement_score_clamps() {
    assert!((compute_engagement_score(200.0, 100.0) - 1.0).abs() < 0.001);
}

#[test]
fn retrieval_weight_no_decay() {
    let w = compute_retrieval_weight(0.8, 0.0, 14.0);
    assert!((w - 0.8).abs() < 0.001);
}

#[test]
fn retrieval_weight_half_life() {
    let w = compute_retrieval_weight(1.0, 14.0, 14.0);
    assert!((w - 0.5).abs() < 0.05, "got {w}");
}

#[test]
fn retrieval_weight_old_content() {
    let w = compute_retrieval_weight(1.0, 56.0, 14.0);
    assert!(w < 0.1, "got {w}");
}

#[test]
fn retrieval_weight_zero_half_life() {
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
    assert!((compute_engagement_score(50.0, -10.0) - COLD_START_WEIGHT).abs() < 0.001);
}

// ============================================================================
// classify_for_row
// ============================================================================

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

// ============================================================================
// compute_days_since
// ============================================================================

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

// ============================================================================
// truncate_at_char_boundary
// ============================================================================

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

// ============================================================================
// combine_prompt_blocks
// ============================================================================

#[test]
fn combine_prompt_blocks_under_limit() {
    let a = "short ancestor";
    let f = "short fragment";
    let result = combine_prompt_blocks(a, f);
    assert_eq!(result, "short ancestorshort fragment");
}

// ============================================================================
// format_seeds_prompt
// ============================================================================

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

// ============================================================================
// Prompt formatting tests
// ============================================================================

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
