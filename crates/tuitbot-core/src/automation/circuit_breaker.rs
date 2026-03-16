//! Circuit breaker for X API rate-limit protection.
//!
//! When the X API returns sustained 429/403 errors, the circuit breaker
//! trips from Closed → Open, pausing all mutations. After a cooldown
//! period it transitions to HalfOpen, allowing a single probe mutation.
//! A success resets to Closed; another failure re-opens.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    /// Normal operation — all mutations allowed.
    Closed,
    /// Tripped — mutations are blocked until cooldown expires.
    Open,
    /// Cooldown expired — one probe mutation is allowed.
    HalfOpen,
}

impl std::fmt::Display for BreakerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakerState::Closed => write!(f, "closed"),
            BreakerState::Open => write!(f, "open"),
            BreakerState::HalfOpen => write!(f, "half_open"),
        }
    }
}

/// Internal sliding window of error timestamps.
struct SlidingWindow {
    timestamps: VecDeque<Instant>,
    window: Duration,
}

impl SlidingWindow {
    fn new(window: Duration) -> Self {
        Self {
            timestamps: VecDeque::new(),
            window,
        }
    }

    /// Push a new error timestamp and prune entries outside the window.
    /// Returns the current count of errors within the window.
    fn push(&mut self, now: Instant) -> u32 {
        self.prune(now);
        self.timestamps.push_back(now);
        self.timestamps.len() as u32
    }

    /// Remove entries older than the window.
    fn prune(&mut self, now: Instant) {
        let cutoff = now.checked_sub(self.window).unwrap_or(now);
        while self.timestamps.front().is_some_and(|&ts| ts < cutoff) {
            self.timestamps.pop_front();
        }
    }

    /// Current error count within the window.
    fn count(&self) -> u32 {
        self.timestamps.len() as u32
    }

    /// Clear all entries.
    fn clear(&mut self) {
        self.timestamps.clear();
    }
}

/// Shared circuit breaker protecting the mutation path.
///
/// Create via [`CircuitBreaker::new`] and share as `Arc<CircuitBreaker>`.
pub struct CircuitBreaker {
    inner: Mutex<BreakerInner>,
    tx: watch::Sender<BreakerState>,
    rx: watch::Receiver<BreakerState>,
    error_threshold: u32,
    cooldown: Duration,
}

struct BreakerInner {
    state: BreakerState,
    window: SlidingWindow,
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// - `error_threshold`: number of errors within `window` to trip the breaker.
    /// - `window`: sliding window duration for counting errors.
    /// - `cooldown`: how long to stay Open before transitioning to HalfOpen.
    pub fn new(error_threshold: u32, window: Duration, cooldown: Duration) -> Arc<Self> {
        let (tx, rx) = watch::channel(BreakerState::Closed);
        Arc::new(Self {
            inner: Mutex::new(BreakerInner {
                state: BreakerState::Closed,
                window: SlidingWindow::new(window),
                opened_at: None,
            }),
            tx,
            rx,
            error_threshold,
            cooldown,
        })
    }

    /// Record an error. Returns the new breaker state.
    pub async fn record_error(&self) -> BreakerState {
        let mut inner = self.inner.lock().await;
        let now = Instant::now();
        let count = inner.window.push(now);

        match inner.state {
            BreakerState::Closed => {
                if count >= self.error_threshold {
                    inner.state = BreakerState::Open;
                    inner.opened_at = Some(now);
                    let _ = self.tx.send(BreakerState::Open);
                    tracing::warn!(
                        error_count = count,
                        threshold = self.error_threshold,
                        cooldown_seconds = self.cooldown.as_secs(),
                        "Circuit breaker OPENED — mutations paused"
                    );
                }
            }
            BreakerState::HalfOpen => {
                // Probe failed — re-open.
                inner.state = BreakerState::Open;
                inner.opened_at = Some(now);
                let _ = self.tx.send(BreakerState::Open);
                tracing::warn!("Circuit breaker probe failed — re-opened");
            }
            BreakerState::Open => {
                // Already open; just record the error.
            }
        }
        inner.state
    }

    /// Record a successful mutation.
    pub async fn record_success(&self) {
        let mut inner = self.inner.lock().await;
        if inner.state == BreakerState::HalfOpen {
            inner.state = BreakerState::Closed;
            inner.window.clear();
            inner.opened_at = None;
            let _ = self.tx.send(BreakerState::Closed);
            tracing::info!("Circuit breaker CLOSED — normal operation resumed");
        }
    }

    /// Current breaker state (checks for cooldown-based HalfOpen transition).
    pub async fn state(&self) -> BreakerState {
        let mut inner = self.inner.lock().await;
        self.maybe_transition_to_half_open(&mut inner);
        inner.state
    }

    /// Whether a mutation should be allowed right now.
    pub async fn should_allow_mutation(&self) -> bool {
        let mut inner = self.inner.lock().await;
        self.maybe_transition_to_half_open(&mut inner);
        match inner.state {
            BreakerState::Closed | BreakerState::HalfOpen => true,
            BreakerState::Open => false,
        }
    }

    /// Async gate: blocks until the breaker leaves Open state (or cancel fires).
    ///
    /// Returns `true` if the breaker is now allowing mutations, `false` if cancelled.
    pub async fn wait_until_closed(&self, cancel: &CancellationToken) -> bool {
        let mut rx = self.rx.clone();
        loop {
            {
                let mut inner = self.inner.lock().await;
                self.maybe_transition_to_half_open(&mut inner);
                match inner.state {
                    BreakerState::Closed | BreakerState::HalfOpen => return true,
                    BreakerState::Open => {}
                }
            }

            tokio::select! {
                biased;
                () = cancel.cancelled() => return false,
                result = rx.changed() => {
                    if result.is_err() {
                        // Sender dropped — treat as closed.
                        return true;
                    }
                }
                () = tokio::time::sleep(Duration::from_secs(1)) => {
                    // Periodic re-check for cooldown expiry.
                }
            }
        }
    }

    /// Current error count within the sliding window.
    pub async fn error_count(&self) -> u32 {
        let inner = self.inner.lock().await;
        inner.window.count()
    }

    /// Remaining cooldown seconds (0 if not open).
    pub async fn cooldown_remaining_seconds(&self) -> u64 {
        let inner = self.inner.lock().await;
        if inner.state != BreakerState::Open {
            return 0;
        }
        if let Some(opened_at) = inner.opened_at {
            let elapsed = opened_at.elapsed();
            if elapsed < self.cooldown {
                return (self.cooldown - elapsed).as_secs();
            }
        }
        0
    }

    /// Check if cooldown has expired and transition Open → HalfOpen.
    fn maybe_transition_to_half_open(&self, inner: &mut BreakerInner) {
        if inner.state == BreakerState::Open {
            if let Some(opened_at) = inner.opened_at {
                if opened_at.elapsed() >= self.cooldown {
                    inner.state = BreakerState::HalfOpen;
                    let _ = self.tx.send(BreakerState::HalfOpen);
                    tracing::info!("Circuit breaker HALF-OPEN — allowing probe mutation");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn trip_after_threshold_errors() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60), Duration::from_secs(10));

        assert_eq!(cb.state().await, BreakerState::Closed);
        assert!(cb.should_allow_mutation().await);

        cb.record_error().await;
        cb.record_error().await;
        assert_eq!(cb.state().await, BreakerState::Closed);

        let state = cb.record_error().await;
        assert_eq!(state, BreakerState::Open);
        assert!(!cb.should_allow_mutation().await);
    }

    #[tokio::test]
    async fn cooldown_to_half_open() {
        // Use 500 ms cooldown (was 50 ms) so the Open→HalfOpen assertion at line
        // +2 stays stable even under tarpaulin's instrumentation overhead.
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_millis(500));

        cb.record_error().await;
        assert_eq!(cb.state().await, BreakerState::Open);

        // Wait for cooldown.
        tokio::time::sleep(Duration::from_millis(600)).await;

        assert_eq!(cb.state().await, BreakerState::HalfOpen);
        assert!(cb.should_allow_mutation().await);
    }

    #[tokio::test]
    async fn success_resets_to_closed() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_millis(500));

        cb.record_error().await;
        assert_eq!(cb.state().await, BreakerState::Open);

        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(cb.state().await, BreakerState::HalfOpen);

        cb.record_success().await;
        assert_eq!(cb.state().await, BreakerState::Closed);
        assert_eq!(cb.error_count().await, 0);
    }

    #[tokio::test]
    async fn half_open_failure_reopens() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_millis(500));

        cb.record_error().await;
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(cb.state().await, BreakerState::HalfOpen);

        let state = cb.record_error().await;
        assert_eq!(state, BreakerState::Open);
        assert!(!cb.should_allow_mutation().await);
    }

    #[tokio::test]
    async fn sliding_window_eviction() {
        // Use 1 s window / 1.2 s sleep (was 100 ms / 150 ms) for tarpaulin stability.
        let cb = CircuitBreaker::new(3, Duration::from_millis(1000), Duration::from_secs(10));

        cb.record_error().await;
        cb.record_error().await;
        assert_eq!(cb.error_count().await, 2);

        // Wait for window to expire.
        tokio::time::sleep(Duration::from_millis(1200)).await;

        // Old errors evicted; this single error shouldn't trip.
        let state = cb.record_error().await;
        assert_eq!(state, BreakerState::Closed);
        assert_eq!(cb.error_count().await, 1);
    }

    #[tokio::test]
    async fn wait_until_closed_returns_on_cancel() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_secs(600));
        cb.record_error().await;

        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            cancel_clone.cancel();
        });

        let result = cb.wait_until_closed(&cancel).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn wait_until_closed_returns_on_transition() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_millis(500));
        cb.record_error().await;

        let cancel = CancellationToken::new();
        let result = cb.wait_until_closed(&cancel).await;
        assert!(result);
    }

    #[tokio::test]
    async fn cooldown_remaining_seconds_reports_correctly() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_secs(10));
        assert_eq!(cb.cooldown_remaining_seconds().await, 0);

        cb.record_error().await;
        let remaining = cb.cooldown_remaining_seconds().await;
        assert!(remaining > 0 && remaining <= 10);
    }

    #[test]
    fn breaker_state_display() {
        assert_eq!(BreakerState::Closed.to_string(), "closed");
        assert_eq!(BreakerState::Open.to_string(), "open");
        assert_eq!(BreakerState::HalfOpen.to_string(), "half_open");
    }

    #[tokio::test]
    async fn success_in_closed_state_is_noop() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60), Duration::from_secs(10));
        assert_eq!(cb.state().await, BreakerState::Closed);

        // Recording success in closed state should not change anything
        cb.record_success().await;
        assert_eq!(cb.state().await, BreakerState::Closed);
        assert_eq!(cb.error_count().await, 0);
    }

    #[tokio::test]
    async fn errors_while_open_accumulate() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60), Duration::from_secs(600));

        cb.record_error().await;
        cb.record_error().await;
        assert_eq!(cb.state().await, BreakerState::Open);

        // More errors while open should not panic or change state
        let state = cb.record_error().await;
        assert_eq!(state, BreakerState::Open);
        assert_eq!(cb.error_count().await, 3);
    }

    #[tokio::test]
    async fn cooldown_remaining_zero_when_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60), Duration::from_secs(10));
        assert_eq!(cb.cooldown_remaining_seconds().await, 0);
    }

    #[tokio::test]
    async fn cooldown_remaining_zero_when_half_open() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_millis(500));
        cb.record_error().await;
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(cb.state().await, BreakerState::HalfOpen);
        assert_eq!(cb.cooldown_remaining_seconds().await, 0);
    }

    #[tokio::test]
    async fn multiple_trip_reset_cycles() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_millis(500));

        // Cycle 1: trip → half-open → success → closed
        cb.record_error().await;
        assert_eq!(cb.state().await, BreakerState::Open);
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(cb.state().await, BreakerState::HalfOpen);
        cb.record_success().await;
        assert_eq!(cb.state().await, BreakerState::Closed);

        // Cycle 2: trip again
        cb.record_error().await;
        assert_eq!(cb.state().await, BreakerState::Open);
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(cb.state().await, BreakerState::HalfOpen);

        // Probe fails → re-opens
        cb.record_error().await;
        assert_eq!(cb.state().await, BreakerState::Open);

        // Wait again → half-open → success → closed
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(cb.state().await, BreakerState::HalfOpen);
        cb.record_success().await;
        assert_eq!(cb.state().await, BreakerState::Closed);
        assert_eq!(cb.error_count().await, 0);
    }

    #[tokio::test]
    async fn should_allow_mutation_half_open() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(60), Duration::from_millis(500));
        cb.record_error().await;
        assert!(!cb.should_allow_mutation().await);

        tokio::time::sleep(Duration::from_millis(600)).await;
        assert!(cb.should_allow_mutation().await);
    }

    #[test]
    fn breaker_state_equality() {
        assert_eq!(BreakerState::Closed, BreakerState::Closed);
        assert_ne!(BreakerState::Open, BreakerState::Closed);
        assert_ne!(BreakerState::HalfOpen, BreakerState::Open);
    }

    #[test]
    fn breaker_state_debug() {
        let debug = format!("{:?}", BreakerState::HalfOpen);
        assert!(debug.contains("HalfOpen"));
    }

    #[test]
    fn breaker_state_clone_copy() {
        let state = BreakerState::Open;
        let cloned = state;
        assert_eq!(state, cloned);
    }
}
