//! Scoring helper utilities: keyword matching, formatting, and text utilities.

use chrono::{DateTime, Utc};

/// Find which keywords matched a tweet (case-insensitive).
///
/// Returns the subset of keywords present in the tweet text.
/// Used for display purposes — the actual scoring uses weighted counts.
pub fn find_matched_keywords(tweet_text: &str, keywords: &[String]) -> Vec<String> {
    let text_lower = tweet_text.to_lowercase();
    keywords
        .iter()
        .filter(|kw| text_lower.contains(&kw.to_lowercase()))
        .cloned()
        .collect()
}

/// Format a follower count for display.
///
/// Examples: 500 → "500", 1200 → "1.2K", 45300 → "45.3K", 1200000 → "1.2M".
pub fn format_follower_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

/// Format a tweet's age for display.
///
/// Parses the ISO-8601 timestamp and returns a human-readable duration
/// like "12 minutes", "2 hours", "1 day". Returns "unknown" on parse failure.
pub fn format_tweet_age(created_at: &str) -> String {
    format_tweet_age_at(created_at, Utc::now())
}

/// Format a tweet's age relative to a specific time (for testability).
pub fn format_tweet_age_at(created_at: &str, now: DateTime<Utc>) -> String {
    let created = match created_at.parse::<DateTime<Utc>>() {
        Ok(dt) => dt,
        Err(_) => return "unknown".to_string(),
    };

    let duration = now - created;
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();

    if minutes < 1 {
        let secs = duration.num_seconds().max(0);
        format!("{secs} seconds")
    } else if minutes < 60 {
        format!("{minutes} minutes")
    } else if hours < 24 {
        format!("{hours} hours")
    } else {
        format!("{days} days")
    }
}

/// Truncate text for display, appending "..." if truncated.
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}
