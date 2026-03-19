//! Tests for preferred-time configuration, overrides, `next_unused_slot`,
//! `slots_for_today`, `next_thread_slot`, and `schedule_gate`.

use std::sync::Arc;

use chrono::{Datelike, Timelike, Utc};
use tokio_util::sync::CancellationToken;

use crate::automation::schedule::{schedule_gate, ActiveSchedule, AUTO_PREFERRED_TIMES};

use super::default_schedule_config;

// -----------------------------------------------------------------------
// Preferred-time config
// -----------------------------------------------------------------------

#[test]
fn has_preferred_times_checks() {
    // Empty
    {
        let config = default_schedule_config();
        let schedule = ActiveSchedule::from_config(&config).unwrap();
        assert!(!schedule.has_preferred_times());
        assert!(schedule.preferred_times.is_empty());
    }

    // Set
    {
        let mut config = default_schedule_config();
        config.preferred_times = vec!["09:00".to_string()];
        let schedule = ActiveSchedule::from_config(&config).unwrap();
        assert!(schedule.has_preferred_times());
    }

    // Many slots
    {
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
}

#[test]
fn from_config_preferred_times_handling() {
    // Sorted
    {
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

    // Deduplicated
    {
        let mut config = default_schedule_config();
        config.preferred_times = vec!["09:15".to_string(), "09:15".to_string()];
        let schedule = ActiveSchedule::from_config(&config).unwrap();
        assert_eq!(schedule.preferred_times.len(), 1);
    }

    // Invalid times ignored
    {
        let mut config = default_schedule_config();
        config.preferred_times = vec!["25:99".to_string(), "09:00".to_string()];
        let schedule = ActiveSchedule::from_config(&config).unwrap();
        assert_eq!(schedule.preferred_times.len(), 1);
        assert_eq!(schedule.preferred_times[0].format(), "09:00");
    }

    // Many times
    {
        let mut config = default_schedule_config();
        config.preferred_times = (0..20).map(|i| format!("{:02}:00", i % 24)).collect();
        let schedule = ActiveSchedule::from_config(&config).unwrap();
        assert!(schedule.preferred_times.len() <= 20);
        for i in 1..schedule.preferred_times.len() {
            assert!(schedule.preferred_times[i - 1] < schedule.preferred_times[i]);
        }
    }
}

// --- Auto expansion ---

#[test]
fn auto_expansion_produces_three_slots() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 3);
    assert_eq!(schedule.preferred_times[0].format(), "09:15");
    assert_eq!(schedule.preferred_times[1].format(), "12:30");
    assert_eq!(schedule.preferred_times[2].format(), "17:00");
}

#[test]
fn auto_expansion_deduped_with_extra() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string(), "12:30".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 3); // 12:30 is already in auto
}

#[test]
fn auto_expansion_mixed_with_new_slot() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string(), "20:00".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 4);
}

#[test]
fn multiple_auto_entries_deduped() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["auto".to_string(), "auto".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.preferred_times.len(), 3);
}

// --- Preferred-time overrides ---

#[test]
fn preferred_times_override_per_day() {
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
fn preferred_times_override_sorted_deduped() {
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
fn preferred_times_override_invalid_times_filtered() {
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
fn preferred_times_override_empty_slot_list() {
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

#[test]
fn overrides_for_multiple_days() {
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

// -----------------------------------------------------------------------
// slots_for_today
// -----------------------------------------------------------------------

#[test]
fn slots_for_today_uses_override_for_today() {
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
fn slots_for_today_uses_base_when_no_override() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["08:00".to_string(), "13:00".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let slots = schedule.slots_for_today();
    assert_eq!(slots.len(), 2);
}

// -----------------------------------------------------------------------
// next_unused_slot
// -----------------------------------------------------------------------

#[test]
fn next_unused_slot_returns_none_when_no_preferred_times() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.next_unused_slot(&[]).is_none());
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
        assert!(schedule.next_unused_slot(&[post_time]).is_none());
    }
}

#[test]
fn next_unused_slot_none_when_all_past() {
    let mut config = default_schedule_config();
    config.preferred_times = vec!["00:01".to_string()];
    let schedule = ActiveSchedule::from_config(&config).unwrap();

    if Utc::now().hour() >= 1 {
        assert!(schedule.next_unused_slot(&[]).is_none());
    }
}

// -----------------------------------------------------------------------
// Thread schedule
// -----------------------------------------------------------------------

#[test]
fn has_thread_preferred_schedule_false_by_default() {
    let config = default_schedule_config();
    assert!(!ActiveSchedule::from_config(&config)
        .unwrap()
        .has_thread_preferred_schedule());
}

#[test]
fn has_thread_preferred_schedule_true_when_set() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("Tue".to_string());
    assert!(ActiveSchedule::from_config(&config)
        .unwrap()
        .has_thread_preferred_schedule());
}

#[test]
fn thread_preferred_invalid_day_gives_none() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("NotADay".to_string());
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(!schedule.has_thread_preferred_schedule());
}

#[test]
fn thread_preferred_invalid_time_uses_default_10_00() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("Wed".to_string());
    config.thread_preferred_time = "bad:time".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.has_thread_preferred_schedule());
    assert_eq!(schedule.thread_preferred_time.hour, 10);
    assert_eq!(schedule.thread_preferred_time.minute, 0);
}

#[test]
fn thread_preferred_time_custom() {
    let mut config = default_schedule_config();
    config.thread_preferred_time = "15:45".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert_eq!(schedule.thread_preferred_time.hour, 15);
    assert_eq!(schedule.thread_preferred_time.minute, 45);
}

#[test]
fn next_thread_slot_none_without_config() {
    let config = default_schedule_config();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.next_thread_slot().is_none());
}

#[test]
fn next_thread_slot_some_when_configured() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("Mon".to_string());
    config.thread_preferred_time = "10:00".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    let result = schedule.next_thread_slot();
    assert!(result.is_some());
    assert!(result.unwrap().as_secs() <= 7 * 86400);
}

#[test]
fn next_thread_slot_all_days_within_bounds() {
    // 10:00 — must be ≤7 days from now
    for day in &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
        let mut config = default_schedule_config();
        config.thread_preferred_day = Some(day.to_string());
        config.thread_preferred_time = "10:00".to_string();
        let dur = ActiveSchedule::from_config(&config)
            .unwrap()
            .next_thread_slot()
            .unwrap();
        assert!(dur.as_secs() <= 7 * 86400, "Day {day}: {}", dur.as_secs());
    }
    // 08:00 — same bound, looser check (some days may already be past 08:00)
    for day in &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
        let mut config = default_schedule_config();
        config.thread_preferred_day = Some(day.to_string());
        config.thread_preferred_time = "08:00".to_string();
        let dur = ActiveSchedule::from_config(&config)
            .unwrap()
            .next_thread_slot()
            .unwrap();
        assert!(dur.as_secs() <= 8 * 86400, "Day {day}: {}", dur.as_secs());
    }
}

#[test]
fn from_config_with_thread_schedule_has_slot() {
    let mut config = default_schedule_config();
    config.thread_preferred_day = Some("Wed".to_string());
    config.thread_preferred_time = "14:30".to_string();
    let schedule = ActiveSchedule::from_config(&config).unwrap();
    assert!(schedule.has_thread_preferred_schedule());
    assert!(schedule.next_thread_slot().is_some());
}

// -----------------------------------------------------------------------
// AUTO_PREFERRED_TIMES in config context
// -----------------------------------------------------------------------

#[test]
fn auto_preferred_times_constant_correct() {
    assert_eq!(AUTO_PREFERRED_TIMES.len(), 3);
    assert_eq!(AUTO_PREFERRED_TIMES[0], "09:15");
    assert_eq!(AUTO_PREFERRED_TIMES[1], "12:30");
    assert_eq!(AUTO_PREFERRED_TIMES[2], "17:00");
}

// -----------------------------------------------------------------------
// schedule_gate
// -----------------------------------------------------------------------

#[tokio::test]
async fn schedule_gate_none_always_returns_true() {
    let cancel = CancellationToken::new();
    assert!(schedule_gate(&None, &cancel).await);
}

#[tokio::test]
async fn schedule_gate_active_schedule_returns_true() {
    let mut config = default_schedule_config();
    config.active_hours_start = 0;
    config.active_hours_end = 23;
    config.active_days = vec![];
    let schedule = Arc::new(ActiveSchedule::from_config(&config).unwrap());
    let cancel = CancellationToken::new();
    let now_hour = Utc::now().hour();
    if now_hour < 23 {
        assert!(schedule_gate(&Some(schedule), &cancel).await);
    }
}

#[tokio::test]
async fn schedule_gate_cancelled_returns_false() {
    let mut config = default_schedule_config();
    let now_hour = Utc::now().hour() as u8;
    config.active_hours_start = (now_hour + 2) % 24;
    config.active_hours_end = (now_hour + 3) % 24;
    let schedule = Arc::new(ActiveSchedule::from_config(&config).unwrap());

    let cancel = CancellationToken::new();
    cancel.cancel();

    assert!(!schedule_gate(&Some(schedule), &cancel).await);
}
