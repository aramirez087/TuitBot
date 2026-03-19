//! Helper functions for language detection, URL parsing, and text similarity.
//!
//! These functions are currently stubbed/unused but preserved for future
//! content evaluation logic once the Config schema is updated to support
//! QA settings (language_policy, glossary, etc.).

#![allow(dead_code)]

use std::collections::HashSet;

use super::types::LanguageDetection;

/// Detect language from text using marker-based heuristics.
pub(super) fn detect_language(text: &str) -> Option<LanguageDetection> {
    let cleaned = text.trim();
    if cleaned.is_empty() {
        return None;
    }

    // English and Spanish marker words for basic detection
    let spanish_markers: HashSet<&'static str> = [
        "el", "la", "los", "las", "de", "que", "y", "en", "un", "una", "por", "para", "con",
    ]
    .into_iter()
    .collect();

    let english_markers: HashSet<&'static str> = [
        "the", "and", "for", "with", "is", "are", "to", "from", "this", "that",
    ]
    .into_iter()
    .collect();

    let mut es_score = 0usize;
    let mut en_score = 0usize;

    for token in cleaned.split_whitespace() {
        let lower = token.to_lowercase();
        if spanish_markers.contains(lower.as_str()) {
            es_score += 1;
        }
        if english_markers.contains(lower.as_str()) {
            en_score += 1;
        }
    }

    if es_score == 0 && en_score == 0 {
        return None;
    }
    if es_score == en_score {
        return None;
    }

    let (code, winner, loser) = if es_score > en_score {
        ("es", es_score, en_score)
    } else {
        ("en", en_score, es_score)
    };

    let confidence = ((winner - loser) as f32 / winner as f32).clamp(0.0, 1.0);
    Some(LanguageDetection {
        code: code.to_string(),
        confidence,
    })
}

/// Extract URLs from text.
pub(super) fn extract_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();

    for word in text.split_whitespace() {
        if word.starts_with("http://") || word.starts_with("https://") {
            let url = word
                .trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | '!' | '?'))
                .to_string();
            urls.push(url);
        }
    }

    urls
}

/// Extract domain from URL.
pub(super) fn extract_domain(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    let host = without_scheme.split('/').next()?.split('?').next()?.trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// Parse query string parameter keys from URL.
pub(super) fn parse_query_keys(url: &str) -> HashSet<String> {
    let query_part = url.split('?').nth(1).unwrap_or("");
    let query = query_part.split('#').next().unwrap_or(query_part);

    query
        .split('&')
        .filter_map(|kv| {
            let key = kv.split('=').next()?.trim();
            if key.is_empty() {
                None
            } else {
                Some(key.to_string())
            }
        })
        .collect()
}

/// Count emoji characters in text.
pub(super) fn count_emoji(text: &str) -> usize {
    text.chars().filter(|ch| is_emoji(*ch)).count()
}

fn is_emoji(ch: char) -> bool {
    let code = ch as u32;
    (0x1F300..=0x1FAFF).contains(&code) || (0x2600..=0x27BF).contains(&code)
}

/// Tokenize text for similarity comparison.
pub(super) fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_ascii_alphanumeric())
                .to_string()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

/// Compute Jaccard similarity between two token sets.
pub(super) fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

/// Normalize domain string.
pub(super) fn normalize_domain(domain: &str) -> String {
    domain.trim().to_lowercase()
}
