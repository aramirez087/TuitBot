//! Cookie-based HTTP transport for X's internal GraphQL API.
//!
//! Uses browser cookies (`auth_token` + `ct0`) to authenticate with X's
//! web API endpoints. This is the transport layer that enables GraphQL
//! reads and mutations without official API credentials.

pub mod features;
pub mod mutations;
pub mod queries;
mod resolve;
pub mod response;

use std::collections::HashMap;

use rand::RngCore;
use rquest::header::{HeaderMap, HeaderValue};

use crate::error::XApiError;
use crate::x_api::types::User;
use crate::x_client_transaction::ClientTransaction;

use super::session::ScraperSession;

/// Static bearer token used by X's web client.
///
/// This token is public and embedded in X's JavaScript bundles. It
/// identifies the "web app" client — cookie auth provides the user identity.
const WEB_BEARER_TOKEN: &str = "AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA";

/// Fallback GraphQL query ID for CreateTweet mutation.
///
/// X rotates this ID with web deploys. On startup we attempt to resolve
/// the current ID from X's JS bundles. This constant is only used when
/// auto-detection fails.
const FALLBACK_CREATE_TWEET_QUERY_ID: &str = "uY34Pldm6W89yqswRmPMSQ";

/// All GraphQL operations whose query IDs we attempt to resolve at startup.
const OPERATION_NAMES: &[&str] = &[
    "CreateTweet",
    "DeleteTweet",
    "FavoriteTweet",
    "UnfavoriteTweet",
    "CreateRetweet",
    "DeleteRetweet",
    "CreateBookmark",
    "DeleteBookmark",
    "SearchTimeline",
    "TweetResultByRestId",
    "UserByScreenName",
    "UserByRestId",
    "UserTweets",
    "Followers",
    "Following",
    "Likes",
    "HomeLatestTimeline",
    "Bookmarks",
];

/// Build an `rquest::Client` that impersonates Chrome's TLS fingerprint.
fn build_browser_client() -> rquest::Client {
    rquest::Client::builder()
        .emulation(rquest_util::Emulation::Chrome134)
        .build()
        .unwrap_or_else(|_| rquest::Client::new())
}

/// Generate a random UUID v4 string for `x-client-uuid`.
fn generate_client_uuid() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    buf[6] = (buf[6] & 0x0f) | 0x40;
    buf[8] = (buf[8] & 0x3f) | 0x80;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
        u16::from_be_bytes([buf[4], buf[5]]),
        u16::from_be_bytes([buf[6], buf[7]]),
        u16::from_be_bytes([buf[8], buf[9]]),
        u64::from_be_bytes([0, 0, buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]]),
    )
}

/// HTTP transport using cookie-based authentication.
///
/// Uses `rquest` (not `reqwest`) to impersonate Chrome's TLS fingerprint
/// so that X's anti-automation checks (JA3/JA4 fingerprinting) pass.
pub struct CookieTransport {
    client: rquest::Client,
    session: ScraperSession,
    /// GraphQL query IDs resolved from X's JS bundles at startup.
    query_ids: HashMap<String, String>,
    /// Persistent client UUID (mimics X web client's localStorage UUID).
    client_uuid: String,
    /// Transaction ID generator (extracted from X's homepage on init).
    /// `None` if the homepage couldn't be parsed — falls back to random IDs.
    transaction: Option<ClientTransaction>,
}

impl CookieTransport {
    /// Create a new cookie transport from a loaded session.
    ///
    /// Prefer `with_resolved_transport` + `resolve_transport` which
    /// auto-detects all query IDs from X's web client.
    pub fn new(session: ScraperSession) -> Self {
        let mut query_ids = HashMap::new();
        let create_tweet_id = std::env::var("TUITBOT_CREATE_TWEET_QUERY_ID")
            .unwrap_or_else(|_| FALLBACK_CREATE_TWEET_QUERY_ID.to_string());
        query_ids.insert("CreateTweet".to_string(), create_tweet_id);

        Self {
            client: build_browser_client(),
            session,
            query_ids,
            client_uuid: generate_client_uuid(),
            transaction: None,
        }
    }

    /// Create a transport with pre-resolved query IDs and transaction generator.
    pub(crate) fn with_resolved_transport(
        session: ScraperSession,
        query_ids: HashMap<String, String>,
        transaction: Option<ClientTransaction>,
    ) -> Self {
        Self {
            client: build_browser_client(),
            session,
            query_ids,
            client_uuid: generate_client_uuid(),
            transaction,
        }
    }

    /// Get the query ID for an operation, returning a clear error if missing.
    fn get_query_id(&self, operation: &str) -> Result<&str, XApiError> {
        self.query_ids
            .get(operation)
            .map(|s| s.as_str())
            .ok_or_else(|| XApiError::ScraperTransportUnavailable {
                message: format!(
                    "{operation}: query ID not found. X may have changed its JS bundles. \
                     Try restarting the application."
                ),
            })
    }

    /// Build the common headers for X internal API requests.
    pub(crate) fn build_headers(&self, method: &str, path: &str) -> Result<HeaderMap, XApiError> {
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
        headers.insert("x-twitter-client-language", HeaderValue::from_static("en"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("origin", HeaderValue::from_static("https://x.com"));
        headers.insert("referer", HeaderValue::from_static("https://x.com/"));

        headers.insert("accept", HeaderValue::from_static("*/*"));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));

        headers.insert(
            "x-client-uuid",
            HeaderValue::from_str(&self.client_uuid).map_err(|e| {
                XApiError::ScraperTransportUnavailable {
                    message: format!("invalid client-uuid header: {e}"),
                }
            })?,
        );

        if let Some(ref txn) = self.transaction {
            match txn.generate_transaction_id(method, path) {
                Ok(txn_id) => {
                    headers.insert(
                        "x-client-transaction-id",
                        HeaderValue::from_str(&txn_id).map_err(|e| {
                            XApiError::ScraperTransportUnavailable {
                                message: format!("invalid transaction-id header: {e}"),
                            }
                        })?,
                    );
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to generate transaction ID, skipping header");
                }
            }
        }

        Ok(headers)
    }

    /// Issue a GraphQL GET request (for read operations).
    pub(crate) async fn graphql_get(
        &self,
        operation: &str,
        variables: &serde_json::Value,
        features: &serde_json::Value,
    ) -> Result<serde_json::Value, XApiError> {
        let query_id = self.get_query_id(operation)?;
        let api_path = format!("/i/api/graphql/{query_id}/{operation}");
        let headers = self.build_headers("GET", &api_path)?;
        let url = format!("https://x.com{api_path}");

        let variables_str = serde_json::to_string(variables).map_err(|e| {
            XApiError::ScraperTransportUnavailable {
                message: format!("failed to serialize variables: {e}"),
            }
        })?;
        let features_str = serde_json::to_string(features).map_err(|e| {
            XApiError::ScraperTransportUnavailable {
                message: format!("failed to serialize features: {e}"),
            }
        })?;

        tracing::debug!(url = %url, "GraphQL GET {operation}");

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&[
                ("variables", variables_str.as_str()),
                ("features", features_str.as_str()),
            ])
            .send()
            .await
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("{operation} HTTP request failed: {e}"),
            })?;

        self.handle_response(operation, response).await
    }

    /// Issue a GraphQL POST request (for mutations).
    pub(crate) async fn graphql_post(
        &self,
        operation: &str,
        variables: &serde_json::Value,
        features: &serde_json::Value,
    ) -> Result<serde_json::Value, XApiError> {
        let query_id = self.get_query_id(operation)?;
        let api_path = format!("/i/api/graphql/{query_id}/{operation}");
        let headers = self.build_headers("POST", &api_path)?;
        let url = format!("https://x.com{api_path}");

        let body = serde_json::json!({
            "variables": variables,
            "features": features,
            "queryId": query_id,
        });

        tracing::debug!(url = %url, "GraphQL POST {operation}");

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("{operation} HTTP request failed: {e}"),
            })?;

        self.handle_response(operation, response).await
    }

    /// Shared response handler for GraphQL requests.
    async fn handle_response(
        &self,
        operation: &str,
        response: rquest::Response,
    ) -> Result<serde_json::Value, XApiError> {
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
                message: format!("{operation} failed: {body_text}"),
            });
        }

        response
            .json()
            .await
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("failed to parse {operation} response JSON: {e}"),
            })
    }

    /// Fetch the authenticated viewer's profile via verify_credentials.
    pub async fn fetch_viewer(&self) -> Result<User, XApiError> {
        let api_path = "/i/api/1.1/account/verify_credentials.json";
        let headers = self.build_headers("GET", api_path)?;

        let url = format!("https://x.com{api_path}?include_email=false&skip_status=true");

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("fetch_viewer HTTP request failed: {e}"),
            })?;

        let status = response.status().as_u16();
        if status == 401 || status == 403 {
            return Err(XApiError::ScraperTransportUnavailable {
                message: "Cookie session expired or invalid. Re-import your browser session."
                    .to_string(),
            });
        }
        if !response.status().is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(XApiError::ApiError {
                status,
                message: format!("verify_credentials failed: {body_text}"),
            });
        }

        let body: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| XApiError::ScraperTransportUnavailable {
                    message: format!("failed to parse verify_credentials JSON: {e}"),
                })?;

        let id = body
            .get("id_str")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let username = body
            .get("screen_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let name = body
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let profile_image_url = body
            .get("profile_image_url_https")
            .and_then(|v| v.as_str())
            .map(|u| u.replace("_normal.", "_400x400."))
            .filter(|u| !u.is_empty());

        if id.is_empty() {
            return Err(XApiError::ApiError {
                status: 0,
                message: "verify_credentials returned no user ID".to_string(),
            });
        }

        Ok(User {
            id,
            username,
            name,
            profile_image_url,
            description: None,
            location: None,
            url: None,
            public_metrics: Default::default(),
        })
    }
}

// Re-export resolution function (used by local_mode/mod.rs).
pub(crate) use resolve::resolve_transport;

// Re-export query ID extraction helpers for tests.
#[cfg(test)]
pub(crate) use resolve::{
    extract_query_id_for_operation, extract_query_id_value, extract_script_urls,
};

#[cfg(test)]
mod tests;
