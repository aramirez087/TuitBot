//! Periodic status summary reporter.
//!
//! Queries action counts at a configurable interval and logs a
//! human-readable summary. Provides users with a heartbeat showing
//! what the agent has been doing without requiring verbose logging.

use super::scheduler::LoopScheduler;
use chrono::{DateTime, Utc};
use tokio_util::sync::CancellationToken;

/// Aggregated action counts for a reporting period.
#[derive(Debug, Clone, Default)]
pub struct ActionCounts {
    /// Number of tweets scored for reply-worthiness.
    pub tweets_scored: u64,
    /// Number of replies sent.
    pub replies_sent: u64,
    /// Number of original tweets posted.
    pub tweets_posted: u64,
    /// Number of thread tweets posted.
    pub threads_posted: u64,
}

impl ActionCounts {
    /// Whether there was any activity in this period.
    pub fn has_activity(&self) -> bool {
        self.tweets_scored > 0
            || self.replies_sent > 0
            || self.tweets_posted > 0
            || self.threads_posted > 0
    }

    /// Format a human-readable summary line.
    pub fn format_summary(&self, interval_label: &str) -> String {
        if self.has_activity() {
            format!(
                "Last {interval_label}: {} tweets scored, {} replies sent, \
                 {} tweets posted, {} threads posted.",
                self.tweets_scored, self.replies_sent, self.tweets_posted, self.threads_posted,
            )
        } else {
            format!("Last {interval_label}: No activity.")
        }
    }
}

/// Trait for querying action counts from the storage layer.
///
/// This trait decouples the status reporter from the actual database,
/// allowing it to be tested with mock implementations.
#[async_trait::async_trait]
pub trait StatusQuerier: Send + Sync {
    /// Query aggregated action counts since the given timestamp.
    async fn query_action_counts_since(&self, since: DateTime<Utc>)
        -> Result<ActionCounts, String>;
}

/// Run the periodic status reporter loop.
///
/// Queries action counts via the `querier` at each scheduler tick and
/// logs a summary. Exits when the cancellation token is triggered.
pub async fn run_status_reporter(
    querier: std::sync::Arc<dyn StatusQuerier>,
    scheduler: LoopScheduler,
    cancel: CancellationToken,
) {
    tracing::info!("Status reporter started");

    let mut last_report = Utc::now();

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                tracing::debug!("Status reporter received cancellation");
                break;
            }
            _ = scheduler.tick() => {}
        }

        if cancel.is_cancelled() {
            break;
        }

        let now = Utc::now();
        let interval_label = format_duration(scheduler.interval());

        match querier.query_action_counts_since(last_report).await {
            Ok(counts) => {
                let summary = counts.format_summary(&interval_label);
                tracing::info!("{summary}");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query action counts for status report");
            }
        }

        last_report = now;
    }

    tracing::info!("Status reporter stopped");
}

/// Format a Duration as a human-readable string (e.g., "5 minutes", "1 hour").
fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{secs} seconds")
    } else if secs < 3600 {
        let mins = secs / 60;
        if mins == 1 {
            "1 minute".to_string()
        } else {
            format!("{mins} minutes")
        }
    } else {
        let hours = secs / 3600;
        if hours == 1 {
            "1 hour".to_string()
        } else {
            format!("{hours} hours")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    struct MockQuerier {
        counts: ActionCounts,
    }

    impl MockQuerier {
        fn with_activity() -> Self {
            Self {
                counts: ActionCounts {
                    tweets_scored: 42,
                    replies_sent: 5,
                    tweets_posted: 2,
                    threads_posted: 1,
                },
            }
        }

        fn no_activity() -> Self {
            Self {
                counts: ActionCounts::default(),
            }
        }
    }

    #[async_trait::async_trait]
    impl StatusQuerier for MockQuerier {
        async fn query_action_counts_since(
            &self,
            _since: DateTime<Utc>,
        ) -> Result<ActionCounts, String> {
            Ok(self.counts.clone())
        }
    }

    struct FailingQuerier;

    #[async_trait::async_trait]
    impl StatusQuerier for FailingQuerier {
        async fn query_action_counts_since(
            &self,
            _since: DateTime<Utc>,
        ) -> Result<ActionCounts, String> {
            Err("database error".to_string())
        }
    }

    #[test]
    fn format_summary_with_activity() {
        let counts = ActionCounts {
            tweets_scored: 10,
            replies_sent: 3,
            tweets_posted: 1,
            threads_posted: 0,
        };
        let summary = counts.format_summary("5 minutes");
        assert!(summary.contains("10 tweets scored"));
        assert!(summary.contains("3 replies sent"));
        assert!(summary.contains("1 tweets posted"));
        assert!(summary.contains("0 threads posted"));
    }

    #[test]
    fn format_summary_no_activity() {
        let counts = ActionCounts::default();
        let summary = counts.format_summary("5 minutes");
        assert_eq!(summary, "Last 5 minutes: No activity.");
    }

    #[test]
    fn has_activity_false_for_default() {
        assert!(!ActionCounts::default().has_activity());
    }

    #[test]
    fn has_activity_true_for_any_field() {
        let mut counts = ActionCounts::default();
        counts.tweets_scored = 1;
        assert!(counts.has_activity());
    }

    #[test]
    fn format_duration_seconds() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30 seconds");
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(300)), "5 minutes");
    }

    #[test]
    fn format_duration_single_minute() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1 minute");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(7200)), "2 hours");
    }

    #[test]
    fn format_duration_single_hour() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "1 hour");
    }

    #[tokio::test]
    async fn reporter_stops_on_cancel() {
        let querier = Arc::new(MockQuerier::with_activity());
        let scheduler = LoopScheduler::new(
            Duration::from_secs(3600), // long interval, won't tick in time
            Duration::ZERO,
            Duration::ZERO,
        );
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let handle = tokio::spawn(async move {
            run_status_reporter(querier, scheduler, cancel_clone).await;
        });

        // Cancel immediately
        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn reporter_runs_one_cycle() {
        let querier = Arc::new(MockQuerier::no_activity());
        let scheduler = LoopScheduler::new(
            Duration::from_millis(10), // short interval
            Duration::ZERO,
            Duration::ZERO,
        );
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let handle = tokio::spawn(async move {
            run_status_reporter(querier, scheduler, cancel_clone).await;
        });

        // Let it run one cycle
        tokio::time::sleep(Duration::from_millis(50)).await;
        cancel.cancel();
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn reporter_handles_query_error() {
        let querier = Arc::new(FailingQuerier);
        let scheduler =
            LoopScheduler::new(Duration::from_millis(10), Duration::ZERO, Duration::ZERO);
        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();
        let handle = tokio::spawn(async move {
            run_status_reporter(querier, scheduler, cancel_clone).await;
        });

        // Let it run one cycle (should not panic on error)
        tokio::time::sleep(Duration::from_millis(50)).await;
        cancel.cancel();
        handle.await.expect("join");
    }
}
