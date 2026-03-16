//! Scraper health tracking for `LocalModeXClient`.
//!
//! Tracks consecutive failures, last success, and last error so the
//! `/api/health/detailed` endpoint can surface scraper-specific status
//! without exposing internal implementation details.

use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;

/// Three-state health classification for the local scraper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScraperState {
    /// Operating normally — no recent failures.
    Healthy,
    /// Partial failures — some requests are failing.
    Degraded,
    /// All recent requests failing — scraper is down.
    Down,
}

impl std::fmt::Display for ScraperState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScraperState::Healthy => write!(f, "healthy"),
            ScraperState::Degraded => write!(f, "degraded"),
            ScraperState::Down => write!(f, "down"),
        }
    }
}

/// Snapshot of scraper health at a point in time.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScraperHealthSnapshot {
    /// Overall health classification.
    pub state: ScraperState,
    /// Number of consecutive failures since last success.
    pub consecutive_failures: u32,
    /// ISO-8601 timestamp of the last successful operation.
    pub last_success_at: Option<String>,
    /// Human-readable description of the last error.
    pub last_error: Option<String>,
    /// ISO-8601 timestamp of the last error.
    pub last_error_at: Option<String>,
}

/// Mutable scraper health state, shared via `Arc<Mutex<ScraperHealthState>>`.
///
/// Callers call `record_success` / `record_failure` after each scraper
/// operation. Thresholds:
/// - `consecutive_failures >= 5` → Down
/// - `consecutive_failures >= 2` → Degraded
/// - `consecutive_failures == 0` → Healthy
#[derive(Debug, Default)]
pub struct ScraperHealthState {
    consecutive_failures: u32,
    last_success_at: Option<String>,
    last_error: Option<String>,
    last_error_at: Option<String>,
}

impl ScraperHealthState {
    /// Record a successful scraper operation.
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_success_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
    }

    /// Record a failed scraper operation.
    pub fn record_failure(&mut self, error: &str) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.last_error = Some(error.to_string());
        self.last_error_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
    }

    /// Derive the overall health state from consecutive failures.
    pub fn state(&self) -> ScraperState {
        match self.consecutive_failures {
            0 => ScraperState::Healthy,
            1..=4 => ScraperState::Degraded,
            _ => ScraperState::Down,
        }
    }

    /// Produce a point-in-time snapshot suitable for serialisation.
    pub fn snapshot(&self) -> ScraperHealthSnapshot {
        ScraperHealthSnapshot {
            state: self.state(),
            consecutive_failures: self.consecutive_failures,
            last_success_at: self.last_success_at.clone(),
            last_error: self.last_error.clone(),
            last_error_at: self.last_error_at.clone(),
        }
    }
}

/// Thread-safe handle for scraper health reporting.
pub type ScraperHealth = Arc<Mutex<ScraperHealthState>>;

/// Create a new, default-initialised [`ScraperHealth`] handle.
pub fn new_scraper_health() -> ScraperHealth {
    Arc::new(Mutex::new(ScraperHealthState::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_healthy() {
        let s = ScraperHealthState::default();
        assert_eq!(s.state(), ScraperState::Healthy);
        assert_eq!(s.consecutive_failures, 0);
    }

    #[test]
    fn one_failure_is_degraded() {
        let mut s = ScraperHealthState::default();
        s.record_failure("timeout");
        assert_eq!(s.state(), ScraperState::Degraded);
        assert_eq!(s.consecutive_failures, 1);
        assert!(s.last_error.as_deref() == Some("timeout"));
    }

    #[test]
    fn four_failures_still_degraded() {
        let mut s = ScraperHealthState::default();
        for _ in 0..4 {
            s.record_failure("err");
        }
        assert_eq!(s.state(), ScraperState::Degraded);
    }

    #[test]
    fn five_failures_is_down() {
        let mut s = ScraperHealthState::default();
        for _ in 0..5 {
            s.record_failure("err");
        }
        assert_eq!(s.state(), ScraperState::Down);
    }

    #[test]
    fn success_resets_to_healthy() {
        let mut s = ScraperHealthState::default();
        for _ in 0..10 {
            s.record_failure("err");
        }
        assert_eq!(s.state(), ScraperState::Down);
        s.record_success();
        assert_eq!(s.state(), ScraperState::Healthy);
        assert_eq!(s.consecutive_failures, 0);
    }

    #[test]
    fn snapshot_fields_populated() {
        let mut s = ScraperHealthState::default();
        s.record_failure("network error");
        let snap = s.snapshot();
        assert_eq!(snap.consecutive_failures, 1);
        assert_eq!(snap.last_error.as_deref(), Some("network error"));
        assert!(snap.last_error_at.is_some());
        assert!(snap.last_success_at.is_none());
    }

    #[test]
    fn state_display() {
        assert_eq!(ScraperState::Healthy.to_string(), "healthy");
        assert_eq!(ScraperState::Degraded.to_string(), "degraded");
        assert_eq!(ScraperState::Down.to_string(), "down");
    }
}
