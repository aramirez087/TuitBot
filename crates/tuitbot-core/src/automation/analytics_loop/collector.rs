//! Port traits and data types for analytics collection.
//!
//! Defines the interfaces for fetching profile/engagement metrics
//! and storing analytics data.

/// Fetches the authenticated user's profile metrics.
#[async_trait::async_trait]
pub trait ProfileFetcher: Send + Sync {
    /// Get current follower count, following count, and tweet count.
    async fn get_profile_metrics(&self) -> Result<ProfileMetrics, AnalyticsError>;
}

/// Fetches engagement metrics for a specific tweet.
#[async_trait::async_trait]
pub trait EngagementFetcher: Send + Sync {
    /// Get engagement metrics for a tweet by its ID.
    async fn get_tweet_metrics(&self, tweet_id: &str) -> Result<TweetMetrics, AnalyticsError>;
}

/// Storage operations for analytics data.
#[async_trait::async_trait]
pub trait AnalyticsStorage: Send + Sync {
    /// Store a daily follower snapshot.
    async fn store_follower_snapshot(
        &self,
        followers: i64,
        following: i64,
        tweets: i64,
    ) -> Result<(), AnalyticsError>;

    /// Get yesterday's follower count (for drop detection).
    async fn get_yesterday_followers(&self) -> Result<Option<i64>, AnalyticsError>;

    /// Get reply IDs posted approximately 24h ago that need performance measurement.
    async fn get_replies_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError>;

    /// Get tweet IDs posted approximately 24h ago that need performance measurement.
    async fn get_tweets_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError>;

    /// Store reply performance metrics.
    async fn store_reply_performance(
        &self,
        reply_id: &str,
        likes: i64,
        replies: i64,
        impressions: i64,
        score: f64,
    ) -> Result<(), AnalyticsError>;

    /// Store tweet performance metrics.
    async fn store_tweet_performance(
        &self,
        tweet_id: &str,
        likes: i64,
        retweets: i64,
        replies: i64,
        impressions: i64,
        score: f64,
    ) -> Result<(), AnalyticsError>;

    /// Update the content score running average for a topic.
    async fn update_content_score(
        &self,
        topic: &str,
        format: &str,
        score: f64,
    ) -> Result<(), AnalyticsError>;

    /// Log an action.
    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), AnalyticsError>;

    /// Run Forge sync if the active content source has analytics sync enabled.
    ///
    /// Returns `Ok(Some(summary))` when sync ran, `Ok(None)` when disabled
    /// or not applicable, `Err` on failure.
    async fn run_forge_sync_if_enabled(&self) -> Result<Option<ForgeSyncResult>, AnalyticsError> {
        // Default: no Forge sync (backwards-compatible for existing impls).
        Ok(None)
    }

    /// Run background aggregation jobs (best-times heatmap, reach snapshots).
    ///
    /// Called at the end of each analytics iteration. Default is a no-op so
    /// existing impls don't break.
    async fn run_aggregations(&self) -> Result<(), AnalyticsError> {
        Ok(())
    }
}

// ============================================================================
// Types
// ============================================================================

/// Profile metrics from X API.
#[derive(Debug, Clone)]
pub struct ProfileMetrics {
    pub follower_count: i64,
    pub following_count: i64,
    pub tweet_count: i64,
}

/// Tweet engagement metrics from X API.
#[derive(Debug, Clone)]
pub struct TweetMetrics {
    pub likes: i64,
    pub retweets: i64,
    pub replies: i64,
    pub impressions: i64,
}

/// Analytics-specific errors.
#[derive(Debug)]
pub enum AnalyticsError {
    /// X API error.
    ApiError(String),
    /// Storage error.
    StorageError(String),
    /// Other error.
    Other(String),
}

impl std::fmt::Display for AnalyticsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiError(msg) => write!(f, "API error: {msg}"),
            Self::StorageError(msg) => write!(f, "storage error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AnalyticsError {}

/// Result of a Forge sync iteration (returned by `run_forge_sync_if_enabled`).
#[derive(Debug, Default, Clone)]
pub struct ForgeSyncResult {
    pub tweets_synced: usize,
    pub threads_synced: usize,
}
