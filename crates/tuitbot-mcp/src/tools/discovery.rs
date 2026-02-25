//! Discovery tools: list discovered tweets, list unreplied tweets.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

#[derive(Serialize)]
struct DiscoveredTweetOut {
    id: String,
    author_id: String,
    author_username: String,
    content: String,
    like_count: i64,
    retweet_count: i64,
    reply_count: i64,
    impression_count: Option<i64>,
    relevance_score: Option<f64>,
    matched_keyword: Option<String>,
    discovered_at: String,
    replied_to: bool,
}

fn tweet_to_out(t: &storage::tweets::DiscoveredTweet) -> DiscoveredTweetOut {
    DiscoveredTweetOut {
        id: t.id.clone(),
        author_id: t.author_id.clone(),
        author_username: t.author_username.clone(),
        content: t.content.clone(),
        like_count: t.like_count,
        retweet_count: t.retweet_count,
        reply_count: t.reply_count,
        impression_count: t.impression_count,
        relevance_score: t.relevance_score,
        matched_keyword: t.matched_keyword.clone(),
        discovered_at: t.discovered_at.clone(),
        replied_to: t.replied_to != 0,
    }
}

/// List unreplied tweets above a score threshold.
pub async fn list_unreplied_tweets(pool: &DbPool, threshold: f64, config: &Config) -> String {
    let start = Instant::now();

    match storage::tweets::get_unreplied_tweets_above_score(pool, threshold).await {
        Ok(tweets) => {
            let out: Vec<DiscoveredTweetOut> = tweets.iter().map(tweet_to_out).collect();
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching unreplied tweets: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// List unreplied tweets above a score threshold with a limit.
pub async fn list_unreplied_tweets_with_limit(
    pool: &DbPool,
    threshold: f64,
    limit: u32,
    config: &Config,
) -> String {
    let start = Instant::now();

    match storage::tweets::get_unreplied_tweets_above_score(pool, threshold).await {
        Ok(tweets) => {
            let out: Vec<DiscoveredTweetOut> = tweets
                .iter()
                .take(limit as usize)
                .map(tweet_to_out)
                .collect();
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching unreplied tweets: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
