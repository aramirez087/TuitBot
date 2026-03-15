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
pub(super) fn build_search_response(
    tweets: Vec<Tweet>,
    next_cursor: Option<String>,
) -> SearchResponse {
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
pub(super) fn build_users_response(users: Vec<User>, next_cursor: Option<String>) -> UsersResponse {
    let result_count = users.len() as u32;

    UsersResponse {
        data: users,
        meta: UsersMeta {
            result_count,
            next_token: next_cursor,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x_api::types::{PublicMetrics, UserMetrics};

    fn make_tweet(id: &str) -> Tweet {
        Tweet {
            id: id.to_string(),
            text: format!("tweet text {id}"),
            author_id: "author1".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        }
    }

    fn make_user(id: &str, username: &str) -> User {
        User {
            id: id.to_string(),
            username: username.to_string(),
            name: format!("User {username}"),
            profile_image_url: None,
            description: None,
            location: None,
            url: None,
            public_metrics: UserMetrics::default(),
        }
    }

    // ── build_search_response ───────────────────────────────────

    #[test]
    fn build_search_response_empty() {
        let resp = build_search_response(vec![], None);
        assert_eq!(resp.data.len(), 0);
        assert_eq!(resp.meta.result_count, 0);
        assert!(resp.meta.newest_id.is_none());
        assert!(resp.meta.oldest_id.is_none());
        assert!(resp.meta.next_token.is_none());
        assert!(resp.includes.is_none());
    }

    #[test]
    fn build_search_response_single_tweet() {
        let tweets = vec![make_tweet("100")];
        let resp = build_search_response(tweets, None);
        assert_eq!(resp.meta.result_count, 1);
        assert_eq!(resp.meta.newest_id.as_deref(), Some("100"));
        assert_eq!(resp.meta.oldest_id.as_deref(), Some("100"));
        assert!(resp.meta.next_token.is_none());
    }

    #[test]
    fn build_search_response_multiple_tweets() {
        let tweets = vec![make_tweet("300"), make_tweet("200"), make_tweet("100")];
        let resp = build_search_response(tweets, Some("cursor_abc".to_string()));
        assert_eq!(resp.meta.result_count, 3);
        assert_eq!(resp.meta.newest_id.as_deref(), Some("300"));
        assert_eq!(resp.meta.oldest_id.as_deref(), Some("100"));
        assert_eq!(resp.meta.next_token.as_deref(), Some("cursor_abc"));
    }

    #[test]
    fn build_search_response_preserves_tweet_data() {
        let mut t = make_tweet("42");
        t.text = "Hello world".to_string();
        t.author_id = "author_99".to_string();
        let resp = build_search_response(vec![t], None);
        assert_eq!(resp.data[0].text, "Hello world");
        assert_eq!(resp.data[0].author_id, "author_99");
    }

    // ── build_users_response ────────────────────────────────────

    #[test]
    fn build_users_response_empty() {
        let resp = build_users_response(vec![], None);
        assert_eq!(resp.data.len(), 0);
        assert_eq!(resp.meta.result_count, 0);
        assert!(resp.meta.next_token.is_none());
    }

    #[test]
    fn build_users_response_single_user() {
        let users = vec![make_user("1", "alice")];
        let resp = build_users_response(users, None);
        assert_eq!(resp.meta.result_count, 1);
        assert_eq!(resp.data[0].username, "alice");
    }

    #[test]
    fn build_users_response_multiple_users_with_cursor() {
        let users = vec![
            make_user("1", "alice"),
            make_user("2", "bob"),
            make_user("3", "carol"),
        ];
        let resp = build_users_response(users, Some("next_page".to_string()));
        assert_eq!(resp.meta.result_count, 3);
        assert_eq!(resp.meta.next_token.as_deref(), Some("next_page"));
        assert_eq!(resp.data[2].username, "carol");
    }

    #[test]
    fn build_users_response_preserves_user_fields() {
        let mut u = make_user("10", "dave");
        u.name = "Dave Smith".to_string();
        u.description = Some("A developer".to_string());
        u.location = Some("NYC".to_string());
        let resp = build_users_response(vec![u], None);
        assert_eq!(resp.data[0].name, "Dave Smith");
        assert_eq!(resp.data[0].description.as_deref(), Some("A developer"));
    }

    // ── features module coverage ────────────────────────────────

    #[test]
    fn read_features_returns_object_with_expected_keys() {
        let f = super::super::features::read_features();
        assert!(f.is_object());
        let obj = f.as_object().unwrap();
        assert!(obj.contains_key("responsive_web_graphql_exclude_directive_enabled"));
        assert!(obj.contains_key("view_counts_everywhere_api_enabled"));
        assert!(obj.len() > 10, "read_features should have many keys");
    }

    #[test]
    fn mutation_features_returns_object_with_expected_keys() {
        let f = super::super::features::mutation_features();
        assert!(f.is_object());
        let obj = f.as_object().unwrap();
        assert!(obj.contains_key("responsive_web_edit_tweet_api_enabled"));
        assert!(obj.contains_key("responsive_web_graphql_exclude_directive_enabled"));
    }

    #[test]
    fn user_features_returns_object_with_expected_keys() {
        let f = super::super::features::user_features();
        assert!(f.is_object());
        let obj = f.as_object().unwrap();
        assert!(obj.contains_key("hidden_profile_subscriptions_enabled"));
        assert!(obj.contains_key("responsive_web_graphql_timeline_navigation_enabled"));
    }

    #[test]
    fn features_are_all_booleans() {
        for features_fn in [
            super::super::features::read_features,
            super::super::features::mutation_features,
            super::super::features::user_features,
        ] {
            let f = features_fn();
            for (key, val) in f.as_object().unwrap() {
                assert!(
                    val.is_boolean(),
                    "feature flag '{key}' should be a boolean, got {val}"
                );
            }
        }
    }

    // ── response module: check_graphql_errors ────────────────────

    #[test]
    fn check_graphql_errors_no_errors() {
        let body = serde_json::json!({"data": {"user": {}}});
        let result = super::super::response::check_graphql_errors(&body);
        assert!(result.is_ok());
    }

    #[test]
    fn check_graphql_errors_with_errors() {
        let body = serde_json::json!({
            "errors": [{"message": "Rate limit exceeded", "code": 88}]
        });
        let result = super::super::response::check_graphql_errors(&body);
        assert!(result.is_err());
    }

    #[test]
    fn check_graphql_errors_empty_errors_array() {
        let body = serde_json::json!({"errors": [], "data": {}});
        let result = super::super::response::check_graphql_errors(&body);
        // Empty errors array should be OK
        assert!(result.is_ok());
    }

    // ── response module: parse_tweet ────────────────────────────

    #[test]
    fn parse_tweet_valid() {
        let result = serde_json::json!({
            "rest_id": "12345",
            "legacy": {
                "full_text": "Hello world",
                "user_id_str": "99",
                "created_at": "Mon Jan 01 00:00:00 +0000 2026",
                "retweet_count": 5,
                "favorite_count": 10,
                "reply_count": 2,
                "quote_count": 1,
                "bookmark_count": 0,
                "conversation_id_str": "12345"
            },
            "views": {"count": "100"}
        });
        let tweet = super::super::response::parse_tweet(&result);
        assert!(tweet.is_some());
        let t = tweet.unwrap();
        assert_eq!(t.id, "12345");
        assert_eq!(t.text, "Hello world");
    }

    #[test]
    fn parse_tweet_tombstone_returns_none() {
        let result = serde_json::json!({
            "__typename": "TweetTombstone"
        });
        assert!(super::super::response::parse_tweet(&result).is_none());
    }

    #[test]
    fn parse_tweet_unavailable_returns_none() {
        let result = serde_json::json!({
            "__typename": "TweetUnavailable"
        });
        assert!(super::super::response::parse_tweet(&result).is_none());
    }

    #[test]
    fn parse_tweet_missing_rest_id_returns_none() {
        let result = serde_json::json!({
            "legacy": {"full_text": "test"}
        });
        assert!(super::super::response::parse_tweet(&result).is_none());
    }

    #[test]
    fn parse_tweet_empty_rest_id_returns_none() {
        let result = serde_json::json!({
            "rest_id": "",
            "legacy": {"full_text": "test"}
        });
        assert!(super::super::response::parse_tweet(&result).is_none());
    }

    // ── response module: parse_timeline ─────────────────────────

    #[test]
    fn parse_timeline_empty_body() {
        let body = serde_json::json!({});
        let (tweets, cursor) = super::super::response::parse_timeline(&body, &["data", "search"]);
        assert!(tweets.is_empty());
        assert!(cursor.is_none());
    }

    #[test]
    fn parse_timeline_no_entries() {
        let body = serde_json::json!({
            "data": {
                "search": {
                    "instructions": []
                }
            }
        });
        let (tweets, cursor) = super::super::response::parse_timeline(&body, &["data", "search"]);
        assert!(tweets.is_empty());
        assert!(cursor.is_none());
    }

    // ── response module: parse_user ─────────────────────────────

    #[test]
    fn parse_user_valid() {
        let result = serde_json::json!({
            "__typename": "User",
            "rest_id": "42",
            "legacy": {
                "screen_name": "testuser",
                "name": "Test User",
                "profile_image_url_https": "https://pbs.twimg.com/profile.jpg",
                "description": "A tester",
                "location": "Earth",
                "followers_count": 100,
                "friends_count": 50,
                "statuses_count": 1000,
                "listed_count": 5
            }
        });
        let user = super::super::response::parse_user(&result);
        assert!(user.is_some());
        let u = user.unwrap();
        assert_eq!(u.id, "42");
        assert_eq!(u.username, "testuser");
    }

    #[test]
    fn parse_user_missing_legacy_returns_none() {
        let result = serde_json::json!({
            "rest_id": "42"
        });
        assert!(super::super::response::parse_user(&result).is_none());
    }

    // ── response module: parse_user_list ─────────────────────────

    #[test]
    fn parse_user_list_empty_body() {
        let body = serde_json::json!({});
        let (users, cursor) =
            super::super::response::parse_user_list(&body, &["data", "user", "result"]);
        assert!(users.is_empty());
        assert!(cursor.is_none());
    }
}
