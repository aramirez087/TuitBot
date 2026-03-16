use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{Confirm, Input, MultiSelect, Select};

// ---------------------------------------------------------------------------
// Change tracking
// ---------------------------------------------------------------------------

pub(super) struct Change {
    pub section: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

pub(super) struct ChangeTracker {
    pub changes: Vec<Change>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub fn record(&mut self, section: &str, field: &str, old_value: &str, new_value: &str) {
        if old_value != new_value {
            self.changes.push(Change {
                section: section.to_string(),
                field: field.to_string(),
                old_value: old_value.to_string(),
                new_value: new_value.to_string(),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Field editing helpers (interactive)
// ---------------------------------------------------------------------------

pub(super) fn edit_string(label: &str, current: &str) -> Result<String> {
    let val: String = Input::new()
        .with_prompt(label)
        .default(current.to_string())
        .interact_text()?;
    Ok(val.trim().to_string())
}

pub(super) fn edit_optional_string(
    label: &str,
    current: &Option<String>,
) -> Result<Option<String>> {
    let default = current.as_deref().unwrap_or("").to_string();
    let prompt = if current.is_some() {
        format!("{label} (type \"none\" to clear)")
    } else {
        format!("{label} (Enter to skip)")
    };
    let val: String = Input::new()
        .with_prompt(prompt)
        .default(default)
        .allow_empty(true)
        .interact_text()?;
    let trimmed = val.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

pub(super) fn edit_bool(label: &str, current: bool) -> Result<bool> {
    let val = Confirm::new()
        .with_prompt(label)
        .default(current)
        .interact()?;
    Ok(val)
}

pub(super) fn edit_u32(label: &str, current: u32, help: Option<&str>) -> Result<u32> {
    if let Some(h) = help {
        let dim = Style::new().dim();
        eprintln!("  {}", dim.apply_to(h));
    }
    let val: String = Input::new()
        .with_prompt(label)
        .default(current.to_string())
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<u32>()
                .map(|_| ())
                .map_err(|_| "Must be a positive number".to_string())
        })
        .interact_text()?;
    Ok(val.trim().parse().unwrap())
}

pub(super) fn edit_u64(label: &str, current: u64, help: Option<&str>) -> Result<u64> {
    if let Some(h) = help {
        let dim = Style::new().dim();
        eprintln!("  {}", dim.apply_to(h));
    }
    let val: String = Input::new()
        .with_prompt(label)
        .default(current.to_string())
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<u64>()
                .map(|_| ())
                .map_err(|_| "Must be a positive number".to_string())
        })
        .interact_text()?;
    Ok(val.trim().parse().unwrap())
}

pub(super) fn edit_u8(label: &str, current: u8) -> Result<u8> {
    let val: String = Input::new()
        .with_prompt(label)
        .default(current.to_string())
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|&v| v <= 23)
                .map(|_| ())
                .ok_or_else(|| "Must be 0-23".to_string())
        })
        .interact_text()?;
    Ok(val.trim().parse().unwrap())
}

pub(super) fn edit_f32(label: &str, current: f32, help: Option<&str>) -> Result<f32> {
    if let Some(h) = help {
        let dim = Style::new().dim();
        eprintln!("  {}", dim.apply_to(h));
    }
    let val: String = Input::new()
        .with_prompt(label)
        .default(format!("{current:.2}"))
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<f32>()
                .map(|_| ())
                .map_err(|_| "Must be a number".to_string())
        })
        .interact_text()?;
    Ok(val.trim().parse().unwrap())
}

pub(super) fn edit_list(label: &str, current: &[String]) -> Result<Vec<String>> {
    let actions = if current.is_empty() {
        vec!["Add items", "Replace all"]
    } else {
        vec!["Add items", "Remove items", "Replace all"]
    };

    let selection = Select::new()
        .with_prompt(format!("{label} — what do you want to do?"))
        .items(&actions)
        .default(0)
        .interact()?;

    let action = actions[selection];

    match action {
        "Add items" => {
            let raw: String = Input::new()
                .with_prompt("Items to add (comma-separated)")
                .interact_text()?;
            let new_items = parse_csv(&raw);
            let mut result = current.to_vec();
            result.extend(new_items);
            Ok(result)
        }
        "Remove items" => {
            if current.is_empty() {
                eprintln!("Nothing to remove.");
                return Ok(current.to_vec());
            }
            let items: Vec<&str> = current.iter().map(|s| s.as_str()).collect();
            let selections = MultiSelect::new()
                .with_prompt("Select items to remove (Space to toggle, Enter to confirm)")
                .items(&items)
                .interact()?;
            let result: Vec<String> = current
                .iter()
                .enumerate()
                .filter(|(i, _)| !selections.contains(i))
                .map(|(_, s)| s.clone())
                .collect();
            Ok(result)
        }
        "Replace all" => {
            let raw: String = Input::new()
                .with_prompt("New items (comma-separated)")
                .allow_empty(true)
                .interact_text()?;
            Ok(parse_csv(&raw))
        }
        _ => Ok(current.to_vec()),
    }
}

pub(super) fn edit_duration_minutes(label: &str, current_seconds: u64) -> Result<u64> {
    let dim = Style::new().dim();
    eprintln!(
        "  {}",
        dim.apply_to(format!(
            "Currently: {}",
            super::show::format_duration(current_seconds)
        ))
    );
    eprintln!(
        "  {}",
        dim.apply_to("Enter value in minutes (e.g., 15) or hours (e.g., 3h)")
    );

    let default_display = if current_seconds >= 3600 && current_seconds % 3600 == 0 {
        format!("{}h", current_seconds / 3600)
    } else {
        format!("{}", current_seconds / 60)
    };

    let val: String = Input::new()
        .with_prompt(format!("{label} (minutes, or Nh for hours)"))
        .default(default_display)
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            parse_duration_input(input.trim())
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .interact_text()?;

    parse_duration_input(val.trim())
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

pub(super) fn parse_duration_input(input: &str) -> Result<u64> {
    let input = input.trim().to_lowercase();
    if let Some(hours) = input.strip_suffix('h') {
        let h: u64 = hours.trim().parse().context("Invalid number of hours")?;
        Ok(h * 3600)
    } else if let Some(days) = input.strip_suffix('d') {
        let d: u64 = days.trim().parse().context("Invalid number of days")?;
        Ok(d * 86400)
    } else {
        let mins: u64 = input
            .parse()
            .context("Enter a number (minutes), or Nh for hours, Nd for days")?;
        Ok(mins * 60)
    }
}

pub(super) fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        _ => bail!("Invalid boolean value: {value} (use true/false, yes/no, 1/0)"),
    }
}

pub(super) fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

pub(super) fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

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

    // ── escape_toml ──────────────────────────────────────────────────

    #[test]
    fn escape_toml_plain_string() {
        assert_eq!(escape_toml("hello world"), "hello world");
    }

    #[test]
    fn escape_toml_backslash() {
        assert_eq!(escape_toml(r"C:\Users\test"), r"C:\\Users\\test");
    }

    #[test]
    fn escape_toml_double_quotes() {
        assert_eq!(escape_toml(r#"say "hi""#), r#"say \"hi\""#);
    }

    #[test]
    fn escape_toml_newlines_tabs() {
        assert_eq!(escape_toml("line1\nline2\ttab"), r"line1\nline2\ttab");
    }

    #[test]
    fn escape_toml_carriage_return() {
        assert_eq!(escape_toml("cr\rhere"), r"cr\rhere");
    }

    #[test]
    fn escape_toml_combined_special_chars() {
        assert_eq!(escape_toml("a\\b\"c\nd\re\tf"), r#"a\\b\"c\nd\re\tf"#);
    }

    #[test]
    fn escape_toml_empty_string() {
        assert_eq!(escape_toml(""), "");
    }

    // ── format_toml_array ────────────────────────────────────────────

    #[test]
    fn format_toml_array_empty() {
        let items: Vec<String> = vec![];
        assert_eq!(format_toml_array(&items), "[]");
    }

    #[test]
    fn format_toml_array_single() {
        let items = vec!["hello".to_string()];
        assert_eq!(format_toml_array(&items), r#"["hello"]"#);
    }

    #[test]
    fn format_toml_array_multiple() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(format_toml_array(&items), r#"["a", "b", "c"]"#);
    }

    #[test]
    fn format_toml_array_with_special_chars() {
        let items = vec!["say \"hi\"".to_string(), "path\\to".to_string()];
        assert_eq!(format_toml_array(&items), r#"["say \"hi\"", "path\\to"]"#);
    }

    // ── parse_csv ────────────────────────────────────────────────────

    #[test]
    fn parse_csv_empty_string() {
        assert!(parse_csv("").is_empty());
    }

    #[test]
    fn parse_csv_whitespace_only() {
        assert!(parse_csv("   ,  ,  ").is_empty());
    }

    #[test]
    fn parse_csv_single_value() {
        assert_eq!(parse_csv("hello"), vec!["hello"]);
    }

    #[test]
    fn parse_csv_trims_whitespace() {
        assert_eq!(parse_csv("  a , b , c  "), vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_csv_filters_empty_entries() {
        assert_eq!(parse_csv("a,,b, ,c"), vec!["a", "b", "c"]);
    }

    // ── parse_bool ───────────────────────────────────────────────────

    #[test]
    fn parse_bool_case_insensitive() {
        assert!(parse_bool("TRUE").unwrap());
        assert!(parse_bool("Yes").unwrap());
        assert!(parse_bool("ON").unwrap());
        assert!(!parse_bool("FALSE").unwrap());
        assert!(!parse_bool("No").unwrap());
        assert!(!parse_bool("OFF").unwrap());
    }

    #[test]
    fn parse_bool_invalid_returns_error() {
        let err = parse_bool("maybe").unwrap_err();
        assert!(err.to_string().contains("Invalid boolean value"));
    }

    // ── parse_duration_input ─────────────────────────────────────────

    #[test]
    fn parse_duration_input_zero_minutes() {
        assert_eq!(parse_duration_input("0").unwrap(), 0);
    }

    #[test]
    fn parse_duration_input_with_whitespace() {
        assert_eq!(parse_duration_input("  15  ").unwrap(), 900);
    }

    #[test]
    fn parse_duration_input_hours_with_whitespace() {
        assert_eq!(parse_duration_input("  3h  ").unwrap(), 10800);
    }

    #[test]
    fn parse_duration_input_days_value() {
        assert_eq!(parse_duration_input("1d").unwrap(), 86400);
        assert_eq!(parse_duration_input("7d").unwrap(), 604800);
    }

    #[test]
    fn parse_duration_input_invalid() {
        assert!(parse_duration_input("abc").is_err());
        assert!(parse_duration_input("").is_err());
    }

    // ── ChangeTracker ────────────────────────────────────────────────

    #[test]
    fn change_tracker_new_is_empty() {
        let tracker = ChangeTracker::new();
        assert!(tracker.changes.is_empty());
    }

    #[test]
    fn change_tracker_records_different_values() {
        let mut tracker = ChangeTracker::new();
        tracker.record("s1", "f1", "old1", "new1");
        tracker.record("s2", "f2", "old2", "new2");
        assert_eq!(tracker.changes.len(), 2);
        assert_eq!(tracker.changes[0].section, "s1");
        assert_eq!(tracker.changes[0].field, "f1");
        assert_eq!(tracker.changes[0].old_value, "old1");
        assert_eq!(tracker.changes[0].new_value, "new1");
    }

    #[test]
    fn change_tracker_skips_identical_values() {
        let mut tracker = ChangeTracker::new();
        tracker.record("s", "f", "same", "same");
        assert!(tracker.changes.is_empty());
    }

    #[test]
    fn change_tracker_empty_string_values() {
        let mut tracker = ChangeTracker::new();
        tracker.record("s", "f", "", "new");
        assert_eq!(tracker.changes.len(), 1);
        tracker.record("s", "f", "", "");
        assert_eq!(tracker.changes.len(), 1); // no change for "" -> ""
    }

    // ── parse_duration_input additional cases ────────────────────────

    #[test]
    fn parse_duration_input_minutes_value() {
        assert_eq!(parse_duration_input("15").unwrap(), 900);
        assert_eq!(parse_duration_input("60").unwrap(), 3600);
        assert_eq!(parse_duration_input("1").unwrap(), 60);
    }

    #[test]
    fn parse_duration_input_hours_uppercase() {
        assert_eq!(parse_duration_input("2H").unwrap(), 7200);
    }

    #[test]
    fn parse_duration_input_days_uppercase() {
        assert_eq!(parse_duration_input("3D").unwrap(), 259200);
    }

    #[test]
    fn parse_duration_input_hours_invalid_number() {
        assert!(parse_duration_input("xh").is_err());
    }

    #[test]
    fn parse_duration_input_days_invalid_number() {
        assert!(parse_duration_input("xd").is_err());
    }

    #[test]
    fn parse_duration_input_leading_trailing_spaces() {
        assert_eq!(parse_duration_input("  30  ").unwrap(), 1800);
    }

    // ── parse_bool additional cases ─────────────────────────────────

    #[test]
    fn parse_bool_numeric_values() {
        assert!(parse_bool("1").unwrap());
        assert!(!parse_bool("0").unwrap());
    }

    #[test]
    fn parse_bool_mixed_case() {
        assert!(parse_bool("True").unwrap());
        assert!(parse_bool("YES").unwrap());
        assert!(!parse_bool("False").unwrap());
        assert!(!parse_bool("NO").unwrap());
    }

    #[test]
    fn parse_bool_on_off() {
        assert!(parse_bool("on").unwrap());
        assert!(!parse_bool("off").unwrap());
    }

    // ── parse_csv additional cases ──────────────────────────────────

    #[test]
    fn parse_csv_preserves_inner_spaces() {
        let result = parse_csv("hello world, foo bar");
        assert_eq!(result, vec!["hello world", "foo bar"]);
    }

    #[test]
    fn parse_csv_single_item_no_comma() {
        assert_eq!(parse_csv("single"), vec!["single"]);
    }

    // ── escape_toml additional cases ────────────────────────────────

    #[test]
    fn escape_toml_unicode_preserved() {
        assert_eq!(escape_toml("emoji 🎉"), "emoji 🎉");
    }

    #[test]
    fn escape_toml_all_special_chars_combined() {
        let input = "a\\b\"c\nd\re\tf";
        let expected = r#"a\\b\"c\nd\re\tf"#;
        assert_eq!(escape_toml(input), expected);
    }

    // ── format_toml_array additional cases ──────────────────────────

    #[test]
    fn format_toml_array_single_item() {
        let items = vec!["item".to_string()];
        assert_eq!(format_toml_array(&items), r#"["item"]"#);
    }

    #[test]
    fn format_toml_array_unicode_items() {
        let items = vec!["café".to_string(), "naïve".to_string()];
        assert_eq!(format_toml_array(&items), r#"["café", "naïve"]"#);
    }

    // ── ChangeTracker field access ──────────────────────────────────

    #[test]
    fn change_fields_are_accessible() {
        let mut tracker = ChangeTracker::new();
        tracker.record("section", "field", "old", "new");
        let change = &tracker.changes[0];
        assert_eq!(change.section, "section");
        assert_eq!(change.field, "field");
        assert_eq!(change.old_value, "old");
        assert_eq!(change.new_value, "new");
    }

    #[test]
    fn change_tracker_multiple_sections() {
        let mut tracker = ChangeTracker::new();
        tracker.record("business", "name", "a", "b");
        tracker.record("llm", "provider", "c", "d");
        tracker.record("scoring", "threshold", "50", "70");
        assert_eq!(tracker.changes.len(), 3);
        assert_eq!(tracker.changes[0].section, "business");
        assert_eq!(tracker.changes[1].section, "llm");
        assert_eq!(tracker.changes[2].section, "scoring");
    }
}
