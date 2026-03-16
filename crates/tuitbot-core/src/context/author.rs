//! Author context aggregation.
//!
//! Builds a rich profile of an author from stored interaction history,
//! conversation records, performance metrics, and risk signals.

use crate::config::Config;
use crate::error::StorageError;
use crate::storage::DbPool;
use serde::Serialize;

/// Complete context profile for an author.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorContext {
    pub author_username: String,
    pub author_id: Option<String>,
    pub interaction_summary: InteractionSummary,
    pub conversation_history: Vec<ConversationRecord>,
    pub topic_affinity: Vec<TopicAffinity>,
    pub risk_signals: Vec<RiskSignal>,
    pub response_metrics: ResponseMetrics,
}

/// Summary of interaction history with an author.
#[derive(Debug, Clone, Serialize)]
pub struct InteractionSummary {
    pub total_replies_sent: i64,
    pub replies_today: i64,
    pub first_interaction: Option<String>,
    pub last_interaction: Option<String>,
    pub distinct_days_active: i64,
}

/// A single conversation record (our reply to their tweet).
#[derive(Debug, Clone, Serialize)]
pub struct ConversationRecord {
    pub tweet_id: String,
    pub tweet_content: String,
    pub reply_content: String,
    pub reply_status: String,
    pub created_at: String,
    pub performance: Option<PerformanceSnapshot>,
}

/// Performance metrics for a single reply.
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceSnapshot {
    pub likes: i64,
    pub replies_received: i64,
    pub impressions: i64,
    pub performance_score: f64,
}

/// A keyword/topic that appears in an author's tweets.
#[derive(Debug, Clone, Serialize)]
pub struct TopicAffinity {
    pub keyword: String,
    pub mention_count: i64,
}

/// A risk signal that may affect engagement decisions.
#[derive(Debug, Clone, Serialize)]
pub struct RiskSignal {
    pub signal_type: String,
    pub severity: String,
    pub description: String,
}

/// Aggregate response metrics for interactions with this author.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseMetrics {
    pub replies_with_engagement: i64,
    pub replies_measured: i64,
    pub response_rate: f64,
    pub avg_performance_score: f64,
}

type ConvRow = (String, String, String, String, String, Option<String>);
type PerfRow = (i64, i64, i64, f64);

/// Build a complete author context from stored data.
///
/// Accepts a username (with or without @) or an author ID.
pub async fn get_author_context(
    pool: &DbPool,
    identifier: &str,
    config: &Config,
) -> Result<AuthorContext, StorageError> {
    let username = identifier.trim_start_matches('@');

    // Resolve author identity from discovered_tweets
    let (author_id, author_username) = resolve_author(pool, username, identifier).await?;

    // Gather interaction summary
    let interaction_summary = query_interaction_summary(pool, &author_id, &author_username).await?;

    // Gather conversation history with performance data
    let conversation_history = query_conversation_history(pool, &author_username).await?;

    // Compute response metrics from conversation history
    let response_metrics = compute_response_metrics(&conversation_history);

    // Extract topic affinity from discovered tweets
    let topic_affinity = query_topic_affinity(pool, &author_username).await?;

    // Generate risk signals
    let risk_signals = generate_risk_signals(
        &interaction_summary,
        &response_metrics,
        config.limits.max_replies_per_author_per_day,
    );

    Ok(AuthorContext {
        author_username,
        author_id,
        interaction_summary,
        conversation_history,
        topic_affinity,
        risk_signals,
        response_metrics,
    })
}

async fn resolve_author(
    pool: &DbPool,
    username: &str,
    raw_identifier: &str,
) -> Result<(Option<String>, String), StorageError> {
    // Try by username first
    let row: Option<(String,)> =
        sqlx::query_as("SELECT author_id FROM discovered_tweets WHERE author_username = ? LIMIT 1")
            .bind(username)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    if let Some((id,)) = row {
        return Ok((Some(id), username.to_string()));
    }

    // Fall back to lookup by author_id
    let row: Option<(String,)> =
        sqlx::query_as("SELECT author_username FROM discovered_tweets WHERE author_id = ? LIMIT 1")
            .bind(raw_identifier)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    match row {
        Some((uname,)) => Ok((Some(raw_identifier.to_string()), uname)),
        None => Ok((None, username.to_string())),
    }
}

async fn query_interaction_summary(
    pool: &DbPool,
    author_id: &Option<String>,
    author_username: &str,
) -> Result<InteractionSummary, StorageError> {
    let row: Option<(i64, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT COALESCE(SUM(reply_count), 0), \
                MIN(interaction_date), \
                MAX(interaction_date), \
                COUNT(DISTINCT interaction_date) \
         FROM author_interactions \
         WHERE author_id = ? OR author_username = ?",
    )
    .bind(author_id.as_deref().unwrap_or(""))
    .bind(author_username)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let (total, first, last, distinct) = row.unwrap_or((0, None, None, 0));

    // Get today's count
    let today_row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(reply_count), 0) \
         FROM author_interactions \
         WHERE (author_id = ? OR author_username = ?) \
           AND interaction_date = date('now')",
    )
    .bind(author_id.as_deref().unwrap_or(""))
    .bind(author_username)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(InteractionSummary {
        total_replies_sent: total,
        replies_today: today_row.0,
        first_interaction: first,
        last_interaction: last,
        distinct_days_active: distinct,
    })
}

async fn query_conversation_history(
    pool: &DbPool,
    author_username: &str,
) -> Result<Vec<ConversationRecord>, StorageError> {
    let rows: Vec<ConvRow> = sqlx::query_as(
        "SELECT dt.id, SUBSTR(dt.content, 1, 200), \
                rs.reply_content, rs.status, rs.created_at, rs.reply_tweet_id \
         FROM replies_sent rs \
         JOIN discovered_tweets dt ON dt.id = rs.target_tweet_id \
         WHERE dt.author_username = ? \
         ORDER BY rs.created_at DESC \
         LIMIT 20",
    )
    .bind(author_username)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let mut records = Vec::with_capacity(rows.len());
    for (tweet_id, tweet_content, reply_content, status, created_at, reply_tweet_id) in rows {
        let performance = if let Some(ref rtid) = reply_tweet_id {
            query_reply_performance(pool, rtid).await?
        } else {
            None
        };
        records.push(ConversationRecord {
            tweet_id,
            tweet_content,
            reply_content,
            reply_status: status,
            created_at,
            performance,
        });
    }
    Ok(records)
}

async fn query_reply_performance(
    pool: &DbPool,
    reply_tweet_id: &str,
) -> Result<Option<PerformanceSnapshot>, StorageError> {
    let row: Option<PerfRow> = sqlx::query_as(
        "SELECT likes_received, replies_received, impressions, performance_score \
         FROM reply_performance WHERE reply_id = ?",
    )
    .bind(reply_tweet_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(
        row.map(|(likes, replies, impressions, score)| PerformanceSnapshot {
            likes,
            replies_received: replies,
            impressions,
            performance_score: score,
        }),
    )
}

fn compute_response_metrics(history: &[ConversationRecord]) -> ResponseMetrics {
    let measured = history.iter().filter(|c| c.performance.is_some()).count() as i64;
    let with_engagement = history
        .iter()
        .filter(|c| {
            c.performance
                .as_ref()
                .is_some_and(|p| p.likes > 0 || p.replies_received > 0)
        })
        .count() as i64;
    let avg_score = if measured > 0 {
        history
            .iter()
            .filter_map(|c| c.performance.as_ref())
            .map(|p| p.performance_score)
            .sum::<f64>()
            / measured as f64
    } else {
        0.0
    };
    let rate = if measured > 0 {
        with_engagement as f64 / measured as f64
    } else {
        0.0
    };

    ResponseMetrics {
        replies_with_engagement: with_engagement,
        replies_measured: measured,
        response_rate: rate,
        avg_performance_score: avg_score,
    }
}

async fn query_topic_affinity(
    pool: &DbPool,
    author_username: &str,
) -> Result<Vec<TopicAffinity>, StorageError> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT matched_keyword, COUNT(*) as cnt \
         FROM discovered_tweets \
         WHERE author_username = ? \
           AND matched_keyword IS NOT NULL AND matched_keyword != '' \
         GROUP BY matched_keyword \
         ORDER BY cnt DESC \
         LIMIT 10",
    )
    .bind(author_username)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|(keyword, count)| TopicAffinity {
            keyword,
            mention_count: count,
        })
        .collect())
}

/// Generate risk signals based on interaction and response data.
fn generate_risk_signals(
    summary: &InteractionSummary,
    metrics: &ResponseMetrics,
    max_per_author_per_day: u32,
) -> Vec<RiskSignal> {
    let mut signals = Vec::new();

    if summary.replies_today >= max_per_author_per_day as i64 {
        signals.push(RiskSignal {
            signal_type: "high_frequency_today".to_string(),
            severity: "high".to_string(),
            description: format!(
                "Already sent {} replies today (limit: {})",
                summary.replies_today, max_per_author_per_day
            ),
        });
    }

    if metrics.replies_measured >= 3 && metrics.response_rate < 0.1 {
        signals.push(RiskSignal {
            signal_type: "low_response_rate".to_string(),
            severity: "medium".to_string(),
            description: format!(
                "Only {:.0}% of replies to this author received engagement ({}/{})",
                metrics.response_rate * 100.0,
                metrics.replies_with_engagement,
                metrics.replies_measured
            ),
        });
    }

    if summary.total_replies_sent > 0 && metrics.replies_measured == 0 {
        signals.push(RiskSignal {
            signal_type: "no_measured_performance".to_string(),
            severity: "low".to_string(),
            description: "Replied before but no performance data collected yet".to_string(),
        });
    }

    if summary.total_replies_sent == 0 {
        signals.push(RiskSignal {
            signal_type: "no_prior_interaction".to_string(),
            severity: "low".to_string(),
            description: "No prior interaction history with this author".to_string(),
        });
    }

    signals
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    fn empty_summary() -> InteractionSummary {
        InteractionSummary {
            total_replies_sent: 0,
            replies_today: 0,
            first_interaction: None,
            last_interaction: None,
            distinct_days_active: 0,
        }
    }

    fn active_summary() -> InteractionSummary {
        InteractionSummary {
            total_replies_sent: 10,
            replies_today: 3,
            first_interaction: Some("2026-01-01T00:00:00Z".to_string()),
            last_interaction: Some("2026-03-15T00:00:00Z".to_string()),
            distinct_days_active: 8,
        }
    }

    fn zero_metrics() -> ResponseMetrics {
        ResponseMetrics {
            replies_with_engagement: 0,
            replies_measured: 0,
            response_rate: 0.0,
            avg_performance_score: 0.0,
        }
    }

    fn low_engagement_metrics() -> ResponseMetrics {
        ResponseMetrics {
            replies_with_engagement: 0,
            replies_measured: 5,
            response_rate: 0.0,
            avg_performance_score: 10.0,
        }
    }

    fn good_metrics() -> ResponseMetrics {
        ResponseMetrics {
            replies_with_engagement: 4,
            replies_measured: 5,
            response_rate: 0.8,
            avg_performance_score: 75.0,
        }
    }

    // ============================================================
    // compute_response_metrics tests
    // ============================================================

    #[test]
    fn compute_metrics_empty_history() {
        let metrics = compute_response_metrics(&[]);
        assert_eq!(metrics.replies_measured, 0);
        assert_eq!(metrics.replies_with_engagement, 0);
        assert_eq!(metrics.response_rate, 0.0);
        assert_eq!(metrics.avg_performance_score, 0.0);
    }

    #[test]
    fn compute_metrics_no_performance_data() {
        let history = vec![ConversationRecord {
            tweet_id: "t1".to_string(),
            tweet_content: "Hello".to_string(),
            reply_content: "Hi there!".to_string(),
            reply_status: "sent".to_string(),
            created_at: "2026-01-01T10:00:00Z".to_string(),
            performance: None,
        }];
        let metrics = compute_response_metrics(&history);
        assert_eq!(metrics.replies_measured, 0);
        assert_eq!(metrics.avg_performance_score, 0.0);
    }

    #[test]
    fn compute_metrics_with_performance() {
        let history = vec![
            ConversationRecord {
                tweet_id: "t1".to_string(),
                tweet_content: "Hello".to_string(),
                reply_content: "Hi!".to_string(),
                reply_status: "sent".to_string(),
                created_at: "2026-01-01T10:00:00Z".to_string(),
                performance: Some(PerformanceSnapshot {
                    likes: 5,
                    replies_received: 2,
                    impressions: 500,
                    performance_score: 80.0,
                }),
            },
            ConversationRecord {
                tweet_id: "t2".to_string(),
                tweet_content: "World".to_string(),
                reply_content: "Hey!".to_string(),
                reply_status: "sent".to_string(),
                created_at: "2026-01-02T10:00:00Z".to_string(),
                performance: Some(PerformanceSnapshot {
                    likes: 0,
                    replies_received: 0,
                    impressions: 100,
                    performance_score: 20.0,
                }),
            },
        ];
        let metrics = compute_response_metrics(&history);
        assert_eq!(metrics.replies_measured, 2);
        assert_eq!(metrics.replies_with_engagement, 1);
        assert!((metrics.response_rate - 0.5).abs() < 0.01);
        assert!((metrics.avg_performance_score - 50.0).abs() < 0.01);
    }

    #[test]
    fn compute_metrics_all_with_engagement() {
        let history = vec![
            ConversationRecord {
                tweet_id: "t1".to_string(),
                tweet_content: "A".to_string(),
                reply_content: "B".to_string(),
                reply_status: "sent".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                performance: Some(PerformanceSnapshot {
                    likes: 10,
                    replies_received: 3,
                    impressions: 1000,
                    performance_score: 90.0,
                }),
            },
            ConversationRecord {
                tweet_id: "t2".to_string(),
                tweet_content: "C".to_string(),
                reply_content: "D".to_string(),
                reply_status: "sent".to_string(),
                created_at: "2026-01-02T00:00:00Z".to_string(),
                performance: Some(PerformanceSnapshot {
                    likes: 3,
                    replies_received: 0,
                    impressions: 200,
                    performance_score: 60.0,
                }),
            },
        ];
        let metrics = compute_response_metrics(&history);
        assert_eq!(metrics.replies_with_engagement, 2);
        assert!((metrics.response_rate - 1.0).abs() < 0.01);
    }

    // ============================================================
    // generate_risk_signals tests
    // ============================================================

    #[test]
    fn risk_no_prior_interaction() {
        let signals = generate_risk_signals(&empty_summary(), &zero_metrics(), 5);
        assert!(signals
            .iter()
            .any(|s| s.signal_type == "no_prior_interaction"));
        assert_eq!(signals.len(), 1);
    }

    #[test]
    fn risk_high_frequency_today() {
        let mut summary = active_summary();
        summary.replies_today = 5;
        let signals = generate_risk_signals(&summary, &good_metrics(), 5);
        assert!(signals
            .iter()
            .any(|s| s.signal_type == "high_frequency_today"));
    }

    #[test]
    fn risk_low_response_rate() {
        let summary = active_summary();
        let signals = generate_risk_signals(&summary, &low_engagement_metrics(), 10);
        assert!(signals.iter().any(|s| s.signal_type == "low_response_rate"));
    }

    #[test]
    fn risk_no_measured_performance() {
        let summary = active_summary();
        let metrics = zero_metrics();
        let signals = generate_risk_signals(&summary, &metrics, 10);
        assert!(signals
            .iter()
            .any(|s| s.signal_type == "no_measured_performance"));
    }

    #[test]
    fn risk_good_engagement_no_signals() {
        let mut summary = active_summary();
        summary.replies_today = 1;
        let signals = generate_risk_signals(&summary, &good_metrics(), 10);
        assert!(signals.is_empty(), "no risk signals expected: {signals:?}");
    }

    #[test]
    fn risk_multiple_signals() {
        let mut summary = active_summary();
        summary.replies_today = 5;
        let signals = generate_risk_signals(&summary, &low_engagement_metrics(), 5);
        assert!(signals.len() >= 2, "expected multiple signals: {signals:?}");
        assert!(signals
            .iter()
            .any(|s| s.signal_type == "high_frequency_today"));
        assert!(signals.iter().any(|s| s.signal_type == "low_response_rate"));
    }

    // ============================================================
    // DB-backed integration tests
    // ============================================================

    #[tokio::test]
    async fn resolve_author_by_username() {
        let pool = init_test_db().await.expect("init db");

        // Seed a discovered tweet
        let tweet = crate::storage::tweets::DiscoveredTweet {
            id: "t_resolve".to_string(),
            author_id: "uid_resolve".to_string(),
            author_username: "resolveuser".to_string(),
            content: "Test tweet".to_string(),
            like_count: 5,
            retweet_count: 1,
            reply_count: 0,
            impression_count: None,
            relevance_score: Some(70.0),
            matched_keyword: Some("test".to_string()),
            discovered_at: "2026-02-20T10:00:00Z".to_string(),
            replied_to: 0,
        };
        crate::storage::tweets::insert_discovered_tweet(&pool, &tweet)
            .await
            .expect("insert");

        let (author_id, username) = resolve_author(&pool, "resolveuser", "resolveuser")
            .await
            .expect("resolve");
        assert_eq!(author_id, Some("uid_resolve".to_string()));
        assert_eq!(username, "resolveuser");
    }

    #[tokio::test]
    async fn resolve_author_by_id() {
        let pool = init_test_db().await.expect("init db");

        let tweet = crate::storage::tweets::DiscoveredTweet {
            id: "t_resolve_id".to_string(),
            author_id: "uid_by_id".to_string(),
            author_username: "byiduser".to_string(),
            content: "Test".to_string(),
            like_count: 0,
            retweet_count: 0,
            reply_count: 0,
            impression_count: None,
            relevance_score: None,
            matched_keyword: None,
            discovered_at: "2026-02-20T10:00:00Z".to_string(),
            replied_to: 0,
        };
        crate::storage::tweets::insert_discovered_tweet(&pool, &tweet)
            .await
            .expect("insert");

        let (author_id, username) = resolve_author(&pool, "uid_by_id", "uid_by_id")
            .await
            .expect("resolve");
        assert_eq!(author_id, Some("uid_by_id".to_string()));
        assert_eq!(username, "byiduser");
    }

    #[tokio::test]
    async fn resolve_author_not_found() {
        let pool = init_test_db().await.expect("init db");

        let (author_id, username) = resolve_author(&pool, "nobody", "nobody")
            .await
            .expect("resolve");
        assert!(author_id.is_none());
        assert_eq!(username, "nobody");
    }

    #[tokio::test]
    async fn interaction_summary_empty_db() {
        let pool = init_test_db().await.expect("init db");

        let summary = query_interaction_summary(&pool, &None, "nobody")
            .await
            .expect("summary");
        assert_eq!(summary.total_replies_sent, 0);
        assert_eq!(summary.replies_today, 0);
        assert!(summary.first_interaction.is_none());
    }

    #[tokio::test]
    async fn topic_affinity_empty_db() {
        let pool = init_test_db().await.expect("init db");

        let topics = query_topic_affinity(&pool, "nobody").await.expect("topics");
        assert!(topics.is_empty());
    }

    #[tokio::test]
    async fn topic_affinity_with_data() {
        let pool = init_test_db().await.expect("init db");

        // Seed multiple tweets with different keywords
        for (i, kw) in ["rust", "rust", "async", "rust"].iter().enumerate() {
            let tweet = crate::storage::tweets::DiscoveredTweet {
                id: format!("ta_{i}"),
                author_id: "uid_ta".to_string(),
                author_username: "tauser".to_string(),
                content: format!("Tweet about {kw}"),
                like_count: 5,
                retweet_count: 0,
                reply_count: 0,
                impression_count: None,
                relevance_score: Some(70.0),
                matched_keyword: Some(kw.to_string()),
                discovered_at: "2026-02-20T10:00:00Z".to_string(),
                replied_to: 0,
            };
            crate::storage::tweets::insert_discovered_tweet(&pool, &tweet)
                .await
                .expect("insert");
        }

        let topics = query_topic_affinity(&pool, "tauser").await.expect("topics");
        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0].keyword, "rust");
        assert_eq!(topics[0].mention_count, 3);
        assert_eq!(topics[1].keyword, "async");
        assert_eq!(topics[1].mention_count, 1);
    }

    #[tokio::test]
    async fn conversation_history_empty() {
        let pool = init_test_db().await.expect("init db");

        let history = query_conversation_history(&pool, "nobody")
            .await
            .expect("history");
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn get_author_context_full_integration() {
        let pool = init_test_db().await.expect("init db");
        let config = crate::config::Config::default();

        let ctx = get_author_context(&pool, "@testuser", &config)
            .await
            .expect("context");
        assert_eq!(ctx.author_username, "testuser");
        assert!(ctx.author_id.is_none());
        assert_eq!(ctx.interaction_summary.total_replies_sent, 0);
        // Should have "no_prior_interaction" signal
        assert!(ctx
            .risk_signals
            .iter()
            .any(|s| s.signal_type == "no_prior_interaction"));
    }

    #[tokio::test]
    async fn get_author_context_with_seeded_data() {
        let pool = init_test_db().await.expect("init db");
        let config = crate::config::Config::default();

        // Seed discovered tweet
        let tweet = crate::storage::tweets::DiscoveredTweet {
            id: "ctx_tweet".to_string(),
            author_id: "uid_ctx".to_string(),
            author_username: "ctxuser".to_string(),
            content: "Context test tweet".to_string(),
            like_count: 10,
            retweet_count: 2,
            reply_count: 1,
            impression_count: Some(500),
            relevance_score: Some(85.0),
            matched_keyword: Some("rust".to_string()),
            discovered_at: "2026-02-20T10:00:00Z".to_string(),
            replied_to: 0,
        };
        crate::storage::tweets::insert_discovered_tweet(&pool, &tweet)
            .await
            .expect("insert");

        let ctx = get_author_context(&pool, "ctxuser", &config)
            .await
            .expect("context");
        assert_eq!(ctx.author_username, "ctxuser");
        assert_eq!(ctx.author_id, Some("uid_ctx".to_string()));
        assert_eq!(ctx.topic_affinity.len(), 1);
        assert_eq!(ctx.topic_affinity[0].keyword, "rust");
    }
}
