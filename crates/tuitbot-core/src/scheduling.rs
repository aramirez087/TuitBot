//! Shared scheduling validation and normalization.
//!
//! # Timestamp contract
//!
//! All `scheduled_for` values are stored as UTC ISO-8601 with trailing `Z`.
//! The posting engine compares `scheduled_for <= datetime('now')` — both sides
//! are UTC, so comparisons are correct without conversion.
//!
//! ## Accepted input formats
//!
//! - `"2026-03-10T14:00:00Z"` — preferred UTC (returned as-is)
//! - `"2026-03-10T14:00:00"` — bare string, treated as UTC for backward compat
//! - `"2026-03-10T14:00:00+05:00"` — offset, converted to UTC
//!
//! ## Account timezone rules
//!
//! The account timezone (`ScheduleConfig.timezone`) is the canonical user-facing
//! timezone. The frontend converts user-selected date/time from account timezone
//! to UTC before sending to the server. The server never interprets timezone —
//! it only validates format, rejects past timestamps, and normalizes to UTC.

use chrono::{DateTime, NaiveDateTime, Utc};

/// Default grace period in seconds for past-schedule rejection.
/// Allows slight clock skew between client and server.
pub const DEFAULT_GRACE_SECONDS: i64 = 300; // 5 minutes

/// Errors from scheduling validation.
#[derive(Debug, thiserror::Error)]
pub enum SchedulingError {
    /// The timestamp string could not be parsed.
    #[error("invalid timestamp format: {0}")]
    InvalidFormat(String),
    /// The scheduled time is in the past (beyond the grace period).
    #[error("scheduled time is in the past")]
    InThePast,
}

/// Validate and normalize a `scheduled_for` timestamp to UTC with `Z` suffix.
///
/// Accepts UTC (`Z`), bare strings (treated as UTC), and offset strings
/// (converted to UTC). Returns a normalized `YYYY-MM-DDTHH:MM:SSZ` string.
pub fn normalize_scheduled_for(raw: &str) -> Result<String, SchedulingError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SchedulingError::InvalidFormat("empty string".to_string()));
    }

    // Try RFC 3339 / ISO 8601 with timezone info (handles both Z and offsets)
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        let utc: DateTime<Utc> = dt.into();
        return Ok(utc.format("%Y-%m-%dT%H:%M:%SZ").to_string());
    }

    // Try bare datetime (no timezone) — treat as UTC
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        let utc = naive.and_utc();
        return Ok(utc.format("%Y-%m-%dT%H:%M:%SZ").to_string());
    }

    // Try with fractional seconds
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%.f") {
        let utc = naive.and_utc();
        return Ok(utc.format("%Y-%m-%dT%H:%M:%SZ").to_string());
    }

    Err(SchedulingError::InvalidFormat(trimmed.to_string()))
}

/// Reject timestamps that are more than `grace_seconds` in the past.
pub fn validate_not_past(utc_iso: &str, grace_seconds: i64) -> Result<(), SchedulingError> {
    let dt = NaiveDateTime::parse_from_str(utc_iso.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| SchedulingError::InvalidFormat(utc_iso.to_string()))?;

    let utc = dt.and_utc();
    let now = Utc::now();
    let diff = utc.signed_duration_since(now);

    if diff.num_seconds() < -grace_seconds {
        return Err(SchedulingError::InThePast);
    }

    Ok(())
}

/// Combined: normalize + validate not in the past.
pub fn validate_and_normalize(raw: &str, grace_seconds: i64) -> Result<String, SchedulingError> {
    let normalized = normalize_scheduled_for(raw)?;
    validate_not_past(&normalized, grace_seconds)?;
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_utc_with_z_normalized_unchanged() {
        let result = normalize_scheduled_for("2099-12-31T23:59:00Z").unwrap();
        assert_eq!(result, "2099-12-31T23:59:00Z");
    }

    #[test]
    fn bare_string_appends_z() {
        let result = normalize_scheduled_for("2099-12-31T23:59:00").unwrap();
        assert_eq!(result, "2099-12-31T23:59:00Z");
    }

    #[test]
    fn offset_string_converts_to_utc() {
        // +05:30 means the local time is 5:30 ahead of UTC
        // 2099-12-31T23:59:00+05:30 → 2099-12-31T18:29:00Z
        let result = normalize_scheduled_for("2099-12-31T23:59:00+05:30").unwrap();
        assert_eq!(result, "2099-12-31T18:29:00Z");
    }

    #[test]
    fn negative_offset_converts_to_utc() {
        // -05:00 means 5 hours behind UTC
        // 2099-12-31T19:00:00-05:00 → 2100-01-01T00:00:00Z
        let result = normalize_scheduled_for("2099-12-31T19:00:00-05:00").unwrap();
        assert_eq!(result, "2100-01-01T00:00:00Z");
    }

    #[test]
    fn fractional_seconds_stripped() {
        let result = normalize_scheduled_for("2099-12-31T23:59:00.123").unwrap();
        assert_eq!(result, "2099-12-31T23:59:00Z");
    }

    #[test]
    fn past_timestamp_rejected() {
        let result = validate_and_normalize("2020-01-01T00:00:00Z", 300);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SchedulingError::InThePast));
    }

    #[test]
    fn future_timestamp_accepted() {
        let result = validate_and_normalize("2099-12-31T23:59:00Z", 300);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2099-12-31T23:59:00Z");
    }

    #[test]
    fn near_past_within_grace_accepted() {
        // Create a timestamp 2 minutes in the past (within 300s grace)
        let near_past = Utc::now() - chrono::Duration::seconds(120);
        let ts = near_past.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let result = validate_and_normalize(&ts, 300);
        assert!(result.is_ok());
    }

    #[test]
    fn near_past_beyond_grace_rejected() {
        // Create a timestamp 10 minutes in the past (beyond 300s grace)
        let far_past = Utc::now() - chrono::Duration::seconds(600);
        let ts = far_past.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let result = validate_and_normalize(&ts, 300);
        assert!(result.is_err());
    }

    #[test]
    fn garbage_string_rejected() {
        let result = normalize_scheduled_for("not-a-date");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchedulingError::InvalidFormat(_)
        ));
    }

    #[test]
    fn empty_string_rejected() {
        let result = normalize_scheduled_for("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchedulingError::InvalidFormat(_)
        ));
    }

    #[test]
    fn whitespace_only_rejected() {
        let result = normalize_scheduled_for("   ");
        assert!(result.is_err());
    }

    #[test]
    fn bare_offset_backward_compat() {
        // Bare string with offset should convert to UTC
        let result = normalize_scheduled_for("2099-06-15T10:00:00+00:00").unwrap();
        assert_eq!(result, "2099-06-15T10:00:00Z");
    }

    #[test]
    fn validate_and_normalize_with_offset() {
        let result = validate_and_normalize("2099-12-31T23:59:00+05:30", 300).unwrap();
        assert_eq!(result, "2099-12-31T18:29:00Z");
    }
}
