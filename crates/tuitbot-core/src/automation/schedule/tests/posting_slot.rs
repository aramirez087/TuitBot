//! Tests for `PostingSlot` — parsing, formatting, ordering, and conversion.

use chrono::Timelike;

use crate::automation::schedule::PostingSlot;

// --- Parse ---

#[test]
fn parse_valid() {
    let slot = PostingSlot::parse("09:15").unwrap();
    assert_eq!(slot.hour, 9);
    assert_eq!(slot.minute, 15);
    assert_eq!(slot.as_minutes(), 9 * 60 + 15);
    assert_eq!(slot.format(), "09:15");
}

#[test]
fn parse_midnight() {
    let slot = PostingSlot::parse("00:00").unwrap();
    assert_eq!(slot.hour, 0);
    assert_eq!(slot.minute, 0);
    assert_eq!(slot.as_minutes(), 0);
}

#[test]
fn parse_end_of_day() {
    let slot = PostingSlot::parse("23:59").unwrap();
    assert_eq!(slot.hour, 23);
    assert_eq!(slot.minute, 59);
    assert_eq!(slot.as_minutes(), 23 * 60 + 59);
}

#[test]
fn parse_boundary_values() {
    // Both boundary values fully checked
    let midnight = PostingSlot::parse("00:00").unwrap();
    assert_eq!(midnight.as_minutes(), 0);
    let eod = PostingSlot::parse("23:59").unwrap();
    assert_eq!(eod.as_minutes(), 23 * 60 + 59);
}

#[test]
fn parse_single_digit_parts() {
    let slot = PostingSlot::parse("9:5").unwrap();
    assert_eq!(slot.hour, 9);
    assert_eq!(slot.minute, 5);
}

#[test]
fn parse_invalid_hour_25() {
    assert!(PostingSlot::parse("25:00").is_none());
}

#[test]
fn parse_invalid_hour_24() {
    assert!(PostingSlot::parse("24:00").is_none());
}

#[test]
fn parse_invalid_minute_60() {
    assert!(PostingSlot::parse("12:60").is_none());
}

#[test]
fn parse_invalid_format_no_colon() {
    assert!(PostingSlot::parse("12").is_none());
    assert!(PostingSlot::parse("").is_none());
    assert!(PostingSlot::parse("12:30:00").is_none());
    assert!(PostingSlot::parse("ab:cd").is_none());
}

#[test]
fn parse_invalid_separator() {
    assert!(PostingSlot::parse("12-30").is_none());
    assert!(PostingSlot::parse("12 30").is_none());
    assert!(PostingSlot::parse("12.30").is_none());
}

#[test]
fn parse_extra_whitespace() {
    assert!(PostingSlot::parse(" 12:30").is_none());
    assert!(PostingSlot::parse("12: 30").is_none());
}

#[test]
fn parse_negative_values_rejected() {
    assert!(PostingSlot::parse("-1:00").is_none());
    assert!(PostingSlot::parse("12:-5").is_none());
}

#[test]
fn parse_large_numbers() {
    assert!(PostingSlot::parse("99:99").is_none());
    assert!(PostingSlot::parse("255:255").is_none());
}

// --- Format ---

#[test]
fn format_zero_padded() {
    let slot = PostingSlot::parse("01:05").unwrap();
    assert_eq!(slot.format(), "01:05");
}

#[test]
fn format_preserves_leading_zeros() {
    assert_eq!(PostingSlot::parse("00:05").unwrap().format(), "00:05");
    assert_eq!(PostingSlot::parse("03:09").unwrap().format(), "03:09");
}

// --- as_minutes ---

#[test]
fn as_minutes_noon() {
    assert_eq!(PostingSlot::parse("12:00").unwrap().as_minutes(), 720);
}

#[test]
fn as_minutes_one_am() {
    assert_eq!(PostingSlot::parse("01:00").unwrap().as_minutes(), 60);
}

// --- to_naive_time ---

#[test]
fn to_naive_time_standard() {
    let slot = PostingSlot::parse("14:30").unwrap();
    let t = slot.to_naive_time();
    assert_eq!(t.hour(), 14);
    assert_eq!(t.minute(), 30);
}

#[test]
fn to_naive_time_midnight() {
    let slot = PostingSlot::parse("00:00").unwrap();
    let t = slot.to_naive_time();
    assert_eq!(t.hour(), 0);
    assert_eq!(t.minute(), 0);
    assert_eq!(t.second(), 0);
}

#[test]
fn to_naive_time_boundary() {
    let slot = PostingSlot::parse("23:59").unwrap();
    let t = slot.to_naive_time();
    assert_eq!(t.hour(), 23);
    assert_eq!(t.minute(), 59);
}

// --- Ordering ---

#[test]
fn ordering_ascending() {
    let a = PostingSlot::parse("09:00").unwrap();
    let b = PostingSlot::parse("12:30").unwrap();
    let c = PostingSlot::parse("17:00").unwrap();
    assert!(a < b);
    assert!(b < c);
}

#[test]
fn ordering_same_hour() {
    let a = PostingSlot::parse("09:00").unwrap();
    let b = PostingSlot::parse("09:30").unwrap();
    assert!(a < b);
}

#[test]
fn ordering_reverse() {
    let a = PostingSlot::parse("23:59").unwrap();
    let b = PostingSlot::parse("00:00").unwrap();
    assert!(a > b);
}

#[test]
fn ordering_equal() {
    let a = PostingSlot::parse("15:30").unwrap();
    let b = PostingSlot::parse("15:30").unwrap();
    assert!(!(a < b));
    assert!(!(a > b));
    assert_eq!(a, b);
}

// --- Clone / Debug / Eq ---

#[test]
fn clone_and_debug() {
    let slot = PostingSlot::parse("12:00").unwrap();
    let cloned = slot.clone();
    assert_eq!(slot, cloned);
    let debug = format!("{:?}", slot);
    assert!(debug.contains("PostingSlot"));
}

#[test]
fn equality() {
    let a = PostingSlot::parse("09:15").unwrap();
    let b = PostingSlot::parse("09:15").unwrap();
    assert_eq!(a, b);
}

#[test]
fn debug_contains_hour_and_minute() {
    let slot = PostingSlot::parse("14:30").unwrap();
    let debug = format!("{:?}", slot);
    assert!(debug.contains("14"));
    assert!(debug.contains("30"));
}
