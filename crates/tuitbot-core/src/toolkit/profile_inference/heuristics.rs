//! Deterministic heuristic extraction from X profile data.
//!
//! First pass of the two-pass inference pipeline. Extracts structured fields
//! from bio text, display name, and profile URL without any LLM calls.

use regex::Regex;
use std::sync::OnceLock;

use crate::x_api::types::Tweet;

use super::{Confidence, InferredField, InferredProfile, ProfileInput, Provenance};

fn business_indicators() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(CEO|CTO|COO|CMO|founder|co-founder|cofounder|building|we build|our product|startup|company|™|®)\b|\.com\b"
        )
        .expect("valid regex")
    })
}

fn url_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"https?://[^\s)>,]+").expect("valid regex"))
}

fn hashtag_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#(\w{2,})").expect("valid regex"))
}

fn at_company_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:@|at\s+)([A-Z][A-Za-z0-9]+)").expect("valid regex"))
}

/// Extract an `InferredProfile` using only deterministic heuristics.
pub fn extract_heuristics(input: &ProfileInput) -> InferredProfile {
    let bio = input.user.description.as_deref().unwrap_or("");
    let tweet_count = input.tweets.len();

    let is_business = detect_business(bio);

    let account_type = infer_account_type(bio, is_business);
    let product_name = infer_product_name(bio, &input.user.name, is_business);
    let product_description = infer_product_description(bio);
    let product_url = infer_product_url(input.user.url.as_deref(), bio);
    let target_audience = infer_target_audience(bio, &input.tweets);
    let product_keywords = infer_product_keywords(bio, &input.tweets);
    let industry_topics = infer_industry_topics(bio, &input.tweets);
    let brand_voice = infer_brand_voice(tweet_count);

    InferredProfile {
        account_type,
        product_name,
        product_description,
        product_url,
        target_audience,
        product_keywords,
        industry_topics,
        brand_voice,
    }
}

fn detect_business(bio: &str) -> bool {
    business_indicators().is_match(bio)
}

fn infer_account_type(bio: &str, is_business: bool) -> InferredField<String> {
    if bio.is_empty() || bio.len() < 10 {
        return InferredField {
            value: "individual".to_string(),
            confidence: Confidence::Low,
            provenance: Provenance::Default,
        };
    }
    if is_business {
        InferredField {
            value: "business".to_string(),
            confidence: Confidence::High,
            provenance: Provenance::Bio,
        }
    } else {
        InferredField {
            value: "individual".to_string(),
            confidence: Confidence::High,
            provenance: Provenance::Bio,
        }
    }
}

fn infer_product_name(bio: &str, display_name: &str, is_business: bool) -> InferredField<String> {
    if is_business {
        if let Some(cap) = at_company_pattern().captures(bio) {
            return InferredField {
                value: cap[1].to_string(),
                confidence: Confidence::High,
                provenance: Provenance::Bio,
            };
        }
    }
    InferredField {
        value: display_name.to_string(),
        confidence: if is_business {
            Confidence::Low
        } else {
            Confidence::High
        },
        provenance: Provenance::DisplayName,
    }
}

fn infer_product_description(bio: &str) -> InferredField<String> {
    if bio.len() > 20 {
        InferredField {
            value: bio.to_string(),
            confidence: Confidence::High,
            provenance: Provenance::Bio,
        }
    } else if bio.len() >= 10 {
        InferredField {
            value: bio.to_string(),
            confidence: Confidence::Medium,
            provenance: Provenance::Bio,
        }
    } else {
        InferredField {
            value: String::new(),
            confidence: Confidence::Low,
            provenance: Provenance::Default,
        }
    }
}

fn infer_product_url(profile_url: Option<&str>, bio: &str) -> InferredField<Option<String>> {
    if let Some(url) = profile_url.filter(|u| !u.is_empty()) {
        return InferredField {
            value: Some(url.to_string()),
            confidence: Confidence::High,
            provenance: Provenance::ProfileUrl,
        };
    }
    if let Some(m) = url_pattern().find(bio) {
        return InferredField {
            value: Some(m.as_str().to_string()),
            confidence: Confidence::Medium,
            provenance: Provenance::Bio,
        };
    }
    InferredField {
        value: None,
        confidence: Confidence::Low,
        provenance: Provenance::Default,
    }
}

fn infer_target_audience(bio: &str, tweets: &[Tweet]) -> InferredField<String> {
    // Look for explicit audience signals in bio: "helping X", "for X", "empowering X"
    static AUDIENCE_RE: OnceLock<Regex> = OnceLock::new();
    let re = AUDIENCE_RE.get_or_init(|| {
        Regex::new(r"(?i)(?:helping|for|empowering|serving|supporting)\s+(.{5,60})(?:\.|,|$)")
            .expect("valid regex")
    });

    if let Some(cap) = re.captures(bio) {
        let audience = cap[1].trim().trim_end_matches('.').to_string();
        if !audience.is_empty() {
            return InferredField {
                value: audience,
                confidence: Confidence::Medium,
                provenance: Provenance::Bio,
            };
        }
    }

    // Fallback: scan tweets for "our users", "our customers", "our community" etc.
    static TWEET_AUDIENCE_RE: OnceLock<Regex> = OnceLock::new();
    let tweet_re = TWEET_AUDIENCE_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:our|my)\s+(users|customers|community|audience|readers|followers|clients|subscribers|developers|teams?)\b")
            .expect("valid regex")
    });

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for tweet in tweets {
        for cap in tweet_re.captures_iter(&tweet.text) {
            let noun = cap[1].to_lowercase();
            *counts.entry(noun).or_default() += 1;
        }
    }

    if let Some((noun, _)) = counts.iter().max_by_key(|(_, c)| *c) {
        return InferredField {
            value: capitalize(noun),
            confidence: Confidence::Low,
            provenance: Provenance::Tweets,
        };
    }

    InferredField {
        value: String::new(),
        confidence: Confidence::Low,
        provenance: Provenance::Default,
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn infer_product_keywords(bio: &str, tweets: &[Tweet]) -> InferredField<Vec<String>> {
    let mut keywords = Vec::new();

    // 1. Extract hashtags (highest signal).
    for cap in hashtag_pattern().captures_iter(bio) {
        let tag = cap[1].to_string();
        if !keywords.contains(&tag) {
            keywords.push(tag);
        }
    }

    for tweet in tweets {
        for cap in hashtag_pattern().captures_iter(&tweet.text) {
            let tag = cap[1].to_string();
            if !keywords.contains(&tag) {
                keywords.push(tag);
            }
        }
    }

    // 2. If hashtags alone aren't enough, extract frequent meaningful words
    //    from bio + tweet text (skip stopwords and short words).
    if keywords.len() < 5 {
        let mut word_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        let all_text = std::iter::once(bio.to_string())
            .chain(tweets.iter().map(|t| t.text.clone()))
            .collect::<Vec<_>>()
            .join(" ");

        for word in all_text.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if clean.len() >= 4
                && !is_stopword(&clean)
                && !clean.starts_with("http")
                && !clean.starts_with('@')
            {
                *word_counts.entry(clean).or_default() += 1;
            }
        }

        // Sort by frequency, take top candidates not already in keywords.
        let mut ranked: Vec<_> = word_counts.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));

        for (word, count) in ranked {
            if count < 2 {
                break;
            }
            if !keywords.iter().any(|k| k.to_lowercase() == word) {
                keywords.push(word);
            }
            if keywords.len() >= 7 {
                break;
            }
        }
    }

    keywords.truncate(7);

    let provenance = if !bio.is_empty() && !tweets.is_empty() {
        Provenance::BioAndTweets
    } else if !bio.is_empty() {
        Provenance::Bio
    } else if !tweets.is_empty() {
        Provenance::Tweets
    } else {
        Provenance::Default
    };

    let confidence = if keywords.len() >= 5 {
        Confidence::High
    } else if keywords.len() >= 2 {
        Confidence::Medium
    } else {
        Confidence::Low
    };

    InferredField {
        value: keywords,
        confidence,
        provenance,
    }
}

/// Common English stopwords that should not be treated as keywords.
fn is_stopword(w: &str) -> bool {
    matches!(
        w,
        "that"
            | "this"
            | "with"
            | "from"
            | "your"
            | "have"
            | "will"
            | "been"
            | "were"
            | "they"
            | "them"
            | "their"
            | "what"
            | "when"
            | "which"
            | "there"
            | "about"
            | "would"
            | "could"
            | "should"
            | "just"
            | "more"
            | "some"
            | "than"
            | "very"
            | "into"
            | "also"
            | "over"
            | "only"
            | "such"
            | "like"
            | "then"
            | "each"
            | "make"
            | "made"
            | "even"
            | "much"
            | "most"
            | "many"
            | "well"
            | "back"
            | "here"
            | "still"
            | "every"
            | "after"
            | "before"
            | "being"
            | "going"
            | "think"
            | "know"
            | "good"
            | "great"
            | "need"
            | "want"
            | "really"
            | "these"
            | "those"
            | "does"
            | "doing"
            | "getting"
            | "thing"
            | "things"
            | "don't"
            | "didn't"
            | "it's"
            | "i'm"
            | "can't"
            | "people"
            | "today"
            | "right"
            | "time"
            | "yeah"
            | "because"
    )
}

fn infer_industry_topics(bio: &str, tweets: &[Tweet]) -> InferredField<Vec<String>> {
    // Match bio + tweets against known industry/topic patterns.
    static TOPIC_MAP: &[(&str, &str)] = &[
        (
            r"(?i)\b(?:ai|artificial intelligence|machine learning|llm|gpt|neural)\b",
            "AI & Machine Learning",
        ),
        (
            r"(?i)\b(?:crypto|blockchain|web3|defi|nft|bitcoin|ethereum|solana)\b",
            "Crypto & Web3",
        ),
        (
            r"(?i)\b(?:saas|software|devtools|developer tools|api)\b",
            "SaaS & Developer Tools",
        ),
        (
            r"(?i)\b(?:marketing|growth|seo|content marketing|social media)\b",
            "Marketing & Growth",
        ),
        (
            r"(?i)\b(?:design|ux|ui|figma|user experience|product design)\b",
            "Design & UX",
        ),
        (
            r"(?i)\b(?:startup|founder|venture|fundrais|seed round|series [a-c]|vc)\b",
            "Startups & Venture",
        ),
        (
            r"(?i)\b(?:fintech|finance|banking|payment|trading|invest)\b",
            "Fintech & Finance",
        ),
        (
            r"(?i)\b(?:health|medical|biotech|wellness|fitness)\b",
            "Health & Wellness",
        ),
        (
            r"(?i)\b(?:education|edtech|learning|teaching|course)\b",
            "Education & EdTech",
        ),
        (
            r"(?i)\b(?:ecommerce|e-commerce|shopify|retail|commerce)\b",
            "E-commerce & Retail",
        ),
        (
            r"(?i)\b(?:devops|kubernetes|docker|cloud|aws|infrastructure|cicd)\b",
            "DevOps & Cloud",
        ),
        (
            r"(?i)\b(?:security|cybersecurity|infosec|privacy|encryption)\b",
            "Security & Privacy",
        ),
        (
            r"(?i)\b(?:gaming|gamedev|game design|esports|unity|unreal)\b",
            "Gaming",
        ),
        (
            r"(?i)\b(?:climate|sustainability|cleantech|renewable|green energy)\b",
            "Climate & Sustainability",
        ),
        (
            r"(?i)\b(?:data science|analytics|data engineer|big data|sql|database)\b",
            "Data & Analytics",
        ),
        (
            r"(?i)\b(?:mobile|ios|android|react native|flutter|swift|kotlin)\b",
            "Mobile Development",
        ),
        (
            r"(?i)\b(?:open source|oss|foss|github|open-source)\b",
            "Open Source",
        ),
        (
            r"(?i)\b(?:product management|pm|roadmap|user research|product-market)\b",
            "Product Management",
        ),
        (
            r"(?i)\b(?:rust|golang|typescript|python|javascript|programming)\b",
            "Software Engineering",
        ),
        (
            r"(?i)\b(?:creator economy|newsletter|podcast|youtube|content creator)\b",
            "Creator Economy",
        ),
    ];

    let all_text = std::iter::once(bio.to_string())
        .chain(tweets.iter().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join(" ");

    let mut matched: Vec<(&str, usize)> = Vec::new();

    for &(pattern, label) in TOPIC_MAP {
        // These patterns are small and compiled per-call, but this runs once
        // per profile analysis — acceptable for 20 patterns.
        if let Ok(re) = Regex::new(pattern) {
            let count = re.find_iter(&all_text).count();
            if count > 0 {
                matched.push((label, count));
            }
        }
    }

    // Sort by match count descending, take top 5.
    matched.sort_by(|a, b| b.1.cmp(&a.1));
    let topics: Vec<String> = matched.iter().take(5).map(|(l, _)| l.to_string()).collect();

    let provenance = if !bio.is_empty() && !tweets.is_empty() {
        Provenance::BioAndTweets
    } else if !bio.is_empty() {
        Provenance::Bio
    } else if !tweets.is_empty() {
        Provenance::Tweets
    } else {
        Provenance::Default
    };

    let confidence = if topics.len() >= 3 {
        Confidence::Medium
    } else {
        Confidence::Low
    };

    InferredField {
        value: topics,
        confidence,
        provenance,
    }
}

fn infer_brand_voice(tweet_count: usize) -> InferredField<Option<String>> {
    let _ = tweet_count;
    InferredField {
        value: None,
        confidence: Confidence::Low,
        provenance: Provenance::Default,
    }
}
