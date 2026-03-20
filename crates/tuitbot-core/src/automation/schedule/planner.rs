//! Schedule planning: `ActiveSchedule` construction, slot resolution,
//! active-window checks, and "time until active" computation.

use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::ScheduleConfig;

use super::recurrence::PostingSlot;

/// Research-backed default posting times (Sprout Social's 2.7B engagement analysis).
pub const AUTO_PREFERRED_TIMES: &[&str] = &["09:15", "12:30", "17:00"];

/// Parse a day abbreviation to a `chrono::Weekday`.
pub(super) fn parse_weekday(s: &str) -> Option<chrono::Weekday> {
    match s.trim() {
        "Mon" => Some(chrono::Weekday::Mon),
        "Tue" => Some(chrono::Weekday::Tue),
        "Wed" => Some(chrono::Weekday::Wed),
        "Thu" => Some(chrono::Weekday::Thu),
        "Fri" => Some(chrono::Weekday::Fri),
        "Sat" => Some(chrono::Weekday::Sat),
        "Sun" => Some(chrono::Weekday::Sun),
        _ => None,
    }
}

/// Parsed active schedule with timezone, hours, weekday filtering, and preferred posting times.
#[derive(Debug, Clone)]
pub struct ActiveSchedule {
    pub(super) tz: Tz,
    pub(super) start_hour: u8,
    pub(super) end_hour: u8,
    pub(super) active_weekdays: Vec<chrono::Weekday>,
    /// Base preferred posting times (sorted). Empty = interval mode.
    pub(super) preferred_times: Vec<PostingSlot>,
    /// Per-day overrides for preferred times.
    pub(super) preferred_times_override: HashMap<chrono::Weekday, Vec<PostingSlot>>,
    /// Preferred weekday for thread posting. None = interval mode.
    pub(super) thread_preferred_day: Option<chrono::Weekday>,
    /// Preferred time for thread posting.
    pub(super) thread_preferred_time: PostingSlot,
}

impl ActiveSchedule {
    /// Create an `ActiveSchedule` from config. Returns `None` if the
    /// timezone string fails to parse.
    pub fn from_config(config: &ScheduleConfig) -> Option<Self> {
        let tz: Tz = config.timezone.parse().ok()?;

        let active_weekdays: Vec<chrono::Weekday> = config
            .active_days
            .iter()
            .filter_map(|d| parse_weekday(d))
            .collect();

        // Parse preferred times, expanding "auto"
        let mut preferred_times: Vec<PostingSlot> = Vec::new();
        for time_str in &config.preferred_times {
            if time_str == "auto" {
                for auto_time in AUTO_PREFERRED_TIMES {
                    if let Some(slot) = PostingSlot::parse(auto_time) {
                        preferred_times.push(slot);
                    }
                }
            } else if let Some(slot) = PostingSlot::parse(time_str) {
                preferred_times.push(slot);
            }
        }
        preferred_times.sort();
        preferred_times.dedup();

        // Parse per-day overrides
        let mut preferred_times_override: HashMap<chrono::Weekday, Vec<PostingSlot>> =
            HashMap::new();
        for (day_str, times) in &config.preferred_times_override {
            if let Some(weekday) = parse_weekday(day_str) {
                let mut slots: Vec<PostingSlot> =
                    times.iter().filter_map(|t| PostingSlot::parse(t)).collect();
                slots.sort();
                slots.dedup();
                preferred_times_override.insert(weekday, slots);
            }
        }

        let thread_preferred_day = config
            .thread_preferred_day
            .as_deref()
            .and_then(parse_weekday);

        let thread_preferred_time =
            PostingSlot::parse(&config.thread_preferred_time).unwrap_or(PostingSlot {
                hour: 10,
                minute: 0,
            });

        Some(Self {
            tz,
            start_hour: config.active_hours_start,
            end_hour: config.active_hours_end,
            active_weekdays,
            preferred_times,
            preferred_times_override,
            thread_preferred_day,
            thread_preferred_time,
        })
    }

    /// Whether preferred posting times are configured (slot mode).
    pub fn has_preferred_times(&self) -> bool {
        !self.preferred_times.is_empty()
    }

    /// Whether a preferred thread schedule is configured.
    pub fn has_thread_preferred_schedule(&self) -> bool {
        self.thread_preferred_day.is_some()
    }

    /// Get the posting slots for today, resolving per-day overrides.
    ///
    /// If today's weekday has an entry in `preferred_times_override`, use that.
    /// Otherwise use the base `preferred_times`.
    pub fn slots_for_today(&self) -> Vec<PostingSlot> {
        let now = Utc::now().with_timezone(&self.tz);
        let weekday = now.weekday();

        if let Some(override_slots) = self.preferred_times_override.get(&weekday) {
            override_slots.clone()
        } else {
            self.preferred_times.clone()
        }
    }

    /// Find the next unused slot for today.
    ///
    /// Compares today's slots against `today_post_times` (actual post times from DB).
    /// A slot is considered "used" if any post occurred within +/- 30 minutes of the slot time.
    ///
    /// Returns `Some((duration_until_slot, slot))` for the next available slot,
    /// or `None` if all slots have been used today.
    pub fn next_unused_slot(
        &self,
        today_post_times: &[DateTime<Utc>],
    ) -> Option<(Duration, PostingSlot)> {
        let now = Utc::now().with_timezone(&self.tz);
        let slots = self.slots_for_today();

        for slot in &slots {
            let slot_time = slot.to_naive_time();

            // Check if this slot has already been used (within +/- 30 min match window)
            let slot_used = today_post_times.iter().any(|post_time| {
                let post_local = post_time.with_timezone(&self.tz);
                let post_naive = post_local.time();
                let diff = (post_naive.num_seconds_from_midnight() as i64)
                    - (slot_time.num_seconds_from_midnight() as i64);
                diff.unsigned_abs() <= 30 * 60
            });

            if slot_used {
                continue;
            }

            // Check if this slot is in the future
            let now_time = now.time();
            if slot_time > now_time {
                let diff_secs = (slot_time.num_seconds_from_midnight() as i64)
                    - (now_time.num_seconds_from_midnight() as i64);
                return Some((Duration::from_secs(diff_secs as u64), slot.clone()));
            }
        }

        None
    }

    /// Compute the duration until the next preferred thread day+time.
    ///
    /// Returns `None` if no preferred thread schedule is configured.
    pub fn next_thread_slot(&self) -> Option<Duration> {
        let target_day = self.thread_preferred_day?;
        let target_time = self.thread_preferred_time.to_naive_time();

        let now = Utc::now().with_timezone(&self.tz);
        let now_weekday = now.weekday();
        let now_time = now.time();

        // Check if target is today and still in the future
        if now_weekday == target_day && now_time < target_time {
            let diff_secs = (target_time.num_seconds_from_midnight() as i64)
                - (now_time.num_seconds_from_midnight() as i64);
            return Some(Duration::from_secs(diff_secs as u64));
        }

        // Find days until next occurrence of target_day
        let now_num = now_weekday.num_days_from_monday();
        let target_num = target_day.num_days_from_monday();
        let days_ahead = if target_num > now_num {
            target_num - now_num
        } else {
            7 - (now_num - target_num)
        };

        // Compute seconds: remaining today + full days + target time
        let secs_remaining_today = (86400 - now_time.num_seconds_from_midnight()) as u64;
        let full_days_between = (days_ahead as u64 - 1) * 86400;
        let secs_into_target_day = target_time.num_seconds_from_midnight() as u64;

        Some(Duration::from_secs(
            secs_remaining_today + full_days_between + secs_into_target_day,
        ))
    }

    /// Check if the current time falls within the active posting window.
    ///
    /// Handles wrapping ranges (e.g. start=22, end=6 for night owls).
    pub fn is_active(&self) -> bool {
        let now = Utc::now().with_timezone(&self.tz);
        let hour = now.hour() as u8;
        let weekday = now.weekday();

        // Check weekday
        if !self.active_weekdays.is_empty() && !self.active_weekdays.contains(&weekday) {
            return false;
        }

        // Check hours — handle wrapping (e.g. 22-06)
        if self.start_hour <= self.end_hour {
            // Normal range: 8-22 means hours 8..22
            hour >= self.start_hour && hour < self.end_hour
        } else {
            // Wrapping range: 22-06 means hours 22..24 or 0..6
            hour >= self.start_hour || hour < self.end_hour
        }
    }

    /// Compute the duration until the next active window starts.
    ///
    /// Returns `Duration::ZERO` if currently active.
    pub fn time_until_active(&self) -> Duration {
        if self.is_active() {
            return Duration::ZERO;
        }

        let now = Utc::now().with_timezone(&self.tz);
        let hour = now.hour() as u8;
        let weekday = now.weekday();

        // First, find how many hours until start_hour today or tomorrow
        let hours_until_start = if hour < self.start_hour {
            (self.start_hour - hour) as u64
        } else {
            // start_hour is tomorrow (or later today if wrapping)
            (24 - hour + self.start_hour) as u64
        };

        // Check if today is an active day
        let is_today_active =
            self.active_weekdays.is_empty() || self.active_weekdays.contains(&weekday);

        // If today is active and start hour hasn't passed yet (non-wrapping case)
        if is_today_active && hour < self.start_hour {
            let wait_secs =
                hours_until_start * 3600 - (now.minute() as u64 * 60) - now.second() as u64;
            return Duration::from_secs(wait_secs.max(1));
        }

        // Look ahead up to 8 days for the next active day
        for day_offset in 1u64..=8 {
            let future_day = now + chrono::Duration::days(day_offset as i64);
            let future_weekday = future_day.weekday();

            if self.active_weekdays.is_empty() || self.active_weekdays.contains(&future_weekday) {
                // Next active day found — compute duration to start_hour on that day
                let secs_remaining_today =
                    (24 - hour as u64) * 3600 - (now.minute() as u64 * 60) - now.second() as u64;
                let full_days_between = (day_offset - 1) * 86400;
                let secs_into_target_day = self.start_hour as u64 * 3600;

                let total = secs_remaining_today + full_days_between + secs_into_target_day;
                return Duration::from_secs(total.max(1));
            }
        }

        // Fallback: sleep 1 hour and re-check
        Duration::from_secs(3600)
    }
}
