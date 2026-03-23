//! Tests for the analytics loop module.

use super::*;
use std::sync::Arc;
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
    forge_sync_result: Option<Result<Option<ForgeSyncResult>, AnalyticsError>>,
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
            forge_sync_result: None,
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

    fn with_forge_sync(mut self, result: Result<Option<ForgeSyncResult>, AnalyticsError>) -> Self {
        self.forge_sync_result = Some(result);
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

    async fn run_forge_sync_if_enabled(&self) -> Result<Option<ForgeSyncResult>, AnalyticsError> {
        match &self.forge_sync_result {
            Some(Ok(v)) => Ok(v.clone()),
            Some(Err(_)) => Err(AnalyticsError::Other("forge sync failed".to_string())),
            None => Ok(None),
        }
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

#[test]
fn analytics_error_display_other() {
    let err = AnalyticsError::Other("unexpected".to_string());
    assert_eq!(err.to_string(), "unexpected");
}

#[test]
fn analytics_error_is_std_error() {
    let err = AnalyticsError::ApiError("test".to_string());
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

#[test]
fn analytics_summary_default() {
    let summary = AnalyticsSummary::default();
    assert_eq!(summary.follower_count, 0);
    assert_eq!(summary.replies_measured, 0);
    assert_eq!(summary.tweets_measured, 0);
}

#[test]
fn profile_metrics_debug_and_clone() {
    let m = ProfileMetrics {
        follower_count: 500,
        following_count: 100,
        tweet_count: 200,
    };
    let m2 = m.clone();
    assert_eq!(m2.follower_count, 500);
    let debug = format!("{m:?}");
    assert!(debug.contains("500"));
}

#[test]
fn tweet_metrics_debug_and_clone() {
    let m = TweetMetrics {
        likes: 5,
        retweets: 2,
        replies: 3,
        impressions: 100,
    };
    let m2 = m.clone();
    assert_eq!(m2.likes, 5);
    let debug = format!("{m:?}");
    assert!(debug.contains("100"));
}

#[test]
fn performance_score_all_zeros() {
    let score = compute_performance_score(0, 0, 0, 0);
    assert!((score - 0.0).abs() < 0.01);
}

#[test]
fn performance_score_high_engagement() {
    let score = compute_performance_score(100, 50, 30, 500);
    // (100*3 + 50*5 + 30*4) / 500 * 1000 = (300+250+120)/500*1000 = 1340
    assert!((score - 1340.0).abs() < 0.01);
}

#[test]
fn performance_score_only_likes() {
    let score = compute_performance_score(10, 0, 0, 100);
    // (10*3 + 0 + 0) / 100 * 1000 = 300
    assert!((score - 300.0).abs() < 0.01);
}

#[test]
fn performance_score_only_replies() {
    let score = compute_performance_score(0, 10, 0, 100);
    // (0 + 10*5 + 0) / 100 * 1000 = 500
    assert!((score - 500.0).abs() < 0.01);
}

#[test]
fn performance_score_only_retweets() {
    let score = compute_performance_score(0, 0, 10, 100);
    // (0 + 0 + 10*4) / 100 * 1000 = 400
    assert!((score - 400.0).abs() < 0.01);
}

#[test]
fn performance_score_negative_impressions_clamped() {
    // Negative impressions should be clamped to 1
    let score = compute_performance_score(1, 1, 1, -5);
    // (3 + 5 + 4) / 1 * 1000 = 12000
    assert!((score - 12000.0).abs() < 0.01);
}

#[test]
fn analytics_error_debug() {
    let err = AnalyticsError::ApiError("timeout".to_string());
    let debug = format!("{err:?}");
    assert!(debug.contains("ApiError"));
    assert!(debug.contains("timeout"));

    let err = AnalyticsError::StorageError("disk full".to_string());
    let debug = format!("{err:?}");
    assert!(debug.contains("StorageError"));

    let err = AnalyticsError::Other("unexpected".to_string());
    let debug = format!("{err:?}");
    assert!(debug.contains("Other"));
}

#[test]
fn analytics_summary_debug() {
    let summary = AnalyticsSummary {
        follower_count: 500,
        replies_measured: 3,
        tweets_measured: 2,
        forge_synced: false,
    };
    let debug = format!("{summary:?}");
    assert!(debug.contains("500"));
    assert!(debug.contains("3"));
    assert!(debug.contains("2"));
}

#[test]
fn analytics_error_source_is_none() {
    let err = AnalyticsError::ApiError("test".to_string());
    // std::error::Error default source() returns None
    assert!(std::error::Error::source(&err).is_none());
}

#[tokio::test]
async fn iteration_with_both_replies_and_tweets() {
    let storage = Arc::new(
        MockAnalyticsStorage::new()
            .with_replies(vec!["r1".to_string()])
            .with_tweets(vec!["t1".to_string(), "t2".to_string()]),
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
    assert_eq!(summary.replies_measured, 1);
    assert_eq!(summary.tweets_measured, 2);
    assert_eq!(summary.follower_count, 1000);
}

#[tokio::test]
async fn iteration_follower_growth_no_alert() {
    // Yesterday: 1000, Today: 1050 (growth, not a drop)
    let storage = Arc::new(MockAnalyticsStorage::new().with_yesterday(1000));
    let analytics = AnalyticsLoop::new(
        Arc::new(MockProfileFetcher {
            metrics: ProfileMetrics {
                follower_count: 1050,
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
    assert_eq!(summary.follower_count, 1050);
}

#[tokio::test]
async fn iteration_no_yesterday_data() {
    let storage = Arc::new(MockAnalyticsStorage::new());
    // No yesterday data — should not crash
    let analytics = AnalyticsLoop::new(
        Arc::new(MockProfileFetcher {
            metrics: default_profile(),
        }),
        Arc::new(MockEngagementFetcher {
            metrics: default_tweet_metrics(),
        }),
        storage,
    );

    let summary = analytics.run_iteration().await.expect("iteration");
    assert_eq!(summary.follower_count, 1000);
}

// --- Failing engagement fetcher ---

struct FailingEngagementFetcher;

#[async_trait::async_trait]
impl EngagementFetcher for FailingEngagementFetcher {
    async fn get_tweet_metrics(&self, _tweet_id: &str) -> Result<TweetMetrics, AnalyticsError> {
        Err(AnalyticsError::ApiError("rate limited".to_string()))
    }
}

#[tokio::test]
async fn iteration_engagement_fetch_failure_continues() {
    let storage = Arc::new(
        MockAnalyticsStorage::new()
            .with_replies(vec!["r1".to_string()])
            .with_tweets(vec!["t1".to_string()]),
    );
    let analytics = AnalyticsLoop::new(
        Arc::new(MockProfileFetcher {
            metrics: default_profile(),
        }),
        Arc::new(FailingEngagementFetcher),
        storage.clone(),
    );

    let summary = analytics.run_iteration().await.expect("iteration");
    // Failures are silently skipped, not counted
    assert_eq!(summary.replies_measured, 0);
    assert_eq!(summary.tweets_measured, 0);
}

// --- Forge sync tests ---

#[tokio::test]
async fn iteration_with_forge_sync_enabled() {
    let storage = Arc::new(MockAnalyticsStorage::new().with_forge_sync(Ok(Some(
        ForgeSyncResult {
            tweets_synced: 5,
            threads_synced: 2,
        },
    ))));
    let analytics = AnalyticsLoop::new(
        Arc::new(MockProfileFetcher {
            metrics: default_profile(),
        }),
        Arc::new(MockEngagementFetcher {
            metrics: default_tweet_metrics(),
        }),
        storage,
    );

    let summary = analytics.run_iteration().await.expect("iteration");
    assert!(summary.forge_synced);
}

#[tokio::test]
async fn iteration_with_forge_sync_disabled() {
    let storage = Arc::new(MockAnalyticsStorage::new().with_forge_sync(Ok(None)));
    let analytics = AnalyticsLoop::new(
        Arc::new(MockProfileFetcher {
            metrics: default_profile(),
        }),
        Arc::new(MockEngagementFetcher {
            metrics: default_tweet_metrics(),
        }),
        storage,
    );

    let summary = analytics.run_iteration().await.expect("iteration");
    assert!(!summary.forge_synced);
}

#[tokio::test]
async fn iteration_forge_sync_failure_non_fatal() {
    let storage = Arc::new(
        MockAnalyticsStorage::new()
            .with_forge_sync(Err(AnalyticsError::Other("disk full".to_string()))),
    );
    let analytics = AnalyticsLoop::new(
        Arc::new(MockProfileFetcher {
            metrics: default_profile(),
        }),
        Arc::new(MockEngagementFetcher {
            metrics: default_tweet_metrics(),
        }),
        storage,
    );

    // Forge sync failure must not fail the iteration
    let summary = analytics.run_iteration().await.expect("iteration");
    assert!(!summary.forge_synced);
    assert_eq!(summary.follower_count, 1000);
}
