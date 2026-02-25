//! Shared tweet length validation.

use std::time::Instant;

use super::super::response::{ToolMeta, ToolResponse};

/// URL-aware tweet length limit.
const MAX_TWEET_LENGTH: usize = 280;

/// X API counts each URL as 23 characters regardless of actual length.
const URL_WEIGHTED_LENGTH: usize = 23;

/// Check if tweet text exceeds the 280-char limit (URL-weighted).
///
/// Returns `Some(error_json)` if the text is too long, `None` if OK.
pub(crate) fn check_tweet_length(text: &str, start: Instant) -> Option<String> {
    let weighted_len = compute_weighted_length(text);
    if weighted_len > MAX_TWEET_LENGTH {
        let elapsed = start.elapsed().as_millis() as u64;
        Some(
            ToolResponse::error(
                "tweet_too_long",
                format!(
                    "Tweet text is {weighted_len} characters (URL-weighted), max is {MAX_TWEET_LENGTH}."
                ),
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json(),
        )
    } else {
        None
    }
}

/// Compute URL-weighted character length.
///
/// Any `http://` or `https://` URL is counted as 23 characters per X API rules.
fn compute_weighted_length(text: &str) -> usize {
    let mut total = 0;
    let mut remaining = text;

    while let Some(url_start) = remaining
        .find("https://")
        .or_else(|| remaining.find("http://"))
    {
        // Count characters before the URL.
        total += remaining[..url_start].chars().count();

        // Find the end of the URL (next whitespace or end of string).
        let url_rest = &remaining[url_start..];
        let url_end = url_rest
            .find(|c: char| c.is_whitespace())
            .unwrap_or(url_rest.len());

        // Each URL counts as URL_WEIGHTED_LENGTH.
        total += URL_WEIGHTED_LENGTH;

        remaining = &remaining[url_start + url_end..];
    }

    // Count remaining non-URL characters.
    total += remaining.chars().count();
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_ok() {
        let start = Instant::now();
        assert!(check_tweet_length("Hello world", start).is_none());
    }

    #[test]
    fn exactly_280_ok() {
        let start = Instant::now();
        let text = "a".repeat(280);
        assert!(check_tweet_length(&text, start).is_none());
    }

    #[test]
    fn over_280_rejected() {
        let start = Instant::now();
        let text = "a".repeat(281);
        assert!(check_tweet_length(&text, start).is_some());
    }

    #[test]
    fn url_counted_as_23() {
        // 257 chars + URL (23 weighted) = 280, should be OK.
        let text = format!(
            "{} https://example.com/very/long/path/that/exceeds/23",
            "a".repeat(256)
        );
        let start = Instant::now();
        assert!(check_tweet_length(&text, start).is_none());
    }

    #[test]
    fn url_weighted_over_280() {
        // 258 chars + URL (23 weighted) = 281, should fail.
        let text = format!("{} https://example.com", "a".repeat(257));
        let start = Instant::now();
        assert!(check_tweet_length(&text, start).is_some());
    }
}
