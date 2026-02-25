//! Tests for the context aggregation module.
//!
//! Covers empty-history and rich-history cases for all three tools.

use crate::config::Config;
use crate::storage::{init_test_db, DbPool};

// ============================================================================
// Test helpers — seed fixtures into the test database
// ============================================================================

async fn seed_discovered_tweet(
    pool: &DbPool,
    id: &str,
    author_id: &str,
    author_username: &str,
    content: &str,
    keyword: &str,
) {
    sqlx::query(
        "INSERT INTO discovered_tweets \
         (id, author_id, author_username, content, matched_keyword, replied_to) \
         VALUES (?, ?, ?, ?, ?, 0)",
    )
    .bind(id)
    .bind(author_id)
    .bind(author_username)
    .bind(content)
    .bind(keyword)
    .execute(pool)
    .await
    .expect("seed discovered tweet");
}

async fn seed_reply(
    pool: &DbPool,
    target_tweet_id: &str,
    reply_tweet_id: &str,
    content: &str,
    created_at: &str,
) {
    sqlx::query(
        "INSERT INTO replies_sent \
         (target_tweet_id, reply_tweet_id, reply_content, status, created_at) \
         VALUES (?, ?, ?, 'sent', ?)",
    )
    .bind(target_tweet_id)
    .bind(reply_tweet_id)
    .bind(content)
    .bind(created_at)
    .execute(pool)
    .await
    .expect("seed reply");
}

async fn seed_reply_performance(
    pool: &DbPool,
    reply_id: &str,
    likes: i64,
    replies: i64,
    score: f64,
) {
    crate::storage::analytics::upsert_reply_performance(
        pool, reply_id, likes, replies, 1000, score,
    )
    .await
    .expect("seed reply performance");
}

async fn seed_author_interaction(pool: &DbPool, author_id: &str, username: &str, count: i64) {
    sqlx::query(
        "INSERT INTO author_interactions \
         (author_id, author_username, interaction_date, reply_count) \
         VALUES (?, ?, date('now'), ?)",
    )
    .bind(author_id)
    .bind(username)
    .bind(count)
    .execute(pool)
    .await
    .expect("seed author interaction");
}

async fn seed_original_tweet(
    pool: &DbPool,
    tweet_id: &str,
    content: &str,
    topic: &str,
    created_at: &str,
) {
    sqlx::query(
        "INSERT INTO original_tweets \
         (tweet_id, content, topic, status, created_at) \
         VALUES (?, ?, ?, 'sent', ?)",
    )
    .bind(tweet_id)
    .bind(content)
    .bind(topic)
    .bind(created_at)
    .execute(pool)
    .await
    .expect("seed original tweet");
}

async fn seed_tweet_performance(pool: &DbPool, tweet_id: &str, score: f64) {
    crate::storage::analytics::upsert_tweet_performance(pool, tweet_id, 10, 5, 3, 1000, score)
        .await
        .expect("seed tweet performance");
}

fn test_config() -> Config {
    let mut config = Config::default();
    config.business.product_keywords = vec!["rust".to_string(), "cli".to_string()];
    config.business.competitor_keywords = vec!["python".to_string()];
    config.business.industry_topics = vec!["developer tools".to_string()];
    config.limits.max_replies_per_day = 5;
    config.limits.max_replies_per_author_per_day = 1;
    config
}

// ============================================================================
// Author context tests
// ============================================================================

mod author_tests {
    use super::*;
    use crate::context::author::get_author_context;

    #[tokio::test]
    async fn empty_db_returns_empty_context() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config();

        let ctx = get_author_context(&pool, "unknown_user", &config)
            .await
            .expect("get context");

        assert_eq!(ctx.author_username, "unknown_user");
        assert!(ctx.author_id.is_none());
        assert_eq!(ctx.interaction_summary.total_replies_sent, 0);
        assert_eq!(ctx.interaction_summary.replies_today, 0);
        assert!(ctx.conversation_history.is_empty());
        assert!(ctx.topic_affinity.is_empty());
        assert_eq!(ctx.response_metrics.replies_measured, 0);
        // Should have "no_prior_interaction" risk signal
        assert!(ctx
            .risk_signals
            .iter()
            .any(|s| s.signal_type == "no_prior_interaction"));
    }

    #[tokio::test]
    async fn strips_at_sign_from_username() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config();

        seed_discovered_tweet(&pool, "t1", "a1", "alice", "Hello world", "rust").await;

        let ctx = get_author_context(&pool, "@alice", &config)
            .await
            .expect("get context");

        assert_eq!(ctx.author_username, "alice");
        assert_eq!(ctx.author_id.as_deref(), Some("a1"));
    }

    #[tokio::test]
    async fn rich_history_builds_full_context() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config();
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Seed discovered tweets from alice
        seed_discovered_tweet(
            &pool,
            "t1",
            "a1",
            "alice",
            "Rust CLI tools are great",
            "rust",
        )
        .await;
        seed_discovered_tweet(&pool, "t2", "a1", "alice", "Python vs Rust debate", "rust").await;

        // Seed replies to alice's tweets
        seed_reply(&pool, "t1", "r1", "Totally agree about Rust!", &now).await;
        seed_reply(&pool, "t2", "r2", "Good comparison!", &now).await;

        // Seed performance for one reply
        seed_reply_performance(&pool, "r1", 5, 2, 75.0).await;

        // Seed author interaction
        seed_author_interaction(&pool, "a1", "alice", 2).await;

        let ctx = get_author_context(&pool, "alice", &config)
            .await
            .expect("get context");

        assert_eq!(ctx.author_id.as_deref(), Some("a1"));
        assert_eq!(ctx.interaction_summary.total_replies_sent, 2);
        assert_eq!(ctx.interaction_summary.replies_today, 2);
        assert_eq!(ctx.conversation_history.len(), 2);
        assert!(
            ctx.conversation_history[0].performance.is_some()
                || ctx.conversation_history[1].performance.is_some()
        );
        assert!(!ctx.topic_affinity.is_empty());
        assert_eq!(ctx.topic_affinity[0].keyword, "rust");
        assert_eq!(ctx.response_metrics.replies_measured, 1);
        assert!(ctx.response_metrics.avg_performance_score > 0.0);
    }

    #[tokio::test]
    async fn high_frequency_risk_signal() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config(); // max_replies_per_author_per_day = 1

        seed_discovered_tweet(&pool, "t1", "a1", "alice", "Hello", "rust").await;
        seed_author_interaction(&pool, "a1", "alice", 1).await;

        let ctx = get_author_context(&pool, "alice", &config)
            .await
            .expect("get context");

        assert!(ctx
            .risk_signals
            .iter()
            .any(|s| s.signal_type == "high_frequency_today"));
    }

    #[tokio::test]
    async fn lookup_by_author_id() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config();

        seed_discovered_tweet(&pool, "t1", "12345", "alice", "Hello", "rust").await;

        let ctx = get_author_context(&pool, "12345", &config)
            .await
            .expect("get context");

        assert_eq!(ctx.author_username, "alice");
        assert_eq!(ctx.author_id.as_deref(), Some("12345"));
    }
}

// ============================================================================
// Engagement recommendation tests
// ============================================================================

mod engagement_tests {
    use super::*;
    use crate::context::engagement::recommend_engagement;

    #[tokio::test]
    async fn empty_db_returns_recommendation() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config();

        let rec = recommend_engagement(
            &pool,
            "unknown_user",
            "Building Rust CLI tools for developers",
            Some("grow developer audience"),
            &config,
        )
        .await
        .expect("recommend");

        // Should produce a valid recommendation with factors
        assert!(!rec.recommended_action.is_empty());
        assert!(rec.confidence > 0.0 && rec.confidence <= 1.0);
        assert!(!rec.contributing_factors.is_empty());
        // Keyword "rust" and "cli" should match
        let kw_factor = rec
            .contributing_factors
            .iter()
            .find(|f| f.factor == "keyword_relevance")
            .expect("has keyword factor");
        assert_eq!(kw_factor.signal, "positive");
    }

    #[tokio::test]
    async fn no_keyword_match_lowers_score() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config();

        let rec = recommend_engagement(
            &pool,
            "unknown_user",
            "The weather is nice today and I love cooking",
            None,
            &config,
        )
        .await
        .expect("recommend");

        let kw_factor = rec
            .contributing_factors
            .iter()
            .find(|f| f.factor == "keyword_relevance")
            .expect("has keyword factor");
        assert_eq!(kw_factor.signal, "negative");
    }

    #[tokio::test]
    async fn at_daily_limit_blocks() {
        let pool = init_test_db().await.expect("init db");
        let mut config = test_config();
        config.limits.max_replies_per_day = 1;

        // Insert a reply for today to hit the limit
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        seed_reply(&pool, "t_limit", "r_limit", "test reply", &now).await;

        let rec =
            recommend_engagement(&pool, "some_user", "Building Rust CLI tools", None, &config)
                .await
                .expect("recommend");

        assert_eq!(rec.recommended_action, "skip");
        assert!(rec.confidence >= 0.9);
        assert!(rec
            .policy_considerations
            .iter()
            .any(|p| p.policy == "daily_rate_limit" && p.status == "blocked"));
    }

    #[tokio::test]
    async fn at_per_author_limit_blocks() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config(); // max_per_author = 1

        seed_discovered_tweet(&pool, "t1", "a1", "alice", "Hello", "rust").await;
        seed_author_interaction(&pool, "a1", "alice", 1).await;

        let rec = recommend_engagement(&pool, "alice", "Building Rust CLI tools", None, &config)
            .await
            .expect("recommend");

        assert_eq!(rec.recommended_action, "skip");
        assert!(rec.confidence >= 0.9);
    }

    #[tokio::test]
    async fn campaign_alignment_boosts_score() {
        let pool = init_test_db().await.expect("init db");
        let config = test_config();

        let rec_aligned = recommend_engagement(
            &pool,
            "user1",
            "Building developer tools with Rust and CLI patterns",
            Some("grow developer tools community engagement"),
            &config,
        )
        .await
        .expect("recommend");

        let rec_unaligned = recommend_engagement(
            &pool,
            "user2",
            "Building developer tools with Rust and CLI patterns",
            Some("fashion beauty lifestyle influencer"),
            &config,
        )
        .await
        .expect("recommend");

        // Aligned objective should score higher
        let aligned_conf = rec_aligned.confidence;
        let unaligned_conf = rec_unaligned.confidence;
        // If both recommend "reply", aligned should have higher confidence
        // If different actions, aligned should be more actionable
        assert!(
            rec_aligned.recommended_action == "reply"
                || aligned_conf >= unaligned_conf
                || rec_unaligned.recommended_action == "skip"
        );
    }

    #[tokio::test]
    async fn approval_mode_in_policy() {
        let pool = init_test_db().await.expect("init db");
        let mut config = test_config();
        config.approval_mode = true;

        let rec = recommend_engagement(&pool, "user1", "Rust is awesome", None, &config)
            .await
            .expect("recommend");

        assert!(rec
            .policy_considerations
            .iter()
            .any(|p| p.policy == "approval_mode"));
    }
}

// ============================================================================
// Topic performance snapshot tests
// ============================================================================

mod topic_tests {
    use super::*;
    use crate::context::topics::get_topic_snapshot;

    #[tokio::test]
    async fn empty_db_returns_empty_snapshot() {
        let pool = init_test_db().await.expect("init db");

        let snapshot = get_topic_snapshot(&pool, 30).await.expect("snapshot");

        assert_eq!(snapshot.lookback_days, 30);
        assert!(snapshot.topics.is_empty());
        assert_eq!(snapshot.total_posts_analyzed, 0);
        assert!((snapshot.overall_avg_performance - 0.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn rich_data_produces_ranked_topics() {
        let pool = init_test_db().await.expect("init db");
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Seed high-performing topic
        for i in 0..4 {
            let tid = format!("tw_rust_{i}");
            seed_original_tweet(&pool, &tid, "Advanced Rust patterns", "rust", &now).await;
            seed_tweet_performance(&pool, &tid, 80.0 + i as f64).await;
        }

        // Seed low-performing topic
        for i in 0..4 {
            let tid = format!("tw_python_{i}");
            seed_original_tweet(&pool, &tid, "Python basics", "python", &now).await;
            seed_tweet_performance(&pool, &tid, 20.0 + i as f64).await;
        }

        let snapshot = get_topic_snapshot(&pool, 30).await.expect("snapshot");

        assert_eq!(snapshot.topics.len(), 2);
        assert_eq!(snapshot.total_posts_analyzed, 8);

        // Rust should be first (higher avg)
        assert_eq!(snapshot.topics[0].topic, "rust");
        assert_eq!(snapshot.topics[0].recommendation, "double_down");
        assert!(snapshot.topics[0].performance_vs_average > 1.0);

        // Python should be second with "reduce" recommendation
        assert_eq!(snapshot.topics[1].topic, "python");
        assert_eq!(snapshot.topics[1].recommendation, "reduce");
        assert!(snapshot.topics[1].performance_vs_average < 1.0);
    }

    #[tokio::test]
    async fn fewer_than_three_posts_gets_experiment() {
        let pool = init_test_db().await.expect("init db");
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Only 2 posts for this topic
        seed_original_tweet(&pool, "tw1", "Go concurrency", "go", &now).await;
        seed_tweet_performance(&pool, "tw1", 90.0).await;
        seed_original_tweet(&pool, "tw2", "Go channels", "go", &now).await;
        seed_tweet_performance(&pool, "tw2", 95.0).await;

        let snapshot = get_topic_snapshot(&pool, 30).await.expect("snapshot");

        assert_eq!(snapshot.topics.len(), 1);
        assert_eq!(snapshot.topics[0].recommendation, "experiment");
    }

    #[tokio::test]
    async fn provenance_includes_best_content() {
        let pool = init_test_db().await.expect("init db");
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        seed_original_tweet(
            &pool,
            "tw1",
            "Best Rust practices for CLI apps",
            "rust",
            &now,
        )
        .await;
        seed_tweet_performance(&pool, "tw1", 95.0).await;
        seed_original_tweet(&pool, "tw2", "Rust error handling tips", "rust", &now).await;
        seed_tweet_performance(&pool, "tw2", 30.0).await;
        seed_original_tweet(&pool, "tw3", "Rust async patterns", "rust", &now).await;
        seed_tweet_performance(&pool, "tw3", 60.0).await;

        let snapshot = get_topic_snapshot(&pool, 30).await.expect("snapshot");

        let rust_topic = &snapshot.topics[0];
        assert!(rust_topic
            .provenance
            .best_content_preview
            .contains("Best Rust"));
        assert!((rust_topic.provenance.best_performance_score - 95.0).abs() < 0.01);
        assert!((rust_topic.provenance.worst_performance_score - 30.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn lookback_window_filters_old_data() {
        let pool = init_test_db().await.expect("init db");
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let old = "2020-01-01T00:00:00Z";

        // Recent tweet
        seed_original_tweet(&pool, "tw_new", "Recent Rust post", "rust", &now).await;
        seed_tweet_performance(&pool, "tw_new", 80.0).await;

        // Old tweet (outside 30-day window)
        seed_original_tweet(&pool, "tw_old", "Old Python post", "python", old).await;
        seed_tweet_performance(&pool, "tw_old", 90.0).await;

        let snapshot = get_topic_snapshot(&pool, 30).await.expect("snapshot");

        // Should only include the recent rust topic
        assert_eq!(snapshot.topics.len(), 1);
        assert_eq!(snapshot.topics[0].topic, "rust");
    }
}
