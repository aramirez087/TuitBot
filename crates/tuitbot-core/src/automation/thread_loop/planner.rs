//! Run loop, scheduling, iteration, and topic selection.
//!
//! Implements `run`, `run_once`, `run_iteration`, and `log_thread_result`
//! on [`ThreadLoop`].

use super::super::schedule::{apply_slot_jitter, schedule_gate, ActiveSchedule};
use super::super::scheduler::LoopScheduler;
use super::{ThreadLoop, ThreadResult};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

impl ThreadLoop {
    /// Run the continuous thread loop until cancellation.
    pub async fn run(
        &self,
        cancel: CancellationToken,
        scheduler: LoopScheduler,
        schedule: Option<Arc<ActiveSchedule>>,
    ) {
        let slot_mode = schedule
            .as_ref()
            .is_some_and(|s| s.has_thread_preferred_schedule());

        tracing::info!(
            dry_run = self.dry_run,
            topics = self.topics.len(),
            thread_interval_secs = self.thread_interval_secs,
            slot_mode = slot_mode,
            "Thread loop started"
        );

        if self.topics.is_empty() {
            tracing::warn!("No topics configured, thread loop has nothing to post");
            cancel.cancelled().await;
            return;
        }

        let min_recent = 3usize;
        let max_recent = (self.topics.len() / 2)
            .max(min_recent)
            .min(self.topics.len());
        let mut recent_topics: Vec<String> = Vec::with_capacity(max_recent);
        let mut rng = rand::rngs::StdRng::from_entropy();

        loop {
            if cancel.is_cancelled() {
                break;
            }

            if !schedule_gate(&schedule, &cancel).await {
                break;
            }

            if slot_mode {
                let sched = schedule.as_ref().expect("slot_mode requires schedule");

                match sched.next_thread_slot() {
                    Some(wait) => {
                        let jittered_wait = apply_slot_jitter(wait);
                        tracing::info!(
                            wait_secs = jittered_wait.as_secs(),
                            "Thread slot mode: sleeping until preferred thread time"
                        );

                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(jittered_wait) => {},
                        }

                        if cancel.is_cancelled() {
                            break;
                        }

                        if !self.safety.can_post_thread().await {
                            Self::log_thread_result(&ThreadResult::RateLimited, self.dry_run);
                            continue;
                        }

                        let topic = super::pick_topic(&self.topics, &mut recent_topics, &mut rng);
                        let result = self.generate_and_post(&topic, None).await;

                        if matches!(result, ThreadResult::Posted { .. }) {
                            if recent_topics.len() >= max_recent {
                                recent_topics.remove(0);
                            }
                            recent_topics.push(topic);
                        }

                        Self::log_thread_result(&result, self.dry_run);
                    }
                    None => {
                        tracing::warn!("Thread slot mode: no next slot found, sleeping 1 hour");
                        tokio::select! {
                            _ = cancel.cancelled() => break,
                            _ = tokio::time::sleep(Duration::from_secs(3600)) => {},
                        }
                    }
                }
            } else {
                let result = self
                    .run_iteration(&mut recent_topics, max_recent, &mut rng)
                    .await;
                Self::log_thread_result(&result, self.dry_run);

                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = scheduler.tick() => {},
                }
            }
        }

        tracing::info!("Thread loop stopped");
    }

    /// Log the result of a thread iteration.
    pub(super) fn log_thread_result(result: &ThreadResult, dry_run: bool) {
        match result {
            ThreadResult::Posted {
                topic, tweet_count, ..
            } => {
                tracing::info!(
                    topic = %topic,
                    tweets = tweet_count,
                    dry_run = dry_run,
                    "Thread iteration: thread posted"
                );
            }
            ThreadResult::PartialFailure {
                tweets_posted,
                total_tweets,
                error,
                ..
            } => {
                tracing::warn!(
                    posted = tweets_posted,
                    total = total_tweets,
                    error = %error,
                    "Thread iteration: partial failure"
                );
            }
            ThreadResult::TooSoon { .. } => {
                tracing::debug!("Thread iteration: too soon since last thread");
            }
            ThreadResult::RateLimited => {
                tracing::info!("Thread iteration: weekly thread limit reached");
            }
            ThreadResult::NoTopics => {
                tracing::warn!("Thread iteration: no topics available");
            }
            ThreadResult::ValidationFailed { error } => {
                tracing::warn!(error = %error, "Thread iteration: validation failed");
            }
            ThreadResult::Failed { error } => {
                tracing::warn!(error = %error, "Thread iteration: failed");
            }
        }
    }

    /// Run a single thread generation (for CLI `tuitbot thread` command).
    ///
    /// If `topic` is provided, uses that topic. Otherwise picks a random one.
    /// If `count` is provided, generates exactly that many tweets (clamped 2-15).
    pub async fn run_once(&self, topic: Option<&str>, count: Option<usize>) -> ThreadResult {
        let chosen_topic = match topic {
            Some(t) => t.to_string(),
            None => {
                if self.topics.is_empty() {
                    return ThreadResult::NoTopics;
                }
                let mut rng = rand::thread_rng();
                self.topics
                    .choose(&mut rng)
                    .expect("topics is non-empty")
                    .clone()
            }
        };

        let clamped_count = count.map(|c| c.clamp(2, 15));

        if !self.safety.can_post_thread().await {
            return ThreadResult::RateLimited;
        }

        self.generate_and_post(&chosen_topic, clamped_count).await
    }

    /// Run a single iteration of the continuous loop.
    pub(super) async fn run_iteration(
        &self,
        recent_topics: &mut Vec<String>,
        max_recent: usize,
        rng: &mut impl rand::Rng,
    ) -> ThreadResult {
        match self.storage.last_thread_time().await {
            Ok(Some(last_time)) => {
                let elapsed = chrono::Utc::now()
                    .signed_duration_since(last_time)
                    .num_seconds()
                    .max(0) as u64;

                if elapsed < self.thread_interval_secs {
                    return ThreadResult::TooSoon {
                        elapsed_secs: elapsed,
                        interval_secs: self.thread_interval_secs,
                    };
                }
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query last thread time, proceeding anyway");
            }
        }

        if !self.safety.can_post_thread().await {
            return ThreadResult::RateLimited;
        }

        let topic = super::pick_topic(&self.topics, recent_topics, rng);

        let result = self.generate_and_post(&topic, None).await;

        if matches!(result, ThreadResult::Posted { .. }) {
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
    use super::super::test_mocks::{
        make_thread_tweets, make_topics, MockPoster, MockSafety, MockStorage, MockThreadGenerator,
    };
    use super::super::{ThreadLoop, ThreadResult};
    use std::sync::Arc;

    #[tokio::test]
    async fn run_once_rate_limited() {
        let loop_ = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: false,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            make_topics(),
            604800,
            false,
        );

        let result = loop_.run_once(None, None).await;
        assert!(matches!(result, ThreadResult::RateLimited));
    }

    #[tokio::test]
    async fn run_once_no_topics() {
        let loop_ = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            Arc::new(MockPoster::new()),
            Vec::new(),
            604800,
            false,
        );

        let result = loop_.run_once(None, None).await;
        assert!(matches!(result, ThreadResult::NoTopics));
    }

    #[tokio::test]
    async fn run_once_clamps_count() {
        let poster = Arc::new(MockPoster::new());
        let tweets = vec![
            "Tweet 1".to_string(),
            "Tweet 2".to_string(),
            "Tweet 3".to_string(),
        ];

        let loop_ = ThreadLoop::new(
            Arc::new(MockThreadGenerator { tweets }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            poster.clone(),
            make_topics(),
            604800,
            false,
        );

        // count=1 clamped to 2; mock ignores count but result should be Posted
        let result = loop_.run_once(Some("Rust"), Some(1)).await;
        assert!(matches!(result, ThreadResult::Posted { .. }));
    }

    #[tokio::test]
    async fn run_iteration_skips_when_too_soon() {
        let now = chrono::Utc::now();
        let last_thread = now - chrono::Duration::days(3);
        let storage = Arc::new(MockStorage::new(Some(last_thread)));

        let loop_ = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            Arc::new(MockPoster::new()),
            make_topics(),
            604800, // 7 days
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = loop_.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ThreadResult::TooSoon { .. }));
    }

    #[tokio::test]
    async fn run_iteration_posts_when_interval_elapsed() {
        let now = chrono::Utc::now();
        let last_thread = now - chrono::Duration::days(8);
        let storage = Arc::new(MockStorage::new(Some(last_thread)));
        let poster = Arc::new(MockPoster::new());

        let loop_ = ThreadLoop::new(
            Arc::new(MockThreadGenerator {
                tweets: make_thread_tweets(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            poster.clone(),
            make_topics(),
            604800, // 7 days
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();
        let result = loop_.run_iteration(&mut recent, 3, &mut rng).await;
        assert!(matches!(result, ThreadResult::Posted { .. }));
        assert_eq!(poster.posted_count(), 5);
        assert_eq!(recent.len(), 1);
    }
}
