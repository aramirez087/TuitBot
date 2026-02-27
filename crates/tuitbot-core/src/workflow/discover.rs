//! Discover step: search tweets via toolkit, score, persist to DB.
//!
//! This is the first step in the reply pipeline: find tweets worth replying to.
//! Routes all X API calls through `toolkit::read::search_tweets`.

use std::collections::HashMap;

use crate::config::Config;
use crate::scoring::{find_matched_keywords, ScoringEngine, TweetData};
use crate::storage;
use crate::storage::tweets::DiscoveredTweet;
use crate::storage::DbPool;
use crate::toolkit;
use crate::x_api::XApiClient;

use super::{ScoreBreakdown, ScoredCandidate, WorkflowError};

/// Input for the discover step.
#[derive(Debug, Clone)]
pub struct DiscoverInput {
    /// Search query. If `None`, uses product keywords from config.
    pub query: Option<String>,
    /// Minimum score threshold. If `None`, uses `config.scoring.threshold`.
    pub min_score: Option<f64>,
    /// Maximum number of results (clamped to 1..100).
    pub limit: Option<u32>,
    /// Only return tweets newer than this ID.
    pub since_id: Option<String>,
}

/// Output from the discover step.
#[derive(Debug, Clone)]
pub struct DiscoverOutput {
    /// Scored and ranked candidates.
    pub candidates: Vec<ScoredCandidate>,
    /// The query that was used.
    pub query_used: String,
    /// The threshold that was applied.
    pub threshold: f64,
}

/// Execute the discover step: search, score, persist, rank.
///
/// All X API access goes through `toolkit::read::search_tweets`.
pub async fn execute(
    db: &DbPool,
    x_client: &dyn XApiClient,
    config: &Config,
    input: DiscoverInput,
) -> Result<DiscoverOutput, WorkflowError> {
    // Build query from input or product keywords
    let search_query = match &input.query {
        Some(q) => q.clone(),
        None => {
            let kw = &config.business.product_keywords;
            if kw.is_empty() {
                return Err(WorkflowError::InvalidInput(
                    "No search query provided and no product_keywords configured.".to_string(),
                ));
            }
            kw.join(" OR ")
        }
    };

    let max_results = input.limit.unwrap_or(10).clamp(1, 100);
    let threshold = input.min_score.unwrap_or(config.scoring.threshold as f64);

    // Search tweets via toolkit (not direct XApiClient)
    let search_response = toolkit::read::search_tweets(
        x_client,
        &search_query,
        max_results,
        input.since_id.as_deref(),
        None, // no pagination token
    )
    .await?;

    if search_response.data.is_empty() {
        return Ok(DiscoverOutput {
            candidates: vec![],
            query_used: search_query,
            threshold,
        });
    }

    // Build author lookup from includes
    let users: HashMap<String, &crate::x_api::types::User> = search_response
        .includes
        .as_ref()
        .map(|inc| inc.users.iter().map(|u| (u.id.clone(), u)).collect())
        .unwrap_or_default();

    // Build scoring engine
    let keywords: Vec<String> = config
        .business
        .product_keywords
        .iter()
        .chain(config.business.competitor_keywords.iter())
        .chain(config.business.effective_industry_topics().iter())
        .cloned()
        .collect();
    let engine = ScoringEngine::new(config.scoring.clone(), keywords.clone());

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

        // Persist to DB (best-effort)
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
        let _ = storage::tweets::insert_discovered_tweet(db, &discovered).await;

        // Check if already replied
        let already_replied = storage::replies::has_replied_to(db, &tweet.id)
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

    // Filter by threshold, sort desc, take limit
    candidates.retain(|c| (c.score_total as f64) >= threshold);
    candidates.sort_by(|a, b| {
        b.score_total
            .partial_cmp(&a.score_total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(max_results as usize);

    Ok(DiscoverOutput {
        candidates,
        query_used: search_query,
        threshold,
    })
}
