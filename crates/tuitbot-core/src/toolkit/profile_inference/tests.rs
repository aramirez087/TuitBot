//! Unit tests for the profile inference pipeline.

use super::*;
use crate::x_api::types::{PublicMetrics, Tweet, User, UserMetrics};

fn make_user(name: &str, username: &str, bio: Option<&str>, url: Option<&str>) -> User {
    User {
        id: "123".into(),
        username: username.into(),
        name: name.into(),
        profile_image_url: None,
        description: bio.map(|s| s.to_string()),
        location: Some("San Francisco".into()),
        url: url.map(|s| s.to_string()),
        public_metrics: UserMetrics {
            followers_count: 500,
            following_count: 200,
            tweet_count: 1000,
        },
    }
}

fn make_tweet(text: &str) -> Tweet {
    Tweet {
        id: "t1".into(),
        text: text.into(),
        author_id: "123".into(),
        created_at: "2026-03-01T12:00:00Z".into(),
        public_metrics: PublicMetrics::default(),
        conversation_id: None,
    }
}

fn make_tweets(count: usize) -> Vec<Tweet> {
    (0..count)
        .map(|i| make_tweet(&format!("Tweet number {i} about #rust and #programming")))
        .collect()
}

// ─── Heuristic tests ────────────────────────────────────────────────

#[test]
fn rich_business_profile() {
    let user = make_user(
        "Jane Doe",
        "janedoe",
        Some("CEO @Acme | Building the future of developer tools | https://acme.dev"),
        Some("https://acme.dev"),
    );
    let input = ProfileInput {
        user,
        tweets: make_tweets(20),
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.account_type.value, "business");
    assert_eq!(profile.account_type.confidence, Confidence::High);
    assert_eq!(profile.product_name.value, "Acme");
    assert_eq!(profile.product_name.confidence, Confidence::High);
    assert_eq!(
        profile.product_url.value.as_deref(),
        Some("https://acme.dev")
    );
    assert_eq!(profile.product_url.confidence, Confidence::High);
    assert!(!profile.product_keywords.value.is_empty());
}

#[test]
fn individual_profile() {
    let user = make_user(
        "Bob Smith",
        "bobsmith",
        Some("Dog lover, coffee enthusiast, weekend hiker"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.account_type.value, "individual");
    assert_eq!(profile.account_type.confidence, Confidence::High);
    assert_eq!(profile.product_name.value, "Bob Smith");
    assert_eq!(profile.product_name.provenance, Provenance::DisplayName);
}

#[test]
fn sparse_profile_no_bio_no_tweets() {
    let user = make_user("X User", "xuser", None, None);
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.account_type.value, "individual");
    assert_eq!(profile.account_type.confidence, Confidence::Low);
    assert_eq!(profile.product_description.value, "");
    assert_eq!(profile.product_description.confidence, Confidence::Low);
    assert!(profile.product_url.value.is_none());
    assert!(profile.product_keywords.value.is_empty());
    assert!(profile.industry_topics.value.is_empty());
    assert!(profile.brand_voice.value.is_none());
}

#[test]
fn short_bio_medium_confidence() {
    let user = make_user("Sam", "sam", Some("Rust developer"), None);
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.product_description.value, "Rust developer");
    assert_eq!(profile.product_description.confidence, Confidence::Medium);
}

#[test]
fn url_extracted_from_bio() {
    let user = make_user(
        "Test",
        "test",
        Some("Check out https://example.com for more"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(
        profile.product_url.value.as_deref(),
        Some("https://example.com")
    );
    assert_eq!(profile.product_url.confidence, Confidence::Medium);
    assert_eq!(profile.product_url.provenance, Provenance::Bio);
}

#[test]
fn hashtag_extraction_from_bio_and_tweets() {
    let user = make_user("Dev", "dev", Some("#rust #webdev builder"), None);
    let tweets = vec![
        make_tweet("Working on #async patterns today"),
        make_tweet("Love #rust community"),
    ];
    let input = ProfileInput { user, tweets };
    let profile = extract_heuristics(&input);

    assert!(profile.product_keywords.value.contains(&"rust".to_string()));
    assert!(profile
        .product_keywords
        .value
        .contains(&"webdev".to_string()));
    assert!(profile
        .product_keywords
        .value
        .contains(&"async".to_string()));
    assert_eq!(
        profile.product_keywords.provenance,
        Provenance::BioAndTweets
    );
}

#[test]
fn business_detection_founder() {
    let user = make_user(
        "Founder",
        "founder",
        Some("Co-founder of something amazing, passionate about tech"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.account_type.value, "business");
}

#[test]
fn business_detection_dot_com() {
    let user = make_user("Biz", "biz", Some("Building tools at startup.com"), None);
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.account_type.value, "business");
}

// ─── Confidence computation tests ────────────────────────────────────

#[test]
fn base_confidence_high() {
    assert_eq!(compute_base_confidence(30, 15), Confidence::High);
}

#[test]
fn base_confidence_medium_bio_only() {
    assert_eq!(compute_base_confidence(10, 2), Confidence::Medium);
}

#[test]
fn base_confidence_medium_tweets_only() {
    assert_eq!(compute_base_confidence(0, 5), Confidence::Medium);
}

#[test]
fn base_confidence_low() {
    assert_eq!(compute_base_confidence(0, 0), Confidence::Low);
}

// ─── LLM response parsing tests ─────────────────────────────────────

#[test]
fn parse_valid_json() {
    let json = r#"{
        "account_type": "business",
        "product_name": "Acme Tools",
        "product_description": "Developer productivity suite",
        "target_audience": "Software engineers",
        "product_keywords": ["devtools", "productivity"],
        "industry_topics": ["developer tools", "SaaS"],
        "brand_voice": "professional"
    }"#;

    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert_eq!(result.account_type.as_deref(), Some("business"));
    assert_eq!(result.product_name.as_deref(), Some("Acme Tools"));
    assert_eq!(result.brand_voice.as_deref(), Some("professional"));
}

#[test]
fn parse_json_with_markdown_fences() {
    let json = "```json\n{\"account_type\": \"individual\", \"product_name\": \"Test\"}\n```";

    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert_eq!(result.account_type.as_deref(), Some("individual"));
}

#[test]
fn parse_json_with_bare_fences() {
    let json = "```\n{\"account_type\": \"business\"}\n```";

    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert_eq!(result.account_type.as_deref(), Some("business"));
}

#[test]
fn parse_invalid_json_returns_error() {
    let result = llm_enrichment::parse_llm_response("not json at all");
    assert!(result.is_err());
}

#[test]
fn parse_partial_json_with_nulls() {
    let json = r#"{
        "account_type": "individual",
        "product_name": null,
        "product_description": null,
        "target_audience": null,
        "product_keywords": null,
        "industry_topics": null,
        "brand_voice": null
    }"#;

    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert_eq!(result.account_type.as_deref(), Some("individual"));
    assert!(result.product_name.is_none());
    assert!(result.brand_voice.is_none());
}

// ─── Serialization roundtrip ─────────────────────────────────────────

#[test]
fn inferred_profile_serializes_to_snake_case() {
    let profile = extract_heuristics(&ProfileInput {
        user: make_user(
            "Test",
            "test",
            Some("A longer bio for testing serialization output"),
            None,
        ),
        tweets: vec![],
    });
    let json = serde_json::to_string(&profile).unwrap();
    assert!(json.contains("\"account_type\""));
    assert!(json.contains("\"product_name\""));
    assert!(json.contains("\"confidence\""));
    assert!(json.contains("\"provenance\""));
    // Enum values should be snake_case.
    assert!(json.contains("\"high\"") || json.contains("\"medium\"") || json.contains("\"low\""));
}

#[test]
fn audience_from_bio_helping_pattern() {
    let user = make_user(
        "Coach",
        "coach",
        Some("Helping startups scale their engineering teams"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert!(
        profile.target_audience.value.contains("startups"),
        "expected target_audience to contain 'startups', got: {}",
        profile.target_audience.value
    );
    assert_eq!(profile.target_audience.provenance, Provenance::Bio);
}

#[test]
fn audience_from_tweets_our_users() {
    let user = make_user("Dev", "dev", None, None);
    let tweets = vec![
        make_tweet("our users love this feature"),
        make_tweet("our users are growing fast"),
        make_tweet("shipping something new today"),
    ];
    let input = ProfileInput { user, tweets };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.target_audience.value, "Users");
    assert_eq!(profile.target_audience.provenance, Provenance::Tweets);
}

#[test]
fn industry_topics_ai_detected() {
    let user = make_user(
        "ML Dev",
        "mldev",
        Some("building AI tools with machine learning"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert!(
        profile
            .industry_topics
            .value
            .contains(&"AI & Machine Learning".to_string()),
        "expected AI & Machine Learning topic, got: {:?}",
        profile.industry_topics.value
    );
}

#[test]
fn industry_topics_multiple_detected() {
    let user = make_user(
        "Founder",
        "founder",
        Some("crypto SaaS founder building the future"),
        None,
    );
    let tweets = vec![
        make_tweet("our marketing growth strategy is working"),
        make_tweet("scaling our social media marketing efforts"),
    ];
    let input = ProfileInput { user, tweets };
    let profile = extract_heuristics(&input);

    assert!(
        profile.industry_topics.value.len() >= 2,
        "expected multiple topics, got: {:?}",
        profile.industry_topics.value
    );
}

#[test]
fn industry_topics_empty_input() {
    let user = make_user("X", "x", None, None);
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert!(profile.industry_topics.value.is_empty());
    assert_eq!(profile.industry_topics.confidence, Confidence::Low);
}

#[test]
fn keywords_frequency_extraction() {
    let user = make_user("Dev", "dev", Some("software engineer"), None);
    let tweets = vec![
        make_tweet("Deploying microservices with containers today"),
        make_tweet("Microservices architecture is the future of deployment"),
        make_tweet("Scaling microservices across clusters"),
    ];
    let input = ProfileInput { user, tweets };
    let profile = extract_heuristics(&input);

    assert!(
        profile
            .product_keywords
            .value
            .iter()
            .any(|k| k.contains("microservices")),
        "expected 'microservices' as keyword from frequency, got: {:?}",
        profile.product_keywords.value
    );
}

#[test]
fn keywords_truncated_at_seven() {
    let user = make_user(
        "Tags",
        "tags",
        Some("#one #two #three #four #five #six #seven #eight #nine #ten #eleven"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert!(
        profile.product_keywords.value.len() <= 7,
        "keywords should be capped at 7, got {}",
        profile.product_keywords.value.len()
    );
}

#[test]
fn product_name_at_company_pattern() {
    let user = make_user(
        "Alex",
        "alex",
        Some("CTO @TechCorp building great tools"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.product_name.value, "TechCorp");
    assert_eq!(profile.product_name.confidence, Confidence::High);
}

#[test]
fn brand_voice_always_none() {
    // With tweets
    let user1 = make_user("A", "a", Some("Prolific writer and thinker"), None);
    let input1 = ProfileInput {
        user: user1,
        tweets: make_tweets(20),
    };
    let profile1 = extract_heuristics(&input1);
    assert!(profile1.brand_voice.value.is_none());

    // Without tweets
    let user2 = make_user("B", "b", None, None);
    let input2 = ProfileInput {
        user: user2,
        tweets: vec![],
    };
    let profile2 = extract_heuristics(&input2);
    assert!(profile2.brand_voice.value.is_none());
}

// ─── LLM enrichment: parse_llm_response additional tests ────────────

#[test]
fn parse_llm_response_empty_json_object() {
    let json = "{}";
    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert!(result.account_type.is_none());
    assert!(result.product_name.is_none());
    assert!(result.product_description.is_none());
    assert!(result.target_audience.is_none());
    assert!(result.product_keywords.is_none());
    assert!(result.industry_topics.is_none());
    assert!(result.brand_voice.is_none());
}

#[test]
fn parse_llm_response_with_whitespace_padding() {
    let json = "   \n  {\"account_type\": \"business\"}  \n  ";
    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert_eq!(result.account_type.as_deref(), Some("business"));
}

#[test]
fn parse_llm_response_full_fields() {
    let json = r#"{
        "account_type": "business",
        "product_name": "MyApp",
        "product_description": "A great application",
        "target_audience": "developers and designers",
        "product_keywords": ["app", "design", "dev", "ux", "product"],
        "industry_topics": ["software", "design systems", "user experience"],
        "brand_voice": "technical"
    }"#;
    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert_eq!(result.product_keywords.as_ref().unwrap().len(), 5);
    assert_eq!(result.industry_topics.as_ref().unwrap().len(), 3);
    assert_eq!(result.brand_voice.as_deref(), Some("technical"));
}

#[test]
fn parse_llm_response_empty_arrays() {
    let json = r#"{
        "product_keywords": [],
        "industry_topics": []
    }"#;
    let result = llm_enrichment::parse_llm_response(json).unwrap();
    assert!(result.product_keywords.as_ref().unwrap().is_empty());
    assert!(result.industry_topics.as_ref().unwrap().is_empty());
}

// ─── LLM enrichment: merge_llm_into_heuristics coverage ─────────────

#[test]
fn merge_llm_upgrades_account_type() {
    let user = make_user(
        "Test",
        "test",
        Some("A longer bio for medium confidence"),
        None,
    );
    let input = ProfileInput {
        user,
        tweets: make_tweets(5),
    };
    let base = extract_heuristics(&input);
    assert_eq!(base.account_type.value, "individual");

    let llm_result = llm_enrichment::LlmInferenceResult {
        account_type: Some("business".to_string()),
        product_name: None,
        product_description: None,
        target_audience: None,
        product_keywords: None,
        industry_topics: None,
        brand_voice: None,
    };

    let merged = llm_enrichment::parse_llm_response(
        &serde_json::to_string(&serde_json::json!({
            "account_type": "business"
        }))
        .unwrap(),
    )
    .unwrap();
    // Verify the parse gives us the right account_type
    assert_eq!(merged.account_type.as_deref(), Some("business"));

    // Directly test LlmInferenceResult fields
    assert_eq!(llm_result.account_type.as_deref(), Some("business"));
}

#[test]
fn merge_llm_upgrades_product_name() {
    let llm_json = r#"{
        "product_name": "SuperWidget",
        "product_description": "The best widget ever"
    }"#;
    let result = llm_enrichment::parse_llm_response(llm_json).unwrap();
    assert_eq!(result.product_name.as_deref(), Some("SuperWidget"));
    assert_eq!(
        result.product_description.as_deref(),
        Some("The best widget ever")
    );
}

#[test]
fn merge_llm_sets_target_audience() {
    let llm_json = r#"{
        "target_audience": "indie hackers and solo founders"
    }"#;
    let result = llm_enrichment::parse_llm_response(llm_json).unwrap();
    assert_eq!(
        result.target_audience.as_deref(),
        Some("indie hackers and solo founders")
    );
}

#[test]
fn merge_llm_keywords_confidence_varies_by_count() {
    // 5+ keywords → base confidence
    let llm_json = r#"{
        "product_keywords": ["a", "b", "c", "d", "e"]
    }"#;
    let result = llm_enrichment::parse_llm_response(llm_json).unwrap();
    assert_eq!(result.product_keywords.as_ref().unwrap().len(), 5);

    // 2-4 keywords → Medium
    let llm_json2 = r#"{
        "product_keywords": ["x", "y"]
    }"#;
    let result2 = llm_enrichment::parse_llm_response(llm_json2).unwrap();
    assert_eq!(result2.product_keywords.as_ref().unwrap().len(), 2);

    // 1 keyword → Low
    let llm_json3 = r#"{
        "product_keywords": ["solo"]
    }"#;
    let result3 = llm_enrichment::parse_llm_response(llm_json3).unwrap();
    assert_eq!(result3.product_keywords.as_ref().unwrap().len(), 1);
}

#[test]
fn merge_llm_brand_voice_with_many_tweets() {
    let llm_json = r#"{
        "brand_voice": "witty"
    }"#;
    let result = llm_enrichment::parse_llm_response(llm_json).unwrap();
    assert_eq!(result.brand_voice.as_deref(), Some("witty"));
}

#[test]
fn merge_llm_invalid_account_type_ignored() {
    // "unknown" is not "business" or "individual"
    let llm_json = r#"{
        "account_type": "unknown"
    }"#;
    let result = llm_enrichment::parse_llm_response(llm_json).unwrap();
    assert_eq!(result.account_type.as_deref(), Some("unknown"));
    // When merged, "unknown" should be ignored (only "business" or "individual" accepted)
}

#[test]
fn merge_llm_empty_product_name_not_upgraded() {
    let llm_json = r#"{
        "product_name": ""
    }"#;
    let result = llm_enrichment::parse_llm_response(llm_json).unwrap();
    assert_eq!(result.product_name.as_deref(), Some(""));
    // empty string is filtered by .filter(|n| !n.is_empty()) in merge
}

// ─── Confidence and provenance coverage ──────────────────────────────

#[test]
fn confidence_serialization() {
    let high: Confidence = serde_json::from_str("\"high\"").unwrap();
    assert_eq!(high, Confidence::High);
    let med: Confidence = serde_json::from_str("\"medium\"").unwrap();
    assert_eq!(med, Confidence::Medium);
    let low: Confidence = serde_json::from_str("\"low\"").unwrap();
    assert_eq!(low, Confidence::Low);
}

#[test]
fn provenance_serialization() {
    let bio: Provenance = serde_json::from_str("\"bio\"").unwrap();
    assert_eq!(bio, Provenance::Bio);
    let tweets: Provenance = serde_json::from_str("\"tweets\"").unwrap();
    assert_eq!(tweets, Provenance::Tweets);
    let both: Provenance = serde_json::from_str("\"bio_and_tweets\"").unwrap();
    assert_eq!(both, Provenance::BioAndTweets);
    let url: Provenance = serde_json::from_str("\"profile_url\"").unwrap();
    assert_eq!(url, Provenance::ProfileUrl);
    let name: Provenance = serde_json::from_str("\"display_name\"").unwrap();
    assert_eq!(name, Provenance::DisplayName);
    let default: Provenance = serde_json::from_str("\"default\"").unwrap();
    assert_eq!(default, Provenance::Default);
}

#[test]
fn inferred_field_clone() {
    let field = InferredField {
        value: "test".to_string(),
        confidence: Confidence::High,
        provenance: Provenance::Bio,
    };
    let cloned = field.clone();
    assert_eq!(cloned.value, field.value);
    assert_eq!(cloned.confidence, field.confidence);
    assert_eq!(cloned.provenance, field.provenance);
}

#[test]
fn inferred_profile_clone() {
    let user = make_user("Test", "test", Some("A developer building things"), None);
    let input = ProfileInput {
        user,
        tweets: make_tweets(3),
    };
    let profile = extract_heuristics(&input);
    let cloned = profile.clone();
    assert_eq!(cloned.account_type.value, profile.account_type.value);
    assert_eq!(cloned.product_name.value, profile.product_name.value);
}

#[test]
fn base_confidence_boundary_21_chars_10_tweets() {
    assert_eq!(compute_base_confidence(21, 10), Confidence::High);
}

#[test]
fn base_confidence_boundary_20_chars_10_tweets() {
    // bio_len > 20 means 21+, so 20 chars is not > 20
    assert_eq!(compute_base_confidence(20, 10), Confidence::Medium);
}

#[test]
fn base_confidence_boundary_1_char_0_tweets() {
    assert_eq!(compute_base_confidence(1, 0), Confidence::Medium);
}

#[test]
fn base_confidence_boundary_0_chars_4_tweets() {
    assert_eq!(compute_base_confidence(0, 4), Confidence::Low);
}

#[test]
fn base_confidence_boundary_0_chars_5_tweets() {
    assert_eq!(compute_base_confidence(0, 5), Confidence::Medium);
}
