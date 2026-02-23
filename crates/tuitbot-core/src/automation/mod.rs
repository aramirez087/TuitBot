//! Automation runtime, engagement loops, and content generation.
//!
//! This module contains the automation runtime for managing concurrent task
//! lifecycles, the core engagement engine (tweet discovery and mention replies),
//! and original content generation (educational tweets and threads).
//!
//! Submodules:
//! - [`scheduler`]: Loop scheduler with configurable interval and jitter.
//! - [`posting_queue`]: Serialized posting queue for concurrent loops.
//! - [`status_reporter`]: Periodic action count summaries.
//! - [`loop_helpers`]: Shared types, traits, and error handling for loops.
//! - [`mentions_loop`]: Monitors @-mentions and generates replies.
//! - [`discovery_loop`]: Searches tweets by keyword, scores, and replies.
//! - [`content_loop`]: Generates and posts educational tweets.
//! - [`thread_loop`]: Generates and posts multi-tweet threads.

pub mod adapters;
pub mod analytics_loop;
pub mod content_loop;
pub mod discovery_loop;
pub mod loop_helpers;
pub mod mentions_loop;
pub mod posting_queue;
pub mod schedule;
pub mod scheduler;
pub mod status_reporter;
pub mod target_loop;
pub mod thread_loop;

pub use analytics_loop::{
    AnalyticsError, AnalyticsLoop, AnalyticsStorage, AnalyticsSummary, EngagementFetcher,
    ProfileFetcher, ProfileMetrics, TweetMetrics,
};
pub use content_loop::{ContentLoop, ContentResult};
pub use discovery_loop::{DiscoveryLoop, DiscoveryResult, DiscoverySummary};
pub use loop_helpers::{
    ConsecutiveErrorTracker, ContentLoopError, ContentSafety, ContentStorage, LoopError,
    LoopStorage, LoopTweet, MentionsFetcher, PostSender, ReplyGenerator, SafetyChecker,
    ScoreResult, ThreadPoster, TopicScorer, TweetGenerator, TweetScorer, TweetSearcher,
};
pub use mentions_loop::{MentionResult, MentionsLoop};
pub use posting_queue::{
    create_posting_queue, run_posting_queue_with_approval, ApprovalQueue, PostAction, PostExecutor,
    QUEUE_CAPACITY,
};
pub use schedule::{schedule_gate, ActiveSchedule};
pub use scheduler::{scheduler_from_config, LoopScheduler};
pub use status_reporter::{ActionCounts, StatusQuerier};
pub use target_loop::{
    TargetLoop, TargetLoopConfig, TargetResult, TargetStorage, TargetTweetFetcher,
    TargetUserManager,
};
pub use thread_loop::{ThreadGenerator, ThreadLoop, ThreadResult};

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::x_api::auth::TokenManager;
use crate::x_api::XApiHttpClient;

/// Background loop that refreshes the X API OAuth token before it expires.
///
/// Checks every 60 seconds whether the token is within 5 minutes of expiry.
/// On successful refresh, updates the `XApiHttpClient`'s bearer token.
/// On `AuthExpired` error (refresh token revoked), cancels the runtime for
/// graceful shutdown.
pub async fn run_token_refresh_loop(
    token_manager: Arc<TokenManager>,
    x_client: Arc<XApiHttpClient>,
    cancel: CancellationToken,
) {
    let interval = Duration::from_secs(60);
    loop {
        tokio::select! {
            () = cancel.cancelled() => {
                tracing::debug!("Token refresh loop cancelled");
                return;
            }
            () = tokio::time::sleep(interval) => {}
        }

        match token_manager.refresh_if_needed().await {
            Ok(()) => {
                // Update the HTTP client's bearer token with whatever is current.
                let token = token_manager
                    .tokens_lock()
                    .read()
                    .await
                    .access_token
                    .clone();
                x_client.set_access_token(token).await;
            }
            Err(crate::error::XApiError::AuthExpired) => {
                tracing::error!(
                    "Token refresh failed: authentication expired. \
                     Run `tuitbot auth` to re-authenticate. Shutting down."
                );
                cancel.cancel();
                return;
            }
            Err(e) => {
                tracing::warn!(error = %e, "Token refresh attempt failed, will retry next cycle");
            }
        }
    }
}

/// Automation runtime that manages concurrent task lifecycles.
///
/// The runtime owns a `CancellationToken` shared by all spawned tasks
/// and collects their `JoinHandle`s for graceful shutdown. It does not
/// own specific business dependencies -- those are passed when spawning
/// individual loops.
pub struct Runtime {
    cancel: CancellationToken,
    handles: Vec<(String, JoinHandle<()>)>,
}

impl Runtime {
    /// Create a new runtime with a fresh cancellation token.
    pub fn new() -> Self {
        Self {
            cancel: CancellationToken::new(),
            handles: Vec::new(),
        }
    }

    /// Return a clone of the cancellation token for passing to tasks.
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    /// Spawn an automation task with a descriptive name.
    ///
    /// The task's `JoinHandle` is tracked for shutdown. The task should
    /// check `CancellationToken::is_cancelled()` in its loop to exit
    /// gracefully when shutdown is initiated.
    pub fn spawn<F>(&mut self, name: impl Into<String>, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let name = name.into();
        tracing::info!(task = %name, "Spawning automation task");
        let handle = tokio::spawn(future);
        self.handles.push((name, handle));
    }

    /// Return the number of spawned tasks.
    pub fn task_count(&self) -> usize {
        self.handles.len()
    }

    /// Initiate graceful shutdown.
    ///
    /// 1. Cancels the token, signaling all tasks to stop.
    /// 2. Awaits all `JoinHandle`s with a 30-second timeout.
    /// 3. If timeout is exceeded, logs a warning (caller decides whether to force-exit).
    pub async fn shutdown(&mut self) {
        tracing::info!("Initiating graceful shutdown");
        self.cancel.cancel();

        let timeout_duration = Duration::from_secs(30);
        let handles: Vec<_> = self.handles.drain(..).collect();

        let shutdown = async {
            for (name, handle) in handles {
                match handle.await {
                    Ok(()) => tracing::info!(task = %name, "Task completed cleanly"),
                    Err(e) => {
                        tracing::warn!(task = %name, error = %e, "Task panicked during shutdown")
                    }
                }
            }
        };

        if tokio::time::timeout(timeout_duration, shutdown)
            .await
            .is_err()
        {
            tracing::warn!("Shutdown timeout exceeded (30s), some tasks may still be running");
        } else {
            tracing::info!("Graceful shutdown complete");
        }
    }

    /// Block until a shutdown signal is received, then gracefully stop all tasks.
    ///
    /// This is the typical entry point for the `tuitbot run` command:
    /// 1. Spawn all tasks.
    /// 2. Call `run_until_shutdown()` to block until Ctrl+C / SIGTERM.
    /// 3. All tasks are stopped and awaited.
    pub async fn run_until_shutdown(mut self) {
        wait_for_shutdown_signal().await;
        self.shutdown().await;
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for an OS shutdown signal (Ctrl+C or SIGTERM).
///
/// On Unix, listens for both Ctrl+C and SIGTERM. On Windows, listens
/// for Ctrl+C only (SIGTERM is not available).
pub async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to register SIGTERM handler, using Ctrl+C only"
                );
                if let Err(e) = tokio::signal::ctrl_c().await {
                    tracing::error!(error = %e, "Failed to listen for Ctrl+C");
                } else {
                    tracing::info!("Received Ctrl+C");
                }
                return;
            }
        };

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                if let Err(e) = result {
                    tracing::error!(error = %e, "Ctrl+C handler error");
                }
                tracing::info!("Received Ctrl+C");
            }
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM");
            }
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!(error = %e, "Failed to listen for Ctrl+C");
        } else {
            tracing::info!("Received Ctrl+C");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn spawn_and_cancel() {
        let mut runtime = Runtime::new();
        let cancel = runtime.cancel_token();
        let ran = Arc::new(AtomicBool::new(false));

        let ran_clone = ran.clone();
        runtime.spawn("test-task", async move {
            ran_clone.store(true, Ordering::SeqCst);
            cancel.cancelled().await;
        });

        assert_eq!(runtime.task_count(), 1);

        // Give task time to start
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(ran.load(Ordering::SeqCst));

        runtime.shutdown().await;
        assert_eq!(runtime.task_count(), 0);
    }

    #[tokio::test]
    async fn multiple_tasks_all_stopped() {
        let mut runtime = Runtime::new();
        let counter = Arc::new(AtomicU32::new(0));

        for i in 0..5 {
            let cancel = runtime.cancel_token();
            let counter_clone = counter.clone();
            runtime.spawn(format!("task-{i}"), async move {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                cancel.cancelled().await;
            });
        }

        assert_eq!(runtime.task_count(), 5);

        // Let all tasks start
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 5);

        runtime.shutdown().await;
        assert_eq!(runtime.task_count(), 0);
    }

    #[tokio::test]
    async fn shutdown_completes_quickly_for_fast_tasks() {
        let mut runtime = Runtime::new();
        let cancel = runtime.cancel_token();

        runtime.spawn("quick-task", async move {
            cancel.cancelled().await;
            // Simulate brief cleanup
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        let start = tokio::time::Instant::now();
        runtime.shutdown().await;
        let elapsed = start.elapsed();

        // Should complete well within the 30s timeout
        assert!(elapsed < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn shutdown_handles_already_completed_tasks() {
        let mut runtime = Runtime::new();

        runtime.spawn("instant-task", async {
            // Task that finishes immediately
        });

        // Let it finish
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Shutdown should handle already-completed tasks gracefully
        runtime.shutdown().await;
    }

    #[tokio::test]
    async fn cancel_token_is_shared() {
        let runtime = Runtime::new();
        let t1 = runtime.cancel_token();
        let t2 = runtime.cancel_token();

        assert!(!t1.is_cancelled());
        assert!(!t2.is_cancelled());

        t1.cancel();

        assert!(t1.is_cancelled());
        assert!(t2.is_cancelled());
    }

    #[tokio::test]
    async fn default_impl() {
        let runtime = Runtime::default();
        assert_eq!(runtime.task_count(), 0);
        assert!(!runtime.cancel_token().is_cancelled());
    }
}
