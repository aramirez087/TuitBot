//! Benchmarks for content utilities: tweet length validation and truncation.
//!
//! These functions run on every draft before it is accepted or retried, so
//! their latency contributes to the overall generation loop.  All are pure
//! and require no I/O.
//!
//! Run locally:
//!   cargo bench -p tuitbot-core --bench content_bench
//!
//! Criterion HTML report: target/criterion/

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use tuitbot_core::content::{
    truncate_at_sentence, tweet_weighted_len, validate_tweet_length, MAX_TWEET_CHARS,
};

// ---------------------------------------------------------------------------
// tweet_weighted_len
// ---------------------------------------------------------------------------

fn bench_tweet_weighted_len(c: &mut Criterion) {
    let cases: &[(&str, &str)] = &[
        ("short_ascii",   "Hello world!"),
        ("medium_ascii",  "Building a SaaS product with Rust — indie hacker life. Day 42 of shipping in public. Small wins today."),
        ("at_limit",      &"x".repeat(MAX_TWEET_CHARS)),
        ("with_url",      "Check out this article https://example.com/some/really/long/path/here — pretty interesting stuff about Rust."),
        ("with_emoji",    "Shipping every day 🚀🦀✨ — the indie hacker way. Count: 1️⃣2️⃣3️⃣"),
        ("unicode_cjk",   "今日はRustでプロダクトを作っています。毎日コードを書くことが大切です。"),
    ];

    let mut group = c.benchmark_group("tweet_weighted_len");
    for &(label, text) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(label), &text, |b, t| {
            b.iter(|| tweet_weighted_len(black_box(t)));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// validate_tweet_length
// ---------------------------------------------------------------------------

fn bench_validate_tweet_length(c: &mut Criterion) {
    let cases: &[(&str, &str)] = &[
        ("well_under", "Short tweet."),
        ("near_limit", &"a".repeat(270)),
        ("exactly_280", &"a".repeat(280)),
        ("over_limit", &"a".repeat(300)),
    ];

    let mut group = c.benchmark_group("validate_tweet_length");
    for &(label, text) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(label), &text, |b, t| {
            b.iter(|| validate_tweet_length(black_box(t), black_box(MAX_TWEET_CHARS)));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// truncate_at_sentence
// ---------------------------------------------------------------------------

fn bench_truncate_at_sentence(c: &mut Criterion) {
    // A tweet that is slightly over the limit and needs trimming.
    let over_limit = "This is the first sentence, which is quite informative. \
        This second sentence pushes it over the limit and must be dropped entirely. \
        And there might even be a third sentence here for good measure.";

    // A tweet that is already under — no work needed.
    let under_limit = "Short and sweet. Fits fine.";

    // A tweet with no sentence boundary — must hard-truncate.
    let no_boundary = &"a".repeat(320);

    let mut group = c.benchmark_group("truncate_at_sentence");
    group.bench_function("over_limit", |b| {
        b.iter(|| truncate_at_sentence(black_box(over_limit), black_box(MAX_TWEET_CHARS)));
    });
    group.bench_function("under_limit", |b| {
        b.iter(|| truncate_at_sentence(black_box(under_limit), black_box(MAX_TWEET_CHARS)));
    });
    group.bench_function("no_sentence_boundary", |b| {
        b.iter(|| {
            truncate_at_sentence(black_box(no_boundary.as_str()), black_box(MAX_TWEET_CHARS))
        });
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

criterion_group!(
    content_benches,
    bench_tweet_weighted_len,
    bench_validate_tweet_length,
    bench_truncate_at_sentence,
);
criterion_main!(content_benches);
