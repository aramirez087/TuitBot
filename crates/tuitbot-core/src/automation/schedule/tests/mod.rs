mod active_schedule;
mod jitter;
mod posting_slot;
mod slots_and_gate;

use crate::config::ScheduleConfig;

pub(self) fn default_schedule_config() -> ScheduleConfig {
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
