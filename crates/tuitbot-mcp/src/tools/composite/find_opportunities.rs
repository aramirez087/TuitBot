//! `find_reply_opportunities` — search, score, and rank tweets in one call.

use std::collections::HashMap;
use std::time::Instant;

use tuitbot_core::scoring::{find_matched_keywords, ScoringEngine, TweetData};
use tuitbot_core::storage;
use tuitbot_core::storage::tweets::DiscoveredTweet;

use crate::state::SharedState;
use crate::tools::response::{ToolMeta, ToolResponse};

use super::{ScoreBreakdown, ScoredCandidate};

/// Execute the `find_reply_opportunities` composite tool.
pub async fn execute(
    state: &SharedState,
    query: Option<&str>,
    min_score: Option<f64>,
    limit: Option<u32>,
    since_id: Option<&str>,
) -> String {
    let start = Instant::now();

    // Require X client
    let x_client = match state.x_client.as_ref() {
        Some(c) => c,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "x_not_configured",
                "X API client not available. Run `tuitbot auth` to authenticate.",
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    // Build query from input or product keywords
    let search_query = match query {
        Some(q) => q.to_string(),
        None => {
            let kw = &state.config.business.product_keywords;
            if kw.is_empty() {
                let elapsed = start.elapsed().as_millis() as u64;
                return ToolResponse::error(
                    "no_keywords",
                    "No search query provided and no product_keywords configured.",
                    false,
                )
                .with_meta(ToolMeta::new(elapsed))
                .to_json();
            }
            kw.join(" OR ")
        }
    };

    let max_results = limit.unwrap_or(10).clamp(1, 100);

    // Search tweets via X API
    let search_response = match x_client
        .search_tweets(&search_query, max_results, since_id, None)
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error("x_api_error", format!("Search failed: {e}"), true)
                .with_meta(ToolMeta::new(elapsed))
                .to_json();
        }
    };

    if search_response.data.is_empty() {
        let elapsed = start.elapsed().as_millis() as u64;
        return ToolResponse::success(serde_json::json!({
            "candidates": [],
            "total_searched": 0,
            "query": search_query,
        }))
        .with_meta(ToolMeta::new(elapsed).with_mode(
            state.config.mode.to_string(),
            state.config.effective_approval_mode(),
        ))
        .to_json();
    }

    // Build author lookup from includes
    let users: HashMap<String, &tuitbot_core::x_api::types::User> = search_response
        .includes
        .as_ref()
        .map(|inc| inc.users.iter().map(|u| (u.id.clone(), u)).collect())
        .unwrap_or_default();

    // Build scoring engine
    let keywords: Vec<String> = state
        .config
        .business
        .product_keywords
        .iter()
        .chain(state.config.business.competitor_keywords.iter())
        .chain(state.config.business.industry_topics.iter())
        .cloned()
        .collect();
    let engine = ScoringEngine::new(state.config.scoring.clone(), keywords.clone());
    let threshold = min_score.unwrap_or(state.config.scoring.threshold as f64);

    let mut candidates = Vec::new();

    for tweet in &search_response.data {
        let user = users.get(&tweet.author_id);
        let author_username = user.map(|u| u.username.as_str()).unwrap_or("unknown");
        let author_followers = user.map(|u| u.public_metrics.followers_count).unwrap_or(0);

        let tweet_data = TweetData {
            text: tweet.text.clone(),
            created_at: tweet.created_at.clone(),
            likes: tweet.public_metrics.like_count,
            retweets: tweet.public_metrics.retweet_count,
            replies: tweet.public_metrics.reply_count,
            author_username: author_username.to_string(),
            author_followers,
            has_media: false,
            is_quote_tweet: false,
        };

        let score = engine.score_tweet(&tweet_data);
        let matched = find_matched_keywords(&tweet.text, &keywords);

        // Persist to DB (best-effort, same as automation loops)
        let discovered = DiscoveredTweet {
            id: tweet.id.clone(),
            author_id: tweet.author_id.clone(),
            author_username: author_username.to_string(),
            content: tweet.text.clone(),
            like_count: tweet.public_metrics.like_count as i64,
            retweet_count: tweet.public_metrics.retweet_count as i64,
            reply_count: tweet.public_metrics.reply_count as i64,
            impression_count: Some(tweet.public_metrics.impression_count as i64),
            relevance_score: Some(score.total as f64),
            matched_keyword: matched.first().cloned(),
            discovered_at: tweet.created_at.clone(),
            replied_to: 0,
        };
        let _ = storage::tweets::insert_discovered_tweet(&state.pool, &discovered).await;

        // Check if already replied
        let already_replied = storage::replies::has_replied_to(&state.pool, &tweet.id)
            .await
            .unwrap_or(false);

        // Determine recommended action
        let recommended_action = if (score.total as f64) >= threshold + 15.0 {
            "strong_reply"
        } else if (score.total as f64) >= threshold {
            "consider"
        } else {
            "skip"
        };

        candidates.push(ScoredCandidate {
            tweet_id: tweet.id.clone(),
            author_username: author_username.to_string(),
            author_followers,
            text: tweet.text.clone(),
            created_at: tweet.created_at.clone(),
            score_total: score.total,
            score_breakdown: ScoreBreakdown {
                keyword_relevance: score.keyword_relevance,
                follower: score.follower,
                recency: score.recency,
                engagement: score.engagement,
                reply_count: score.reply_count,
                content_type: score.content_type,
            },
            matched_keywords: matched,
            recommended_action: recommended_action.to_string(),
            already_replied,
        });
    }

    // Filter by min_score, sort desc, take limit
    candidates.retain(|c| (c.score_total as f64) >= threshold);
    candidates.sort_by(|a, b| {
        b.score_total
            .partial_cmp(&a.score_total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(max_results as usize);

    let total = candidates.len();
    let elapsed = start.elapsed().as_millis() as u64;
    crate::tools::telemetry::record(
        &state.pool,
        "find_reply_opportunities",
        "composite",
        elapsed,
        true,
        None,
        None,
    )
    .await;
    ToolResponse::success(serde_json::json!({
        "candidates": candidates,
        "total_found": total,
        "query": search_query,
        "threshold": threshold,
    }))
    .with_meta(ToolMeta::new(elapsed).with_mode(
        state.config.mode.to_string(),
        state.config.effective_approval_mode(),
    ))
    .to_json()
}
