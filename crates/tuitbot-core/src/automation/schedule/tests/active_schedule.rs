//! Tests for `ActiveSchedule::from_config`, `is_active`, `time_until_active`,
//! and the `parse_weekday` helper.

use std::time::Duration;

use chrono::{Datelike, Timelike, Utc};

use crate::automation::schedule::planner::parse_weekday;
use crate::automation::schedule::ActiveSchedule;

use super::default_schedule_config;

// --- from_config: timezone parsing ---

#[test]
fn valid_utc_timezone() {
    let config = default_schedule_config();
    assert!(ActiveSchedule::from_config(&config).is_some());
}

#[test]
fn invalid_timezone_returns_none() {
    let mut config = default_schedule_config();
    config.timezone = "Invalid/Timezone".to_string();
    assert!(ActiveSchedule::from_config(&config).is_none());
}

#[test]
fn america_new_york_parses() {
    let mut config = default_schedule_config();
    config.timezone = "America/New_York".to_string();
    assert!(ActiveSchedule::from_config(&config).is_some());
}

#[test]
fn multiple_timezones_parse() {
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

// --- from_config: active_days ---

#[test]
fn all_seven_days_parsed() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 7);
}

#[test]
fn empty_active_days_means_all_days() {
    let mut config = default_schedule_config();
    config.active_days = vec![];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.active_weekdays.is_empty());
}

#[test]
fn invalid_day_ignored() {
    let mut config = default_schedule_config();
    config.active_days = vec!["Mon".to_string(), "Funday".to_string(), "Fri".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 2);
}

#[test]
fn subset_of_days_parsed() {
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
fn weekday_only_active_days() {
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
fn weekend_only_active_days() {
    let mut config = default_schedule_config();
    config.active_days = vec!["Sat".to_string(), "Sun".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.active_weekdays.len(), 2);
    assert!(schedule.active_weekdays.contains(&chrono::Weekday::Sat));
    assert!(schedule.active_weekdays.contains(&chrono::Weekday::Sun));
}

// --- from_config: active_hours bounds ---

#[test]
fn wrapping_range_fields_stored() {
    let mut config = default_schedule_config();
    config.active_hours_start = 22;
    config.active_hours_end = 6;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 22);
    assert_eq!(schedule.end_hour, 6);
}

#[test]
fn wrapping_hours_night_owl() {
    let mut config = default_schedule_config();
    config.active_hours_start = 20;
    config.active_hours_end = 4;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 20);
    assert_eq!(schedule.end_hour, 4);
}

#[test]
fn wrapping_range_late_start() {
    let mut config = default_schedule_config();
    config.active_hours_start = 23;
    config.active_hours_end = 5;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 23);
    assert_eq!(schedule.end_hour, 5);
}

#[test]
fn narrow_active_hours() {
    let mut config = default_schedule_config();
    config.active_hours_start = 12;
    config.active_hours_end = 13;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 12);
    assert_eq!(schedule.end_hour, 13);
}

#[test]
fn same_start_end_degenerate() {
    let mut config = default_schedule_config();
    config.active_hours_start = 15;
    config.active_hours_end = 15;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.start_hour, 15);
    assert_eq!(schedule.end_hour, 15);
}

#[test]
fn degenerate_zero_zero_creates_schedule() {
    let mut config = default_schedule_config();
    config.active_hours_start = 0;
    config.active_hours_end = 0;
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let _ = schedule; // degenerate; just verify it constructs
}

// --- is_active ---

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

// --- time_until_active ---

#[test]
fn time_until_active_zero_when_active() {
    let mut config = default_schedule_config();
    config.active_hours_start = 0;
    config.active_hours_end = 23;
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    if schedule.is_active() {
        assert_eq!(schedule.time_until_active(), Duration::ZERO);
    }
}

#[test]
fn time_until_active_nonzero_when_outside() {
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
fn empty_active_days_allows_is_active_check() {
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

// --- parse_weekday ---

#[test]
fn parse_weekday_all_valid() {
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
fn parse_weekday_case_sensitive() {
    assert_eq!(parse_weekday("mon"), None);
    assert_eq!(parse_weekday("MON"), None);
    assert_eq!(parse_weekday("monday"), None);
}

#[test]
fn parse_weekday_with_whitespace() {
    assert_eq!(parse_weekday(" Mon "), Some(chrono::Weekday::Mon));
    assert_eq!(parse_weekday("  Fri  "), Some(chrono::Weekday::Fri));
}
