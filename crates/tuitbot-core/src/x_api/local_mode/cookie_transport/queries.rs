//! GraphQL read operations for X's internal API.
//!
//! All methods are on `CookieTransport` and return domain types from
//! `crate::x_api::types`. Each method builds the appropriate variables
//! and features, issues a `graphql_get`, and parses the response.

use crate::error::XApiError;
use crate::x_api::types::{SearchMeta, SearchResponse, Tweet, User, UsersMeta, UsersResponse};

use super::features;
use super::response;
use super::CookieTransport;

impl CookieTransport {
    /// Search tweets via the SearchTimeline GraphQL endpoint.
    pub async fn search_timeline(
        &self,
        query: &str,
        max_results: u32,
        cursor: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let mut variables = serde_json::json!({
            "rawQuery": query,
            "count": max_results.min(20),
            "querySource": "typed_query",
            "product": "Latest",
        });
        if let Some(c) = cursor {
            variables["cursor"] = serde_json::Value::String(c.to_string());
        }

        let body = self
            .graphql_get("SearchTimeline", &variables, &features::read_features())
            .await?;

        let (tweets, next_cursor) = response::parse_timeline(
            &body,
            &["data", "search_by_raw_query", "search_timeline", "timeline"],
        );

        Ok(build_search_response(tweets, next_cursor))
    }

    /// Get a single tweet by ID via TweetResultByRestId.
    pub async fn get_tweet_by_id(&self, tweet_id: &str) -> Result<Tweet, XApiError> {
        let variables = serde_json::json!({
            "tweetId": tweet_id,
            "withCommunity": false,
            "includePromotedContent": false,
            "withVoice": false,
        });

        let body = self
            .graphql_get(
                "TweetResultByRestId",
                &variables,
                &features::read_features(),
            )
            .await?;

        response::check_graphql_errors(&body)?;

        let result = body
            .get("data")
            .and_then(|d| d.get("tweetResult"))
            .and_then(|tr| tr.get("result"))
            .ok_or_else(|| XApiError::ApiError {
                status: 0,
                message: format!("tweet {tweet_id} not found or unavailable"),
            })?;

        response::parse_tweet(result).ok_or_else(|| XApiError::ApiError {
            status: 0,
            message: format!("tweet {tweet_id} could not be parsed (tombstone or unavailable)"),
        })
    }

    /// Look up a user by screen name via UserByScreenName.
    pub async fn get_user_by_screen_name(&self, username: &str) -> Result<User, XApiError> {
        let variables = serde_json::json!({
            "screen_name": username,
            "withSafetyModeUserFields": true,
        });

        let body = self
            .graphql_get("UserByScreenName", &variables, &features::user_features())
            .await?;

        response::check_graphql_errors(&body)?;

        let result = body
            .get("data")
            .and_then(|d| d.get("user"))
            .and_then(|u| u.get("result"))
            .ok_or_else(|| XApiError::ApiError {
                status: 0,
                message: format!("user @{username} not found"),
            })?;

        response::parse_user(result).ok_or_else(|| XApiError::ApiError {
            status: 0,
            message: format!("user @{username} could not be parsed"),
        })
    }

    /// Look up a user by rest ID via UserByRestId.
    pub async fn get_user_by_rest_id(&self, user_id: &str) -> Result<User, XApiError> {
        let variables = serde_json::json!({
            "userId": user_id,
            "withSafetyModeUserFields": true,
        });

        let body = self
            .graphql_get("UserByRestId", &variables, &features::user_features())
            .await?;

        response::check_graphql_errors(&body)?;

        let result = body
            .get("data")
            .and_then(|d| d.get("user"))
            .and_then(|u| u.get("result"))
            .ok_or_else(|| XApiError::ApiError {
                status: 0,
                message: format!("user ID {user_id} not found"),
            })?;

        response::parse_user(result).ok_or_else(|| XApiError::ApiError {
            status: 0,
            message: format!("user ID {user_id} could not be parsed"),
        })
    }

    /// Get recent tweets from a user via UserTweets.
    pub async fn get_user_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        cursor: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let mut variables = serde_json::json!({
            "userId": user_id,
            "count": max_results.min(20),
            "includePromotedContent": false,
            "withQuickPromoteEligibilityTweetFields": false,
            "withVoice": false,
            "withV2Timeline": true,
        });
        if let Some(c) = cursor {
            variables["cursor"] = serde_json::Value::String(c.to_string());
        }

        let body = self
            .graphql_get("UserTweets", &variables, &features::read_features())
            .await?;

        let (tweets, next_cursor) = response::parse_timeline(
            &body,
            &["data", "user", "result", "timeline_v2", "timeline"],
        );

        Ok(build_search_response(tweets, next_cursor))
    }

    /// Get a user's followers via the Followers GraphQL endpoint.
    pub async fn get_followers(
        &self,
        user_id: &str,
        max_results: u32,
        cursor: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        let mut variables = serde_json::json!({
            "userId": user_id,
            "count": max_results.min(20),
            "includePromotedContent": false,
        });
        if let Some(c) = cursor {
            variables["cursor"] = serde_json::Value::String(c.to_string());
        }

        let body = self
            .graphql_get("Followers", &variables, &features::read_features())
            .await?;

        let (users, next_cursor) =
            response::parse_user_list(&body, &["data", "user", "result", "timeline", "timeline"]);

        Ok(build_users_response(users, next_cursor))
    }

    /// Get accounts a user is following via the Following GraphQL endpoint.
    pub async fn get_following(
        &self,
        user_id: &str,
        max_results: u32,
        cursor: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        let mut variables = serde_json::json!({
            "userId": user_id,
            "count": max_results.min(20),
            "includePromotedContent": false,
        });
        if let Some(c) = cursor {
            variables["cursor"] = serde_json::Value::String(c.to_string());
        }

        let body = self
            .graphql_get("Following", &variables, &features::read_features())
            .await?;

        let (users, next_cursor) =
            response::parse_user_list(&body, &["data", "user", "result", "timeline", "timeline"]);

        Ok(build_users_response(users, next_cursor))
    }

    /// Get tweets liked by a user via the Likes GraphQL endpoint.
    pub async fn get_liked_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        cursor: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let mut variables = serde_json::json!({
            "userId": user_id,
            "count": max_results.min(20),
            "includePromotedContent": false,
            "withClientEventToken": false,
            "withBirdwatchNotes": false,
            "withVoice": false,
            "withV2Timeline": true,
        });
        if let Some(c) = cursor {
            variables["cursor"] = serde_json::Value::String(c.to_string());
        }

        let body = self
            .graphql_get("Likes", &variables, &features::read_features())
            .await?;

        let (tweets, next_cursor) = response::parse_timeline(
            &body,
            &["data", "user", "result", "timeline_v2", "timeline"],
        );

        Ok(build_search_response(tweets, next_cursor))
    }

    /// Get the authenticated user's home timeline via HomeLatestTimeline.
    pub async fn get_home_timeline(
        &self,
        max_results: u32,
        cursor: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let mut variables = serde_json::json!({
            "count": max_results.min(20),
            "includePromotedContent": false,
            "latestControlAvailable": true,
            "requestContext": "launch",
            "withCommunity": true,
        });
        if let Some(c) = cursor {
            variables["cursor"] = serde_json::Value::String(c.to_string());
        }

        let body = self
            .graphql_get("HomeLatestTimeline", &variables, &features::read_features())
            .await?;

        let (tweets, next_cursor) =
            response::parse_timeline(&body, &["data", "home", "home_timeline_urt"]);

        Ok(build_search_response(tweets, next_cursor))
    }

    /// Get the authenticated user's bookmarks via the Bookmarks GraphQL endpoint.
    pub async fn get_bookmarks(
        &self,
        max_results: u32,
        cursor: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let mut variables = serde_json::json!({
            "count": max_results.min(20),
            "includePromotedContent": false,
        });
        if let Some(c) = cursor {
            variables["cursor"] = serde_json::Value::String(c.to_string());
        }

        let body = self
            .graphql_get("Bookmarks", &variables, &features::read_features())
            .await?;

        let (tweets, next_cursor) =
            response::parse_timeline(&body, &["data", "bookmark_timeline_v2", "timeline"]);

        Ok(build_search_response(tweets, next_cursor))
    }
}

/// Build a `SearchResponse` from parsed tweets and cursor.
fn build_search_response(tweets: Vec<Tweet>, next_cursor: Option<String>) -> SearchResponse {
    let result_count = tweets.len() as u32;
    let newest_id = tweets.first().map(|t| t.id.clone());
    let oldest_id = tweets.last().map(|t| t.id.clone());

    SearchResponse {
        data: tweets,
        includes: None,
        meta: SearchMeta {
            newest_id,
            oldest_id,
            result_count,
            next_token: next_cursor,
        },
    }
}

/// Build a `UsersResponse` from parsed users and cursor.
fn build_users_response(users: Vec<User>, next_cursor: Option<String>) -> UsersResponse {
    let result_count = users.len() as u32;

    UsersResponse {
        data: users,
        meta: UsersMeta {
            result_count,
            next_token: next_cursor,
        },
    }
}
