//! Additional edge case tests for classification, scoring, and helper functions.
//! All deterministic — no DB required.

use crate::context::winning_dna::analysis::{
    classify_for_row, classify_reply_archetype, classify_tweet_format, compute_days_since,
    truncate_at_char_boundary,
};
use crate::context::winning_dna::scoring::{compute_engagement_score, compute_retrieval_weight};


// ============================================================================
// classify_reply_archetype edge cases
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

// ============================================================================
// classify_tweet_format edge cases
// ============================================================================

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

// ============================================================================
// compute_engagement_score edge cases
// ============================================================================

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

// ============================================================================
// compute_retrieval_weight edge cases
// ============================================================================

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

// ============================================================================
// compute_days_since edge cases
// ============================================================================

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

// ============================================================================
// classify_for_row edge cases
// ============================================================================

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

// ============================================================================
// truncate_at_char_boundary edge cases
// ============================================================================

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
