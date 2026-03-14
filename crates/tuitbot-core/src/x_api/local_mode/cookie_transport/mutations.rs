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
