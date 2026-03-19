//! Schedule gate: async helper that sleeps until the active posting window opens.

use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use super::planner::ActiveSchedule;

/// Async gate that sleeps until the active window opens.
///
/// Returns `true` if the loop should continue, `false` if cancelled.
/// If `schedule` is `None`, always returns `true` immediately.
pub async fn schedule_gate(
    schedule: &Option<Arc<ActiveSchedule>>,
    cancel: &CancellationToken,
) -> bool {
    let schedule = match schedule {
        Some(s) => s,
        None => return true,
    };

    if schedule.is_active() {
        return true;
    }

    let wait = schedule.time_until_active();
    tracing::info!(
        wait_secs = wait.as_secs(),
        "Outside active hours, sleeping until active window"
    );

    tokio::select! {
        _ = cancel.cancelled() => false,
        _ = tokio::time::sleep(wait) => true,
    }
}
