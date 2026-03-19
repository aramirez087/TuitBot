//! Posting slot types and jitter utilities.
//!
//! `PostingSlot` represents a parsed HH:MM posting time. `apply_slot_jitter`
//! adds random ±15-minute noise to a scheduling delay.

use rand::Rng;
use std::time::Duration;

/// Maximum jitter applied to slot-based scheduling (in seconds): +/- 15 minutes.
pub(super) const SLOT_JITTER_SECS: u64 = 15 * 60;

/// A parsed posting time slot (HH:MM).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostingSlot {
    pub(super) hour: u8,
    pub(super) minute: u8,
}

impl PostingSlot {
    /// Parse an "HH:MM" string into a `PostingSlot`.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let hour: u8 = parts[0].parse().ok()?;
        let minute: u8 = parts[1].parse().ok()?;
        if hour > 23 || minute > 59 {
            return None;
        }
        Some(Self { hour, minute })
    }

    /// Total minutes since midnight.
    pub fn as_minutes(&self) -> u32 {
        self.hour as u32 * 60 + self.minute as u32
    }

    /// Format as "HH:MM".
    pub fn format(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    /// Convert to a `NaiveTime`.
    pub fn to_naive_time(&self) -> chrono::NaiveTime {
        chrono::NaiveTime::from_hms_opt(self.hour as u32, self.minute as u32, 0)
            .expect("PostingSlot values are validated on construction")
    }
}

/// Apply random jitter to a slot wait duration (+/- 15 minutes).
///
/// The output is clamped to at least 0 to prevent negative waits.
pub fn apply_slot_jitter(wait: Duration) -> Duration {
    let jitter_secs = rand::rng().random_range(0..=SLOT_JITTER_SECS * 2);
    // offset from -SLOT_JITTER_SECS to +SLOT_JITTER_SECS
    let wait_secs = wait.as_secs() as i64 + jitter_secs as i64 - SLOT_JITTER_SECS as i64;
    Duration::from_secs(wait_secs.max(0) as u64)
}
