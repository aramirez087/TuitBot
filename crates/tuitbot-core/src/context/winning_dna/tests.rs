//! Tests for winning_dna: classification, scoring, formatting, retrieval.
//!
//! Unit tests (deterministic, no DB) come first; async integration tests follow.

use super::analysis::{
    classify_for_row, classify_reply_archetype, classify_tweet_format, combine_prompt_blocks,
    compute_days_since, format_ancestors_prompt, format_ancestors_prompt_capped,
    format_seeds_prompt, truncate_at_char_boundary,
};
use super::scoring::{compute_engagement_score, compute_retrieval_weight};
use super::{
    build_draft_context, retrieve_ancestors, ContentSeedContext, DraftContext, WinningAncestor,
    COLD_START_WEIGHT, MAX_ANCESTORS, MAX_ANCESTOR_CHARS, MAX_COLD_START_SEEDS,
    MIN_ENGAGEMENT_SCORE, RAG_MAX_CHARS, RECENCY_HALF_LIFE_DAYS,
};
use crate::storage::{analytics, watchtower};

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

// ============================================================================
// Additional edge case tests for coverage push
// ============================================================================

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

// ============================================================================
// DB integration tests
// ============================================================================

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

    let ancestors = retrieve_ancestors(&pool, "account-b", &[], 5, 14.0)
        .await
        .expect("retrieve");
    assert!(
        ancestors.is_empty(),
        "account B should see no ancestors from account A"
    );

    let ancestors = retrieve_ancestors(&pool, "account-a", &[], 5, 14.0)
        .await
        .expect("retrieve");
    assert_eq!(ancestors.len(), 1);
}

#[tokio::test]
async fn build_draft_context_with_fragments() {
    let pool = crate::storage::init_test_db().await.expect("init db");

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
