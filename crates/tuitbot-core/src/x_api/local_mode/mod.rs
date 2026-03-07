//! Local-mode X API client for scraper backend.
//!
//! Implements `XApiClient` without requiring OAuth credentials.
//! When a browser session file (`scraper_session.json`) is present,
//! write methods use cookie-based authentication to post via X's
//! internal GraphQL API. Without a session, write methods return
//! `ScraperTransportUnavailable`.

pub mod cookie_transport;
pub mod session;

use std::path::{Path, PathBuf};

use crate::error::XApiError;
use crate::x_api::types::{
    MediaId, MediaType, MentionResponse, PostedTweet, RawApiResponse, SearchResponse, Tweet, User,
    UsersResponse,
};
use crate::x_api::XApiClient;

use cookie_transport::CookieTransport;
use session::ScraperSession;

/// X API client for local/scraper mode — no OAuth credentials required.
///
/// When a valid `scraper_session.json` exists in the data directory,
/// write operations are dispatched to the cookie-based transport.
/// Otherwise, they return `ScraperTransportUnavailable`.
pub struct LocalModeXClient {
    allow_mutations: bool,
    cookie_transport: Option<CookieTransport>,
}

impl LocalModeXClient {
    /// Create a new local-mode client.
    ///
    /// `allow_mutations` controls whether write operations are attempted
    /// (when `true`) or immediately rejected (when `false`).
    pub fn new(allow_mutations: bool) -> Self {
        Self {
            allow_mutations,
            cookie_transport: None,
        }
    }

    /// Create a local-mode client with cookie-auth from a session file.
    ///
    /// If the session file exists and is valid, write operations will use
    /// the cookie transport. Otherwise, falls back to stub behavior.
    ///
    /// Auto-detects the current CreateTweet GraphQL query ID from X's
    /// web client JS bundles at startup.
    pub async fn with_session(allow_mutations: bool, data_dir: &Path) -> Self {
        let session_path = data_dir.join("scraper_session.json");
        let session = ScraperSession::load(&session_path).ok().flatten();

        let cookie_transport = if let Some(session) = session {
            let resolved = cookie_transport::resolve_transport().await;
            tracing::info!("Cookie-auth transport loaded from scraper_session.json");
            Some(CookieTransport::with_query_id(
                session,
                resolved.query_id,
                resolved.transaction,
            ))
        } else {
            None
        };

        Self {
            allow_mutations,
            cookie_transport,
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
        Err(Self::auth_required("get_me"))
    }

    async fn get_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        Err(Self::auth_required("get_mentions"))
    }

    async fn get_home_timeline(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(Self::auth_required("get_home_timeline"))
    }

    async fn get_bookmarks(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(Self::auth_required("get_bookmarks"))
    }

    async fn bookmark_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        Err(Self::auth_required("bookmark_tweet"))
    }

    async fn unbookmark_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        Err(Self::auth_required("unbookmark_tweet"))
    }

    // --- Read methods (transport stubs) ---

    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(Self::read_stub("search_tweets"))
    }

    async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, XApiError> {
        Err(Self::read_stub("get_tweet"))
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<User, XApiError> {
        Err(Self::read_stub("get_user_by_username"))
    }

    async fn get_user_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(Self::read_stub("get_user_tweets"))
    }

    async fn get_user_by_id(&self, _user_id: &str) -> Result<User, XApiError> {
        Err(Self::read_stub("get_user_by_id"))
    }

    async fn get_followers(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        Err(Self::read_stub("get_followers"))
    }

    async fn get_following(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        Err(Self::read_stub("get_following"))
    }

    async fn get_users_by_ids(&self, _user_ids: &[&str]) -> Result<UsersResponse, XApiError> {
        Err(Self::read_stub("get_users_by_ids"))
    }

    async fn get_liked_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(Self::read_stub("get_liked_tweets"))
    }

    async fn get_tweet_liking_users(
        &self,
        _tweet_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
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

    // --- Write methods (mutation-gated) ---

    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        self.check_mutation("post_tweet")?;
        self.cookie_transport
            .as_ref()
            .unwrap()
            .post_tweet(text)
            .await
    }

    async fn reply_to_tweet(
        &self,
        text: &str,
        in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("reply_to_tweet")?;
        self.cookie_transport
            .as_ref()
            .unwrap()
            .reply_to_tweet(text, in_reply_to_id)
            .await
    }

    async fn post_tweet_with_media(
        &self,
        text: &str,
        _media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("post_tweet_with_media")?;
        // Media upload not yet supported via cookie transport — post text only.
        self.cookie_transport
            .as_ref()
            .unwrap()
            .post_tweet(text)
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
        self.cookie_transport
            .as_ref()
            .unwrap()
            .reply_to_tweet(text, in_reply_to_id)
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

    async fn like_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("like_tweet")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "like_tweet not yet supported via cookie transport".to_string(),
        })
    }

    async fn unlike_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unlike_tweet")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "unlike_tweet not yet supported via cookie transport".to_string(),
        })
    }

    async fn follow_user(&self, _user_id: &str, _target_user_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("follow_user")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "follow_user not yet supported via cookie transport".to_string(),
        })
    }

    async fn unfollow_user(
        &self,
        _user_id: &str,
        _target_user_id: &str,
    ) -> Result<bool, XApiError> {
        self.check_mutation("unfollow_user")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "unfollow_user not yet supported via cookie transport".to_string(),
        })
    }

    async fn retweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("retweet")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "retweet not yet supported via cookie transport".to_string(),
        })
    }

    async fn unretweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unretweet")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "unretweet not yet supported via cookie transport".to_string(),
        })
    }

    async fn delete_tweet(&self, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("delete_tweet")?;
        Err(XApiError::ScraperTransportUnavailable {
            message: "delete_tweet not yet supported via cookie transport".to_string(),
        })
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
