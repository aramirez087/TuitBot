//! Active hours schedule for timezone-aware posting windows.
//!
//! Prevents the bot from posting during off-hours by gating automation
//! loops behind a configurable active window. Supports IANA timezones
//! with automatic DST handling via `chrono-tz`.
//!
//! Submodules:
//! - [`recurrence`]: `PostingSlot`, `apply_slot_jitter`, jitter constant.
//! - [`planner`]: `ActiveSchedule` construction, slot resolution, active-window logic.
//! - [`executor`]: `schedule_gate` async gate function.

mod executor;
mod planner;
mod recurrence;
#[cfg(test)]
mod tests;

pub use executor::schedule_gate;
pub use planner::{ActiveSchedule, AUTO_PREFERRED_TIMES};
pub use recurrence::{apply_slot_jitter, PostingSlot};
