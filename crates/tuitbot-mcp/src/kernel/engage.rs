//! DB-free engagement functions taking `&dyn XApiClient` directly.
//!
//! These kernel engage tools bypass policy gating and mutation recording,
//! making them suitable for the API profile where no DB is available.

use std::time::Instant;

use serde::Serialize;

use crate::contract::envelope::{ToolMeta, ToolResponse};
use crate::contract::error::provider_error_to_response;
use crate::provider::x_api::map_x_error;
use tuitbot_core::x_api::XApiClient;

/// Like a tweet.
pub async fn like_tweet(client: &dyn XApiClient, user_id: &str, tweet_id: &str) -> String {
    let start = Instant::now();
    match client.like_tweet(user_id, tweet_id).await {
        Ok(liked) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct LikeResult {
                liked: bool,
                tweet_id: String,
            }
            ToolResponse::success(LikeResult {
                liked,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}

/// Follow a user.
pub async fn follow_user(client: &dyn XApiClient, user_id: &str, target_user_id: &str) -> String {
    let start = Instant::now();
    match client.follow_user(user_id, target_user_id).await {
        Ok(following) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct FollowResult {
                following: bool,
                target_user_id: String,
            }
            ToolResponse::success(FollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}

/// Unfollow a user.
pub async fn unfollow_user(client: &dyn XApiClient, user_id: &str, target_user_id: &str) -> String {
    let start = Instant::now();
    match client.unfollow_user(user_id, target_user_id).await {
        Ok(following) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UnfollowResult {
                following: bool,
                target_user_id: String,
            }
            ToolResponse::success(UnfollowResult {
                following,
                target_user_id: target_user_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}

/// Retweet a tweet.
pub async fn retweet(client: &dyn XApiClient, user_id: &str, tweet_id: &str) -> String {
    let start = Instant::now();
    match client.retweet(user_id, tweet_id).await {
        Ok(retweeted) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct RetweetResult {
                retweeted: bool,
                tweet_id: String,
            }
            ToolResponse::success(RetweetResult {
                retweeted,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}

/// Undo a retweet.
pub async fn unretweet(client: &dyn XApiClient, user_id: &str, tweet_id: &str) -> String {
    let start = Instant::now();
    match client.unretweet(user_id, tweet_id).await {
        Ok(retweeted) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UnretweetResult {
                retweeted: bool,
                tweet_id: String,
            }
            ToolResponse::success(UnretweetResult {
                retweeted,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}

/// Unlike a tweet.
pub async fn unlike_tweet(client: &dyn XApiClient, user_id: &str, tweet_id: &str) -> String {
    let start = Instant::now();
    match client.unlike_tweet(user_id, tweet_id).await {
        Ok(liked) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UnlikeResult {
                liked: bool,
                tweet_id: String,
            }
            ToolResponse::success(UnlikeResult {
                liked,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}

/// Bookmark a tweet.
pub async fn bookmark_tweet(client: &dyn XApiClient, user_id: &str, tweet_id: &str) -> String {
    let start = Instant::now();
    match client.bookmark_tweet(user_id, tweet_id).await {
        Ok(bookmarked) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct BookmarkResult {
                bookmarked: bool,
                tweet_id: String,
            }
            ToolResponse::success(BookmarkResult {
                bookmarked,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}

/// Remove a bookmark.
pub async fn unbookmark_tweet(client: &dyn XApiClient, user_id: &str, tweet_id: &str) -> String {
    let start = Instant::now();
    match client.unbookmark_tweet(user_id, tweet_id).await {
        Ok(bookmarked) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UnbookmarkResult {
                bookmarked: bool,
                tweet_id: String,
            }
            ToolResponse::success(UnbookmarkResult {
                bookmarked,
                tweet_id: tweet_id.to_string(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => provider_error_to_response(&map_x_error(&e), start),
    }
}
