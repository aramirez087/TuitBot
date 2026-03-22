//! Local-mode X API client for scraper backend.
//!
//! Implements `XApiClient` without requiring OAuth credentials.
//! When a browser session file (`scraper_session.json`) is present,
//! read and write methods use cookie-based authentication via X's
//! internal GraphQL API. Without a session, write methods return
//! `ScraperTransportUnavailable`.

pub mod cookie_transport;
pub mod session;

use std::path::{Path, PathBuf};

use crate::error::XApiError;
use crate::x_api::retry::{retry_with_backoff, RetryConfig};
use crate::x_api::scraper_health::{new_scraper_health, ScraperHealth};
use crate::x_api::types::{
    MediaId, MediaType, MentionResponse, PostedTweet, RawApiResponse, SearchResponse, Tweet, User,
    UsersResponse,
};
use crate::x_api::XApiClient;

use cookie_transport::CookieTransport;
use session::ScraperSession;

/// Default retry policy for scraper operations.
const SCRAPER_RETRY: RetryConfig = RetryConfig {
    max_attempts: 3,
    base_delay: std::time::Duration::from_millis(500),
    max_delay: std::time::Duration::from_secs(8),
};

/// X API client for local/scraper mode — no OAuth credentials required.
///
/// When a valid `scraper_session.json` exists in the data directory,
/// operations are dispatched to the cookie-based transport.
/// Otherwise, they return `ScraperTransportUnavailable`.
///
/// All transport calls are wrapped with `retry_with_backoff` so transient
/// network errors and 5xx responses are retried automatically.  Health
/// is tracked in `health` and exposed via [`LocalModeXClient::health`].
pub struct LocalModeXClient {
    allow_mutations: bool,
    cookie_transport: Option<CookieTransport>,
    /// Shared health tracker — updated after every transport call.
    health: ScraperHealth,
}

impl LocalModeXClient {
    /// Create a new local-mode client (no session — stub mode).
    ///
    /// `allow_mutations` controls whether write operations are attempted
    /// (when `true`) or immediately rejected (when `false`).
    pub fn new(allow_mutations: bool) -> Self {
        Self {
            allow_mutations,
            cookie_transport: None,
            health: new_scraper_health(),
        }
    }

    /// Create a local-mode client with cookie-auth from a session file.
    ///
    /// If the session file exists and is valid, operations will use
    /// the cookie transport. Otherwise, falls back to stub behavior.
    ///
    /// Auto-detects GraphQL query IDs from X's web client JS bundles at startup.
    pub async fn with_session(allow_mutations: bool, data_dir: &Path) -> Self {
        Self::with_session_and_health(allow_mutations, data_dir, new_scraper_health()).await
    }

    /// Create a local-mode client with cookie-auth and a **shared** health handle.
    ///
    /// Use this instead of [`with_session`] when you want multiple ephemeral
    /// clients (e.g. one per HTTP request) to update the same health tracker
    /// held in `AppState`. Enables the `/health` endpoint to reflect real
    /// scraper health across the lifetime of the server process.
    pub async fn with_session_and_health(
        allow_mutations: bool,
        data_dir: &Path,
        health: ScraperHealth,
    ) -> Self {
        let session_path = data_dir.join("scraper_session.json");
        let session = ScraperSession::load(&session_path).ok().flatten();

        let cookie_transport = if let Some(session) = session {
            let resolved = cookie_transport::resolve_transport().await;
            tracing::info!("Cookie-auth transport loaded from scraper_session.json");
            Some(CookieTransport::with_resolved_transport(
                session,
                resolved.query_ids,
                resolved.transaction,
            ))
        } else {
            None
        };

        Self {
            allow_mutations,
            cookie_transport,
            health,
        }
    }

    /// Return a clone of the shared health handle.
    ///
    /// Callers (e.g. the health endpoint) can snapshot the current state
    /// without coupling to the client implementation.
    pub fn health(&self) -> ScraperHealth {
        self.health.clone()
    }

    /// Wrap a transport call with retry logic and health tracking.
    ///
    /// On success: resets consecutive failure counter.
    /// On final failure: increments counter and records last error.
    async fn with_retry_and_health<F, Fut, T>(&self, op: F) -> Result<T, XApiError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, XApiError>>,
    {
        match retry_with_backoff(SCRAPER_RETRY, op).await {
            Ok(v) => {
                self.health.lock().await.record_success();
                Ok(v)
            }
            Err(e) => {
                self.health.lock().await.record_failure(&e.to_string());
                Err(e)
            }
        }
    }

    /// Path to the session file in a given data directory.
    pub fn session_path(data_dir: &Path) -> PathBuf {
        data_dir.join("scraper_session.json")
    }

    /// Check mutation gate and delegate to cookie transport if available.
    fn check_mutation(&self, method: &str) -> Result<(), XApiError> {
        if !self.allow_mutations {
            return Err(XApiError::ScraperMutationBlocked {
                message: method.to_string(),
            });
        }
        if self.cookie_transport.is_none() {
            return Err(XApiError::ScraperTransportUnavailable {
                message: format!(
                    "{method}: no browser session imported. \
                     Import cookies via Settings → X API → Import Browser Session."
                ),
            });
        }
        Ok(())
    }

    /// Return the cookie transport, or an error if unavailable.
    ///
    /// Must only be called after `check_mutation()` succeeds (which already
    /// guards against `cookie_transport.is_none()`).  Using this helper
    /// avoids `.unwrap()` scatter across every mutation method.
    fn transport(&self) -> Result<&CookieTransport, XApiError> {
        self.cookie_transport.as_ref().ok_or_else(|| {
            XApiError::ScraperTransportUnavailable {
                message: "no browser session imported. \
                          Import cookies via Settings → X API → Import Browser Session."
                    .to_string(),
            }
        })
    }

    /// Return a transport-unavailable error for read methods.
    fn read_stub(method: &str) -> XApiError {
        XApiError::ScraperTransportUnavailable {
            message: format!("{method}: scraper transport not yet implemented"),
        }
    }

    /// Return a feature-requires-auth error.
    fn auth_required(method: &str) -> XApiError {
        XApiError::FeatureRequiresAuth {
            message: format!("{method} requires authenticated API access"),
        }
    }
}

#[async_trait::async_trait]
impl XApiClient for LocalModeXClient {
    // --- Auth-gated methods ---

    async fn get_me(&self) -> Result<User, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport.fetch_viewer().await;
        }
        Err(Self::auth_required("get_me"))
    }

    async fn get_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        // No direct GraphQL endpoint for mentions
        Err(Self::auth_required("get_mentions"))
    }

    async fn get_home_timeline(
        &self,
        _user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport
                .get_home_timeline(max_results, pagination_token)
                .await;
        }
        Err(Self::auth_required("get_home_timeline"))
    }

    async fn get_bookmarks(
        &self,
        _user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport.get_bookmarks(max_results, pagination_token).await;
        }
        Err(Self::auth_required("get_bookmarks"))
    }

    async fn bookmark_tweet(&self, _user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("bookmark_tweet")?;
        self.transport()?.create_bookmark(tweet_id).await
    }

    async fn unbookmark_tweet(&self, _user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unbookmark_tweet")?;
        self.transport()?.delete_bookmark(tweet_id).await
    }

    // --- Read methods (delegated to cookie transport when available) ---

    async fn search_tweets(
        &self,
        query: &str,
        max_results: u32,
        _since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport
                .search_timeline(query, max_results, pagination_token)
                .await;
        }
        Err(Self::read_stub("search_tweets"))
    }

    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport.get_tweet_by_id(tweet_id).await;
        }
        Err(Self::read_stub("get_tweet"))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport.get_user_by_screen_name(username).await;
        }
        Err(Self::read_stub("get_user_by_username"))
    }

    async fn get_user_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport
                .get_user_tweets(user_id, max_results, pagination_token)
                .await;
        }
        Err(Self::read_stub("get_user_tweets"))
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<User, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport.get_user_by_rest_id(user_id).await;
        }
        Err(Self::read_stub("get_user_by_id"))
    }

    async fn get_followers(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport
                .get_followers(user_id, max_results, pagination_token)
                .await;
        }
        Err(Self::read_stub("get_followers"))
    }

    async fn get_following(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport
                .get_following(user_id, max_results, pagination_token)
                .await;
        }
        Err(Self::read_stub("get_following"))
    }

    async fn get_users_by_ids(&self, _user_ids: &[&str]) -> Result<UsersResponse, XApiError> {
        // No clean GraphQL batch endpoint
        Err(Self::read_stub("get_users_by_ids"))
    }

    async fn get_liked_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        if let Some(ref transport) = self.cookie_transport {
            return transport
                .get_liked_tweets(user_id, max_results, pagination_token)
                .await;
        }
        Err(Self::read_stub("get_liked_tweets"))
    }

    async fn get_tweet_liking_users(
        &self,
        _tweet_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        // No clean GraphQL endpoint
        Err(Self::read_stub("get_tweet_liking_users"))
    }

    async fn raw_request(
        &self,
        _method: &str,
        _url: &str,
        _query: Option<&[(String, String)]>,
        _body: Option<&str>,
        _headers: Option<&[(String, String)]>,
    ) -> Result<RawApiResponse, XApiError> {
        Err(Self::read_stub("raw_request"))
    }

    // --- Write methods (mutation-gated, delegated to cookie transport) ---

    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        self.check_mutation("post_tweet")?;
        let transport = self.transport()?;
        let text = text.to_string();
        self.with_retry_and_health(|| {
            let t = text.clone();
            async move { transport.post_tweet(&t).await }
        })
        .await
    }

    async fn reply_to_tweet(
        &self,
        text: &str,
        in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("reply_to_tweet")?;
        let transport = self.transport()?;
        let text = text.to_string();
        let reply_id = in_reply_to_id.to_string();
        self.with_retry_and_health(|| {
            let t = text.clone();
            let r = reply_id.clone();
            async move { transport.reply_to_tweet(&t, &r).await }
        })
        .await
    }

    async fn post_tweet_with_media(
        &self,
        text: &str,
        _media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("post_tweet_with_media")?;
        // Media upload not yet supported via cookie transport — post text only.
        let transport = self.transport()?;
        let text = text.to_string();
        self.with_retry_and_health(|| {
            let t = text.clone();
            async move { transport.post_tweet(&t).await }
        })
        .await
    }

    async fn reply_to_tweet_with_media(
        &self,
        text: &str,
        in_reply_to_id: &str,
        _media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("reply_to_tweet_with_media")?;
        // Media upload not yet supported via cookie transport — post text only.
        let transport = self.transport()?;
        let text = text.to_string();
        let reply_id = in_reply_to_id.to_string();
        self.with_retry_and_health(|| {
            let t = text.clone();
            let r = reply_id.clone();
            async move { transport.reply_to_tweet(&t, &r).await }
        })
        .await
    }

    async fn quote_tweet(
        &self,
        _text: &str,
        _quoted_tweet_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("quote_tweet")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "quote_tweet not yet supported via cookie transport".to_string(),
        })
    }

    async fn like_tweet(&self, _user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("like_tweet")?;
        self.transport()?.favorite_tweet(tweet_id).await
    }

    async fn unlike_tweet(&self, _user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unlike_tweet")?;
        self.transport()?.unfavorite_tweet(tweet_id).await
    }

    async fn follow_user(&self, _user_id: &str, target_user_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("follow_user")?;
        self.transport()?.follow_user(target_user_id).await
    }

    async fn unfollow_user(&self, _user_id: &str, target_user_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unfollow_user")?;
        self.transport()?.unfollow_user(target_user_id).await
    }

    async fn retweet(&self, _user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("retweet")?;
        self.transport()?.create_retweet(tweet_id).await
    }

    async fn unretweet(&self, _user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unretweet")?;
        self.transport()?.delete_retweet(tweet_id).await
    }

    async fn delete_tweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("delete_tweet")?;
        self.transport()?.delete_tweet(tweet_id).await
    }

    // --- Media (always unavailable in scraper mode) ---

    async fn upload_media(
        &self,
        _data: &[u8],
        _media_type: MediaType,
    ) -> Result<MediaId, XApiError> {
        Err(XApiError::MediaUploadError {
            message: "media upload unavailable in scraper mode".to_string(),
        })
    }
}

#[cfg(test)]
mod tests;
