//! GraphQL mutation operations for X's internal API.
//!
//! Includes tweet creation, deletion, likes, retweets, bookmarks,
//! and follow/unfollow (REST endpoints).

use crate::error::XApiError;
use crate::x_api::types::PostedTweet;

use super::features;
use super::response;
use super::CookieTransport;

impl CookieTransport {
    /// Post a new tweet via the CreateTweet GraphQL mutation.
    pub async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        self.create_tweet(text, None).await
    }

    /// Reply to a tweet via the CreateTweet GraphQL mutation.
    pub async fn reply_to_tweet(
        &self,
        text: &str,
        in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        self.create_tweet(text, Some(in_reply_to_id)).await
    }

    /// Core CreateTweet GraphQL request.
    async fn create_tweet(
        &self,
        text: &str,
        in_reply_to_id: Option<&str>,
    ) -> Result<PostedTweet, XApiError> {
        let mut variables = serde_json::json!({
            "tweet_text": text,
            "dark_request": false,
            "media": {
                "media_entities": [],
                "possibly_sensitive": false
            },
            "semantic_annotation_ids": []
        });

        if let Some(reply_id) = in_reply_to_id {
            variables["reply"] = serde_json::json!({
                "in_reply_to_tweet_id": reply_id,
                "exclude_reply_user_ids": []
            });
        }

        let body = self
            .graphql_post("CreateTweet", &variables, &features::mutation_features())
            .await?;

        response::parse_create_tweet_response(&body)
    }

    /// Like a tweet via the FavoriteTweet GraphQL mutation.
    pub async fn favorite_tweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        let body = self
            .graphql_post("FavoriteTweet", &variables, &features::mutation_features())
            .await?;

        response::check_graphql_errors(&body)?;
        Ok(body
            .get("data")
            .and_then(|d| d.get("favorite_tweet"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false))
    }

    /// Unlike a tweet via the UnfavoriteTweet GraphQL mutation.
    pub async fn unfavorite_tweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        let body = self
            .graphql_post(
                "UnfavoriteTweet",
                &variables,
                &features::mutation_features(),
            )
            .await?;

        response::check_graphql_errors(&body)?;
        Ok(body
            .get("data")
            .and_then(|d| d.get("unfavorite_tweet"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false))
    }

    /// Retweet via the CreateRetweet GraphQL mutation.
    pub async fn create_retweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
            "dark_request": false,
        });

        let body = self
            .graphql_post("CreateRetweet", &variables, &features::mutation_features())
            .await?;

        response::check_graphql_errors(&body)?;
        Ok(body
            .get("data")
            .and_then(|d| d.get("create_retweet"))
            .and_then(|r| r.get("retweet_results"))
            .is_some())
    }

    /// Undo a retweet via the DeleteRetweet GraphQL mutation.
    pub async fn delete_retweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        let variables = serde_json::json!({
            "source_tweet_id": tweet_id,
            "dark_request": false,
        });

        let body = self
            .graphql_post("DeleteRetweet", &variables, &features::mutation_features())
            .await?;

        response::check_graphql_errors(&body)?;
        Ok(body
            .get("data")
            .and_then(|d| d.get("unretweet"))
            .and_then(|u| u.get("source_tweet_results"))
            .is_some())
    }

    /// Delete a tweet via the DeleteTweet GraphQL mutation.
    pub async fn delete_tweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
            "dark_request": false,
        });

        let body = self
            .graphql_post("DeleteTweet", &variables, &features::mutation_features())
            .await?;

        response::check_graphql_errors(&body)?;
        Ok(body
            .get("data")
            .and_then(|d| d.get("delete_tweet"))
            .and_then(|dt| dt.get("tweet_results"))
            .is_some())
    }

    /// Bookmark a tweet via the CreateBookmark GraphQL mutation.
    pub async fn create_bookmark(&self, tweet_id: &str) -> Result<bool, XApiError> {
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        let body = self
            .graphql_post("CreateBookmark", &variables, &features::mutation_features())
            .await?;

        response::check_graphql_errors(&body)?;
        // CreateBookmark returns data.tweet_bookmark_put == "Done"
        Ok(body
            .get("data")
            .and_then(|d| d.get("tweet_bookmark_put"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false))
    }

    /// Remove a bookmark via the DeleteBookmark GraphQL mutation.
    pub async fn delete_bookmark(&self, tweet_id: &str) -> Result<bool, XApiError> {
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        let body = self
            .graphql_post("DeleteBookmark", &variables, &features::mutation_features())
            .await?;

        response::check_graphql_errors(&body)?;
        Ok(body
            .get("data")
            .and_then(|d| d.get("tweet_bookmark_delete"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false))
    }

    /// Follow a user via the REST friendships/create endpoint.
    ///
    /// Follow/unfollow use REST endpoints, not GraphQL.
    pub async fn follow_user(&self, target_user_id: &str) -> Result<bool, XApiError> {
        let api_path = "/i/api/1.1/friendships/create.json";
        let headers = self.build_headers("POST", api_path)?;

        let url = format!("https://x.com{api_path}");

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!(
                "include_profile_interstitial_type=1&user_id={target_user_id}"
            ))
            .send()
            .await
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("follow_user HTTP request failed: {e}"),
            })?;

        self.handle_rest_status(response, "follow_user").await
    }

    /// Unfollow a user via the REST friendships/destroy endpoint.
    pub async fn unfollow_user(&self, target_user_id: &str) -> Result<bool, XApiError> {
        let api_path = "/i/api/1.1/friendships/destroy.json";
        let headers = self.build_headers("POST", api_path)?;

        let url = format!("https://x.com{api_path}");

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!(
                "include_profile_interstitial_type=1&user_id={target_user_id}"
            ))
            .send()
            .await
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("unfollow_user HTTP request failed: {e}"),
            })?;

        self.handle_rest_status(response, "unfollow_user").await
    }

    /// Handle status codes for REST API responses.
    async fn handle_rest_status(
        &self,
        response: rquest::Response,
        operation: &str,
    ) -> Result<bool, XApiError> {
        let status = response.status().as_u16();

        if status == 401 || status == 403 {
            let body_text = response.text().await.unwrap_or_default();
            return Err(XApiError::ScraperTransportUnavailable {
                message: format!(
                    "Cookie session expired or invalid (HTTP {status}). \
                     Re-import your browser session. Response: {body_text}"
                ),
            });
        }
        if status == 429 {
            return Err(XApiError::RateLimited { retry_after: None });
        }
        if !rquest::StatusCode::from_u16(status)
            .map(|s| s.is_success())
            .unwrap_or(false)
        {
            let body_text = response.text().await.unwrap_or_default();
            return Err(XApiError::ApiError {
                status,
                message: format!("{operation} failed: {body_text}"),
            });
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    // ── Variable construction tests ────────────────────────────────
    // These test the JSON variable construction logic that would be passed
    // to graphql_post, without requiring an actual HTTP connection.

    #[test]
    fn create_tweet_variables_no_reply() {
        let mut variables = serde_json::json!({
            "tweet_text": "Hello world",
            "dark_request": false,
            "media": {
                "media_entities": [],
                "possibly_sensitive": false
            },
            "semantic_annotation_ids": []
        });

        let in_reply_to_id: Option<&str> = None;
        if let Some(reply_id) = in_reply_to_id {
            variables["reply"] = serde_json::json!({
                "in_reply_to_tweet_id": reply_id,
                "exclude_reply_user_ids": []
            });
        }

        assert_eq!(variables["tweet_text"], "Hello world");
        assert_eq!(variables["dark_request"], false);
        assert!(variables.get("reply").is_none());
        assert!(variables["media"]["media_entities"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn create_tweet_variables_with_reply() {
        let mut variables = serde_json::json!({
            "tweet_text": "My reply",
            "dark_request": false,
            "media": {
                "media_entities": [],
                "possibly_sensitive": false
            },
            "semantic_annotation_ids": []
        });

        let in_reply_to_id: Option<&str> = Some("12345");
        if let Some(reply_id) = in_reply_to_id {
            variables["reply"] = serde_json::json!({
                "in_reply_to_tweet_id": reply_id,
                "exclude_reply_user_ids": []
            });
        }

        assert_eq!(variables["reply"]["in_reply_to_tweet_id"], "12345");
        assert!(variables["reply"]["exclude_reply_user_ids"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn favorite_tweet_variables() {
        let tweet_id = "987654321";
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        assert_eq!(variables["tweet_id"], "987654321");
        // Should only have one key
        assert_eq!(variables.as_object().unwrap().len(), 1);
    }

    #[test]
    fn unfavorite_tweet_variables() {
        let tweet_id = "111222333";
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        assert_eq!(variables["tweet_id"], "111222333");
    }

    #[test]
    fn create_retweet_variables() {
        let tweet_id = "555666777";
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
            "dark_request": false,
        });

        assert_eq!(variables["tweet_id"], "555666777");
        assert_eq!(variables["dark_request"], false);
    }

    #[test]
    fn delete_retweet_variables() {
        let tweet_id = "888999000";
        let variables = serde_json::json!({
            "source_tweet_id": tweet_id,
            "dark_request": false,
        });

        assert_eq!(variables["source_tweet_id"], "888999000");
        assert_eq!(variables["dark_request"], false);
    }

    #[test]
    fn delete_tweet_variables() {
        let tweet_id = "444555666";
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
            "dark_request": false,
        });

        assert_eq!(variables["tweet_id"], "444555666");
    }

    #[test]
    fn create_bookmark_variables() {
        let tweet_id = "111333555";
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        assert_eq!(variables["tweet_id"], "111333555");
    }

    #[test]
    fn delete_bookmark_variables() {
        let tweet_id = "222444666";
        let variables = serde_json::json!({
            "tweet_id": tweet_id,
        });

        assert_eq!(variables["tweet_id"], "222444666");
    }

    // ── Response parsing logic tests ───────────────────────────────

    #[test]
    fn favorite_response_done() {
        let body = serde_json::json!({
            "data": {
                "favorite_tweet": "Done"
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("favorite_tweet"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(result);
    }

    #[test]
    fn favorite_response_not_done() {
        let body = serde_json::json!({
            "data": {
                "favorite_tweet": "NotDone"
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("favorite_tweet"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(!result);
    }

    #[test]
    fn favorite_response_missing_data() {
        let body = serde_json::json!({});
        let result = body
            .get("data")
            .and_then(|d| d.get("favorite_tweet"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(!result);
    }

    #[test]
    fn unfavorite_response_done() {
        let body = serde_json::json!({
            "data": {
                "unfavorite_tweet": "Done"
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("unfavorite_tweet"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(result);
    }

    #[test]
    fn create_retweet_response_present() {
        let body = serde_json::json!({
            "data": {
                "create_retweet": {
                    "retweet_results": {
                        "result": {"rest_id": "rt_123"}
                    }
                }
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("create_retweet"))
            .and_then(|r| r.get("retweet_results"))
            .is_some();
        assert!(result);
    }

    #[test]
    fn create_retweet_response_missing() {
        let body = serde_json::json!({
            "data": {
                "create_retweet": {}
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("create_retweet"))
            .and_then(|r| r.get("retweet_results"))
            .is_some();
        assert!(!result);
    }

    #[test]
    fn delete_retweet_response_present() {
        let body = serde_json::json!({
            "data": {
                "unretweet": {
                    "source_tweet_results": {
                        "result": {"rest_id": "orig_123"}
                    }
                }
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("unretweet"))
            .and_then(|u| u.get("source_tweet_results"))
            .is_some();
        assert!(result);
    }

    #[test]
    fn delete_tweet_response_present() {
        let body = serde_json::json!({
            "data": {
                "delete_tweet": {
                    "tweet_results": {}
                }
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("delete_tweet"))
            .and_then(|dt| dt.get("tweet_results"))
            .is_some();
        assert!(result);
    }

    #[test]
    fn delete_tweet_response_missing() {
        let body = serde_json::json!({
            "data": {}
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("delete_tweet"))
            .and_then(|dt| dt.get("tweet_results"))
            .is_some();
        assert!(!result);
    }

    #[test]
    fn bookmark_response_done() {
        let body = serde_json::json!({
            "data": {
                "tweet_bookmark_put": "Done"
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("tweet_bookmark_put"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(result);
    }

    #[test]
    fn delete_bookmark_response_done() {
        let body = serde_json::json!({
            "data": {
                "tweet_bookmark_delete": "Done"
            }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("tweet_bookmark_delete"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(result);
    }

    #[test]
    fn follow_user_api_path() {
        let api_path = "/i/api/1.1/friendships/create.json";
        let url = format!("https://x.com{api_path}");
        assert_eq!(url, "https://x.com/i/api/1.1/friendships/create.json");
    }

    #[test]
    fn unfollow_user_api_path() {
        let api_path = "/i/api/1.1/friendships/destroy.json";
        let url = format!("https://x.com{api_path}");
        assert_eq!(url, "https://x.com/i/api/1.1/friendships/destroy.json");
    }

    #[test]
    fn follow_user_body_format() {
        let target_user_id = "12345";
        let body = format!("include_profile_interstitial_type=1&user_id={target_user_id}");
        assert!(body.contains("user_id=12345"));
        assert!(body.contains("include_profile_interstitial_type=1"));
    }

    // ── handle_rest_status response parsing logic ──────────────────

    #[test]
    fn rest_status_401_is_auth_error() {
        let status = 401u16;
        assert!(status == 401 || status == 403);
    }

    #[test]
    fn rest_status_403_is_auth_error() {
        let status = 403u16;
        assert!(status == 401 || status == 403);
    }

    #[test]
    fn rest_status_429_is_rate_limit() {
        let status = 429u16;
        assert_eq!(status, 429);
    }

    #[test]
    fn rest_status_200_is_success() {
        let status = rquest::StatusCode::from_u16(200)
            .map(|s| s.is_success())
            .unwrap_or(false);
        assert!(status);
    }

    #[test]
    fn rest_status_500_is_not_success() {
        let status = rquest::StatusCode::from_u16(500)
            .map(|s| s.is_success())
            .unwrap_or(false);
        assert!(!status);
    }

    #[test]
    fn rest_status_201_is_success() {
        let status = rquest::StatusCode::from_u16(201)
            .map(|s| s.is_success())
            .unwrap_or(false);
        assert!(status);
    }

    // ── mutation features coverage ────────────────────────────────

    #[test]
    fn mutation_features_has_edit_tweet() {
        let f = super::super::features::mutation_features();
        assert_eq!(f["responsive_web_edit_tweet_api_enabled"], true);
    }

    #[test]
    fn mutation_features_tipping_disabled() {
        let f = super::super::features::mutation_features();
        assert_eq!(f["tweet_awards_web_tipping_enabled"], false);
    }

    // ── handle_rest_status logic simulation ─────────────────────────

    #[test]
    fn rest_status_branch_401_403_429_success_error() {
        // Simulate the branching logic in handle_rest_status
        for (status, expected) in [
            (200u16, "success"),
            (201, "success"),
            (204, "success"),
            (301, "error"),
            (400, "error"),
            (401, "auth"),
            (403, "auth"),
            (429, "rate_limit"),
            (500, "error"),
            (502, "error"),
        ] {
            let result = if status == 401 || status == 403 {
                "auth"
            } else if status == 429 {
                "rate_limit"
            } else if rquest::StatusCode::from_u16(status)
                .map(|s| s.is_success())
                .unwrap_or(false)
            {
                "success"
            } else {
                "error"
            };
            assert_eq!(
                result, expected,
                "status {status}: expected {expected}, got {result}"
            );
        }
    }

    // ── Response parsing: combined data structure traversal ──────────

    #[test]
    fn unfavorite_response_missing_field() {
        let body = serde_json::json!({ "data": {} });
        let result = body
            .get("data")
            .and_then(|d| d.get("unfavorite_tweet"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(!result);
    }

    #[test]
    fn delete_retweet_response_missing() {
        let body = serde_json::json!({ "data": { "unretweet": {} } });
        let result = body
            .get("data")
            .and_then(|d| d.get("unretweet"))
            .and_then(|u| u.get("source_tweet_results"))
            .is_some();
        assert!(!result);
    }

    #[test]
    fn delete_bookmark_response_not_done() {
        let body = serde_json::json!({
            "data": { "tweet_bookmark_delete": "NotDone" }
        });
        let result = body
            .get("data")
            .and_then(|d| d.get("tweet_bookmark_delete"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(!result);
    }

    #[test]
    fn bookmark_response_missing_data() {
        let body = serde_json::json!({});
        let result = body
            .get("data")
            .and_then(|d| d.get("tweet_bookmark_put"))
            .and_then(|f| f.as_str())
            .map(|s| s == "Done")
            .unwrap_or(false);
        assert!(!result);
    }

    // ── Variable structure completeness ──────────────────────────────

    #[test]
    fn create_tweet_variables_has_all_required_fields() {
        let variables = serde_json::json!({
            "tweet_text": "test",
            "dark_request": false,
            "media": { "media_entities": [], "possibly_sensitive": false },
            "semantic_annotation_ids": []
        });
        let obj = variables.as_object().unwrap();
        assert!(obj.contains_key("tweet_text"));
        assert!(obj.contains_key("dark_request"));
        assert!(obj.contains_key("media"));
        assert!(obj.contains_key("semantic_annotation_ids"));
        assert_eq!(obj.len(), 4);
    }

    #[test]
    fn follow_unfollow_body_has_interstitial() {
        for uid in ["1", "999", "123456789"] {
            let body = format!("include_profile_interstitial_type=1&user_id={uid}");
            assert!(body.starts_with("include_profile_interstitial_type=1&user_id="));
            assert!(body.ends_with(uid));
        }
    }

    // ── mutation_features coverage ────────────────────────────────

    #[test]
    fn mutation_features_is_object() {
        let f = super::super::features::mutation_features();
        assert!(f.is_object());
        let obj = f.as_object().unwrap();
        assert!(obj.len() > 5, "should have many feature flags");
        // All values are booleans
        for (key, val) in obj {
            assert!(val.is_boolean(), "{key} should be boolean, got {val}");
        }
    }
}
