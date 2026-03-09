//! Tests for the onboarding profile analysis pipeline.
//!
//! These test the core inference logic directly (not through HTTP) since
//! the handler delegates all work to `tuitbot_core::toolkit::profile_inference`.

use tuitbot_core::toolkit::profile_inference::{
    compute_base_confidence, extract_heuristics, Confidence, InferredProfile, ProfileInput,
    Provenance,
};
use tuitbot_core::x_api::types::{PublicMetrics, Tweet, User, UserMetrics};

fn make_user(bio: Option<&str>, url: Option<&str>) -> User {
    User {
        id: "999".into(),
        username: "testuser".into(),
        name: "Test User".into(),
        profile_image_url: Some("https://img.example.com/pic.jpg".into()),
        description: bio.map(|s| s.to_string()),
        location: Some("New York".into()),
        url: url.map(|s| s.to_string()),
        public_metrics: UserMetrics {
            followers_count: 1200,
            following_count: 300,
            tweet_count: 5000,
        },
    }
}

fn make_tweets(count: usize) -> Vec<Tweet> {
    (0..count)
        .map(|i| Tweet {
            id: format!("t{i}"),
            text: format!("Example tweet #{i} about #tech and #startups"),
            author_id: "999".into(),
            created_at: "2026-03-01T00:00:00Z".into(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        })
        .collect()
}

#[test]
fn rich_profile_all_fields_populated() {
    let user = make_user(
        Some("CEO @WidgetCo | Building developer tools for the modern era"),
        Some("https://widgetco.dev"),
    );
    let input = ProfileInput {
        user,
        tweets: make_tweets(20),
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.account_type.value, "business");
    assert_eq!(profile.product_name.value, "WidgetCo");
    assert!(!profile.product_description.value.is_empty());
    assert_eq!(
        profile.product_url.value.as_deref(),
        Some("https://widgetco.dev")
    );
    assert_eq!(profile.product_url.confidence, Confidence::High);
    // Keywords extracted from tweets
    assert!(profile.product_keywords.value.contains(&"tech".to_string()));
    assert!(profile
        .product_keywords
        .value
        .contains(&"startups".to_string()));
}

#[test]
fn sparse_profile_degrades_gracefully() {
    let user = make_user(None, None);
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    // All fields should have defaults with low confidence.
    assert_eq!(profile.account_type.confidence, Confidence::Low);
    assert_eq!(profile.product_description.confidence, Confidence::Low);
    assert_eq!(profile.product_url.confidence, Confidence::Low);
    assert_eq!(profile.target_audience.confidence, Confidence::Low);
    assert_eq!(profile.brand_voice.confidence, Confidence::Low);
    assert!(profile.product_url.value.is_none());
    assert!(profile.brand_voice.value.is_none());
}

#[test]
fn no_tweets_uses_bio_only() {
    let user = make_user(
        Some("Software engineer passionate about Rust and distributed systems"),
        Some("https://myblog.dev"),
    );
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    assert_eq!(profile.product_description.provenance, Provenance::Bio);
    assert_eq!(profile.product_url.provenance, Provenance::ProfileUrl);
    // Industry topics and brand voice need LLM — should be low confidence.
    assert_eq!(profile.industry_topics.confidence, Confidence::Low);
    assert_eq!(profile.brand_voice.confidence, Confidence::Low);
}

#[test]
fn low_confidence_when_minimal_data() {
    let base = compute_base_confidence(0, 3);
    assert_eq!(base, Confidence::Low);

    let base = compute_base_confidence(5, 0);
    assert_eq!(base, Confidence::Medium);

    let base = compute_base_confidence(25, 10);
    assert_eq!(base, Confidence::High);
}

#[test]
fn profile_serialization_roundtrip() {
    let user = make_user(Some("A bio for testing"), None);
    let input = ProfileInput {
        user,
        tweets: vec![],
    };
    let profile = extract_heuristics(&input);

    let json = serde_json::to_string(&profile).expect("serialize");
    let _parsed: InferredProfile = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn analysis_response_shape() {
    let user = make_user(
        Some("Founder @StartupX | AI-powered analytics for SaaS"),
        Some("https://startupx.io"),
    );
    let input = ProfileInput {
        user,
        tweets: make_tweets(10),
    };
    let profile = extract_heuristics(&input);

    // Verify the JSON shape matches the inference contract.
    let json = serde_json::to_value(&profile).expect("serialize");

    assert!(json["account_type"]["value"].is_string());
    assert!(json["account_type"]["confidence"].is_string());
    assert!(json["account_type"]["provenance"].is_string());
    assert!(json["product_name"]["value"].is_string());
    assert!(json["product_keywords"]["value"].is_array());
    assert!(json["industry_topics"]["value"].is_array());
}
