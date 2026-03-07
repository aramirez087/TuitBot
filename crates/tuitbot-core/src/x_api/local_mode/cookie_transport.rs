//! Cookie-based HTTP transport for X's internal GraphQL API.
//!
//! Uses browser cookies (`auth_token` + `ct0`) to authenticate with X's
//! web API endpoints. This is the transport layer that enables posting
//! tweets without official API credentials.

use rand::RngCore;
use rquest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use x_client_transaction::ClientTransaction;

use crate::error::XApiError;
use crate::x_api::types::PostedTweet;

use super::session::ScraperSession;

/// Static bearer token used by X's web client.
///
/// This token is public and embedded in X's JavaScript bundles. It
/// identifies the "web app" client — cookie auth provides the user identity.
const WEB_BEARER_TOKEN: &str = "AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA";

/// Fallback GraphQL query ID for CreateTweet mutation.
///
/// X rotates this ID with web deploys. On startup we attempt to resolve
/// the current ID from X's JS bundles (`resolve_create_tweet_query_id`).
/// This constant is only used when auto-detection fails.
const FALLBACK_CREATE_TWEET_QUERY_ID: &str = "uY34Pldm6W89yqswRmPMSQ";

/// Build an `rquest::Client` that impersonates Chrome's TLS fingerprint.
///
/// X uses TLS fingerprinting (JA3/JA4) to detect non-browser clients.
/// `rquest` reproduces Chrome's exact TLS handshake, HTTP/2 settings,
/// and header order so the connection is indistinguishable from a real browser.
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
/// Generates valid `x-client-transaction-id` headers using the algorithm
/// extracted from X's web client (animation key + SHA256 + XOR).
pub struct CookieTransport {
    client: rquest::Client,
    session: ScraperSession,
    create_tweet_query_id: String,
    /// Persistent client UUID (mimics X web client's localStorage UUID).
    client_uuid: String,
    /// Transaction ID generator (extracted from X's homepage on init).
    /// `None` if the homepage couldn't be parsed — falls back to random IDs.
    transaction: Option<ClientTransaction>,
}

impl CookieTransport {
    /// Create a new cookie transport from a loaded session.
    ///
    /// Prefer `with_query_id` + `resolve_create_tweet_query_id` which
    /// auto-detects the current CreateTweet query ID from X's web client.
    pub fn new(session: ScraperSession) -> Self {
        let query_id = std::env::var("TUITBOT_CREATE_TWEET_QUERY_ID")
            .unwrap_or_else(|_| FALLBACK_CREATE_TWEET_QUERY_ID.to_string());

        Self {
            client: build_browser_client(),
            session,
            create_tweet_query_id: query_id,
            client_uuid: generate_client_uuid(),
            transaction: None,
        }
    }

    /// Create a transport with a pre-resolved query ID and transaction generator.
    pub fn with_query_id(
        session: ScraperSession,
        query_id: String,
        transaction: Option<ClientTransaction>,
    ) -> Self {
        Self {
            client: build_browser_client(),
            session,
            create_tweet_query_id: query_id,
            client_uuid: generate_client_uuid(),
            transaction,
        }
    }

    /// Build the common headers for X internal API requests.
    ///
    /// `method` and `path` are used to generate a valid `x-client-transaction-id`.
    fn build_headers(&self, method: &str, path: &str) -> Result<HeaderMap, XApiError> {
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

        // Override the emulation's navigation-style defaults with XHR/fetch values.
        // The emulation sets sec-fetch-dest=document, sec-fetch-mode=navigate, etc.
        // which are wrong for an API POST — real browsers use these values for fetch().
        headers.insert("accept", HeaderValue::from_static("*/*"));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));

        // Anti-automation headers: X's web client sends these with every request.
        // `x-client-uuid` is a persistent UUID stored in the browser's localStorage.
        headers.insert(
            "x-client-uuid",
            HeaderValue::from_str(&self.client_uuid).map_err(|e| {
                XApiError::ScraperTransportUnavailable {
                    message: format!("invalid client-uuid header: {e}"),
                }
            })?,
        );

        // `x-client-transaction-id` is a cryptographic hash derived from X's
        // homepage animation data, the HTTP method/path, and current time.
        // If we have a valid ClientTransaction, generate the real value;
        // otherwise fall back to a random one (which X may reject).
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
        let api_path = format!("/i/api/graphql/{}/CreateTweet", self.create_tweet_query_id);
        let headers = self.build_headers("POST", &api_path)?;

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

        let url = format!("https://x.com{api_path}");

        tracing::debug!(url = %url, "Sending CreateTweet via cookie transport");

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| XApiError::ScraperTransportUnavailable {
                message: format!("HTTP request failed: {e}"),
            })?;

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

        let resp_body: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| XApiError::ScraperTransportUnavailable {
                    message: format!("failed to parse response JSON: {e}"),
                })?;

        parse_create_tweet_response(&resp_body)
    }

    /// Fetch the authenticated viewer's profile via the legacy verify_credentials endpoint.
    ///
    /// Returns a `User` with id, username, display name, and avatar URL.
    pub async fn fetch_viewer(&self) -> Result<crate::x_api::types::User, XApiError> {
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
            // X returns "_normal" size by default; replace for higher quality.
            .map(|u| u.replace("_normal.", "_400x400."))
            .filter(|u| !u.is_empty());

        if id.is_empty() {
            return Err(XApiError::ApiError {
                status: 0,
                message: "verify_credentials returned no user ID".to_string(),
            });
        }

        Ok(crate::x_api::types::User {
            id,
            username,
            name,
            profile_image_url,
            public_metrics: Default::default(),
        })
    }
}

/// Result of startup resolution: query ID + transaction generator.
pub struct ResolvedTransport {
    /// The CreateTweet GraphQL query ID.
    pub query_id: String,
    /// Transaction ID generator (extracted from X's homepage).
    /// `None` if initialization failed — requests will omit the header.
    pub transaction: Option<ClientTransaction>,
}

/// Resolve the current CreateTweet query ID and initialize the transaction
/// ID generator from X's web client JS bundles and homepage.
///
/// 1. Fetches the X homepage to discover `<script>` bundle URLs.
/// 2. Fetches each bundle and searches for the `CreateTweet` operation.
/// 3. Extracts the query ID from the surrounding pattern.
/// 4. Initializes the `ClientTransaction` for valid `x-client-transaction-id`.
///
/// Returns the env-var override if set, falls back to the hardcoded default
/// if auto-detection fails. Logs the outcome at info/warn level.
pub async fn resolve_transport() -> ResolvedTransport {
    let query_id = resolve_query_id().await;
    let transaction = resolve_client_transaction().await;

    ResolvedTransport {
        query_id,
        transaction,
    }
}

/// Resolve the CreateTweet query ID from X's JS bundles.
async fn resolve_query_id() -> String {
    // Env-var override always wins.
    if let Ok(id) = std::env::var("TUITBOT_CREATE_TWEET_QUERY_ID") {
        tracing::info!(query_id = %id, "Using CreateTweet query ID from env var");
        return id;
    }

    match detect_query_id_from_bundles().await {
        Ok(id) => {
            tracing::info!(query_id = %id, "Auto-detected CreateTweet query ID from X web client");
            id
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                fallback = FALLBACK_CREATE_TWEET_QUERY_ID,
                "Failed to auto-detect CreateTweet query ID, using fallback"
            );
            FALLBACK_CREATE_TWEET_QUERY_ID.to_string()
        }
    }
}

/// Initialize the `ClientTransaction` for generating valid transaction IDs.
///
/// Uses `reqwest::blocking` internally (via the `x-client-transaction` crate),
/// so this runs on a blocking thread via `spawn_blocking`.
async fn resolve_client_transaction() -> Option<ClientTransaction> {
    match tokio::task::spawn_blocking(|| {
        let client = reqwest::blocking::Client::builder()
            .user_agent(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
                 AppleWebKit/537.36 (KHTML, like Gecko) \
                 Chrome/134.0.0.0 Safari/537.36",
            )
            .build()
            .ok()?;
        ClientTransaction::new(&client).ok()
    })
    .await
    {
        Ok(Some(ct)) => {
            tracing::info!("Initialized x-client-transaction-id generator");
            Some(ct)
        }
        Ok(None) => {
            tracing::warn!(
                "Failed to initialize x-client-transaction-id generator; \
                 requests will omit the header"
            );
            None
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "spawn_blocking for ClientTransaction panicked"
            );
            None
        }
    }
}

/// Fetch X's homepage, find JS bundle URLs, and extract the CreateTweet query ID.
async fn detect_query_id_from_bundles() -> Result<String, String> {
    let client = rquest::Client::builder()
        .emulation(rquest_util::Emulation::Chrome134)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    // Step 1: Fetch the X homepage to find JS bundle URLs.
    let html = client
        .get("https://x.com")
        .send()
        .await
        .map_err(|e| format!("failed to fetch x.com: {e}"))?
        .text()
        .await
        .map_err(|e| format!("failed to read x.com response: {e}"))?;

    // Extract script src URLs from the HTML.
    let script_urls: Vec<String> = extract_script_urls(&html);

    if script_urls.is_empty() {
        return Err("no JS bundle URLs found in x.com HTML".to_string());
    }

    tracing::debug!(count = script_urls.len(), "Found JS bundle URLs to scan");

    // Step 2: Fetch each bundle and search for the CreateTweet query ID.
    // X typically has ~10-15 bundles; the query ID is in one of the main API bundles.
    for url in &script_urls {
        let js = match client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if let Some(id) = extract_query_id_for_operation(&js, "CreateTweet") {
            return Ok(id);
        }
    }

    Err("CreateTweet query ID not found in any JS bundle".to_string())
}

/// Extract `<script src="...">` URLs from HTML that look like X's JS bundles.
fn extract_script_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    // Match src="..." in script tags. X uses absolute paths like
    // "https://abs.twimg.com/responsive-web/client-web/main.XXXX.js"
    for segment in html.split("src=\"") {
        if let Some(end) = segment.find('"') {
            let url = &segment[..end];
            if url.ends_with(".js") && (url.contains("twimg.com") || url.contains("x.com")) {
                urls.push(url.to_string());
            }
        }
    }
    urls
}

/// Search a JS bundle for a GraphQL operation and extract its query ID.
///
/// X's bundles contain patterns like:
/// - `queryId:"abc123",operationName:"CreateTweet"`
/// - `{queryId:"abc123",...,operationName:"CreateTweet"}`
fn extract_query_id_for_operation(js: &str, operation: &str) -> Option<String> {
    // Find the operation name in the JS source.
    let op_pattern = format!("\"{}\"", operation);

    // Search for queryId near the operation name.
    // The queryId and operationName are usually within the same object literal,
    // so we search a window around each occurrence of the operation name.
    for (idx, _) in js.match_indices(&op_pattern) {
        // Look backwards from the match (up to 200 chars) for a queryId.
        let start = idx.saturating_sub(200);
        let window = &js[start..idx];

        if let Some(id) = extract_query_id_value(window) {
            return Some(id);
        }

        // Also look forward (the queryId might come after operationName).
        let end = (idx + op_pattern.len() + 200).min(js.len());
        let window = &js[idx..end];

        if let Some(id) = extract_query_id_value(window) {
            return Some(id);
        }
    }
    None
}

/// Extract a queryId value from a JS snippet like `queryId:"abc123"`.
fn extract_query_id_value(snippet: &str) -> Option<String> {
    let marker = "queryId:\"";
    let pos = snippet.rfind(marker)?;
    let after = &snippet[pos + marker.len()..];
    let end = after.find('"')?;
    let id = &after[..end];
    // Sanity: query IDs are short alphanumeric+underscore+hyphen strings.
    if !id.is_empty()
        && id.len() < 64
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Some(id.to_string())
    } else {
        None
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

    // --- Query ID extraction tests ---

    #[test]
    fn extract_query_id_before_operation_name() {
        let js = r#"e.exports={queryId:"uY34Pldm6W89yqswRmPMSQ",operationName:"CreateTweet",operationType:"mutation"}"#;
        assert_eq!(
            extract_query_id_for_operation(js, "CreateTweet"),
            Some("uY34Pldm6W89yqswRmPMSQ".to_string())
        );
    }

    #[test]
    fn extract_query_id_after_operation_name() {
        let js = r#"{operationName:"CreateTweet",queryId:"abc123XYZ"}"#;
        assert_eq!(
            extract_query_id_for_operation(js, "CreateTweet"),
            Some("abc123XYZ".to_string())
        );
    }

    #[test]
    fn extract_query_id_returns_none_for_missing_operation() {
        let js = r#"{queryId:"abc123",operationName:"DeleteTweet"}"#;
        assert_eq!(extract_query_id_for_operation(js, "CreateTweet"), None);
    }

    #[test]
    fn extract_script_urls_finds_twimg_bundles() {
        let html = r#"<script src="https://abs.twimg.com/responsive-web/client-web/main.abc123.js"></script>"#;
        let urls = extract_script_urls(html);
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("twimg.com"));
    }

    #[test]
    fn extract_script_urls_ignores_non_js() {
        let html = r#"<script src="https://abs.twimg.com/data.json"></script>"#;
        let urls = extract_script_urls(html);
        assert!(urls.is_empty());
    }

    #[test]
    fn extract_query_id_value_basic() {
        assert_eq!(
            extract_query_id_value(r#"queryId:"hello123""#),
            Some("hello123".to_string())
        );
    }

    #[test]
    fn extract_query_id_value_rejects_empty() {
        assert_eq!(extract_query_id_value(r#"queryId:"""#), None);
    }
}
