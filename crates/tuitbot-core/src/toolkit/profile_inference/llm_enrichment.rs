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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolkit::profile_inference::{
        Confidence, InferredField, InferredProfile, Provenance,
    };
    use crate::x_api::types::{User, UserMetrics};

    fn make_user(bio: Option<&str>, name: &str, username: &str) -> User {
        User {
            id: "123".to_string(),
            username: username.to_string(),
            name: name.to_string(),
            profile_image_url: None,
            description: bio.map(|s| s.to_string()),
            location: None,
            url: None,
            public_metrics: UserMetrics {
                followers_count: 1000,
                following_count: 200,
                tweet_count: 500,
            },
        }
    }

    fn make_input(bio: Option<&str>) -> ProfileInput {
        ProfileInput {
            user: make_user(bio, "Test User", "testuser"),
            tweets: vec![],
        }
    }

    fn make_base_profile() -> InferredProfile {
        InferredProfile {
            account_type: InferredField {
                value: "individual".to_string(),
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
            product_name: InferredField {
                value: "Test".to_string(),
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
            product_description: InferredField {
                value: "A test product".to_string(),
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
            product_url: InferredField {
                value: None,
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
            target_audience: InferredField {
                value: "developers".to_string(),
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
            product_keywords: InferredField {
                value: vec!["test".to_string()],
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
            industry_topics: InferredField {
                value: vec!["tech".to_string()],
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
            brand_voice: InferredField {
                value: None,
                confidence: Confidence::Low,
                provenance: Provenance::Default,
            },
        }
    }

    // ── build_llm_prompt ─────────────────────────────────────────

    #[test]
    fn build_llm_prompt_with_bio() {
        let input = make_input(Some("Rust developer building cool things"));
        let (system, user_msg) = build_llm_prompt(&input);
        assert!(system.contains("analyzing an X"));
        assert!(user_msg.contains("Rust developer"));
        assert!(user_msg.contains("@testuser"));
        assert!(user_msg.contains("Test User"));
        assert!(user_msg.contains("1000")); // followers
    }

    #[test]
    fn build_llm_prompt_without_bio() {
        let input = make_input(None);
        let (_, user_msg) = build_llm_prompt(&input);
        assert!(user_msg.contains("(none)"));
    }

    #[test]
    fn build_llm_prompt_with_tweets() {
        let mut input = make_input(Some("dev"));
        input.tweets.push(crate::x_api::types::Tweet {
            id: "1".to_string(),
            text: "Just shipped a new feature!".to_string(),
            author_id: "123".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            public_metrics: Default::default(),
            conversation_id: None,
        });
        let (_, user_msg) = build_llm_prompt(&input);
        assert!(user_msg.contains("shipped a new feature"));
    }

    #[test]
    fn build_llm_prompt_no_tweets_placeholder() {
        let input = make_input(Some("dev"));
        let (_, user_msg) = build_llm_prompt(&input);
        assert!(user_msg.contains("no recent tweets available"));
    }

    #[test]
    fn build_llm_prompt_system_prompt_instructs_json() {
        let input = make_input(Some("dev"));
        let (system, _) = build_llm_prompt(&input);
        assert!(system.contains("JSON"));
    }

    // ── parse_llm_response ───────────────────────────────────────

    #[test]
    fn parse_llm_response_valid_json() {
        let json = r#"{
            "account_type": "business",
            "product_name": "Acme Corp",
            "product_description": "Cloud hosting",
            "target_audience": "startups",
            "product_keywords": ["cloud", "hosting"],
            "industry_topics": ["SaaS", "DevOps"],
            "brand_voice": "professional"
        }"#;
        let result = parse_llm_response(json).unwrap();
        assert_eq!(result.account_type.as_deref(), Some("business"));
        assert_eq!(result.product_name.as_deref(), Some("Acme Corp"));
        assert_eq!(result.brand_voice.as_deref(), Some("professional"));
        assert_eq!(result.product_keywords.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn parse_llm_response_with_code_fence() {
        let json = "```json\n{\"account_type\": \"individual\"}\n```";
        let result = parse_llm_response(json).unwrap();
        assert_eq!(result.account_type.as_deref(), Some("individual"));
    }

    #[test]
    fn parse_llm_response_with_bare_code_fence() {
        let json = "```\n{\"account_type\": \"business\"}\n```";
        let result = parse_llm_response(json).unwrap();
        assert_eq!(result.account_type.as_deref(), Some("business"));
    }

    #[test]
    fn parse_llm_response_invalid_json() {
        let result = parse_llm_response("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_llm_response_empty_object() {
        let result = parse_llm_response("{}").unwrap();
        assert!(result.account_type.is_none());
        assert!(result.product_name.is_none());
    }

    #[test]
    fn parse_llm_response_all_null() {
        let json = r#"{
            "account_type": null,
            "product_name": null,
            "product_description": null,
            "target_audience": null,
            "product_keywords": null,
            "industry_topics": null,
            "brand_voice": null
        }"#;
        let result = parse_llm_response(json).unwrap();
        assert!(result.account_type.is_none());
        assert!(result.product_keywords.is_none());
    }

    // ── merge_llm_into_heuristics ─────────────────────────────────

    #[test]
    fn merge_upgrades_account_type() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: Some("business".to_string()),
            product_name: None,
            product_description: None,
            target_audience: None,
            product_keywords: None,
            industry_topics: None,
            brand_voice: None,
        };
        let input = make_input(Some("We sell stuff"));
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.account_type.value, "business");
    }

    #[test]
    fn merge_ignores_invalid_account_type() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: Some("unknown_type".to_string()),
            product_name: None,
            product_description: None,
            target_audience: None,
            product_keywords: None,
            industry_topics: None,
            brand_voice: None,
        };
        let input = make_input(Some("bio"));
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.account_type.value, "individual");
    }

    #[test]
    fn merge_upgrades_product_name() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: None,
            product_name: Some("Better Name".to_string()),
            product_description: None,
            target_audience: None,
            product_keywords: None,
            industry_topics: None,
            brand_voice: None,
        };
        let input = make_input(Some("bio"));
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.product_name.value, "Better Name");
    }

    #[test]
    fn merge_skips_empty_product_name() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: None,
            product_name: Some(String::new()),
            product_description: None,
            target_audience: None,
            product_keywords: None,
            industry_topics: None,
            brand_voice: None,
        };
        let input = make_input(Some("bio"));
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.product_name.value, "Test");
    }

    #[test]
    fn merge_brand_voice_high_confidence_with_many_tweets() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: None,
            product_name: None,
            product_description: None,
            target_audience: None,
            product_keywords: None,
            industry_topics: None,
            brand_voice: Some("witty".to_string()),
        };
        let mut input = make_input(Some("bio"));
        for i in 0..10 {
            input.tweets.push(crate::x_api::types::Tweet {
                id: i.to_string(),
                text: format!("Tweet {i}"),
                author_id: "123".to_string(),
                created_at: String::new(),
                public_metrics: Default::default(),
                conversation_id: None,
            });
        }
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.brand_voice.value, Some("witty".to_string()));
        assert_eq!(merged.brand_voice.confidence, Confidence::High);
    }

    #[test]
    fn merge_brand_voice_low_confidence_few_tweets() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: None,
            product_name: None,
            product_description: None,
            target_audience: None,
            product_keywords: None,
            industry_topics: None,
            brand_voice: Some("technical".to_string()),
        };
        let input = make_input(Some("bio"));
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.brand_voice.confidence, Confidence::Low);
    }

    #[test]
    fn merge_keywords_confidence_scales_with_count() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: None,
            product_name: None,
            product_description: None,
            target_audience: None,
            product_keywords: Some(vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into(),
                "e".into(),
            ]),
            industry_topics: None,
            brand_voice: None,
        };
        let input = make_input(Some("bio"));
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.product_keywords.value.len(), 5);
    }

    #[test]
    fn merge_target_audience_from_llm() {
        let base = make_base_profile();
        let llm = LlmInferenceResult {
            account_type: None,
            product_name: None,
            product_description: None,
            target_audience: Some("enterprise CTOs".to_string()),
            product_keywords: None,
            industry_topics: None,
            brand_voice: None,
        };
        let input = make_input(Some("bio"));
        let merged = merge_llm_into_heuristics(base, llm, &input);
        assert_eq!(merged.target_audience.value, "enterprise CTOs");
    }

    // ── LlmInferenceResult struct ────────────────────────────────

    #[test]
    fn llm_inference_result_debug() {
        let result = LlmInferenceResult {
            account_type: Some("business".to_string()),
            product_name: None,
            product_description: None,
            target_audience: None,
            product_keywords: None,
            industry_topics: None,
            brand_voice: None,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("business"));
    }
}
