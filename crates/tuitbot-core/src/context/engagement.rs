//! Engagement recommendation engine.
//!
//! Analyzes author context, tweet content, rate limits, and campaign
//! objectives to produce an explainable engagement recommendation.
//! Every recommendation includes the contributing factors and their
//! weights so the agent can understand *why* it was made.

use crate::config::Config;
use crate::error::StorageError;
use crate::scoring::find_matched_keywords;
use crate::storage::DbPool;
use serde::Serialize;

use super::author;

/// The complete recommendation output.
#[derive(Debug, Clone, Serialize)]
pub struct EngagementRecommendation {
    pub recommended_action: String,
    pub confidence: f64,
    pub contributing_factors: Vec<ContributingFactor>,
    pub policy_considerations: Vec<PolicyConsideration>,
}

/// A single factor that contributed to the recommendation.
#[derive(Debug, Clone, Serialize)]
pub struct ContributingFactor {
    pub factor: String,
    pub signal: String,
    pub weight: f64,
    pub detail: String,
}

/// A policy constraint relevant to the recommendation.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyConsideration {
    pub policy: String,
    pub status: String,
    pub detail: String,
}

/// Produce an explainable engagement recommendation.
///
/// Evaluates five weighted factors:
/// 1. Keyword relevance (30%) — does the tweet match configured keywords?
/// 2. Author relationship (20%) — interaction history and engagement rate
/// 3. Author frequency (15%) — per-author daily limit proximity
/// 4. Daily capacity (15%) — global daily limit proximity
/// 5. Campaign alignment (20%) — overlap with the stated objective
pub async fn recommend_engagement(
    pool: &DbPool,
    author_username: &str,
    tweet_text: &str,
    campaign_objective: Option<&str>,
    config: &Config,
) -> Result<EngagementRecommendation, StorageError> {
    let ctx = author::get_author_context(pool, author_username, config).await?;
    let replies_today_total = crate::storage::replies::count_replies_today(pool).await?;

    let mut factors = Vec::new();
    let mut blocked = false;

    // --- 1. Keyword relevance (weight: 30) ---
    let mut keywords: Vec<String> = config.business.product_keywords.clone();
    keywords.extend(config.business.competitor_keywords.clone());
    keywords.extend(config.business.industry_topics.clone());
    let matched = find_matched_keywords(tweet_text, &keywords);
    let relevance_score = if matched.is_empty() {
        factors.push(ContributingFactor {
            factor: "keyword_relevance".into(),
            signal: "negative".into(),
            weight: 30.0,
            detail: "No configured keyword matches in tweet text".into(),
        });
        10.0
    } else {
        let score = (matched.len() as f64 * 30.0).min(100.0);
        factors.push(ContributingFactor {
            factor: "keyword_relevance".into(),
            signal: "positive".into(),
            weight: 30.0,
            detail: format!("Matched {} keywords: {}", matched.len(), matched.join(", ")),
        });
        score
    };

    // --- 2. Author relationship (weight: 20) ---
    let relationship_score = evaluate_relationship(&ctx, &mut factors);

    // --- 3. Author frequency (weight: 15) ---
    let max_per_author = config.limits.max_replies_per_author_per_day as i64;
    let frequency_score = if ctx.interaction_summary.replies_today >= max_per_author {
        blocked = true;
        factors.push(ContributingFactor {
            factor: "author_frequency".into(),
            signal: "negative".into(),
            weight: 15.0,
            detail: format!(
                "At per-author daily limit ({}/{})",
                ctx.interaction_summary.replies_today, max_per_author
            ),
        });
        0.0
    } else if ctx.interaction_summary.replies_today > 0 {
        factors.push(ContributingFactor {
            factor: "author_frequency".into(),
            signal: "neutral".into(),
            weight: 15.0,
            detail: format!(
                "Replied {} time(s) today (limit: {})",
                ctx.interaction_summary.replies_today, max_per_author
            ),
        });
        40.0
    } else {
        factors.push(ContributingFactor {
            factor: "author_frequency".into(),
            signal: "positive".into(),
            weight: 15.0,
            detail: "No replies to this author today".into(),
        });
        100.0
    };

    // --- 4. Daily capacity (weight: 15) ---
    let max_per_day = config.limits.max_replies_per_day as i64;
    let capacity_score =
        evaluate_capacity(replies_today_total, max_per_day, &mut factors, &mut blocked);

    // --- 5. Campaign alignment (weight: 20) ---
    let alignment_score = evaluate_campaign(tweet_text, campaign_objective, &mut factors);

    // --- Weighted total ---
    let weighted_total = (relevance_score * 30.0
        + relationship_score * 20.0
        + frequency_score * 15.0
        + capacity_score * 15.0
        + alignment_score * 20.0)
        / 100.0;

    let (action, confidence) = decide_action(weighted_total, blocked);

    // --- Policy considerations ---
    let policies = build_policy_considerations(
        config,
        replies_today_total,
        max_per_day,
        ctx.interaction_summary.replies_today,
        max_per_author,
    );

    Ok(EngagementRecommendation {
        recommended_action: action,
        confidence,
        contributing_factors: factors,
        policy_considerations: policies,
    })
}

fn evaluate_relationship(
    ctx: &author::AuthorContext,
    factors: &mut Vec<ContributingFactor>,
) -> f64 {
    if ctx.interaction_summary.total_replies_sent > 0 {
        if ctx.response_metrics.response_rate > 0.2 {
            factors.push(ContributingFactor {
                factor: "author_relationship".into(),
                signal: "positive".into(),
                weight: 20.0,
                detail: format!(
                    "Good engagement history ({:.0}% response rate over {} interactions)",
                    ctx.response_metrics.response_rate * 100.0,
                    ctx.response_metrics.replies_measured
                ),
            });
            90.0
        } else if ctx.response_metrics.response_rate > 0.0 {
            factors.push(ContributingFactor {
                factor: "author_relationship".into(),
                signal: "neutral".into(),
                weight: 20.0,
                detail: format!(
                    "Some engagement history ({:.0}% response rate)",
                    ctx.response_metrics.response_rate * 100.0
                ),
            });
            60.0
        } else {
            factors.push(ContributingFactor {
                factor: "author_relationship".into(),
                signal: "negative".into(),
                weight: 20.0,
                detail: "Previous interactions received no engagement".into(),
            });
            30.0
        }
    } else {
        factors.push(ContributingFactor {
            factor: "author_relationship".into(),
            signal: "neutral".into(),
            weight: 20.0,
            detail: "No prior interaction — fresh engagement opportunity".into(),
        });
        50.0
    }
}

fn evaluate_capacity(
    replies_today: i64,
    max_per_day: i64,
    factors: &mut Vec<ContributingFactor>,
    blocked: &mut bool,
) -> f64 {
    if replies_today >= max_per_day {
        *blocked = true;
        factors.push(ContributingFactor {
            factor: "daily_capacity".into(),
            signal: "negative".into(),
            weight: 15.0,
            detail: format!("Daily limit reached ({}/{})", replies_today, max_per_day),
        });
        0.0
    } else {
        let utilization = replies_today as f64 / max_per_day.max(1) as f64;
        if utilization > 0.8 {
            factors.push(ContributingFactor {
                factor: "daily_capacity".into(),
                signal: "negative".into(),
                weight: 15.0,
                detail: format!(
                    "Nearing daily limit ({}/{}, {:.0}% used)",
                    replies_today,
                    max_per_day,
                    utilization * 100.0
                ),
            });
            30.0
        } else {
            factors.push(ContributingFactor {
                factor: "daily_capacity".into(),
                signal: "positive".into(),
                weight: 15.0,
                detail: format!(
                    "Capacity available ({}/{}, {:.0}% used)",
                    replies_today,
                    max_per_day,
                    utilization * 100.0
                ),
            });
            100.0
        }
    }
}

fn evaluate_campaign(
    tweet_text: &str,
    campaign_objective: Option<&str>,
    factors: &mut Vec<ContributingFactor>,
) -> f64 {
    let Some(objective) = campaign_objective.filter(|o| !o.is_empty()) else {
        factors.push(ContributingFactor {
            factor: "campaign_alignment".into(),
            signal: "neutral".into(),
            weight: 20.0,
            detail: "No campaign objective specified".into(),
        });
        return 50.0;
    };

    let tweet_lower = tweet_text.to_lowercase();
    let objective_words: Vec<&str> = objective
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .collect();
    let matching: Vec<&&str> = objective_words
        .iter()
        .filter(|w| tweet_lower.contains(&w.to_lowercase()))
        .collect();

    if matching.len() >= 3 {
        factors.push(ContributingFactor {
            factor: "campaign_alignment".into(),
            signal: "positive".into(),
            weight: 20.0,
            detail: format!(
                "Strong alignment — {} objective terms found in tweet",
                matching.len()
            ),
        });
        90.0
    } else if !matching.is_empty() {
        factors.push(ContributingFactor {
            factor: "campaign_alignment".into(),
            signal: "neutral".into(),
            weight: 20.0,
            detail: format!(
                "Partial alignment — {} objective term(s) found in tweet",
                matching.len()
            ),
        });
        60.0
    } else {
        factors.push(ContributingFactor {
            factor: "campaign_alignment".into(),
            signal: "negative".into(),
            weight: 20.0,
            detail: "No objective terms found in tweet text".into(),
        });
        20.0
    }
}

fn decide_action(weighted_total: f64, blocked: bool) -> (String, f64) {
    if blocked {
        return ("skip".to_string(), 0.95);
    }
    if weighted_total >= 65.0 {
        let confidence = (0.5 + (weighted_total - 65.0) / 70.0).clamp(0.6, 0.95);
        ("reply".to_string(), confidence)
    } else if weighted_total >= 40.0 {
        let confidence = (0.4 + (weighted_total - 40.0) / 62.5).clamp(0.4, 0.8);
        ("observe".to_string(), confidence)
    } else {
        let confidence = (0.5 + (40.0 - weighted_total) / 80.0).clamp(0.5, 0.95);
        ("skip".to_string(), confidence)
    }
}

fn build_policy_considerations(
    config: &Config,
    replies_today: i64,
    max_per_day: i64,
    replies_to_author: i64,
    max_per_author: i64,
) -> Vec<PolicyConsideration> {
    let mut policies = Vec::new();

    if config.effective_approval_mode() {
        policies.push(PolicyConsideration {
            policy: "approval_mode".into(),
            status: "warning".into(),
            detail: "Approval mode active — replies require manual review".into(),
        });
    }

    if replies_today >= max_per_day {
        policies.push(PolicyConsideration {
            policy: "daily_rate_limit".into(),
            status: "blocked".into(),
            detail: format!("Daily limit reached ({}/{})", replies_today, max_per_day),
        });
    } else if replies_today as f64 > max_per_day as f64 * 0.8 {
        policies.push(PolicyConsideration {
            policy: "daily_rate_limit".into(),
            status: "warning".into(),
            detail: format!(
                "Approaching daily limit ({}/{})",
                replies_today, max_per_day
            ),
        });
    }

    if replies_to_author >= max_per_author {
        policies.push(PolicyConsideration {
            policy: "per_author_limit".into(),
            status: "blocked".into(),
            detail: format!(
                "Per-author limit reached ({}/{})",
                replies_to_author, max_per_author
            ),
        });
    }

    policies
}
