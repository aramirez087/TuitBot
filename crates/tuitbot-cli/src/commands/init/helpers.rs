//! Pure helper functions for string manipulation and parsing.

/// Convert a trimmed string to `Some` or `None` if empty.
pub(super) fn non_empty(s: String) -> Option<String> {
    let trimmed = s.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Split a comma-separated string into trimmed, non-empty values.
pub(super) fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Escape special TOML characters inside a double-quoted string value.
pub(super) fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Capitalize the first letter of a string.
pub(super) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Format a `Vec<String>` as a TOML inline array: `["a", "b", "c"]`.
pub(super) fn format_toml_array(items: &[String]) -> String {
    let inner: Vec<String> = items
        .iter()
        .map(|s| format!("\"{}\"", escape_toml(s)))
        .collect();
    format!("[{}]", inner.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── non_empty ────────────────────────────────────────────────────

    #[test]
    fn non_empty_returns_none_for_empty() {
        assert_eq!(non_empty(String::new()), None);
    }

    #[test]
    fn non_empty_returns_none_for_whitespace() {
        assert_eq!(non_empty("   ".to_string()), None);
    }

    #[test]
    fn non_empty_returns_some_trimmed() {
        assert_eq!(
            non_empty("  hello  ".to_string()),
            Some("hello".to_string())
        );
    }

    #[test]
    fn non_empty_preserves_content() {
        assert_eq!(
            non_empty("some value".to_string()),
            Some("some value".to_string())
        );
    }

    // ── parse_csv ────────────────────────────────────────────────────

    #[test]
    fn parse_csv_empty_string() {
        assert!(parse_csv("").is_empty());
    }

    #[test]
    fn parse_csv_single_item() {
        assert_eq!(parse_csv("rust"), vec!["rust"]);
    }

    #[test]
    fn parse_csv_multiple_items_trimmed() {
        assert_eq!(parse_csv(" a , b , c "), vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_csv_filters_empty_segments() {
        assert_eq!(parse_csv("a,,b, ,c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_csv_only_commas() {
        assert!(parse_csv(",,,").is_empty());
    }

    // ── escape_toml ──────────────────────────────────────────────────

    #[test]
    fn escape_toml_no_special_chars() {
        assert_eq!(escape_toml("hello"), "hello");
    }

    #[test]
    fn escape_toml_backslash() {
        assert_eq!(escape_toml(r"a\b"), r"a\\b");
    }

    #[test]
    fn escape_toml_double_quote() {
        assert_eq!(escape_toml(r#"say "hi""#), r#"say \"hi\""#);
    }

    #[test]
    fn escape_toml_newline_and_tab() {
        assert_eq!(escape_toml("a\nb\tc"), r"a\nb\tc");
    }

    #[test]
    fn escape_toml_carriage_return() {
        assert_eq!(escape_toml("a\rb"), r"a\rb");
    }

    #[test]
    fn escape_toml_empty() {
        assert_eq!(escape_toml(""), "");
    }

    // ── capitalize ───────────────────────────────────────────────────

    #[test]
    fn capitalize_empty() {
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn capitalize_lowercase() {
        assert_eq!(capitalize("openai"), "Openai");
    }

    #[test]
    fn capitalize_already_uppercase() {
        assert_eq!(capitalize("Ollama"), "Ollama");
    }

    #[test]
    fn capitalize_single_char() {
        assert_eq!(capitalize("a"), "A");
    }

    #[test]
    fn capitalize_all_lowercase() {
        assert_eq!(capitalize("anthropic"), "Anthropic");
    }

    // ── format_toml_array ────────────────────────────────────────────

    #[test]
    fn format_toml_array_empty() {
        let items: Vec<String> = vec![];
        assert_eq!(format_toml_array(&items), "[]");
    }

    #[test]
    fn format_toml_array_single_item() {
        let items = vec!["rust".to_string()];
        assert_eq!(format_toml_array(&items), r#"["rust"]"#);
    }

    #[test]
    fn format_toml_array_multiple_items() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(format_toml_array(&items), r#"["a", "b", "c"]"#);
    }

    #[test]
    fn format_toml_array_with_quotes_in_content() {
        let items = vec![r#"say "hi""#.to_string()];
        assert_eq!(format_toml_array(&items), r#"["say \"hi\""]"#);
    }
}
