//! Tests for `LocalModeXClient` and the `create_local_client` factory.

use std::sync::Arc;

use crate::config::XApiConfig;
use crate::error::XApiError;
use crate::x_api::local_mode::LocalModeXClient;
use crate::x_api::{create_local_client, XApiClient};

// --- Factory function tests ---

#[tokio::test]
async fn factory_returns_some_for_scraper_backend() {
    let config = XApiConfig {
        provider_backend: "scraper".to_string(),
        scraper_allow_mutations: false,
        ..Default::default()
    };
    assert!(create_local_client(&config).await.is_some());
}

#[tokio::test]
async fn factory_returns_none_for_empty_backend() {
    let config = XApiConfig {
        provider_backend: String::new(),
        ..Default::default()
    };
    assert!(create_local_client(&config).await.is_none());
}

#[tokio::test]
async fn factory_returns_none_for_x_api_backend() {
    let config = XApiConfig {
        provider_backend: "x_api".to_string(),
        ..Default::default()
    };
    assert!(create_local_client(&config).await.is_none());
}

#[tokio::test]
async fn factory_passes_allow_mutations_true() {
    let config = XApiConfig {
        provider_backend: "scraper".to_string(),
        scraper_allow_mutations: true,
        ..Default::default()
    };
    let client = create_local_client(&config).await.unwrap();
    // Verify it's a valid Arc<dyn XApiClient>
    let _: &dyn XApiClient = &*client;
}

// --- Auth-gated method tests ---

#[tokio::test]
async fn get_me_returns_feature_requires_auth() {
    let client = LocalModeXClient::new(false);
    let err = client.get_me().await.unwrap_err();
    assert!(
        matches!(err, XApiError::FeatureRequiresAuth { .. }),
        "Expected FeatureRequiresAuth, got: {err}"
    );
}

#[tokio::test]
async fn get_mentions_returns_feature_requires_auth() {
    let client = LocalModeXClient::new(false);
    let err = client
        .get_mentions("user123", None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, XApiError::FeatureRequiresAuth { .. }));
}

#[tokio::test]
async fn get_home_timeline_returns_feature_requires_auth() {
    let client = LocalModeXClient::new(false);
    let err = client
        .get_home_timeline("user123", 10, None)
        .await
        .unwrap_err();
    assert!(matches!(err, XApiError::FeatureRequiresAuth { .. }));
}

#[tokio::test]
async fn get_bookmarks_returns_feature_requires_auth() {
    let client = LocalModeXClient::new(false);
    let err = client.get_bookmarks("user123", 10, None).await.unwrap_err();
    assert!(matches!(err, XApiError::FeatureRequiresAuth { .. }));
}

// --- Mutation-gated tests (mutations disabled) ---

#[tokio::test]
async fn post_tweet_blocked_when_mutations_disabled() {
    let client = LocalModeXClient::new(false);
    let err = client.post_tweet("hello").await.unwrap_err();
    assert!(
        matches!(err, XApiError::ScraperMutationBlocked { .. }),
        "Expected ScraperMutationBlocked, got: {err}"
    );
}

#[tokio::test]
async fn reply_to_tweet_blocked_when_mutations_disabled() {
    let client = LocalModeXClient::new(false);
    let err = client
        .reply_to_tweet("reply text", "12345")
        .await
        .unwrap_err();
    assert!(matches!(err, XApiError::ScraperMutationBlocked { .. }));
}

#[tokio::test]
async fn like_tweet_blocked_when_mutations_disabled() {
    let client = LocalModeXClient::new(false);
    let err = client.like_tweet("user1", "tweet1").await.unwrap_err();
    assert!(matches!(err, XApiError::ScraperMutationBlocked { .. }));
}

#[tokio::test]
async fn follow_user_blocked_when_mutations_disabled() {
    let client = LocalModeXClient::new(false);
    let err = client.follow_user("user1", "user2").await.unwrap_err();
    assert!(matches!(err, XApiError::ScraperMutationBlocked { .. }));
}

#[tokio::test]
async fn delete_tweet_blocked_when_mutations_disabled() {
    let client = LocalModeXClient::new(false);
    let err = client.delete_tweet("tweet1").await.unwrap_err();
    assert!(matches!(err, XApiError::ScraperMutationBlocked { .. }));
}

// --- Mutation-gated tests (mutations enabled, transport unavailable) ---

#[tokio::test]
async fn post_tweet_transport_unavailable_when_mutations_enabled() {
    let client = LocalModeXClient::new(true);
    let err = client.post_tweet("hello").await.unwrap_err();
    assert!(
        matches!(err, XApiError::ScraperTransportUnavailable { .. }),
        "Expected ScraperTransportUnavailable, got: {err}"
    );
}

#[tokio::test]
async fn reply_to_tweet_transport_unavailable_when_mutations_enabled() {
    let client = LocalModeXClient::new(true);
    let err = client.reply_to_tweet("reply", "12345").await.unwrap_err();
    assert!(matches!(err, XApiError::ScraperTransportUnavailable { .. }));
}

// --- Read method stubs ---

#[tokio::test]
async fn search_tweets_returns_transport_unavailable() {
    let client = LocalModeXClient::new(false);
    let err = client
        .search_tweets("query", 10, None, None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, XApiError::ScraperTransportUnavailable { .. }),
        "Expected ScraperTransportUnavailable, got: {err}"
    );
}

#[tokio::test]
async fn get_tweet_returns_transport_unavailable() {
    let client = LocalModeXClient::new(false);
    let err = client.get_tweet("tweet1").await.unwrap_err();
    assert!(matches!(err, XApiError::ScraperTransportUnavailable { .. }));
}

#[tokio::test]
async fn get_user_by_username_returns_transport_unavailable() {
    let client = LocalModeXClient::new(false);
    let err = client.get_user_by_username("alice").await.unwrap_err();
    assert!(matches!(err, XApiError::ScraperTransportUnavailable { .. }));
}

// --- Media ---

#[tokio::test]
async fn upload_media_returns_media_upload_error() {
    use crate::x_api::types::{ImageFormat, MediaType};
    let client = LocalModeXClient::new(true);
    let err = client
        .upload_media(&[0u8; 10], MediaType::Image(ImageFormat::Jpeg))
        .await
        .unwrap_err();
    assert!(
        matches!(err, XApiError::MediaUploadError { .. }),
        "Expected MediaUploadError, got: {err}"
    );
}

// --- Trait object compatibility ---

#[tokio::test]
async fn local_mode_client_is_send_sync() {
    let client: Arc<dyn XApiClient> = Arc::new(LocalModeXClient::new(false));
    // Verify the client can be sent across threads
    let handle = tokio::spawn(async move {
        let err = client.get_me().await.unwrap_err();
        matches!(err, XApiError::FeatureRequiresAuth { .. })
    });
    assert!(handle.await.unwrap());
}
