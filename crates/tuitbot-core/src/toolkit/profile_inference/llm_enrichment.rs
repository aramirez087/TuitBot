//! LLM-based enrichment for profile inference.
//!
//! Second pass of the two-pass inference pipeline. Sends a structured prompt
//! to the LLM and merges the results into the heuristic baseline.

use serde::Deserialize;

use crate::error::LlmError;
use crate::llm::{GenerationParams, LlmProvider};

use super::{Confidence, InferredField, InferredProfile, ProfileInput, Provenance};

/// Intermediate struct for parsing the LLM's JSON response.
#[derive(Debug, Deserialize)]
pub(super) struct LlmInferenceResult {
    pub(super) account_type: Option<String>,
    pub(super) product_name: Option<String>,
    pub(super) product_description: Option<String>,
    pub(super) target_audience: Option<String>,
    pub(super) product_keywords: Option<Vec<String>>,
    pub(super) industry_topics: Option<Vec<String>>,
    pub(super) brand_voice: Option<String>,
}

/// Enrich an existing heuristic profile with LLM analysis.
///
/// If the LLM call or response parsing fails, the heuristic baseline is
/// returned unchanged alongside the error.
pub async fn enrich_with_llm(
    base: InferredProfile,
    input: &ProfileInput,
    llm: &dyn LlmProvider,
) -> Result<InferredProfile, LlmError> {
    let (system, user_msg) = build_llm_prompt(input);

    let params = GenerationParams {
        max_tokens: 1024,
        temperature: 0.3,
        system_prompt: Some(system),
    };

    let response = llm.complete("", &user_msg, &params).await?;

    let parsed = parse_llm_response(&response.text)
        .map_err(|e| LlmError::Parse(format!("profile inference parse error: {e}")))?;

    Ok(merge_llm_into_heuristics(base, parsed, input))
}

/// Build the system and user prompts per the inference contract.
fn build_llm_prompt(input: &ProfileInput) -> (String, String) {
    let system =
        "You are analyzing an X (Twitter) profile to configure a content automation tool. \
        Extract structured profile information from the user's bio and recent tweets. \
        Return a JSON object with the specified fields. Be specific and actionable — \
        generic descriptions are not useful."
            .to_string();

    let bio = input.user.description.as_deref().unwrap_or("(none)");
    let url = input.user.url.as_deref().unwrap_or("(none)");
    let location = input.user.location.as_deref().unwrap_or("(none)");
    let followers = input.user.public_metrics.followers_count;
    let following = input.user.public_metrics.following_count;

    let tweets_formatted = if input.tweets.is_empty() {
        "(no recent tweets available)".to_string()
    } else {
        input
            .tweets
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t.text))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let user_msg = format!(
        r#"Analyze this X profile and return a JSON object with the following fields.

## Profile
- Display name: {name}
- Username: @{username}
- Bio: {bio}
- URL: {url}
- Location: {location}
- Followers: {followers}
- Following: {following}

## Recent Tweets (most recent first)
{tweets_formatted}

## Required Output (JSON)
{{
  "account_type": "individual" or "business",
  "product_name": "name of the person or product",
  "product_description": "one-sentence description",
  "target_audience": "who they want to reach",
  "product_keywords": ["keyword1", "keyword2", ...],
  "industry_topics": ["topic1", "topic2", ...],
  "brand_voice": "casual" | "balanced" | "professional" | "witty" | "technical" | null
}}

Rules:
- product_keywords: 3-7 terms that would find relevant tweets to reply to
- industry_topics: 2-5 broader themes for original content creation
- Keywords and topics should not overlap significantly
- If bio is empty or uninformative, rely more on tweet content
- If you cannot determine a field, use null"#,
        name = input.user.name,
        username = input.user.username,
    );

    (system, user_msg)
}

/// Parse the LLM response text into structured data.
///
/// Handles markdown code fences (```json ... ```) wrapping.
pub(super) fn parse_llm_response(text: &str) -> Result<LlmInferenceResult, String> {
    let trimmed = text.trim();

    // Strip markdown code fences if present.
    let json_str = if trimmed.starts_with("```") {
        let inner = trimmed
            .strip_prefix("```json")
            .or_else(|| trimmed.strip_prefix("```"))
            .unwrap_or(trimmed);
        inner.strip_suffix("```").unwrap_or(inner).trim()
    } else {
        trimmed
    };

    serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))
}

/// Merge LLM results into the heuristic baseline, upgrading fields
/// where the LLM provides better data.
fn merge_llm_into_heuristics(
    mut base: InferredProfile,
    llm: LlmInferenceResult,
    input: &ProfileInput,
) -> InferredProfile {
    let bio_len = input.user.description.as_deref().map_or(0, |b| b.len());
    let tweet_count = input.tweets.len();
    let base_conf = super::compute_base_confidence(bio_len, tweet_count);

    let provenance = if bio_len > 0 && tweet_count > 0 {
        Provenance::BioAndTweets
    } else if bio_len > 0 {
        Provenance::Bio
    } else if tweet_count > 0 {
        Provenance::Tweets
    } else {
        Provenance::Default
    };

    // Upgrade account_type if LLM provided one.
    if let Some(at) = llm.account_type {
        if at == "business" || at == "individual" {
            base.account_type = InferredField {
                value: at,
                confidence: base_conf.clone(),
                provenance: provenance.clone(),
            };
        }
    }

    // Upgrade product_name if LLM provided one.
    if let Some(name) = llm.product_name.filter(|n| !n.is_empty()) {
        base.product_name = InferredField {
            value: name,
            confidence: base_conf.clone(),
            provenance: provenance.clone(),
        };
    }

    // Upgrade product_description if LLM provided a better one.
    if let Some(desc) = llm.product_description.filter(|d| !d.is_empty()) {
        base.product_description = InferredField {
            value: desc,
            confidence: base_conf.clone(),
            provenance: provenance.clone(),
        };
    }

    // target_audience — only LLM can provide this meaningfully.
    if let Some(audience) = llm.target_audience.filter(|a| !a.is_empty()) {
        base.target_audience = InferredField {
            value: audience,
            confidence: base_conf.clone(),
            provenance: provenance.clone(),
        };
    }

    // product_keywords — merge LLM keywords with heuristic hashtags.
    if let Some(kw) = llm.product_keywords.filter(|k| !k.is_empty()) {
        let confidence = if kw.len() >= 5 {
            base_conf.clone()
        } else if kw.len() >= 2 {
            Confidence::Medium
        } else {
            Confidence::Low
        };
        base.product_keywords = InferredField {
            value: kw,
            confidence,
            provenance: provenance.clone(),
        };
    }

    // industry_topics — only LLM can provide this meaningfully.
    if let Some(topics) = llm.industry_topics.filter(|t| !t.is_empty()) {
        let confidence = if topics.len() >= 3 && tweet_count >= 5 {
            base_conf.clone()
        } else {
            Confidence::Medium
        };
        base.industry_topics = InferredField {
            value: topics,
            confidence,
            provenance: provenance.clone(),
        };
    }

    // brand_voice — downgrade if fewer than 5 tweets per contract.
    if let Some(voice) = llm.brand_voice {
        let confidence = if tweet_count >= 10 {
            Confidence::High
        } else if tweet_count >= 5 {
            Confidence::Medium
        } else {
            Confidence::Low
        };
        base.brand_voice = InferredField {
            value: Some(voice),
            confidence,
            provenance: if tweet_count > 0 {
                Provenance::Tweets
            } else {
                Provenance::Default
            },
        };
    }

    base
}
