//! Loop scheduler with configurable interval and randomized jitter.
//!
//! Each automation loop instantiates its own `LoopScheduler` with a
//! per-loop base interval (from `[intervals]` config) and global jitter
//! range (from `[limits]` config). The jitter prevents predictable
//! patterns, making the agent's behavior appear more natural.

use rand::Rng;
use std::time::Duration;

/// A scheduler that paces automation loop iterations with jitter.
///
/// Each call to [`tick()`](LoopScheduler::tick) sleeps for `interval + random_jitter`,
/// where `random_jitter` is drawn uniformly from `[min_delay, max_delay]`.
#[derive(Debug, Clone)]
pub struct LoopScheduler {
    interval: Duration,
    min_delay: Duration,
    max_delay: Duration,
}

impl LoopScheduler {
    /// Create a new scheduler.
    ///
    /// If `min_delay > max_delay`, the values are swapped to prevent panics.
    pub fn new(interval: Duration, min_delay: Duration, max_delay: Duration) -> Self {
        let (actual_min, actual_max) = if min_delay <= max_delay {
            (min_delay, max_delay)
        } else {
            tracing::warn!(
                min_ms = min_delay.as_millis() as u64,
                max_ms = max_delay.as_millis() as u64,
                "min_delay > max_delay, swapping values"
            );
            (max_delay, min_delay)
        };

        Self {
            interval,
            min_delay: actual_min,
            max_delay: actual_max,
        }
    }

    /// Compute the next sleep duration: `interval + random_jitter`.
    ///
    /// The jitter is drawn uniformly from `[min_delay, max_delay]`.
    /// If `min_delay == max_delay`, the jitter is fixed (no randomness).
    pub fn next_delay(&self) -> Duration {
        let jitter = if self.min_delay == self.max_delay {
            self.min_delay
        } else {
            let min_ms = self.min_delay.as_millis() as u64;
            let max_ms = self.max_delay.as_millis() as u64;
            Duration::from_millis(rand::thread_rng().gen_range(min_ms..=max_ms))
        };

        self.interval + jitter
    }

    /// Sleep for the next computed delay (interval + jitter).
    pub async fn tick(&self) {
        let delay = self.next_delay();
        tracing::debug!(
            delay_ms = delay.as_millis() as u64,
            interval_ms = self.interval.as_millis() as u64,
            "Scheduler tick sleeping"
        );
        tokio::time::sleep(delay).await;
    }

    /// Return the base interval (without jitter).
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Return the jitter range as `(min_delay, max_delay)`.
    pub fn jitter_range(&self) -> (Duration, Duration) {
        (self.min_delay, self.max_delay)
    }
}

/// Create a `LoopScheduler` from config values.
///
/// Convenience constructor using the config's interval and jitter fields.
pub fn scheduler_from_config(
    interval_seconds: u64,
    min_delay_seconds: u64,
    max_delay_seconds: u64,
) -> LoopScheduler {
    LoopScheduler::new(
        Duration::from_secs(interval_seconds),
        Duration::from_secs(min_delay_seconds),
        Duration::from_secs(max_delay_seconds),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_delay_within_bounds() {
        let scheduler = LoopScheduler::new(
            Duration::from_secs(10),
            Duration::from_secs(1),
            Duration::from_secs(5),
        );

        // Run multiple times to exercise randomness
        for _ in 0..100 {
            let delay = scheduler.next_delay();
            assert!(delay >= Duration::from_secs(11)); // interval + min
            assert!(delay <= Duration::from_secs(15)); // interval + max
        }
    }

    #[test]
    fn next_delay_fixed_jitter() {
        let scheduler = LoopScheduler::new(
            Duration::from_secs(5),
            Duration::from_secs(2),
            Duration::from_secs(2),
        );

        // With equal min/max, delay should always be interval + jitter
        for _ in 0..10 {
            assert_eq!(scheduler.next_delay(), Duration::from_secs(7));
        }
    }

    #[test]
    fn next_delay_zero_jitter() {
        let scheduler = LoopScheduler::new(Duration::from_secs(5), Duration::ZERO, Duration::ZERO);

        assert_eq!(scheduler.next_delay(), Duration::from_secs(5));
    }

    #[test]
    fn constructor_swaps_inverted_min_max() {
        let scheduler = LoopScheduler::new(
            Duration::from_secs(10),
            Duration::from_secs(5), // min > max
            Duration::from_secs(1),
        );

        // After swap: min=1, max=5
        let (min, max) = scheduler.jitter_range();
        assert_eq!(min, Duration::from_secs(1));
        assert_eq!(max, Duration::from_secs(5));
    }

    #[test]
    fn interval_accessor() {
        let scheduler = LoopScheduler::new(Duration::from_secs(42), Duration::ZERO, Duration::ZERO);
        assert_eq!(scheduler.interval(), Duration::from_secs(42));
    }

    #[test]
    fn scheduler_from_config_creates_correctly() {
        let scheduler = scheduler_from_config(300, 30, 120);
        assert_eq!(scheduler.interval(), Duration::from_secs(300));
        let (min, max) = scheduler.jitter_range();
        assert_eq!(min, Duration::from_secs(30));
        assert_eq!(max, Duration::from_secs(120));
    }

    #[tokio::test]
    async fn tick_completes() {
        let scheduler =
            LoopScheduler::new(Duration::from_millis(10), Duration::ZERO, Duration::ZERO);

        let start = tokio::time::Instant::now();
        scheduler.tick().await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
    }
}
