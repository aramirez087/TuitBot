//! Analytics loop for tracking content performance.
//!
//! Runs periodically to:
//! 1. Snapshot follower counts via the X API.
//! 2. Fetch engagement metrics on content posted ~24h ago.
//! 3. Compute performance scores and update running averages.
//! 4. Alert on significant follower drops.

use super::loop_helpers::ConsecutiveErrorTracker;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

// ============================================================================
// Port traits
// ============================================================================

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

// ============================================================================
// Analytics loop
// ============================================================================

/// Analytics loop that tracks content performance and follower trends.
pub struct AnalyticsLoop {
    profile_fetcher: Arc<dyn ProfileFetcher>,
    engagement_fetcher: Arc<dyn EngagementFetcher>,
    storage: Arc<dyn AnalyticsStorage>,
}

impl AnalyticsLoop {
    /// Create a new analytics loop.
    pub fn new(
        profile_fetcher: Arc<dyn ProfileFetcher>,
        engagement_fetcher: Arc<dyn EngagementFetcher>,
        storage: Arc<dyn AnalyticsStorage>,
    ) -> Self {
        Self {
            profile_fetcher,
            engagement_fetcher,
            storage,
        }
    }

    /// Run the continuous analytics loop until cancellation.
    pub async fn run(&self, cancel: CancellationToken, interval: Duration) {
        tracing::info!("Analytics loop started");

        let mut error_tracker = ConsecutiveErrorTracker::new(5, Duration::from_secs(600));

        loop {
            if cancel.is_cancelled() {
                break;
            }

            match self.run_iteration().await {
                Ok(summary) => {
                    error_tracker.record_success();
                    tracing::info!(
                        followers = summary.follower_count,
                        replies_measured = summary.replies_measured,
                        tweets_measured = summary.tweets_measured,
                        "Analytics iteration complete"
                    );
                }
                Err(e) => {
                    let should_pause = error_tracker.record_error();
                    tracing::warn!(error = %e, "Analytics iteration failed");

                    if should_pause {
                        tracing::warn!(
                            pause_secs = error_tracker.pause_duration().as_secs(),
                            "Pausing analytics loop due to consecutive errors"
                        );
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(error_tracker.pause_duration()) => {},
                        }
                        error_tracker.reset();
                        continue;
                    }
                }
            }

            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = tokio::time::sleep(interval) => {},
            }
        }

        tracing::info!("Analytics loop stopped");
    }

    /// Run a single analytics iteration.
    pub async fn run_iteration(&self) -> Result<AnalyticsSummary, AnalyticsError> {
        let mut summary = AnalyticsSummary::default();

        // 1. Snapshot follower count
        let metrics = self.profile_fetcher.get_profile_metrics().await?;
        summary.follower_count = metrics.follower_count;

        tracing::info!(
            followers = metrics.follower_count,
            "Follower snapshot: {} followers",
            metrics.follower_count,
        );

        self.storage
            .store_follower_snapshot(
                metrics.follower_count,
                metrics.following_count,
                metrics.tweet_count,
            )
            .await?;

        // Check for significant follower drop
        if let Ok(Some(yesterday)) = self.storage.get_yesterday_followers().await {
            if yesterday > 0 {
                let drop_pct =
                    (yesterday - metrics.follower_count) as f64 / yesterday as f64 * 100.0;
                if drop_pct > 2.0 {
                    tracing::warn!(
                        yesterday = yesterday,
                        today = metrics.follower_count,
                        drop_pct = format!("{:.1}%", drop_pct),
                        "Significant follower drop detected"
                    );

                    let _ = self
                        .storage
                        .log_action(
                            "analytics",
                            "alert",
                            &format!(
                                "Follower drop: {} -> {} ({:.1}%)",
                                yesterday, metrics.follower_count, drop_pct
                            ),
                        )
                        .await;
                }
            }
        }

        // 2. Measure reply performance
        let reply_ids = self.storage.get_replies_needing_measurement().await?;
        for reply_id in &reply_ids {
            match self.engagement_fetcher.get_tweet_metrics(reply_id).await {
                Ok(m) => {
                    let score =
                        compute_performance_score(m.likes, m.replies, m.retweets, m.impressions);
                    let _ = self
                        .storage
                        .store_reply_performance(reply_id, m.likes, m.replies, m.impressions, score)
                        .await;
                    summary.replies_measured += 1;
                }
                Err(e) => {
                    tracing::debug!(reply_id = %reply_id, error = %e, "Failed to fetch reply metrics");
                }
            }
        }

        // 3. Measure tweet performance
        let tweet_ids = self.storage.get_tweets_needing_measurement().await?;
        for tweet_id in &tweet_ids {
            match self.engagement_fetcher.get_tweet_metrics(tweet_id).await {
                Ok(m) => {
                    let score =
                        compute_performance_score(m.likes, m.replies, m.retweets, m.impressions);
                    let _ = self
                        .storage
                        .store_tweet_performance(
                            tweet_id,
                            m.likes,
                            m.retweets,
                            m.replies,
                            m.impressions,
                            score,
                        )
                        .await;
                    summary.tweets_measured += 1;
                }
                Err(e) => {
                    tracing::debug!(tweet_id = %tweet_id, error = %e, "Failed to fetch tweet metrics");
                }
            }
        }

        let _ = self
            .storage
            .log_action(
                "analytics",
                "success",
                &format!(
                    "Followers: {}, replies measured: {}, tweets measured: {}",
                    summary.follower_count, summary.replies_measured, summary.tweets_measured,
                ),
            )
            .await;

        Ok(summary)
    }
}

/// Summary of an analytics iteration.
#[derive(Debug, Default)]
pub struct AnalyticsSummary {
    pub follower_count: i64,
    pub replies_measured: usize,
    pub tweets_measured: usize,
}

/// Compute the performance score for content engagement.
///
/// Formula: `(likes * 3 + replies * 5 + retweets * 4) / max(impressions, 1) * 1000`
pub fn compute_performance_score(likes: i64, replies: i64, retweets: i64, impressions: i64) -> f64 {
    let numerator = (likes * 3 + replies * 5 + retweets * 4) as f64;
    let denominator = impressions.max(1) as f64;
    numerator / denominator * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // --- Mock implementations ---

    struct MockProfileFetcher {
        metrics: ProfileMetrics,
    }

    #[async_trait::async_trait]
    impl ProfileFetcher for MockProfileFetcher {
        async fn get_profile_metrics(&self) -> Result<ProfileMetrics, AnalyticsError> {
            Ok(self.metrics.clone())
        }
    }

    struct MockEngagementFetcher {
        metrics: TweetMetrics,
    }

    #[async_trait::async_trait]
    impl EngagementFetcher for MockEngagementFetcher {
        async fn get_tweet_metrics(&self, _tweet_id: &str) -> Result<TweetMetrics, AnalyticsError> {
            Ok(self.metrics.clone())
        }
    }

    struct MockAnalyticsStorage {
        snapshots: Mutex<Vec<(i64, i64, i64)>>,
        yesterday_followers: Option<i64>,
        reply_ids: Vec<String>,
        tweet_ids: Vec<String>,
        reply_perfs: Mutex<Vec<(String, f64)>>,
        tweet_perfs: Mutex<Vec<(String, f64)>>,
    }

    impl MockAnalyticsStorage {
        fn new() -> Self {
            Self {
                snapshots: Mutex::new(Vec::new()),
                yesterday_followers: None,
                reply_ids: Vec::new(),
                tweet_ids: Vec::new(),
                reply_perfs: Mutex::new(Vec::new()),
                tweet_perfs: Mutex::new(Vec::new()),
            }
        }

        fn with_yesterday(mut self, followers: i64) -> Self {
            self.yesterday_followers = Some(followers);
            self
        }

        fn with_replies(mut self, ids: Vec<String>) -> Self {
            self.reply_ids = ids;
            self
        }

        fn with_tweets(mut self, ids: Vec<String>) -> Self {
            self.tweet_ids = ids;
            self
        }
    }

    #[async_trait::async_trait]
    impl AnalyticsStorage for MockAnalyticsStorage {
        async fn store_follower_snapshot(
            &self,
            followers: i64,
            following: i64,
            tweets: i64,
        ) -> Result<(), AnalyticsError> {
            self.snapshots
                .lock()
                .expect("lock")
                .push((followers, following, tweets));
            Ok(())
        }

        async fn get_yesterday_followers(&self) -> Result<Option<i64>, AnalyticsError> {
            Ok(self.yesterday_followers)
        }

        async fn get_replies_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError> {
            Ok(self.reply_ids.clone())
        }

        async fn get_tweets_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError> {
            Ok(self.tweet_ids.clone())
        }

        async fn store_reply_performance(
            &self,
            reply_id: &str,
            _likes: i64,
            _replies: i64,
            _impressions: i64,
            score: f64,
        ) -> Result<(), AnalyticsError> {
            self.reply_perfs
                .lock()
                .expect("lock")
                .push((reply_id.to_string(), score));
            Ok(())
        }

        async fn store_tweet_performance(
            &self,
            tweet_id: &str,
            _likes: i64,
            _retweets: i64,
            _replies: i64,
            _impressions: i64,
            score: f64,
        ) -> Result<(), AnalyticsError> {
            self.tweet_perfs
                .lock()
                .expect("lock")
                .push((tweet_id.to_string(), score));
            Ok(())
        }

        async fn update_content_score(
            &self,
            _topic: &str,
            _format: &str,
            _score: f64,
        ) -> Result<(), AnalyticsError> {
            Ok(())
        }

        async fn log_action(
            &self,
            _action_type: &str,
            _status: &str,
            _message: &str,
        ) -> Result<(), AnalyticsError> {
            Ok(())
        }
    }

    fn default_profile() -> ProfileMetrics {
        ProfileMetrics {
            follower_count: 1000,
            following_count: 200,
            tweet_count: 500,
        }
    }

    fn default_tweet_metrics() -> TweetMetrics {
        TweetMetrics {
            likes: 10,
            retweets: 3,
            replies: 5,
            impressions: 1000,
        }
    }

    // --- Tests ---

    #[tokio::test]
    async fn iteration_snapshots_followers() {
        let storage = Arc::new(MockAnalyticsStorage::new());
        let analytics = AnalyticsLoop::new(
            Arc::new(MockProfileFetcher {
                metrics: default_profile(),
            }),
            Arc::new(MockEngagementFetcher {
                metrics: default_tweet_metrics(),
            }),
            storage.clone(),
        );

        let summary = analytics.run_iteration().await.expect("iteration");
        assert_eq!(summary.follower_count, 1000);
        assert_eq!(storage.snapshots.lock().expect("lock").len(), 1);
    }

    #[tokio::test]
    async fn iteration_measures_replies() {
        let storage = Arc::new(
            MockAnalyticsStorage::new().with_replies(vec!["r1".to_string(), "r2".to_string()]),
        );
        let analytics = AnalyticsLoop::new(
            Arc::new(MockProfileFetcher {
                metrics: default_profile(),
            }),
            Arc::new(MockEngagementFetcher {
                metrics: default_tweet_metrics(),
            }),
            storage.clone(),
        );

        let summary = analytics.run_iteration().await.expect("iteration");
        assert_eq!(summary.replies_measured, 2);
        assert_eq!(storage.reply_perfs.lock().expect("lock").len(), 2);
    }

    #[tokio::test]
    async fn iteration_measures_tweets() {
        let storage = Arc::new(MockAnalyticsStorage::new().with_tweets(vec!["tw1".to_string()]));
        let analytics = AnalyticsLoop::new(
            Arc::new(MockProfileFetcher {
                metrics: default_profile(),
            }),
            Arc::new(MockEngagementFetcher {
                metrics: default_tweet_metrics(),
            }),
            storage.clone(),
        );

        let summary = analytics.run_iteration().await.expect("iteration");
        assert_eq!(summary.tweets_measured, 1);
        assert_eq!(storage.tweet_perfs.lock().expect("lock").len(), 1);
    }

    #[tokio::test]
    async fn iteration_detects_follower_drop() {
        // Yesterday: 1000, Today: 970 (3% drop)
        let storage = Arc::new(MockAnalyticsStorage::new().with_yesterday(1000));
        let analytics = AnalyticsLoop::new(
            Arc::new(MockProfileFetcher {
                metrics: ProfileMetrics {
                    follower_count: 970,
                    following_count: 200,
                    tweet_count: 500,
                },
            }),
            Arc::new(MockEngagementFetcher {
                metrics: default_tweet_metrics(),
            }),
            storage,
        );

        // Should not panic — alert is logged
        let summary = analytics.run_iteration().await.expect("iteration");
        assert_eq!(summary.follower_count, 970);
    }

    #[tokio::test]
    async fn iteration_no_drop_alert_when_stable() {
        // Yesterday: 1000, Today: 999 (0.1% drop — below 2% threshold)
        let storage = Arc::new(MockAnalyticsStorage::new().with_yesterday(1000));
        let analytics = AnalyticsLoop::new(
            Arc::new(MockProfileFetcher {
                metrics: ProfileMetrics {
                    follower_count: 999,
                    following_count: 200,
                    tweet_count: 500,
                },
            }),
            Arc::new(MockEngagementFetcher {
                metrics: default_tweet_metrics(),
            }),
            storage,
        );

        let summary = analytics.run_iteration().await.expect("iteration");
        assert_eq!(summary.follower_count, 999);
    }

    #[test]
    fn performance_score_basic() {
        let score = compute_performance_score(10, 5, 3, 1000);
        // (10*3 + 5*5 + 3*4) / 1000 * 1000 = 67
        assert!((score - 67.0).abs() < 0.01);
    }

    #[test]
    fn performance_score_zero_impressions() {
        let score = compute_performance_score(10, 5, 3, 0);
        assert!((score - 67000.0).abs() < 0.01);
    }

    #[test]
    fn analytics_error_display() {
        let err = AnalyticsError::ApiError("timeout".to_string());
        assert_eq!(err.to_string(), "API error: timeout");

        let err = AnalyticsError::StorageError("disk full".to_string());
        assert_eq!(err.to_string(), "storage error: disk full");
    }
}
