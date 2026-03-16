//! Run loop, iteration logic, and slot/interval scheduling.
//!
//! Implements the `run`, `run_once`, `run_iteration`, `run_slot_iteration`,
//! and `log_content_result` methods on [`ContentLoop`].

use super::super::schedule::{apply_slot_jitter, schedule_gate, ActiveSchedule};
use super::super::scheduler::LoopScheduler;
use super::{ContentLoop, ContentResult};
use rand::seq::IndexedRandom;
use rand::SeedableRng;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

impl ContentLoop {
    /// Run the continuous content loop until cancellation.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        scheduler: LoopScheduler,
        schedule: Option<Arc<ActiveSchedule>>,
    ) {
        let slot_mode = schedule.as_ref().is_some_and(|s| s.has_preferred_times());

        tracing::info!(
            dry_run = self.dry_run,
            topics = self.topics.len(),
            window_secs = self.post_window_secs,
            slot_mode = slot_mode,
            "Content loop started"
        );

        if self.topics.is_empty() {
            tracing::warn!("No topics configured, content loop has nothing to post");
            cancel.cancelled().await;
            return;
        }

        let min_recent = 3usize;
        let max_recent = (self.topics.len() / 2)
            .max(min_recent)
            .min(self.topics.len());
        let mut recent_topics: Vec<String> = Vec::with_capacity(max_recent);
        let mut rng = rand::rngs::StdRng::from_rng(&mut rand::rng());

        loop {
            if cancel.is_cancelled() {
                break;
            }

            if !schedule_gate(&schedule, &cancel).await {
                break;
            }

            if slot_mode {
                // Slot-based scheduling: post at preferred times
                let sched = schedule.as_ref().expect("slot_mode requires schedule");

                // Query today's post times from storage
                let today_posts = match self.storage.todays_tweet_times().await {
                    Ok(times) => times,
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to query today's tweet times");
                        Vec::new()
                    }
                };

                match sched.next_unused_slot(&today_posts) {
                    Some((wait, slot)) => {
                        let jittered_wait = apply_slot_jitter(wait);
                        tracing::info!(
                            slot = %slot.format(),
                            wait_secs = jittered_wait.as_secs(),
                            "Slot mode: sleeping until next posting slot"
                        );

                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(jittered_wait) => {},
                        }

                        if cancel.is_cancelled() {
                            break;
                        }

                        // In slot mode, skip the elapsed-time check — post directly
                        let result = self
                            .run_slot_iteration(&mut recent_topics, max_recent, &mut rng)
                            .await;
                        self.log_content_result(&result);
                    }
                    None => {
                        // All slots used today — sleep until next active day
                        tracing::info!(
                            "Slot mode: all slots used today, sleeping until next active period"
                        );
                        let wait = sched.time_until_active();
                        if wait.is_zero() {
                            // Currently active but all slots used — sleep 1 hour and recheck
                            tokio::select! {
                                _ = cancel.cancelled() => break,
                                _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {},
                            }
                        } else {
                            tokio::select! {
                                _ = cancel.cancelled() => break,
                                _ = tokio::time::sleep(wait) => {},
                            }
                        }
                    }
                }
            } else {
                // Interval-based scheduling (existing behavior)
                let result = self
                    .run_iteration(&mut recent_topics, max_recent, &mut rng)
                    .await;
                self.log_content_result(&result);

                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = scheduler.tick() => {},
                }
            }
        }

        tracing::info!("Content loop stopped");
    }

    /// Log the result of a content iteration.
    pub(super) fn log_content_result(&self, result: &ContentResult) {
        match result {
            ContentResult::Posted { topic, content } => {
                tracing::info!(
                    topic = %topic,
                    chars = content.len(),
                    dry_run = self.dry_run,
                    "Content iteration: tweet posted"
                );
            }
            ContentResult::TooSoon {
                elapsed_secs,
                window_secs,
            } => {
                tracing::debug!(
                    elapsed = elapsed_secs,
                    window = window_secs,
                    "Content iteration: too soon since last tweet"
                );
            }
            ContentResult::RateLimited => {
                tracing::info!("Content iteration: daily tweet limit reached");
            }
            ContentResult::NoTopics => {
                tracing::warn!("Content iteration: no topics available");
            }
            ContentResult::Failed { error } => {
                tracing::warn!(error = %error, "Content iteration: failed");
            }
        }
    }

    /// Run a single iteration in slot mode (skips elapsed-time check).
    pub(super) async fn run_slot_iteration(
        &self,
        recent_topics: &mut Vec<String>,
        max_recent: usize,
        rng: &mut impl rand::Rng,
    ) -> ContentResult {
        // Check for manually scheduled content due for posting
        if let Some(result) = self.try_post_scheduled().await {
            return result;
        }

        // Check safety (daily tweet limit)
        if !self.safety.can_post_tweet().await {
            return ContentResult::RateLimited;
        }

        // Pick a topic using epsilon-greedy if scorer is available
        let topic = self.pick_topic_epsilon_greedy(recent_topics, rng).await;

        let result = self.generate_and_post(&topic).await;

        // Update recent_topics on success
        if matches!(result, ContentResult::Posted { .. }) {
            if recent_topics.len() >= max_recent {
                recent_topics.remove(0);
            }
            recent_topics.push(topic);
        }

        result
    }

    /// Run a single content generation (for CLI `tuitbot post` command).
    ///
    /// If `topic` is provided, uses that topic. Otherwise picks a random
    /// topic from the configured list.
    pub async fn run_once(&self, topic: Option<&str>) -> ContentResult {
        let chosen_topic = match topic {
            Some(t) => t.to_string(),
            None => {
                if self.topics.is_empty() {
                    return ContentResult::NoTopics;
                }
                let mut rng = rand::rng();
                self.topics
                    .choose(&mut rng)
                    .expect("topics is non-empty")
                    .clone()
            }
        };

        // Skip window check for single-shot mode, but still check safety
        if !self.safety.can_post_tweet().await {
            return ContentResult::RateLimited;
        }

        self.generate_and_post(&chosen_topic).await
    }

    /// Run a single iteration of the continuous loop.
    pub(super) async fn run_iteration(
        &self,
        recent_topics: &mut Vec<String>,
        max_recent: usize,
        rng: &mut impl rand::Rng,
    ) -> ContentResult {
        // Check for manually scheduled content due for posting
        if let Some(result) = self.try_post_scheduled().await {
            return result;
        }

        // Check elapsed time since last tweet
        match self.storage.last_tweet_time().await {
            Ok(Some(last_time)) => {
                let elapsed = chrono::Utc::now()
                    .signed_duration_since(last_time)
                    .num_seconds()
                    .max(0) as u64;

                if elapsed < self.post_window_secs {
                    return ContentResult::TooSoon {
                        elapsed_secs: elapsed,
                        window_secs: self.post_window_secs,
                    };
                }
            }
            Ok(None) => {
                // No previous tweets -- proceed
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query last tweet time, proceeding anyway");
            }
        }

        // Check safety (daily tweet limit)
        if !self.safety.can_post_tweet().await {
            return ContentResult::RateLimited;
        }

        // Pick a topic using epsilon-greedy if scorer is available
        let topic = self.pick_topic_epsilon_greedy(recent_topics, rng).await;

        let result = self.generate_and_post(&topic).await;

        // Update recent_topics on success
        if matches!(result, ContentResult::Posted { .. }) {
            if recent_topics.len() >= max_recent {
                recent_topics.remove(0);
            }
            recent_topics.push(topic);
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::test_mocks::{make_topics, MockGenerator, MockSafety, MockStorage};
    use super::super::{ContentLoop, ContentResult};
    use std::sync::Arc;

    #[tokio::test]
    async fn run_once_posts_tweet() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Great tweet about Rust!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1);
    }

    #[tokio::test]
    async fn run_once_dry_run_does_not_post() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Great tweet about Rust!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            true,
        );

        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 0); // Not posted in dry-run
        assert_eq!(storage.action_count(), 1); // Action logged
    }

    #[tokio::test]
    async fn run_once_rate_limited() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: false,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(None).await;
        assert!(matches!(result, ContentResult::RateLimited));
    }

    #[tokio::test]
    async fn run_once_no_topics_returns_no_topics() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Vec::new(),
            14400,
            false,
        );

        let result = content.run_once(None).await;
        assert!(matches!(result, ContentResult::NoTopics));
    }

    #[tokio::test]
    async fn run_iteration_skips_when_too_soon() {
        let now = chrono::Utc::now();
        // Last tweet was 1 hour ago, window is 4 hours
        let last_tweet = now - chrono::Duration::hours(1);
        let storage = Arc::new(MockStorage::new(Some(last_tweet)));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400, // 4 hours
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::TooSoon { .. }));
    }

    #[tokio::test]
    async fn run_iteration_posts_when_window_elapsed() {
        let now = chrono::Utc::now();
        // Last tweet was 5 hours ago, window is 4 hours
        let last_tweet = now - chrono::Duration::hours(5);
        let storage = Arc::new(MockStorage::new(Some(last_tweet)));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Fresh tweet!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1);
        assert_eq!(recent.len(), 1);
    }

    // ---------------------------------------------------------------------------
    // run() loop — cancellation coverage
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn run_cancels_immediately_with_topics() {
        // Pre-cancel the token: loop should see is_cancelled() == true and exit
        // without doing any work. Covers lines: slot_mode setup, loop entry, break.
        use crate::automation::scheduler::LoopScheduler;
        use std::time::Duration;
        use tokio_util::sync::CancellationToken;

        let cancel = CancellationToken::new();
        cancel.cancel(); // already cancelled before run() is called

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            3600,
            false,
        );

        let scheduler =
            LoopScheduler::new(Duration::from_secs(3600), Duration::ZERO, Duration::ZERO);
        // Should return immediately — no panic, no post
        content.run(cancel, scheduler, None).await;
    }

    #[tokio::test]
    async fn run_no_topics_exits_on_cancel() {
        // Empty topics: run() logs a warning then awaits cancel.
        // Pre-cancelling means cancel.cancelled().await resolves immediately.
        use crate::automation::scheduler::LoopScheduler;
        use std::time::Duration;
        use tokio_util::sync::CancellationToken;

        let cancel = CancellationToken::new();
        cancel.cancel();

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            vec![], // no topics
            3600,
            false,
        );

        let scheduler = LoopScheduler::new(Duration::from_secs(1), Duration::ZERO, Duration::ZERO);
        content.run(cancel, scheduler, None).await;
    }

    #[tokio::test]
    async fn run_interval_mode_one_iteration_then_cancel() {
        // Interval mode with a very short scheduler interval.
        // The loop runs one iteration, then gets cancelled via a background task.
        use crate::automation::scheduler::LoopScheduler;
        use std::time::Duration;
        use tokio_util::sync::CancellationToken;

        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "interval tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            0, // post_window_secs=0 → always elapsed
            false,
        );

        // Scheduler with 1ms interval so tick() resolves immediately
        let scheduler =
            LoopScheduler::new(Duration::from_millis(1), Duration::ZERO, Duration::ZERO);

        // Cancel after 50ms to let one iteration complete
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            cancel_clone.cancel();
        });

        tokio::time::timeout(Duration::from_secs(5), content.run(cancel, scheduler, None))
            .await
            .expect("run() should complete within timeout");
    }

    // ---------------------------------------------------------------------------
    // log_content_result — all arms
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn log_content_result_all_variants() {
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "t".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            3600,
            false,
        );

        // Exercise every arm — no panics, no assertions needed (these are tracing calls)
        content.log_content_result(&ContentResult::Posted {
            topic: "Rust".to_string(),
            content: "hello".to_string(),
        });
        content.log_content_result(&ContentResult::TooSoon {
            elapsed_secs: 10,
            window_secs: 3600,
        });
        content.log_content_result(&ContentResult::RateLimited);
        content.log_content_result(&ContentResult::NoTopics);
        content.log_content_result(&ContentResult::Failed {
            error: "oops".to_string(),
        });
    }

    // -----------------------------------------------------------------------
    // Additional scheduler coverage tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn run_once_with_specific_topic() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Topic-specific tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(Some("CLI tools")).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        if let ContentResult::Posted { topic, .. } = result {
            assert_eq!(topic, "CLI tools");
        }
    }

    #[tokio::test]
    async fn run_once_random_topic_when_none() {
        let storage = Arc::new(MockStorage::new(None));
        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Random topic tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(None).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
    }

    #[tokio::test]
    async fn run_iteration_posts_when_no_previous_tweet() {
        let storage = Arc::new(MockStorage::new(None)); // No last tweet

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "First ever tweet!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(storage.posted_count(), 1);
    }

    #[tokio::test]
    async fn run_iteration_rate_limited() {
        let now = chrono::Utc::now();
        let last_tweet = now - chrono::Duration::hours(5);
        let storage = Arc::new(MockStorage::new(Some(last_tweet)));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: false, // rate limited
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::RateLimited));
    }

    #[tokio::test]
    async fn run_slot_iteration_rate_limited() {
        let storage = Arc::new(MockStorage::new(None));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: false, // rate limited
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::rng();
        let result = content.run_slot_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::RateLimited));
    }

    #[tokio::test]
    async fn run_slot_iteration_success_updates_recent() {
        let storage = Arc::new(MockStorage::new(None));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "Slot tweet!".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage.clone(),
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::rng();
        let result = content.run_slot_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn run_slot_iteration_caps_recent_topics() {
        let storage = Arc::new(MockStorage::new(None));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        );

        let mut recent = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let max_recent = 3;
        let mut rng = rand::rng();
        let result = content
            .run_slot_iteration(&mut recent, max_recent, &mut rng)
            .await;
        if matches!(result, ContentResult::Posted { .. }) {
            // Recent should have removed oldest and added new
            assert_eq!(recent.len(), max_recent);
        }
    }

    #[tokio::test]
    async fn run_iteration_updates_recent_on_success() {
        let now = chrono::Utc::now();
        let last_tweet = now - chrono::Duration::hours(5);
        let storage = Arc::new(MockStorage::new(Some(last_tweet)));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::rng();
        let result = content.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ContentResult::Posted { .. }));
        assert_eq!(recent.len(), 1);
    }
}
