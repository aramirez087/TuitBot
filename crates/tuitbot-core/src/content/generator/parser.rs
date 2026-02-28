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
