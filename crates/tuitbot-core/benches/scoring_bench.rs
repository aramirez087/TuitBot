//! Benchmarks for the tweet scoring engine.
//!
//! These cover the hot path that runs on every discovered tweet during the
//! discovery loop.  All functions are pure (no I/O, no async) so they make
//! clean benchmark targets without mock setup overhead.
//!
//! Run locally:
//!   cargo bench -p tuitbot-core --bench scoring_bench
//!
//! Criterion HTML report: target/criterion/

use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use tuitbot_core::{
    config::ScoringConfig,
    scoring::{
        signals::{
            content_type_score, engagement_rate, follower_score, keyword_relevance,
            recency_score_at, reply_count_score,
        },
        ScoringEngine, TweetData,
    },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_engine(keywords: Vec<String>) -> ScoringEngine {
    ScoringEngine::new(ScoringConfig::default(), keywords)
}

fn make_tweet(text: &str, followers: u64, likes: u64, retweets: u64, replies: u64) -> TweetData {
    let ts = (Utc::now() - Duration::minutes(5)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    TweetData {
        text: text.to_string(),
        created_at: ts,
        likes,
        retweets,
        replies,
        author_username: "bench_user".to_string(),
        author_followers: followers,
        has_media: false,
        is_quote_tweet: false,
    }
}

// ---------------------------------------------------------------------------
// Individual signal benchmarks
// ---------------------------------------------------------------------------

fn bench_keyword_relevance(c: &mut Criterion) {
    let keywords_small: Vec<String> = vec!["rust".to_string(), "async".to_string()];
    let keywords_large: Vec<String> = (0..50)
        .map(|i| format!("keyword_{i}"))
        .chain(["rust programming".to_string(), "async tokio".to_string()])
        .collect();

    let tweet_text = "Building async Rust services with Tokio is a great developer experience.";

    let mut group = c.benchmark_group("keyword_relevance");
    group.bench_with_input(
        BenchmarkId::new("small_list", 2),
        &keywords_small,
        |b, kw| {
            b.iter(|| keyword_relevance(black_box(tweet_text), black_box(kw), black_box(25.0)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("large_list", 52),
        &keywords_large,
        |b, kw| {
            b.iter(|| keyword_relevance(black_box(tweet_text), black_box(kw), black_box(25.0)));
        },
    );
    group.finish();
}

fn bench_follower_score(c: &mut Criterion) {
    let mut group = c.benchmark_group("follower_score");
    for &count in &[0u64, 100, 1_000, 10_000, 100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &fc| {
            b.iter(|| follower_score(black_box(fc), black_box(15.0)));
        });
    }
    group.finish();
}

fn bench_recency_score(c: &mut Criterion) {
    let now = Utc::now();
    let ages_min: &[i64] = &[1, 10, 30, 60, 180, 360, 720];

    let mut group = c.benchmark_group("recency_score");
    for &age in ages_min {
        let ts = (now - Duration::minutes(age)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        group.bench_with_input(BenchmarkId::new("age_min", age), &ts, |b, ts| {
            b.iter(|| recency_score_at(black_box(ts.as_str()), black_box(10.0), now));
        });
    }
    group.finish();
}

fn bench_engagement_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("engagement_rate");
    // (label, likes, retweets, replies, followers)
    let cases: &[(&str, u64, u64, u64, u64)] = &[
        ("cold", 0, 0, 0, 100),
        ("warm", 10, 2, 1, 1_000),
        ("viral", 5_000, 1_000, 200, 50_000),
    ];
    for &(label, likes, retweets, replies, followers) in cases {
        group.bench_function(label, |b| {
            b.iter(|| {
                engagement_rate(
                    black_box(likes),
                    black_box(retweets),
                    black_box(replies),
                    black_box(followers),
                    black_box(15.0),
                )
            });
        });
    }
    group.finish();
}

fn bench_reply_count_score(c: &mut Criterion) {
    c.bench_function("reply_count_score", |b| {
        b.iter(|| reply_count_score(black_box(3u64), black_box(15.0)));
    });
}

fn bench_content_type_score(c: &mut Criterion) {
    c.bench_function("content_type_score/text_only", |b| {
        b.iter(|| content_type_score(black_box(false), black_box(false), black_box(10.0)));
    });
    c.bench_function("content_type_score/with_media", |b| {
        b.iter(|| content_type_score(black_box(true), black_box(false), black_box(10.0)));
    });
}

// ---------------------------------------------------------------------------
// Full scoring engine
// ---------------------------------------------------------------------------

fn bench_score_tweet(c: &mut Criterion) {
    let keywords: Vec<String> = vec![
        "rust".to_string(),
        "async".to_string(),
        "tokio".to_string(),
        "indie hacker".to_string(),
        "SaaS founder".to_string(),
    ];
    let engine = make_engine(keywords);

    let tweets: &[(&str, TweetData)] = &[
        (
            "high_score",
            make_tweet(
                "Building a SaaS product with async Rust and Tokio — indie hacker life",
                12_000,
                80,
                15,
                2,
            ),
        ),
        ("low_score", make_tweet("Just had coffee ☕", 50, 0, 0, 0)),
        (
            "mid_score",
            make_tweet("Working on a new startup feature today", 3_000, 12, 3, 1),
        ),
    ];

    let mut group = c.benchmark_group("score_tweet");
    for (label, tweet) in tweets {
        group.bench_with_input(BenchmarkId::from_parameter(label), tweet, |b, t| {
            b.iter(|| engine.score_tweet(black_box(t)));
        });
    }
    group.finish();
}

/// Scores a batch of 20 tweets — the typical discovery loop batch size.
fn bench_score_tweet_batch(c: &mut Criterion) {
    let keywords: Vec<String> = vec![
        "rust".to_string(),
        "startup".to_string(),
        "indie hacker".to_string(),
    ];
    let engine = make_engine(keywords);

    let batch: Vec<TweetData> = (0u64..20)
        .map(|i| {
            make_tweet(
                &format!("Tweet number {i} about rust and startup ideas"),
                (i + 1) * 500,
                i * 3,
                i,
                (i / 3).max(0),
            )
        })
        .collect();

    c.bench_function("score_tweet_batch_20", |b| {
        b.iter(|| {
            for tweet in &batch {
                let _ = engine.score_tweet(black_box(tweet));
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

criterion_group!(
    signal_benches,
    bench_keyword_relevance,
    bench_follower_score,
    bench_recency_score,
    bench_engagement_rate,
    bench_reply_count_score,
    bench_content_type_score,
);
criterion_group!(engine_benches, bench_score_tweet, bench_score_tweet_batch);
criterion_main!(signal_benches, engine_benches);
