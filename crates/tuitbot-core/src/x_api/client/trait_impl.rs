//! `XApiClient` trait implementation for `XApiHttpClient`.
//!
//! Each method is a thin wrapper that builds request parameters and
//! delegates to the HTTP helpers in `mod.rs` (`get`, `post_json`, `delete`).

use crate::error::XApiError;
use crate::x_api::types::{
    ActionResultResponse, BookmarkTweetRequest, DeleteTweetResponse, FollowUserRequest,
    LikeTweetRequest, MediaId, MediaPayload, MediaType, MentionResponse, PostTweetRequest,
    PostTweetResponse, PostedTweet, RawApiResponse, ReplyTo, RetweetRequest, SearchResponse,
    SingleTweetResponse, Tweet, User, UserResponse, UsersResponse,
};
use crate::x_api::XApiClient;

use super::{XApiHttpClient, EXPANSIONS, TWEET_FIELDS, USER_FIELDS};

#[async_trait::async_trait]
impl XApiClient for XApiHttpClient {
    async fn search_tweets(
        &self,
        query: &str,
        max_results: u32,
        since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        tracing::debug!(query = %query, max_results = max_results, "Search tweets");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("query", query),
            ("max_results", &max_str),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let since_id_owned;
        if let Some(sid) = since_id {
            since_id_owned = sid.to_string();
            params.push(("since_id", &since_id_owned));
        }

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get("/tweets/search/recent", &params).await?;
        let resp: SearchResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        tracing::debug!(
            query = %query,
            results = resp.data.len(),
            "Search tweets completed",
        );
        Ok(resp)
    }

    async fn get_mentions(
        &self,
        user_id: &str,
        since_id: Option<&str>,
        pagination_token: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        let path = format!("/users/{user_id}/mentions");
        let mut params = vec![
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let since_id_owned;
        if let Some(sid) = since_id {
            since_id_owned = sid.to_string();
            params.push(("since_id", &since_id_owned));
        }

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<MentionResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        tracing::debug!(chars = text.len(), "Posting tweet");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: None,
            media: None,
            quote_tweet_id: None,
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn reply_to_tweet(
        &self,
        text: &str,
        in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(in_reply_to = %in_reply_to_id, chars = text.len(), "Posting reply");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: in_reply_to_id.to_string(),
            }),
            media: None,
            quote_tweet_id: None,
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn upload_media(&self, data: &[u8], media_type: MediaType) -> Result<MediaId, XApiError> {
        super::super::media::upload_media(
            &self.client,
            &self.upload_base_url,
            &self.access_token.read().await,
            data,
            media_type,
        )
        .await
    }

    async fn post_tweet_with_media(
        &self,
        text: &str,
        media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(
            chars = text.len(),
            media_count = media_ids.len(),
            "Posting tweet with media"
        );
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: None,
            media: Some(MediaPayload {
                media_ids: media_ids.to_vec(),
            }),
            quote_tweet_id: None,
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn reply_to_tweet_with_media(
        &self,
        text: &str,
        in_reply_to_id: &str,
        media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(in_reply_to = %in_reply_to_id, chars = text.len(), media_count = media_ids.len(), "Posting reply with media");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: in_reply_to_id.to_string(),
            }),
            media: Some(MediaPayload {
                media_ids: media_ids.to_vec(),
            }),
            quote_tweet_id: None,
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, XApiError> {
        let path = format!("/tweets/{tweet_id}");
        let params = [
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let response = self.get(&path, &params).await?;
        let resp: SingleTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn get_me(&self) -> Result<User, XApiError> {
        let params = [("user.fields", USER_FIELDS)];

        let response = self.get("/users/me", &params).await?;
        let resp: UserResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn get_user_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let path = format!("/users/{user_id}/tweets");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<SearchResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, XApiError> {
        let path = format!("/users/by/username/{username}");
        let params = [("user.fields", USER_FIELDS)];

        let response = self.get(&path, &params).await?;
        let resp: UserResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn quote_tweet(
        &self,
        text: &str,
        quoted_tweet_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(chars = text.len(), quoted = %quoted_tweet_id, "Posting quote tweet");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: None,
            media: None,
            quote_tweet_id: Some(quoted_tweet_id.to_string()),
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn like_tweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Liking tweet");
        let path = format!("/users/{user_id}/likes");
        let body = LikeTweetRequest {
            tweet_id: tweet_id.to_string(),
        };

        let response = self.post_json(&path, &body).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn follow_user(&self, user_id: &str, target_user_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, target = %target_user_id, "Following user");
        let path = format!("/users/{user_id}/following");
        let body = FollowUserRequest {
            target_user_id: target_user_id.to_string(),
        };

        let response = self.post_json(&path, &body).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn unfollow_user(&self, user_id: &str, target_user_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, target = %target_user_id, "Unfollowing user");
        let path = format!("/users/{user_id}/following/{target_user_id}");

        let response = self.delete(&path).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn retweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Retweeting");
        let path = format!("/users/{user_id}/retweets");
        let body = RetweetRequest {
            tweet_id: tweet_id.to_string(),
        };

        let response = self.post_json(&path, &body).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn unretweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Unretweeting");
        let path = format!("/users/{user_id}/retweets/{tweet_id}");

        let response = self.delete(&path).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn delete_tweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(tweet_id = %tweet_id, "Deleting tweet");
        let path = format!("/tweets/{tweet_id}");

        let response = self.delete(&path).await?;
        let resp: DeleteTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.deleted)
    }

    async fn get_home_timeline(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        tracing::debug!(user_id = %user_id, max_results = max_results, "Getting home timeline");
        let path = format!("/users/{user_id}/timelines/reverse_chronological");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<SearchResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn unlike_tweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Unliking tweet");
        let path = format!("/users/{user_id}/likes/{tweet_id}");

        let response = self.delete(&path).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn get_followers(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        tracing::debug!(user_id = %user_id, max_results = max_results, "Getting followers");
        let path = format!("/users/{user_id}/followers");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<UsersResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn get_following(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        tracing::debug!(user_id = %user_id, max_results = max_results, "Getting following");
        let path = format!("/users/{user_id}/following");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<UsersResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<User, XApiError> {
        tracing::debug!(user_id = %user_id, "Getting user by ID");
        let path = format!("/users/{user_id}");
        let params = [("user.fields", USER_FIELDS)];

        let response = self.get(&path, &params).await?;
        let resp: UserResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn get_liked_tweets(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        tracing::debug!(user_id = %user_id, max_results = max_results, "Getting liked tweets");
        let path = format!("/users/{user_id}/liked_tweets");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<SearchResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn get_bookmarks(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        tracing::debug!(user_id = %user_id, max_results = max_results, "Getting bookmarks");
        let path = format!("/users/{user_id}/bookmarks");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<SearchResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn bookmark_tweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Bookmarking tweet");
        let path = format!("/users/{user_id}/bookmarks");
        let body = BookmarkTweetRequest {
            tweet_id: tweet_id.to_string(),
        };

        let response = self.post_json(&path, &body).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn unbookmark_tweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Unbookmarking tweet");
        let path = format!("/users/{user_id}/bookmarks/{tweet_id}");

        let response = self.delete(&path).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn get_users_by_ids(&self, user_ids: &[&str]) -> Result<UsersResponse, XApiError> {
        tracing::debug!(count = user_ids.len(), "Getting users by IDs");
        let ids = user_ids.join(",");
        let params = [("ids", ids.as_str()), ("user.fields", USER_FIELDS)];

        let response = self.get("/users", &params).await?;
        response
            .json::<UsersResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn get_tweet_liking_users(
        &self,
        tweet_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<UsersResponse, XApiError> {
        tracing::debug!(tweet_id = %tweet_id, max_results = max_results, "Getting liking users");
        let path = format!("/tweets/{tweet_id}/liking_users");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<UsersResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn raw_request(
        &self,
        method: &str,
        url: &str,
        query: Option<&[(String, String)]>,
        body: Option<&str>,
        headers: Option<&[(String, String)]>,
    ) -> Result<RawApiResponse, XApiError> {
        let token = self.access_token.read().await;
        let req_method = match method.to_ascii_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            other => {
                return Err(XApiError::ApiError {
                    status: 0,
                    message: format!("unsupported HTTP method: {other}"),
                })
            }
        };

        let mut builder = self.client.request(req_method, url).bearer_auth(&*token);

        if let Some(pairs) = query {
            builder = builder.query(pairs);
        }
        if let Some(json_body) = body {
            builder = builder
                .header("Content-Type", "application/json")
                .body(json_body.to_string());
        }
        if let Some(extra_headers) = headers {
            for (k, v) in extra_headers {
                builder = builder.header(k.as_str(), v.as_str());
            }
        }

        let response = builder
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        let status = response.status().as_u16();
        let rate_limit = Self::parse_rate_limit_headers(response.headers());

        // Extract a small set of useful headers.
        let mut resp_headers = std::collections::HashMap::new();
        for key in [
            "content-type",
            "x-rate-limit-remaining",
            "x-rate-limit-reset",
            "x-rate-limit-limit",
        ] {
            if let Some(val) = response.headers().get(key) {
                if let Ok(s) = val.to_str() {
                    resp_headers.insert(key.to_string(), s.to_string());
                }
            }
        }

        // Extract path for usage tracking (best-effort parse from URL).
        if let Ok(parsed) = reqwest::Url::parse(url) {
            self.record_usage(parsed.path(), method, status);
        }

        let response_body = response
            .text()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        Ok(RawApiResponse {
            status,
            headers: resp_headers,
            body: response_body,
            rate_limit: Some(rate_limit),
        })
    }
}
