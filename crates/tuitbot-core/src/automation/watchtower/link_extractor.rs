//! Link and tag extraction from markdown note bodies.
//!
//! Pure functions — no async, no DB access. Takes `&str`, returns extracted
//! links and tags. All resolution (DB lookups) happens in `graph_ingest.rs`.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

/// A raw link or tag extracted from a note body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLink {
    /// The raw target string (note name for wikilinks, path for md links, tag text for tags).
    pub target_raw: String,
    /// The type of link.
    pub link_type: LinkType,
    /// Display text (alias) if provided.
    pub display_text: Option<String>,
    /// Heading anchor (e.g. `#Heading` in `[[Note#Heading]]`).
    pub heading_anchor: Option<String>,
}

/// The kind of link extracted from a note body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkType {
    Wikilink,
    MarkdownLink,
    InlineTag,
}

// ---------------------------------------------------------------------------
// Regex singletons
// ---------------------------------------------------------------------------

fn wikilink_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\[\[([^\]\|#]+)(?:#([^\]\|]+))?(?:\|([^\]]+))?\]\]").expect("wikilink regex")
    })
}

fn md_link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("md link regex"))
}

fn inline_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?:^|[\s])#([a-zA-Z][a-zA-Z0-9_/-]*)").expect("inline tag regex")
    })
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

/// Extract all wikilinks, markdown note links, and inline tags from a note body.
///
/// Skips content inside fenced code blocks (`` ``` ``). Deduplicates by
/// `(target_raw, link_type)`.
pub fn extract_links(body: &str) -> Vec<RawLink> {
    let mut results = Vec::new();
    let mut seen: HashSet<(String, LinkType)> = HashSet::new();
    let mut in_code_block = false;

    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        // Wikilinks: [[Target]], [[Target#Heading]], [[Target|Display]], [[Target#Heading|Display]]
        for caps in wikilink_re().captures_iter(line) {
            let target = caps[1].trim().to_string();
            let heading = caps.get(2).map(|m| m.as_str().trim().to_string());
            let display = caps.get(3).map(|m| m.as_str().trim().to_string());

            let key = (target.clone(), LinkType::Wikilink);
            if seen.insert(key) {
                results.push(RawLink {
                    target_raw: target,
                    link_type: LinkType::Wikilink,
                    display_text: display,
                    heading_anchor: heading,
                });
            }
        }

        // Markdown links: [text](path) — only local refs (no http(s)://)
        for caps in md_link_re().captures_iter(line) {
            let display = caps[1].trim().to_string();
            let path = caps[2].trim().to_string();

            // Skip external URLs.
            if path.starts_with("http://") || path.starts_with("https://") {
                continue;
            }

            // Only include if path ends with .md, has no extension, or is a relative path
            let has_ext = path.rsplit('/').next().is_some_and(|f| f.contains('.'));
            let is_md = path.ends_with(".md");
            if has_ext && !is_md {
                continue;
            }

            let key = (path.clone(), LinkType::MarkdownLink);
            if seen.insert(key) {
                results.push(RawLink {
                    target_raw: path,
                    link_type: LinkType::MarkdownLink,
                    display_text: Some(display),
                    heading_anchor: None,
                });
            }
        }

        // Inline tags: #tag-name (preceded by whitespace or start-of-line)
        for caps in inline_tag_re().captures_iter(line) {
            let tag = caps[1].to_string();
            let key = (tag.clone(), LinkType::InlineTag);
            if seen.insert(key) {
                results.push(RawLink {
                    target_raw: tag,
                    link_type: LinkType::InlineTag,
                    display_text: None,
                    heading_anchor: None,
                });
            }
        }
    }

    results
}

/// Normalize a tag string: strip leading `#`, trim, lowercase, collapse whitespace to `-`.
pub fn normalize_tag(raw: &str) -> String {
    let stripped = raw.strip_prefix('#').unwrap_or(raw);
    stripped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_wikilinks_simple() {
        let links = extract_links("Check out [[Target Note]] for details.");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_raw, "Target Note");
        assert_eq!(links[0].link_type, LinkType::Wikilink);
        assert_eq!(links[0].display_text, None);
        assert_eq!(links[0].heading_anchor, None);
    }

    #[test]
    fn extract_wikilinks_with_display() {
        let links = extract_links("See [[Target|Display Text]] here.");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_raw, "Target");
        assert_eq!(links[0].display_text, Some("Display Text".to_string()));
    }

    #[test]
    fn extract_wikilinks_with_heading() {
        let links = extract_links("Jump to [[Target#Section One]].");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_raw, "Target");
        assert_eq!(links[0].heading_anchor, Some("Section One".to_string()));
    }

    #[test]
    fn extract_wikilinks_full() {
        let links = extract_links("See [[Target#Heading|Alias]].");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_raw, "Target");
        assert_eq!(links[0].heading_anchor, Some("Heading".to_string()));
        assert_eq!(links[0].display_text, Some("Alias".to_string()));
    }

    #[test]
    fn extract_markdown_links() {
        let links = extract_links("Read [the guide](./path.md) now.");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_raw, "./path.md");
        assert_eq!(links[0].link_type, LinkType::MarkdownLink);
        assert_eq!(links[0].display_text, Some("the guide".to_string()));
    }

    #[test]
    fn extract_markdown_links_skip_external() {
        let links = extract_links("[docs](https://example.com/docs)");
        let md_links: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::MarkdownLink)
            .collect();
        assert!(md_links.is_empty());
    }

    #[test]
    fn extract_markdown_links_no_extension() {
        let links = extract_links("[other](other-note)");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_raw, "other-note");
        assert_eq!(links[0].link_type, LinkType::MarkdownLink);
    }

    #[test]
    fn extract_markdown_links_skip_images() {
        let links = extract_links("[photo](assets/image.png)");
        let md_links: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::MarkdownLink)
            .collect();
        assert!(md_links.is_empty(), "should skip non-.md file extensions");
    }

    #[test]
    fn extract_inline_tags() {
        let links = extract_links("Topics: #rust #distributed-systems");
        let tags: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::InlineTag)
            .collect();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].target_raw, "rust");
        assert_eq!(tags[1].target_raw, "distributed-systems");
    }

    #[test]
    fn extract_inline_tags_skip_numeric_hex() {
        // Pure numeric hex like #123456 is skipped because the tag regex
        // requires the first char after # to be [a-zA-Z].
        let links = extract_links("color: #123456;");
        let tags: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::InlineTag)
            .collect();
        assert!(tags.is_empty(), "numeric hex should not be a tag");
    }

    #[test]
    fn extract_inline_tags_no_preceding_space() {
        // Tags glued to a word (no whitespace) are not extracted.
        let links = extract_links("foo#bar");
        let tags: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::InlineTag)
            .collect();
        assert!(tags.is_empty(), "no space before # means not a tag");
    }

    #[test]
    fn extract_code_block_skipped() {
        let body = "\
[[Real Link]]

```
[[Not A Link]]
#not-a-tag
[fake](./fake.md)
```

#real-tag
";
        let links = extract_links(body);
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l.target_raw == "Real Link"));
        assert!(links.iter().any(|l| l.target_raw == "real-tag"));
    }

    #[test]
    fn extract_mixed_links() {
        let body = "\
See [[Wiki Note]] and [guide](./guide.md).

Tags: #architecture #design
";
        let links = extract_links(body);
        assert_eq!(links.len(), 4);
        let types: Vec<LinkType> = links.iter().map(|l| l.link_type).collect();
        assert!(types.contains(&LinkType::Wikilink));
        assert!(types.contains(&LinkType::MarkdownLink));
        assert!(types.contains(&LinkType::InlineTag));
    }

    #[test]
    fn extract_deduplicates() {
        let body = "See [[Target]] and also [[Target]] again.";
        let links = extract_links(body);
        let wiki: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::Wikilink)
            .collect();
        assert_eq!(wiki.len(), 1, "duplicate wikilink should be deduplicated");
    }

    #[test]
    fn normalize_tag_cases() {
        assert_eq!(normalize_tag("#Rust"), "rust");
        assert_eq!(normalize_tag("Rust"), "rust");
        assert_eq!(normalize_tag("# My Tag"), "my-tag");
        assert_eq!(normalize_tag("  spaces  "), "spaces");
        assert_eq!(normalize_tag("#distributed-systems"), "distributed-systems");
    }

    #[test]
    fn extract_tag_at_line_start() {
        let links = extract_links("#startup");
        let tags: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::InlineTag)
            .collect();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].target_raw, "startup");
    }

    #[test]
    fn extract_multiple_wikilinks_same_line() {
        let links = extract_links("See [[Alpha]] and [[Beta]] and [[Gamma]].");
        let wikis: Vec<_> = links
            .iter()
            .filter(|l| l.link_type == LinkType::Wikilink)
            .collect();
        assert_eq!(wikis.len(), 3);
    }
}
