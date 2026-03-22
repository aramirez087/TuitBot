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
