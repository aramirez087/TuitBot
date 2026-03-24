/// Parse a thread response by splitting on `---` delimiters.
///
/// Also tries numbered patterns (e.g., "1/8", "1.") as a fallback.
pub fn parse_thread(text: &str) -> Vec<String> {
    // Primary: split on "---" delimiter
    let tweets: Vec<String> = text
        .split("---")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if !tweets.is_empty() && text.contains("---") {
        return tweets;
    }

    // Fallback: try splitting on numbered patterns like "1/8", "2/8" or "1.", "2."
    let lines: Vec<&str> = text.lines().collect();
    let mut tweets = Vec::new();
    let mut current = String::new();

    for line in &lines {
        let trimmed = line.trim();
        let is_numbered = trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
            && (trimmed.contains('/')
                || trimmed.starts_with(|c: char| c.is_ascii_digit())
                    && trimmed.chars().nth(1).is_some_and(|c| c == '.' || c == ')'));

        if is_numbered && !current.is_empty() {
            tweets.push(current.trim().to_string());
            current = String::new();
        }

        if !trimmed.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            // Strip the number prefix if present
            if is_numbered {
                let content = trimmed
                    .find(|c: char| !c.is_ascii_digit() && c != '/' && c != '.' && c != ')')
                    .map(|i| trimmed[i..].trim_start())
                    .unwrap_or(trimmed);
                current.push_str(content);
            } else {
                current.push_str(trimmed);
            }
        }
    }

    if !current.trim().is_empty() {
        tweets.push(current.trim().to_string());
    }

    tweets
}

/// Parse a hooks response with `STYLE:` and `HOOK:` lines separated by `---`.
///
/// Returns `(style, hook_text)` pairs. Falls back to `"general"` if STYLE line
/// is missing for a block.
///
/// Tolerant of common LLM formatting variations: case-insensitive prefixes,
/// markdown bold (`**STYLE:**`), numbered prefixes (`1. STYLE:`), and
/// leading/trailing quotes. When strict parsing yields fewer than 2 hooks,
/// falls back to extracting non-empty text blocks (separated by `---` or
/// numbered/bulleted lines).
pub fn parse_hooks_response(text: &str) -> Vec<(String, String)> {
    let results = parse_hooks_strict(text);
    if !results.is_empty() {
        return results;
    }

    // Fallback: strict parser found nothing — try extracting text blocks.
    parse_hooks_fallback(text)
}

/// Strict parser: looks for STYLE:/HOOK: prefixed lines separated by ---.
fn parse_hooks_strict(text: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut current_hook = String::new();
    let mut current_style = String::new();

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed == "---" || trimmed == "- - -" {
            if !current_hook.is_empty() {
                let style = if current_style.is_empty() {
                    "general".to_string()
                } else {
                    current_style.clone()
                };
                results.push((style, current_hook.clone()));
                current_hook.clear();
                current_style.clear();
            }
            continue;
        }

        // Strip markdown bold, numbering prefixes, and leading bullets
        let cleaned = strip_line_noise(trimmed);

        if let Some(s) = strip_prefix_ci(&cleaned, "style:") {
            current_style = s.trim().to_string();
        } else if let Some(h) = strip_prefix_ci(&cleaned, "hook:") {
            current_hook = strip_quotes(h.trim());
        }
    }

    // Capture the last block
    if !current_hook.is_empty() {
        let style = if current_style.is_empty() {
            "general".to_string()
        } else {
            current_style
        };
        results.push((style, current_hook));
    }

    results
}

/// Fallback parser: splits on `---` separators, or treats each non-empty
/// line (after stripping numbering/bullets) as a hook with style "general".
fn parse_hooks_fallback(text: &str) -> Vec<(String, String)> {
    // Try --- separated blocks first
    if text.contains("---") {
        let blocks: Vec<(String, String)> = text
            .split("---")
            .map(|block| {
                let cleaned = block
                    .lines()
                    .map(|l| strip_line_noise(l.trim()))
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                strip_quotes(cleaned.trim())
            })
            .filter(|s| !s.is_empty())
            .map(|s| extract_inline_style(&s))
            .collect();

        if blocks.len() >= 2 {
            return blocks;
        }
    }

    // Treat each non-empty line as a separate hook
    text.lines()
        .map(|line| strip_line_noise(line.trim()))
        .map(|s| strip_quotes(s.trim()))
        .filter(|s| !s.is_empty() && s.len() > 10) // skip very short lines (headers, etc.)
        .map(|s| extract_inline_style(&s))
        .collect()
}

/// Try to extract an inline style label like `[Question] hook text` or
/// `Question: hook text` from the beginning of text.
fn extract_inline_style(text: &str) -> (String, String) {
    // Pattern: [Style] rest
    if text.starts_with('[') {
        if let Some(end) = text.find(']') {
            let style = text[1..end].trim().to_string();
            let rest = text[end + 1..].trim().to_string();
            if !style.is_empty() && !rest.is_empty() {
                return (style, strip_quotes(&rest));
            }
        }
    }

    // Pattern: Style: rest (short label before colon, up to 3 words)
    if let Some(colon) = text.find(':') {
        let candidate = text[..colon].trim();
        if !candidate.is_empty()
            && candidate.len() <= 25
            && candidate.split_whitespace().count() <= 3
        {
            let rest = text[colon + 1..].trim();
            if !rest.is_empty() && rest.len() > 10 {
                return (candidate.to_string(), strip_quotes(rest));
            }
        }
    }

    ("general".to_string(), text.to_string())
}

/// Case-insensitive prefix strip. Returns the remainder after the prefix.
fn strip_prefix_ci<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = text.to_ascii_lowercase();
    if lower.starts_with(prefix) {
        Some(&text[prefix.len()..])
    } else {
        None
    }
}

/// Remove markdown bold markers, numbered prefixes, and bullet chars.
fn strip_line_noise(line: &str) -> String {
    let mut s = line.replace("**", "");
    // Strip leading number+punctuation like "1. ", "1) ", "1: "
    if let Some(first) = s.chars().next() {
        if first.is_ascii_digit() {
            if let Some(pos) = s.find(|c: char| !c.is_ascii_digit()) {
                let after = &s[pos..];
                if after.starts_with(". ") || after.starts_with(") ") || after.starts_with(": ") {
                    s = after[2..].to_string();
                }
            }
        }
    }
    // Strip leading bullet
    if s.starts_with("- ") || s.starts_with("• ") {
        s = s[2..].to_string();
    }
    s
}

/// Remove surrounding quotes (single or double) from hook text.
fn strip_quotes(text: &str) -> String {
    let t = text.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_thread: --- delimiter ──────────────────────────────

    #[test]
    fn parse_thread_basic_delimiter() {
        let input = "First tweet\n---\nSecond tweet\n---\nThird tweet";
        let tweets = parse_thread(input);
        assert_eq!(tweets.len(), 3);
        assert_eq!(tweets[0], "First tweet");
        assert_eq!(tweets[1], "Second tweet");
        assert_eq!(tweets[2], "Third tweet");
    }

    #[test]
    fn parse_thread_trims_whitespace() {
        let input = "  First tweet  \n---\n  Second tweet  ";
        let tweets = parse_thread(input);
        assert_eq!(tweets.len(), 2);
        assert_eq!(tweets[0], "First tweet");
        assert_eq!(tweets[1], "Second tweet");
    }

    #[test]
    fn parse_thread_skips_empty_segments() {
        let input = "First tweet\n---\n\n---\nThird tweet";
        let tweets = parse_thread(input);
        assert_eq!(tweets.len(), 2);
    }

    #[test]
    fn parse_thread_single_no_delimiter() {
        let input = "Just a single tweet, no delimiter here.";
        let tweets = parse_thread(input);
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0], "Just a single tweet, no delimiter here.");
    }

    #[test]
    fn parse_thread_empty_input() {
        assert!(parse_thread("").is_empty());
    }

    #[test]
    fn parse_thread_whitespace_only() {
        assert!(parse_thread("   \n  \n   ").is_empty());
    }

    #[test]
    fn parse_thread_multiline_with_delimiter() {
        let input = "First tweet\nwith two lines\n---\nSecond tweet\nalso two lines";
        let tweets = parse_thread(input);
        assert_eq!(tweets.len(), 2);
        assert!(tweets[0].contains("First tweet"));
        assert!(tweets[1].contains("Second tweet"));
    }

    #[test]
    fn parse_thread_special_chars() {
        let input = "Tweet with @mentions and #hashtags\n---\nTweet with https://example.com";
        let tweets = parse_thread(input);
        assert_eq!(tweets.len(), 2);
        assert!(tweets[0].contains("@mentions"));
        assert!(tweets[1].contains("https://"));
    }

    #[test]
    fn parse_thread_unicode() {
        let input = "Tweet with émojis 🚀🔥\n---\nTweet with ñ and ü";
        let tweets = parse_thread(input);
        assert_eq!(tweets.len(), 2);
        assert!(tweets[0].contains("🚀"));
    }

    #[test]
    fn parse_thread_numbered_dot() {
        let input = "1. First tweet\n2. Second tweet\n3. Third tweet";
        let tweets = parse_thread(input);
        assert!(tweets.len() >= 2);
    }

    // ── parse_hooks_response: strict ────────────────────────────

    #[test]
    fn parse_hooks_strict_basic() {
        let input = "STYLE: Question\nHOOK: What if you could 10x?\n---\nSTYLE: Bold\nHOOK: Most devs are wrong.";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].0, "Question");
        assert_eq!(hooks[0].1, "What if you could 10x?");
        assert_eq!(hooks[1].0, "Bold");
    }

    #[test]
    fn parse_hooks_missing_style() {
        let input = "HOOK: A hook without style\n---\nSTYLE: Question\nHOOK: Why not?";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].0, "general");
        assert_eq!(hooks[1].0, "Question");
    }

    #[test]
    fn parse_hooks_case_insensitive() {
        let input = "style: lowercase\nhook: This is a test hook.";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].0, "lowercase");
    }

    #[test]
    fn parse_hooks_strips_quotes() {
        let input = "STYLE: Q\nHOOK: \"What if testing was fun?\"";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks[0].1, "What if testing was fun?");
    }

    #[test]
    fn parse_hooks_empty() {
        assert!(parse_hooks_response("").is_empty());
    }

    #[test]
    fn parse_hooks_markdown_bold() {
        let input = "**STYLE:** Bold\n**HOOK:** A bold format hook\n---\n**STYLE:** Q\n**HOOK:** Another hook?";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn parse_hooks_numbered() {
        let input = "1. STYLE: First\n1. HOOK: First hook text here\n---\n2. STYLE: Second\n2. HOOK: Second hook text here";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn parse_hooks_single_no_separator() {
        let input = "STYLE: Story\nHOOK: I spent 3 years building the wrong thing.";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].0, "Story");
    }

    // ── parse_hooks_response: fallback ──────────────────────────

    #[test]
    fn parse_hooks_fallback_blocks() {
        let input =
            "A great opening hook that grabs attention\n---\nAnother compelling hook text here";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn parse_hooks_fallback_filters_short() {
        let input =
            "Short\nA longer hook that should be captured here\nAnother real hook text long enough";
        let hooks = parse_hooks_response(input);
        for (_, hook) in &hooks {
            assert!(hook.len() > 10, "Short lines should be filtered: {hook}");
        }
    }

    #[test]
    fn parse_hooks_no_keywords_plain_blocks() {
        let input = "Plain text block that could be a hook\n---\nAnother plain block of text";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 2);
    }

    // ── strip_line_noise (exercised via parse_hooks) ────────────

    #[test]
    fn parse_hooks_strips_bullet_prefix() {
        let input = "- STYLE: Bullet\n- HOOK: A hook with bullet prefix here";
        let hooks = parse_hooks_response(input);
        assert!(!hooks.is_empty());
    }

    // ── extract_inline_style (exercised via fallback) ───────────

    #[test]
    fn parse_hooks_bracket_style_extraction() {
        let input = "[Question] What if you could double test coverage?\n---\n[Bold] Most teams ship broken code daily";
        let hooks = parse_hooks_response(input);
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].0, "Question");
        assert_eq!(hooks[1].0, "Bold");
    }
}
