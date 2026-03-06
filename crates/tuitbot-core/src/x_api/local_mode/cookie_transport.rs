//! Cookie-based HTTP transport for X's internal GraphQL API.
//!
//! Uses browser cookies (`auth_token` + `ct0`) to authenticate with X's
//! web API endpoints. This is the transport layer that enables posting
//! tweets without official API credentials.

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::error::XApiError;
use crate::x_api::types::PostedTweet;

use super::session::ScraperSession;

/// Static bearer token used by X's web client.
///
/// This token is public and embedded in X's JavaScript bundles. It
/// identifies the "web app" client — cookie auth provides the user identity.
const WEB_BEARER_TOKEN: &str = "AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA";

/// Base URL for X's internal API.
const X_API_BASE: &str = "https://x.com/i/api";

/// GraphQL query ID for CreateTweet mutation.
///
/// This ID changes with X web deploys. If requests start returning 404,
/// the ID needs updating. Override via `TUITBOT_CREATE_TWEET_QUERY_ID` env var.
const DEFAULT_CREATE_TWEET_QUERY_ID: &str = "znCbxBEHRELYLRSELA_J1g";

/// HTTP transport using cookie-based authentication.
pub struct CookieTransport {
    client: reqwest::Client,
    session: ScraperSession,
    create_tweet_query_id: String,
}

impl CookieTransport {
    /// Create a new cookie transport from a loaded session.
    pub fn new(session: ScraperSession) -> Self {
        let query_id = std::env::var("TUITBOT_CREATE_TWEET_QUERY_ID")
            .unwrap_or_else(|_| DEFAULT_CREATE_TWEET_QUERY_ID.to_string());

        Self {
            client: reqwest::Client::new(),
            session,
            create_tweet_query_id: query_id,
        }
    }

    /// Build the common headers for X internal API requests.
    fn build_headers(&self) -> Result<HeaderMap, XApiError> {
        let mut headers = HeaderMap::new();

        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {WEB_BEARER_TOKEN}")).map_err(|e| {
                XApiError::ScraperTransportUnavailable {
                    message: format!("invalid bearer header: {e}"),
                }
            })?,
        );

        headers.insert(
            "cookie",
            HeaderValue::from_str(&format!(
                "auth_token={}; ct0={}",
                self.session.auth_token, self.session.ct0
            ))
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("invalid cookie header: {e}"),
            })?,
        );

        headers.insert(
            "x-csrf-token",
            HeaderValue::from_str(&self.session.ct0).map_err(|e| {
                XApiError::ScraperTransportUnavailable {
                    message: format!("invalid csrf header: {e}"),
                }
            })?,
        );

        headers.insert(
            "x-twitter-auth-type",
            HeaderValue::from_static("OAuth2Session"),
        );
        headers.insert("x-twitter-active-user", HeaderValue::from_static("yes"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    /// Post a new tweet via the GraphQL CreateTweet mutation.
    pub async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        self.create_tweet(text, None).await
    }

    /// Reply to a tweet via the GraphQL CreateTweet mutation.
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
        let headers = self.build_headers()?;

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

        let body = CreateTweetRequest {
            variables,
            features: default_features(),
            query_id: self.create_tweet_query_id.clone(),
        };

        let url = format!(
            "{X_API_BASE}/graphql/{}/CreateTweet",
            self.create_tweet_query_id
        );

        tracing::debug!(url = %url, "Sending CreateTweet via cookie transport");

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

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

        if !response.status().is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(XApiError::ApiError {
                status,
                message: format!("CreateTweet failed: {body_text}"),
            });
        }

        let resp_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        parse_create_tweet_response(&resp_body)
    }
}

/// Parse the GraphQL CreateTweet response to extract the posted tweet.
fn parse_create_tweet_response(body: &serde_json::Value) -> Result<PostedTweet, XApiError> {
    // Check for GraphQL-level errors first.
    if let Some(errors) = body.get("errors").and_then(|e| e.as_array()) {
        if let Some(first) = errors.first() {
            let msg = first
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown GraphQL error");
            return Err(XApiError::ApiError {
                status: 0,
                message: format!("CreateTweet GraphQL error: {msg}"),
            });
        }
    }

    // Navigate: data.create_tweet.tweet_results.result
    let result = body
        .get("data")
        .and_then(|d| d.get("create_tweet"))
        .and_then(|ct| ct.get("tweet_results"))
        .and_then(|tr| tr.get("result"))
        .ok_or_else(|| XApiError::ApiError {
            status: 0,
            message: format!(
                "unexpected CreateTweet response structure: {}",
                serde_json::to_string(body).unwrap_or_default()
            ),
        })?;

    let tweet_id = result
        .get("rest_id")
        .and_then(|id| id.as_str())
        .unwrap_or_default()
        .to_string();

    let text = result
        .get("legacy")
        .and_then(|l| l.get("full_text"))
        .and_then(|t| t.as_str())
        .unwrap_or_default()
        .to_string();

    if tweet_id.is_empty() {
        return Err(XApiError::ApiError {
            status: 0,
            message: "CreateTweet returned no tweet ID".to_string(),
        });
    }

    Ok(PostedTweet { id: tweet_id, text })
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CreateTweetRequest {
    variables: serde_json::Value,
    features: CreateTweetFeatures,
    #[serde(rename = "queryId")]
    query_id: String,
}

/// Feature flags required by the CreateTweet GraphQL mutation.
///
/// These are the minimum set needed for the mutation to succeed.
/// X periodically adds new required features; if requests start failing
/// with feature-related errors, new fields may need to be added here.
#[derive(Serialize)]
struct CreateTweetFeatures {
    communities_web_enable_tweet_community_results_fetch: bool,
    c9s_tweet_anatomy_moderator_badge_enabled: bool,
    tweetypie_unmention_optimization_enabled: bool,
    responsive_web_edit_tweet_api_enabled: bool,
    graphql_is_translatable_rweb_tweet_is_translatable_enabled: bool,
    view_counts_everywhere_api_enabled: bool,
    longform_notetweets_consumption_enabled: bool,
    responsive_web_twitter_article_tweet_consumption_enabled: bool,
    tweet_awards_web_tipping_enabled: bool,
    longform_notetweets_rich_text_read_enabled: bool,
    longform_notetweets_inline_media_enabled: bool,
    responsive_web_graphql_exclude_directive_enabled: bool,
    verified_phone_label_enabled: bool,
    freedom_of_speech_not_reach_fetch_enabled: bool,
    standardized_nudges_misinfo: bool,
    tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled: bool,
    responsive_web_media_download_video_enabled: bool,
    responsive_web_graphql_skip_user_profile_image_extensions_enabled: bool,
    responsive_web_graphql_timeline_navigation_enabled: bool,
    responsive_web_enhance_cards_enabled: bool,
}

fn default_features() -> CreateTweetFeatures {
    CreateTweetFeatures {
        communities_web_enable_tweet_community_results_fetch: true,
        c9s_tweet_anatomy_moderator_badge_enabled: true,
        tweetypie_unmention_optimization_enabled: true,
        responsive_web_edit_tweet_api_enabled: true,
        graphql_is_translatable_rweb_tweet_is_translatable_enabled: true,
        view_counts_everywhere_api_enabled: true,
        longform_notetweets_consumption_enabled: true,
        responsive_web_twitter_article_tweet_consumption_enabled: true,
        tweet_awards_web_tipping_enabled: false,
        longform_notetweets_rich_text_read_enabled: true,
        longform_notetweets_inline_media_enabled: true,
        responsive_web_graphql_exclude_directive_enabled: true,
        verified_phone_label_enabled: false,
        freedom_of_speech_not_reach_fetch_enabled: true,
        standardized_nudges_misinfo: true,
        tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled: true,
        responsive_web_media_download_video_enabled: false,
        responsive_web_graphql_skip_user_profile_image_extensions_enabled: false,
        responsive_web_graphql_timeline_navigation_enabled: true,
        responsive_web_enhance_cards_enabled: false,
    }
}

// ---------------------------------------------------------------------------
// Response types (for deserialization)
// ---------------------------------------------------------------------------

/// Minimal GraphQL error shape.
#[derive(Deserialize)]
struct _GraphQLError {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_successful_create_tweet() {
        let body = serde_json::json!({
            "data": {
                "create_tweet": {
                    "tweet_results": {
                        "result": {
                            "rest_id": "1234567890",
                            "legacy": {
                                "full_text": "Hello world"
                            }
                        }
                    }
                }
            }
        });
        let result = parse_create_tweet_response(&body).unwrap();
        assert_eq!(result.id, "1234567890");
        assert_eq!(result.text, "Hello world");
    }

    #[test]
    fn parse_graphql_error() {
        let body = serde_json::json!({
            "errors": [{"message": "Rate limit exceeded"}]
        });
        let err = parse_create_tweet_response(&body).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Rate limit exceeded"), "got: {msg}");
    }

    #[test]
    fn parse_missing_result() {
        let body = serde_json::json!({"data": {}});
        let err = parse_create_tweet_response(&body).unwrap_err();
        assert!(err.to_string().contains("unexpected"), "got: {}", err);
    }

    #[test]
    fn parse_empty_tweet_id() {
        let body = serde_json::json!({
            "data": {
                "create_tweet": {
                    "tweet_results": {
                        "result": {
                            "rest_id": "",
                            "legacy": {"full_text": "hi"}
                        }
                    }
                }
            }
        });
        let err = parse_create_tweet_response(&body).unwrap_err();
        assert!(err.to_string().contains("no tweet ID"));
    }

    #[test]
    fn default_features_serializes() {
        let features = default_features();
        let json = serde_json::to_value(features).unwrap();
        assert!(json.is_object());
        assert_eq!(
            json["tweet_awards_web_tipping_enabled"],
            serde_json::Value::Bool(false)
        );
    }
}
