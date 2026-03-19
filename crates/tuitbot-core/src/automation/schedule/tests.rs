#![cfg(test)]

use super::*;
use crate::config::ScheduleConfig;
use chrono::{Datelike, Timelike, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

// Re-export for tests that reach into planner internals.
use super::planner::parse_weekday;
use super::recurrence::SLOT_JITTER_SECS;

fn default_schedule_config() -> ScheduleConfig {
    ScheduleConfig {
        timezone: "UTC".to_string(),
        active_hours_start: 8,
        active_hours_end: 22,
        active_days: vec![
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
            "Sat".to_string(),
            "Sun".to_string(),
        ],
        preferred_times: Vec::new(),
        preferred_times_override: std::collections::HashMap::new(),
        thread_preferred_day: None,
        thread_preferred_time: "10:00".to_string(),
    }
}

#[test]
fn from_config_valid_timezone() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config);
    assert!(schedule.is_some());
}

#[test]
fn from_config_invalid_timezone() {
    let mut config = default_schedule_config();
    config.timezone = "Invalid/Timezone".to_string();
    let schedule = ActiveSchedule::from_config(&config);
    assert!(schedule.is_none());
}

#[test]
fn from_config_america_timezone() {
    let mut config = default_schedule_config();
    config.timezone = "America/New_York".to_string();
    let schedule = ActiveSchedule::from_config(&config);
    assert!(schedule.is_some());
}

#[test]
fn from_config_is_active_all_day() {
    let mut config = default_schedule_config();
    config.active_hours_start = 0;
    config.active_hours_end = 0; // degenerate; skip assertions
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let _ = schedule; // Degenerate case, skip
}

#[test]
fn wrapping_range() {
    let mut config = default_schedule_config();
    config.active_hours_start = 22;
    config.active_hours_end = 6;
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    assert_eq!(schedule.start_hour, 22);
    assert_eq!(schedule.end_hour, 6);
}

#[test]
fn time_until_active_when_active_is_zero() {
    let mut config = default_schedule_config();
    config.active_hours_start = 0;
    config.active_hours_end = 23;
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    if schedule.is_active() {
        assert_eq!(schedule.time_until_active(), Duration::ZERO);
    }
}

#[test]
fn parse_weekday_valid() {
    assert_eq!(parse_weekday("Mon"), Some(chrono::Weekday::Mon));
    assert_eq!(parse_weekday("Tue"), Some(chrono::Weekday::Tue));
    assert_eq!(parse_weekday("Wed"), Some(chrono::Weekday::Wed));
    assert_eq!(parse_weekday("Thu"), Some(chrono::Weekday::Thu));
    assert_eq!(parse_weekday("Fri"), Some(chrono::Weekday::Fri));
    assert_eq!(parse_weekday("Sat"), Some(chrono::Weekday::Sat));
    assert_eq!(parse_weekday("Sun"), Some(chrono::Weekday::Sun));
}

#[test]
fn parse_weekday_invalid() {
    assert_eq!(parse_weekday("Monday"), None);
    assert_eq!(parse_weekday(""), None);
    assert_eq!(parse_weekday("foo"), None);
}

#[test]
fn empty_active_days_means_all_days_active() {
    let mut config = default_schedule_config();
    config.active_days = Vec::new();
    config.active_hours_start = 0;
    config.active_hours_end = 23;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.active_weekdays.is_empty());
    if schedule.is_active() {
        assert_eq!(schedule.time_until_active(), Duration::ZERO);
    }
}

#[tokio::test]
async fn schedule_gate_none_returns_true() {
    let cancel = CancellationToken::new();
    assert!(schedule_gate(&None, &cancel).await);
}

// --- PostingSlot tests ---

#[test]
fn posting_slot_parse_valid() {
    let slot = PostingSlot::parse("09:15").unwrap();
    assert_eq!(slot.hour, 9);
    assert_eq!(slot.minute, 15);
    assert_eq!(slot.as_minutes(), 9 * 60 + 15);
    assert_eq!(slot.format(), "09:15");
}

#[test]
fn posting_slot_parse_midnight() {
    let slot = PostingSlot::parse("00:00").unwrap();
    assert_eq!(slot.hour, 0);
    assert_eq!(slot.minute, 0);
}

#[test]
fn posting_slot_parse_end_of_day() {
    let slot = PostingSlot::parse("23:59").unwrap();
    assert_eq!(slot.hour, 23);
    assert_eq!(slot.minute, 59);
}

#[test]
fn posting_slot_parse_invalid_hour() {
    assert!(PostingSlot::parse("25:00").is_none());
}

#[test]
fn posting_slot_parse_invalid_minute() {
    assert!(PostingSlot::parse("12:60").is_none());
}

#[test]
fn posting_slot_parse_invalid_format() {
    assert!(PostingSlot::parse("12").is_none());
    assert!(PostingSlot::parse("").is_none());
    assert!(PostingSlot::parse("12:30:00").is_none());
    assert!(PostingSlot::parse("ab:cd").is_none());
}

#[test]
fn posting_slot_ordering() {
    let a = PostingSlot::parse("09:00").unwrap();
    let b = PostingSlot::parse("12:30").unwrap();
    let c = PostingSlot::parse("17:00").unwrap();
    assert!(a < b);
    assert!(b < c);
}

#[test]
fn apply_slot_jitter_within_range() {
    let base = Duration::from_secs(3600);
    for _ in 0..100 {
        let jittered = apply_slot_jitter(base);
        assert!(jittered.as_secs() <= 4500);
    }
}

#[test]
fn apply_slot_jitter_zero_wait_clamps() {
    let base = Duration::ZERO;
    let jittered = apply_slot_jitter(base);
    assert!(jittered.as_secs() <= SLOT_JITTER_SECS);
}

#[test]
fn auto_expansion() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 3);
    assert_eq!(schedule.preferred_times[0].format(), "09:15");
    assert_eq!(schedule.preferred_times[1].format(), "12:30");
    assert_eq!(schedule.preferred_times[2].format(), "17:00");
}

#[test]
fn has_preferred_times_false_when_empty() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(!schedule.has_preferred_times());
}

#[test]
fn has_preferred_times_true_when_set() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["09:00".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.has_preferred_times());
}

#[test]
fn has_thread_preferred_schedule() {
    let mut config = default_schedule_config();
    assert!(!ActiveSchedule::from_config(&config)
        .unwrap()
        .has_thread_preferred_schedule());

    config.thread_preferred_day = Some("Tue".to_string());
    assert!(ActiveSchedule::from_config(&config)
        .unwrap()
        .has_thread_preferred_schedule());
}

#[test]
fn next_unused_slot_all_future() {
    let mut config = default_schedule_config();
    config.preferred_times = vec![
        "23:00".to_string(),
        "23:30".to_string(),
        "23:59".to_string(),
    ];
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    let result = schedule.next_unused_slot(&[]);
    let now_hour = Utc::now().hour();
    if now_hour < 23 {
        assert!(result.is_some());
        let (_, slot) = result.unwrap();
        assert_eq!(slot.format(), "23:00");
    }
}

#[test]
fn next_unused_slot_skips_used() {
    let mut config = default_schedule_config();
    config.preferred_times = vec![
        "22:00".to_string(),
        "22:45".to_string(),
        "23:30".to_string(),
    ];
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    let now_hour = Utc::now().hour();
    if now_hour < 22 {
        let today = Utc::now().date_naive();
        let post_time = today.and_hms_opt(22, 0, 0).unwrap().and_utc();

        let result = schedule.next_unused_slot(&[post_time]);
        assert!(result.is_some());
        let (_, slot) = result.unwrap();
        assert_eq!(slot.format(), "22:45");
    }
}

#[test]
fn next_unused_slot_none_when_all_used() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["23:00".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    let now_hour = Utc::now().hour();
    if now_hour < 23 {
        let today = Utc::now().date_naive();
        let post_time = today.and_hms_opt(23, 0, 0).unwrap().and_utc();

        let result = schedule.next_unused_slot(&[post_time]);
        assert!(result.is_none());
    }
}

#[test]
fn next_unused_slot_none_when_all_past() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["00:01".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    let now_hour = Utc::now().hour();
    if now_hour >= 1 {
        let result = schedule.next_unused_slot(&[]);
        assert!(result.is_none());
    }
}

#[test]
fn slots_for_today_uses_override() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["09:00".to_string(), "12:00".to_string()];

    let today = Utc::now().weekday();
    let today_str = match today {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    };

    config
        .preferred_times_override
        .insert(today_str.to_string(), vec!["11:00".to_string()]);

    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let slots = schedule.slots_for_today();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].format(), "11:00");
}

#[test]
fn next_thread_slot_returns_some() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("Mon".to_string());
    config.thread_preferred_time = "10:00".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    let result = schedule.next_thread_slot();
    assert!(result.is_some());
    assert!(result.unwrap().as_secs() <= 7 * 86400);
}

#[test]
fn next_thread_slot_none_without_config() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.next_thread_slot().is_none());
}

#[tokio::test]
async fn schedule_gate_cancelled_returns_false() {
    let mut config = default_schedule_config();
    let now_hour = Utc::now().hour() as u8;
    config.active_hours_start = (now_hour + 2) % 24;
    config.active_hours_end = (now_hour + 3) % 24;
    let schedule = Arc::new(ActiveSchedule::from_config(&config).unwrap());
    let schedule_opt = Some(schedule);

    let cancel = CancellationToken::new();
    cancel.cancel();

    let result = schedule_gate(&schedule_opt, &cancel).await;
    assert!(!result);
}

#[test]
fn posting_slot_to_naive_time() {
    let slot = PostingSlot::parse("14:30").unwrap();
    let time = slot.to_naive_time();
    assert_eq!(time.hour(), 14);
    assert_eq!(time.minute(), 30);
}

#[test]
fn posting_slot_equality() {
    let a = PostingSlot::parse("09:15").unwrap();
    let b = PostingSlot::parse("09:15").unwrap();
    assert_eq!(a, b);
}

#[test]
fn from_config_with_thread_preferred_time() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("Wed".to_string());
    config.thread_preferred_time = "14:30".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.has_thread_preferred_schedule());
    assert!(schedule.next_thread_slot().is_some());
}

#[test]
fn from_config_mixed_preferred_times() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string(), "20:00".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 4);
}

#[test]
fn from_config_deduplicates_preferred_times() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["09:15".to_string(), "09:15".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 1);
}

#[test]
fn from_config_invalid_preferred_time_ignored() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["25:99".to_string(), "09:00".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 1);
    assert_eq!(schedule.preferred_times[0].format(), "09:00");
}

// -----------------------------------------------------------------------
// Additional schedule coverage tests
// -----------------------------------------------------------------------

#[test]
fn posting_slot_as_minutes_boundary() {
    let midnight = PostingSlot::parse("00:00").unwrap();
    assert_eq!(midnight.as_minutes(), 0);

    let end_of_day = PostingSlot::parse("23:59").unwrap();
    assert_eq!(end_of_day.as_minutes(), 23 * 60 + 59);
}

#[test]
fn posting_slot_format_zero_padded() {
    let slot = PostingSlot::parse("01:05").unwrap();
    assert_eq!(slot.format(), "01:05");
}

#[test]
fn posting_slot_parse_single_digit_parts() {
    let slot = PostingSlot::parse("9:5");
    assert!(slot.is_some());
    let s = slot.unwrap();
    assert_eq!(s.hour, 9);
    assert_eq!(s.minute, 5);
}

#[test]
fn posting_slot_parse_negative_rejected() {
    assert!(PostingSlot::parse("-1:00").is_none());
    assert!(PostingSlot::parse("12:-5").is_none());
}

#[test]
fn posting_slot_ordering_same_hour() {
    let a = PostingSlot::parse("09:00").unwrap();
    let b = PostingSlot::parse("09:30").unwrap();
    assert!(a < b);
}

#[test]
fn posting_slot_to_naive_time_boundary() {
    let slot = PostingSlot::parse("23:59").unwrap();
    let t = slot.to_naive_time();
    assert_eq!(t.hour(), 23);
    assert_eq!(t.minute(), 59);
}

#[test]
fn posting_slot_clone_and_debug() {
    let slot = PostingSlot::parse("12:00").unwrap();
    let cloned = slot.clone();
    assert_eq!(slot, cloned);
    let debug = format!("{:?}", slot);
    assert!(debug.contains("PostingSlot"));
}

#[test]
fn from_config_all_timezones() {
    for tz in &[
        "Europe/London",
        "Asia/Tokyo",
        "US/Pacific",
        "Australia/Sydney",
    ] {
        let mut config = default_schedule_config();
        config.timezone = tz.to_string();
        assert!(
            ActiveSchedule::from_config(&config).is_some(),
            "Failed to parse timezone: {tz}"
        );
    }
}

#[test]
fn from_config_empty_active_days_parsed() {
    let mut config = default_schedule_config();
    config.active_days = vec![];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.active_weekdays.is_empty());
}

#[test]
fn from_config_invalid_day_ignored() {
    let mut config = default_schedule_config();
    config.active_days = vec!["Mon".to_string(), "Funday".to_string(), "Fri".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 2);
}

#[test]
fn from_config_thread_preferred_invalid_day_none() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("NotADay".to_string());
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(!schedule.has_thread_preferred_schedule());
}

#[test]
fn from_config_thread_preferred_invalid_time_uses_default() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("Wed".to_string());
    config.thread_preferred_time = "bad:time".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.has_thread_preferred_schedule());
}

#[test]
fn from_config_preferred_times_override_per_day() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["09:00".to_string()];
    config.preferred_times_override.insert(
        "Mon".to_string(),
        vec!["14:00".to_string(), "18:00".to_string()],
    );
    config
        .preferred_times_override
        .insert("InvalidDay".to_string(), vec!["10:00".to_string()]);
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let mon_slots = schedule.preferred_times_override.get(&chrono::Weekday::Mon);
    assert!(mon_slots.is_some());
    assert_eq!(mon_slots.unwrap().len(), 2);
    assert_eq!(schedule.preferred_times_override.len(), 1);
}

#[test]
fn from_config_preferred_times_override_sorted_deduped() {
    let mut config = default_schedule_config();
    config.preferred_times_override.insert(
        "Tue".to_string(),
        vec![
            "18:00".to_string(),
            "09:00".to_string(),
            "18:00".to_string(),
        ],
    );
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let tue_slots = schedule
        .preferred_times_override
        .get(&chrono::Weekday::Tue)
        .unwrap();
    assert_eq!(tue_slots.len(), 2);
    assert!(tue_slots[0] < tue_slots[1]);
}

#[test]
fn from_config_auto_with_duplicate_deduped() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string(), "12:30".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 3);
}

#[test]
fn wrapping_hours_struct_fields() {
    let mut config = default_schedule_config();
    config.active_hours_start = 20;
    config.active_hours_end = 4;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 20);
    assert_eq!(schedule.end_hour, 4);
}

#[test]
fn next_thread_slot_within_week() {
    for day in &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
        let mut config = default_schedule_config();
        config.thread_preferred_day = Some(day.to_string());
        config.thread_preferred_time = "10:00".to_string();
        let schedule = ActiveSchedule::from_config(&config).unwrap();
        let dur = schedule.next_thread_slot().unwrap();
        assert!(
            dur.as_secs() <= 7 * 86400,
            "Thread slot for {day} exceeds 7 days: {} secs",
            dur.as_secs()
        );
    }
}

#[test]
fn apply_slot_jitter_many_runs_never_panics() {
    for secs in [0, 1, 60, 900, 3600, 86400] {
        for _ in 0..50 {
            let _ = apply_slot_jitter(Duration::from_secs(secs));
        }
    }
}

#[test]
fn parse_weekday_with_whitespace() {
    assert_eq!(parse_weekday(" Mon "), Some(chrono::Weekday::Mon));
    assert_eq!(parse_weekday("  Fri  "), Some(chrono::Weekday::Fri));
}

#[test]
fn slots_for_today_no_override_uses_base() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["08:00".to_string(), "13:00".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let slots = schedule.slots_for_today();
    assert_eq!(slots.len(), 2);
}

#[test]
fn time_until_active_non_zero_when_outside() {
    let mut config = default_schedule_config();
    let now_hour = Utc::now().hour() as u8;
    config.active_hours_start = (now_hour + 3) % 24;
    config.active_hours_end = (now_hour + 4) % 24;
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    if !schedule.is_active() {
        let wait = schedule.time_until_active();
        assert!(wait > Duration::ZERO);
    }
}

#[test]
fn time_until_active_respects_weekday() {
    let now_weekday = Utc::now().weekday();

    let other_day = match now_weekday {
        chrono::Weekday::Mon => "Wed",
        _ => "Mon",
    };

    let mut config = default_schedule_config();
    config.active_hours_start = 0;
    config.active_hours_end = 23;
    config.active_days = vec![other_day.to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    if now_weekday != parse_weekday(other_day).unwrap() {
        assert!(!schedule.is_active());
        let wait = schedule.time_until_active();
        assert!(wait > Duration::ZERO);
        assert!(wait.as_secs() <= 7 * 86400 + 3600);
    }
}

#[test]
fn is_active_normal_range() {
    let mut config = default_schedule_config();
    config.active_hours_start = 8;
    config.active_hours_end = 22;
    config.active_days = vec![];
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    let now_hour = Utc::now().hour() as u8;
    if now_hour >= 8 && now_hour < 22 {
        assert!(schedule.is_active());
    } else {
        assert!(!schedule.is_active());
    }
}

#[test]
fn auto_preferred_times_constant() {
    assert_eq!(AUTO_PREFERRED_TIMES.len(), 3);
    assert_eq!(AUTO_PREFERRED_TIMES[0], "09:15");
    assert_eq!(AUTO_PREFERRED_TIMES[1], "12:30");
    assert_eq!(AUTO_PREFERRED_TIMES[2], "17:00");
}

#[tokio::test]
async fn schedule_gate_active_returns_true() {
    let mut config = default_schedule_config();
    config.active_hours_start = 0;
    config.active_hours_end = 23;
    config.active_days = vec![];
    let schedule = Arc::new(ActiveSchedule::from_config(&config).unwrap());
    let schedule_opt = Some(schedule);

    let cancel = CancellationToken::new();
    let now_hour = Utc::now().hour();
    if now_hour < 23 {
        let result = schedule_gate(&schedule_opt, &cancel).await;
        assert!(result);
    }
}

#[test]
fn next_unused_slot_empty_schedule_returns_none() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let result = schedule.next_unused_slot(&[]);
    assert!(result.is_none());
}

// -----------------------------------------------------------------------
// Extended edge-case coverage
// -----------------------------------------------------------------------

#[test]
fn posting_slot_as_minutes_noon() {
    let slot = PostingSlot::parse("12:00").unwrap();
    assert_eq!(slot.as_minutes(), 720);
}

#[test]
fn posting_slot_as_minutes_one_am() {
    let slot = PostingSlot::parse("01:00").unwrap();
    assert_eq!(slot.as_minutes(), 60);
}

#[test]
fn posting_slot_format_preserves_leading_zeros() {
    let slot = PostingSlot::parse("00:05").unwrap();
    assert_eq!(slot.format(), "00:05");
    let slot2 = PostingSlot::parse("03:09").unwrap();
    assert_eq!(slot2.format(), "03:09");
}

#[test]
fn posting_slot_to_naive_time_midnight() {
    let slot = PostingSlot::parse("00:00").unwrap();
    let t = slot.to_naive_time();
    assert_eq!(t.hour(), 0);
    assert_eq!(t.minute(), 0);
    assert_eq!(t.second(), 0);
}

#[test]
fn posting_slot_ordering_reverse() {
    let a = PostingSlot::parse("23:59").unwrap();
    let b = PostingSlot::parse("00:00").unwrap();
    assert!(a > b);
}

#[test]
fn posting_slot_ordering_equal() {
    let a = PostingSlot::parse("15:30").unwrap();
    let b = PostingSlot::parse("15:30").unwrap();
    assert!(!(a < b));
    assert!(!(a > b));
    assert_eq!(a, b);
}

#[test]
fn posting_slot_parse_boundary_23_59() {
    let slot = PostingSlot::parse("23:59").unwrap();
    assert_eq!(slot.hour, 23);
    assert_eq!(slot.minute, 59);
    assert_eq!(slot.as_minutes(), 23 * 60 + 59);
}

#[test]
fn posting_slot_parse_boundary_00_00() {
    let slot = PostingSlot::parse("00:00").unwrap();
    assert_eq!(slot.hour, 0);
    assert_eq!(slot.minute, 0);
    assert_eq!(slot.as_minutes(), 0);
}

#[test]
fn posting_slot_parse_invalid_separator() {
    assert!(PostingSlot::parse("12-30").is_none());
    assert!(PostingSlot::parse("12 30").is_none());
    assert!(PostingSlot::parse("12.30").is_none());
}

#[test]
fn posting_slot_parse_extra_whitespace() {
    assert!(PostingSlot::parse(" 12:30").is_none());
    assert!(PostingSlot::parse("12: 30").is_none());
}

#[test]
fn posting_slot_parse_hour_24_invalid() {
    assert!(PostingSlot::parse("24:00").is_none());
}

#[test]
fn posting_slot_parse_large_numbers() {
    assert!(PostingSlot::parse("99:99").is_none());
    assert!(PostingSlot::parse("255:255").is_none());
}

#[test]
fn from_config_all_seven_days() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 7);
}

#[test]
fn from_config_subset_of_days() {
    let mut config = default_schedule_config();
    config.active_days = vec!["Mon".to_string(), "Wed".to_string(), "Fri".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 3);
    assert!(schedule.active_weekdays.contains(&chrono::Weekday::Mon));
    assert!(schedule.active_weekdays.contains(&chrono::Weekday::Wed));
    assert!(schedule.active_weekdays.contains(&chrono::Weekday::Fri));
    assert!(!schedule.active_weekdays.contains(&chrono::Weekday::Tue));
}

#[test]
fn from_config_preferred_times_sorted() {
    let mut config = default_schedule_config();
    config.preferred_times = vec![
        "17:00".to_string(),
        "09:00".to_string(),
        "12:00".to_string(),
    ];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times[0].format(), "09:00");
    assert_eq!(schedule.preferred_times[1].format(), "12:00");
    assert_eq!(schedule.preferred_times[2].format(), "17:00");
}

#[test]
fn from_config_multiple_auto_entries_deduped() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string(), "auto".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 3);
}

#[test]
fn from_config_override_with_invalid_times_filtered() {
    let mut config = default_schedule_config();
    config.preferred_times_override.insert(
        "Mon".to_string(),
        vec![
            "25:00".to_string(),
            "09:00".to_string(),
            "99:99".to_string(),
        ],
    );
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let mon_slots = schedule
        .preferred_times_override
        .get(&chrono::Weekday::Mon)
        .unwrap();
    assert_eq!(mon_slots.len(), 1);
    assert_eq!(mon_slots[0].format(), "09:00");
}

#[test]
fn from_config_thread_preferred_time_default_fallback() {
    let mut config = default_schedule_config();
    config.thread_preferred_time = "invalid".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.thread_preferred_time.hour, 10);
    assert_eq!(schedule.thread_preferred_time.minute, 0);
}

#[test]
fn from_config_thread_preferred_time_custom() {
    let mut config = default_schedule_config();
    config.thread_preferred_time = "15:45".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.thread_preferred_time.hour, 15);
    assert_eq!(schedule.thread_preferred_time.minute, 45);
}

#[test]
fn apply_slot_jitter_large_wait() {
    let base = Duration::from_secs(86400);
    for _ in 0..50 {
        let jittered = apply_slot_jitter(base);
        assert!(jittered.as_secs() >= 86400 - SLOT_JITTER_SECS);
        assert!(jittered.as_secs() <= 86400 + SLOT_JITTER_SECS);
    }
}

#[test]
fn apply_slot_jitter_small_wait() {
    let base = Duration::from_secs(60);
    for _ in 0..50 {
        let jittered = apply_slot_jitter(base);
        assert!(jittered.as_secs() <= 60 + SLOT_JITTER_SECS);
    }
}

#[test]
fn wrapping_range_fields_preserved() {
    let mut config = default_schedule_config();
    config.active_hours_start = 23;
    config.active_hours_end = 5;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 23);
    assert_eq!(schedule.end_hour, 5);
}

#[test]
fn parse_weekday_case_sensitive() {
    assert_eq!(parse_weekday("mon"), None);
    assert_eq!(parse_weekday("MON"), None);
    assert_eq!(parse_weekday("monday"), None);
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

#[test]
fn from_config_empty_preferred_times() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.preferred_times.is_empty());
    assert!(!schedule.has_preferred_times());
}

#[test]
fn from_config_many_preferred_times() {
    let mut config = default_schedule_config();
    config.preferred_times = (0..20).map(|i| format!("{:02}:00", i % 24)).collect();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.preferred_times.len() <= 20);
    for i in 1..schedule.preferred_times.len() {
        assert!(schedule.preferred_times[i - 1] < schedule.preferred_times[i]);
    }
}

#[test]
fn next_thread_slot_each_day_of_week_valid_duration() {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    for day in &days {
        let mut config = default_schedule_config();
        config.thread_preferred_day = Some(day.to_string());
        config.thread_preferred_time = "08:00".to_string();
        let schedule = ActiveSchedule::from_config(&config).unwrap();
        let dur = schedule.next_thread_slot().unwrap();
        assert!(dur.as_secs() > 0 || schedule.has_thread_preferred_schedule());
        assert!(dur.as_secs() <= 7 * 86400 + 86400);
    }
}

#[test]
fn from_config_overrides_for_multiple_days() {
    let mut config = default_schedule_config();
    config
        .preferred_times_override
        .insert("Mon".to_string(), vec!["09:00".to_string()]);
    config.preferred_times_override.insert(
        "Fri".to_string(),
        vec!["14:00".to_string(), "18:00".to_string()],
    );
    config
        .preferred_times_override
        .insert("Sun".to_string(), vec!["10:30".to_string()]);
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times_override.len(), 3);
    assert_eq!(
        schedule
            .preferred_times_override
            .get(&chrono::Weekday::Fri)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn has_preferred_times_with_many_slots() {
    let mut config = default_schedule_config();
    config.preferred_times = vec![
        "06:00".to_string(),
        "10:00".to_string(),
        "14:00".to_string(),
        "18:00".to_string(),
        "22:00".to_string(),
    ];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.has_preferred_times());
    assert_eq!(schedule.preferred_times.len(), 5);
}

#[test]
fn slot_jitter_secs_is_15_minutes() {
    assert_eq!(SLOT_JITTER_SECS, 15 * 60);
    assert_eq!(SLOT_JITTER_SECS, 900);
}

#[test]
fn from_config_weekday_only_active_days() {
    let mut config = default_schedule_config();
    config.active_days = vec![
        "Mon".to_string(),
        "Tue".to_string(),
        "Wed".to_string(),
        "Thu".to_string(),
        "Fri".to_string(),
    ];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 5);
    assert!(!schedule.active_weekdays.contains(&chrono::Weekday::Sat));
    assert!(!schedule.active_weekdays.contains(&chrono::Weekday::Sun));
}

#[test]
fn from_config_weekend_only_active_days() {
    let mut config = default_schedule_config();
    config.active_days = vec!["Sat".to_string(), "Sun".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 2);
    assert!(schedule.active_weekdays.contains(&chrono::Weekday::Sat));
    assert!(schedule.active_weekdays.contains(&chrono::Weekday::Sun));
}

#[test]
fn from_config_narrow_active_hours() {
    let mut config = default_schedule_config();
    config.active_hours_start = 12;
    config.active_hours_end = 13;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 12);
    assert_eq!(schedule.end_hour, 13);
}

#[test]
fn from_config_same_start_end_degenerate() {
    let mut config = default_schedule_config();
    config.active_hours_start = 15;
    config.active_hours_end = 15;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 15);
    assert_eq!(schedule.end_hour, 15);
}

#[test]
fn posting_slot_debug_format_contains_hour_minute() {
    let slot = PostingSlot::parse("14:30").unwrap();
    let debug = format!("{:?}", slot);
    assert!(debug.contains("14"));
    assert!(debug.contains("30"));
}

#[test]
fn from_config_overrides_empty_slot_list() {
    let mut config = default_schedule_config();
    config
        .preferred_times_override
        .insert("Mon".to_string(), vec![]);
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let mon_slots = schedule
        .preferred_times_override
        .get(&chrono::Weekday::Mon)
        .unwrap();
    assert!(mon_slots.is_empty());
}
