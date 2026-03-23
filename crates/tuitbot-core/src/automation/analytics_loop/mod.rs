//! Analytics loop for tracking content performance.
//!
//! Runs periodically to:
//! 1. Snapshot follower counts via the X API.
//! 2. Fetch engagement metrics on content posted ~24h ago.
//! 3. Compute performance scores and update running averages.
//! 4. Alert on significant follower drops.

mod collector;
mod reporter;
#[cfg(test)]
mod tests;

pub use collector::{
    AnalyticsError, AnalyticsStorage, EngagementFetcher, ForgeSyncResult, ProfileFetcher,
    ProfileMetrics, TweetMetrics,
};
pub use reporter::{compute_performance_score, AnalyticsSummary};

use super::loop_helpers::ConsecutiveErrorTracker;
use super::scheduler::LoopScheduler;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

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
    pub async fn run(&self, cancel: CancellationToken, scheduler: LoopScheduler) {
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
                _ = scheduler.tick() => {},
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

        // 4. Forge sync (if enabled)
        match self.storage.run_forge_sync_if_enabled().await {
            Ok(Some(forge_result)) => {
                tracing::info!(
                    tweets_synced = forge_result.tweets_synced,
                    threads_synced = forge_result.threads_synced,
                    "Forge sync complete"
                );
                summary.forge_synced = true;
            }
            Ok(None) => {
                // Forge sync not enabled — no action
            }
            Err(e) => {
                // Forge sync failure is non-fatal
                tracing::warn!(error = %e, "Forge sync failed");
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
