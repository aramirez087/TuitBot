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
    let bio_len = bio.len();
    let tweet_count = input.tweets.len();
    let base = super::compute_base_confidence(bio_len, tweet_count);

    let is_business = detect_business(bio);

    let account_type = infer_account_type(bio, is_business);
    let product_name = infer_product_name(bio, &input.user.name, is_business);
    let product_description = infer_product_description(bio);
    let product_url = infer_product_url(input.user.url.as_deref(), bio);
    let target_audience = infer_target_audience(&base);
    let product_keywords = infer_product_keywords(bio, &input.tweets);
    let industry_topics = infer_industry_topics(&base);
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

fn infer_target_audience(_base: &Confidence) -> InferredField<String> {
    InferredField {
        value: String::new(),
        confidence: Confidence::Low,
        provenance: Provenance::Default,
    }
}

fn infer_product_keywords(bio: &str, tweets: &[Tweet]) -> InferredField<Vec<String>> {
    let mut keywords = Vec::new();

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

fn infer_industry_topics(_base: &Confidence) -> InferredField<Vec<String>> {
    InferredField {
        value: Vec::new(),
        confidence: Confidence::Low,
        provenance: Provenance::Default,
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
