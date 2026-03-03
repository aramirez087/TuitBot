//! Local-mode X API client for scraper backend.
//!
//! Implements `XApiClient` without requiring OAuth credentials.
//! Read methods return transport-unavailable stubs (pending actual scraper
//! transport in a future session). Write methods are gated by
//! `scraper_allow_mutations`. Auth-gated methods (get_me, mentions, etc.)
//! return `FeatureRequiresAuth`.

use crate::error::XApiError;
use crate::x_api::types::{
    MediaId, MediaType, MentionResponse, PostedTweet, RawApiResponse, SearchResponse, Tweet, User,
    UsersResponse,
};
use crate::x_api::XApiClient;

/// X API client for local/scraper mode — no OAuth credentials required.
///
/// All methods return appropriate error variants:
/// - Auth-gated methods → `FeatureRequiresAuth`
/// - Write methods → `ScraperMutationBlocked` (if disabled) or `ScraperTransportUnavailable`
/// - Read methods → `ScraperTransportUnavailable` (pending actual transport)
/// - Media methods → `MediaUploadError`
pub struct LocalModeXClient {
    allow_mutations: bool,
}

impl LocalModeXClient {
    /// Create a new local-mode client.
    ///
    /// `allow_mutations` controls whether write operations are attempted
    /// (when `true`) or immediately rejected (when `false`).
    pub fn new(allow_mutations: bool) -> Self {
        Self { allow_mutations }
    }

    /// Check mutation gate and return appropriate error.
    fn check_mutation(&self, method: &str) -> Result<(), XApiError> {
        if !self.allow_mutations {
            return Err(XApiError::ScraperMutationBlocked {
                message: method.to_string(),
            });
        }
        Err(XApiError::ScraperTransportUnavailable {
            message: format!("{method}: scraper write transport not yet implemented"),
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

    async fn post_tweet(&self, _text: &str) -> Result<PostedTweet, XApiError> {
        self.check_mutation("post_tweet")?;
        unreachable!()
    }

    async fn reply_to_tweet(
        &self,
        _text: &str,
        _in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("reply_to_tweet")?;
        unreachable!()
    }

    async fn post_tweet_with_media(
        &self,
        _text: &str,
        _media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("post_tweet_with_media")?;
        unreachable!()
    }

    async fn reply_to_tweet_with_media(
        &self,
        _text: &str,
        _in_reply_to_id: &str,
        _media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("reply_to_tweet_with_media")?;
        unreachable!()
    }

    async fn quote_tweet(
        &self,
        _text: &str,
        _quoted_tweet_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        self.check_mutation("quote_tweet")?;
        unreachable!()
    }

    async fn like_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("like_tweet")?;
        unreachable!()
    }

    async fn unlike_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unlike_tweet")?;
        unreachable!()
    }

    async fn follow_user(&self, _user_id: &str, _target_user_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("follow_user")?;
        unreachable!()
    }

    async fn unfollow_user(
        &self,
        _user_id: &str,
        _target_user_id: &str,
    ) -> Result<bool, XApiError> {
        self.check_mutation("unfollow_user")?;
        unreachable!()
    }

    async fn retweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("retweet")?;
        unreachable!()
    }

    async fn unretweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("unretweet")?;
        unreachable!()
    }

    async fn delete_tweet(&self, _tweet_id: &str) -> Result<bool, XApiError> {
        self.check_mutation("delete_tweet")?;
        unreachable!()
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
