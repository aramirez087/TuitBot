//! Tests for `apply_slot_jitter`, `SLOT_JITTER_SECS`, and `AUTO_PREFERRED_TIMES`.

use std::time::Duration;

use crate::automation::schedule::recurrence::SLOT_JITTER_SECS;
use crate::automation::schedule::{apply_slot_jitter, PostingSlot, AUTO_PREFERRED_TIMES};

// --- SLOT_JITTER_SECS constant ---

#[test]
fn slot_jitter_secs_is_15_minutes() {
    assert_eq!(SLOT_JITTER_SECS, 15 * 60);
    assert_eq!(SLOT_JITTER_SECS, 900);
}

// --- apply_slot_jitter ---

#[test]
fn jitter_within_range_for_1_hour_base() {
    let base = Duration::from_secs(3600);
    for _ in 0..100 {
        let jittered = apply_slot_jitter(base);
        // base ± 15 min → max 3600 + 900 = 4500
        assert!(jittered.as_secs() <= 4500);
    }
}

#[test]
fn jitter_zero_wait_clamps_to_nonnegative() {
    let base = Duration::ZERO;
    let jittered = apply_slot_jitter(base);
    // Should never underflow; max is SLOT_JITTER_SECS
    assert!(jittered.as_secs() <= SLOT_JITTER_SECS);
}

#[test]
fn jitter_large_wait_stays_within_window() {
    let base = Duration::from_secs(86400); // 1 day
    for _ in 0..50 {
        let jittered = apply_slot_jitter(base);
        assert!(jittered.as_secs() >= 86400 - SLOT_JITTER_SECS);
        assert!(jittered.as_secs() <= 86400 + SLOT_JITTER_SECS);
    }
}

#[test]
fn jitter_small_wait_stays_bounded() {
    let base = Duration::from_secs(60);
    for _ in 0..50 {
        let jittered = apply_slot_jitter(base);
        assert!(jittered.as_secs() <= 60 + SLOT_JITTER_SECS);
    }
}

#[test]
fn jitter_many_durations_never_panics() {
    for secs in [0, 1, 60, 900, 3600, 86400] {
        for _ in 0..50 {
            let _ = apply_slot_jitter(Duration::from_secs(secs));
        }
    }
}

// --- AUTO_PREFERRED_TIMES ---

#[test]
fn auto_preferred_times_has_three_entries() {
    assert_eq!(AUTO_PREFERRED_TIMES.len(), 3);
    assert_eq!(AUTO_PREFERRED_TIMES[0], "09:15");
    assert_eq!(AUTO_PREFERRED_TIMES[1], "12:30");
    assert_eq!(AUTO_PREFERRED_TIMES[2], "17:00");
}

#[test]
fn auto_preferred_times_all_parseable() {
    for time_str in AUTO_PREFERRED_TIMES {
        let slot = PostingSlot::parse(time_str);
        assert!(
            slot.is_some(),
            "AUTO_PREFERRED_TIMES entry {time_str} should parse"
        );
    }
}
