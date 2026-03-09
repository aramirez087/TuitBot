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
