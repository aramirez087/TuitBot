//! X API tier detection and capability mapping.
//!
//! Detects the user's X API tier (Free, Basic, Pro) at startup
//! by probing the search endpoint. Adapts agent behavior based
//! on available capabilities.

use super::XApiClient;
use crate::error::XApiError;

/// X API access tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiTier {
    /// Free tier: posting only, no search or mentions.
    Free,
    /// Basic tier ($200/mo): search, mentions, and posting.
    Basic,
    /// Pro tier: all features with higher rate limits.
    Pro,
}

/// Capabilities available at each API tier.
#[derive(Debug, Clone)]
pub struct TierCapabilities {
    /// Whether the tweet search endpoint is available.
    pub search_available: bool,
    /// Whether the mentions endpoint is available.
    pub mentions_available: bool,
    /// Whether posting tweets is available.
    pub posting_available: bool,
    /// Whether the discovery loop should be enabled.
    pub discovery_loop_enabled: bool,
}

impl ApiTier {
    /// Get the capabilities available at this tier.
    pub fn capabilities(&self) -> TierCapabilities {
        match self {
            ApiTier::Free => TierCapabilities {
                search_available: false,
                mentions_available: false,
                posting_available: true,
                discovery_loop_enabled: false,
            },
            ApiTier::Basic | ApiTier::Pro => TierCapabilities {
                search_available: true,
                mentions_available: true,
                posting_available: true,
                discovery_loop_enabled: true,
            },
        }
    }
}

impl std::fmt::Display for ApiTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiTier::Free => write!(f, "Free"),
            ApiTier::Basic => write!(f, "Basic"),
            ApiTier::Pro => write!(f, "Pro"),
        }
    }
}

/// Detect the user's X API tier by probing the search endpoint.
///
/// Uses a minimal test query to avoid wasting rate limit quota.
/// Falls back to `Free` tier on network errors as the safe default.
/// Propagates auth errors (do not infer tier from auth failure).
pub async fn detect_tier(client: &dyn XApiClient) -> Result<ApiTier, XApiError> {
    match client.search_tweets("test", 10, None, None).await {
        Ok(_) => {
            let tier = ApiTier::Basic;
            log_tier_detection(&tier);
            Ok(tier)
        }
        Err(XApiError::Forbidden { .. }) => {
            let tier = ApiTier::Free;
            log_tier_detection(&tier);
            Ok(tier)
        }
        Err(XApiError::RateLimited { .. }) => {
            // Rate limited implies the endpoint exists
            let tier = ApiTier::Basic;
            log_tier_detection(&tier);
            Ok(tier)
        }
        Err(XApiError::AuthExpired) => {
            // Do not infer tier from auth failure
            Err(XApiError::AuthExpired)
        }
        Err(XApiError::Network { .. }) => {
            tracing::warn!("Network error during tier detection, defaulting to Free tier");
            let tier = ApiTier::Free;
            log_tier_detection(&tier);
            Ok(tier)
        }
        Err(e) => {
            tracing::warn!(error = %e, "Unexpected error during tier detection, defaulting to Free tier");
            let tier = ApiTier::Free;
            log_tier_detection(&tier);
            Ok(tier)
        }
    }
}

/// Log the detected tier and its capabilities.
fn log_tier_detection(tier: &ApiTier) {
    let caps = tier.capabilities();
    tracing::info!(
        tier = %tier,
        search = caps.search_available,
        mentions = caps.mentions_available,
        posting = caps.posting_available,
        discovery_loop = caps.discovery_loop_enabled,
        "Detected X API tier"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x_api::types::*;

    /// Mock client that returns a configurable result for search_tweets.
    struct MockClient {
        search_result: Result<SearchResponse, XApiError>,
    }

    impl MockClient {
        fn ok() -> Self {
            Self {
                search_result: Ok(SearchResponse {
                    data: vec![],
                    includes: None,
                    meta: SearchMeta {
                        newest_id: None,
                        oldest_id: None,
                        result_count: 0,
                        next_token: None,
                    },
                }),
            }
        }

        fn forbidden() -> Self {
            Self {
                search_result: Err(XApiError::Forbidden {
                    message: "Not permitted".to_string(),
                }),
            }
        }

        fn rate_limited() -> Self {
            Self {
                search_result: Err(XApiError::RateLimited {
                    retry_after: Some(60),
                }),
            }
        }

        fn auth_expired() -> Self {
            Self {
                search_result: Err(XApiError::AuthExpired),
            }
        }

        fn api_error() -> Self {
            Self {
                search_result: Err(XApiError::ApiError {
                    status: 500,
                    message: "Internal error".to_string(),
                }),
            }
        }
    }

    #[async_trait::async_trait]
    impl XApiClient for MockClient {
        async fn search_tweets(
            &self,
            _query: &str,
            _max_results: u32,
            _since_id: Option<&str>,
            _pagination_token: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            match &self.search_result {
                Ok(r) => Ok(r.clone()),
                Err(e) => match e {
                    XApiError::Forbidden { message } => Err(XApiError::Forbidden {
                        message: message.clone(),
                    }),
                    XApiError::RateLimited { retry_after } => Err(XApiError::RateLimited {
                        retry_after: *retry_after,
                    }),
                    XApiError::AuthExpired => Err(XApiError::AuthExpired),
                    XApiError::ApiError { status, message } => Err(XApiError::ApiError {
                        status: *status,
                        message: message.clone(),
                    }),
                    _ => Err(XApiError::ApiError {
                        status: 0,
                        message: "test error".to_string(),
                    }),
                },
            }
        }

        async fn get_mentions(
            &self,
            _user_id: &str,
            _since_id: Option<&str>,
            _pagination_token: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            unimplemented!()
        }

        async fn post_tweet(&self, _text: &str) -> Result<PostedTweet, XApiError> {
            unimplemented!()
        }

        async fn reply_to_tweet(
            &self,
            _text: &str,
            _in_reply_to_id: &str,
        ) -> Result<PostedTweet, XApiError> {
            unimplemented!()
        }

        async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, XApiError> {
            unimplemented!()
        }

        async fn get_me(&self) -> Result<User, XApiError> {
            unimplemented!()
        }

        async fn get_user_tweets(
            &self,
            _user_id: &str,
            _max_results: u32,
            _pagination_token: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            unimplemented!()
        }

        async fn get_user_by_username(&self, _username: &str) -> Result<User, XApiError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn detect_basic_on_search_success() {
        let client = MockClient::ok();
        let tier = detect_tier(&client).await.expect("detect");
        assert_eq!(tier, ApiTier::Basic);
    }

    #[tokio::test]
    async fn detect_free_on_forbidden() {
        let client = MockClient::forbidden();
        let tier = detect_tier(&client).await.expect("detect");
        assert_eq!(tier, ApiTier::Free);
    }

    #[tokio::test]
    async fn detect_basic_on_rate_limited() {
        let client = MockClient::rate_limited();
        let tier = detect_tier(&client).await.expect("detect");
        assert_eq!(tier, ApiTier::Basic);
    }

    #[tokio::test]
    async fn detect_propagates_auth_expired() {
        let client = MockClient::auth_expired();
        let result = detect_tier(&client).await;
        assert!(matches!(result, Err(XApiError::AuthExpired)));
    }

    #[tokio::test]
    async fn detect_defaults_to_free_on_other_errors() {
        let client = MockClient::api_error();
        let tier = detect_tier(&client).await.expect("detect");
        assert_eq!(tier, ApiTier::Free);
    }

    #[test]
    fn free_tier_capabilities() {
        let caps = ApiTier::Free.capabilities();
        assert!(!caps.search_available);
        assert!(!caps.mentions_available);
        assert!(caps.posting_available);
        assert!(!caps.discovery_loop_enabled);
    }

    #[test]
    fn basic_tier_capabilities() {
        let caps = ApiTier::Basic.capabilities();
        assert!(caps.search_available);
        assert!(caps.mentions_available);
        assert!(caps.posting_available);
        assert!(caps.discovery_loop_enabled);
    }

    #[test]
    fn pro_tier_same_as_basic() {
        let basic = ApiTier::Basic.capabilities();
        let pro = ApiTier::Pro.capabilities();
        assert_eq!(basic.search_available, pro.search_available);
        assert_eq!(basic.mentions_available, pro.mentions_available);
    }

    #[test]
    fn tier_display() {
        assert_eq!(ApiTier::Free.to_string(), "Free");
        assert_eq!(ApiTier::Basic.to_string(), "Basic");
        assert_eq!(ApiTier::Pro.to_string(), "Pro");
    }
}
