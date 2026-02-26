use serde_json::Value;

use crate::kernel::{engage, read, utils, write};
use crate::tools::test_mocks::{artifacts_dir, MockProvider, MockXApiClient};

#[tokio::test]
async fn conformance_all_kernel_tools_produce_valid_envelope() {
    let mut results: Vec<(&str, bool)> = Vec::new();

    // Read tools (14)
    let tools: Vec<(&str, String)> = vec![
        ("get_tweet", read::get_tweet(&MockProvider, "t1").await),
        (
            "get_user_by_username",
            read::get_user_by_username(&MockProvider, "a").await,
        ),
        (
            "search_tweets",
            read::search_tweets(&MockProvider, "q", 10, None, None).await,
        ),
        (
            "get_user_mentions",
            read::get_user_mentions(&MockProvider, "u1", None, None).await,
        ),
        (
            "get_user_tweets",
            read::get_user_tweets(&MockProvider, "u1", 10, None).await,
        ),
        (
            "get_home_timeline",
            read::get_home_timeline(&MockProvider, "u1", 20, None).await,
        ),
        ("get_me", utils::get_me(&MockProvider).await),
        (
            "get_followers",
            read::get_followers(&MockProvider, "u1", 10, None).await,
        ),
        (
            "get_following",
            read::get_following(&MockProvider, "u1", 10, None).await,
        ),
        (
            "get_user_by_id",
            read::get_user_by_id(&MockProvider, "u1").await,
        ),
        (
            "get_liked_tweets",
            read::get_liked_tweets(&MockProvider, "u1", 10, None).await,
        ),
        (
            "get_bookmarks",
            read::get_bookmarks(&MockProvider, "u1", 10, None).await,
        ),
        (
            "get_users_by_ids",
            read::get_users_by_ids(&MockProvider, &["u1"]).await,
        ),
        (
            "get_tweet_liking_users",
            read::get_tweet_liking_users(&MockProvider, "t1", 10, None).await,
        ),
    ];
    for (name, json) in &tools {
        let parsed: Value = serde_json::from_str(json).unwrap_or_default();
        let valid = parsed["success"].as_bool().unwrap_or(false)
            && parsed.get("meta").is_some()
            && parsed["meta"]["tool_version"] == "1.0";
        results.push((name, valid));
    }

    // Write tools (5)
    let write_tools: Vec<(&str, String)> = vec![
        (
            "post_tweet",
            write::post_tweet(&MockXApiClient, "Hi", None).await,
        ),
        (
            "reply_to_tweet",
            write::reply_to_tweet(&MockXApiClient, "Re", "t1", None).await,
        ),
        (
            "quote_tweet",
            write::quote_tweet(&MockXApiClient, "QT", "t1").await,
        ),
        (
            "delete_tweet",
            write::delete_tweet(&MockXApiClient, "t1").await,
        ),
        (
            "post_thread",
            write::post_thread(&MockXApiClient, &["A".to_string()], None).await,
        ),
    ];
    for (name, json) in &write_tools {
        let parsed: Value = serde_json::from_str(json).unwrap_or_default();
        let valid = parsed["success"].as_bool().unwrap_or(false)
            && parsed.get("meta").is_some()
            && parsed["meta"]["tool_version"] == "1.0";
        results.push((name, valid));
    }

    // Engage tools (8)
    let engage_tools: Vec<(&str, String)> = vec![
        (
            "like_tweet",
            engage::like_tweet(&MockXApiClient, "u1", "t1").await,
        ),
        (
            "unlike_tweet",
            engage::unlike_tweet(&MockXApiClient, "u1", "t1").await,
        ),
        (
            "follow_user",
            engage::follow_user(&MockXApiClient, "u1", "t1").await,
        ),
        (
            "unfollow_user",
            engage::unfollow_user(&MockXApiClient, "u1", "t1").await,
        ),
        (
            "retweet",
            engage::retweet(&MockXApiClient, "u1", "t1").await,
        ),
        (
            "unretweet",
            engage::unretweet(&MockXApiClient, "u1", "t1").await,
        ),
        (
            "bookmark_tweet",
            engage::bookmark_tweet(&MockXApiClient, "u1", "t1").await,
        ),
        (
            "unbookmark_tweet",
            engage::unbookmark_tweet(&MockXApiClient, "u1", "t1").await,
        ),
    ];
    for (name, json) in &engage_tools {
        let parsed: Value = serde_json::from_str(json).unwrap_or_default();
        let valid = parsed["success"].as_bool().unwrap_or(false)
            && parsed.get("meta").is_some()
            && parsed["meta"]["tool_version"] == "1.0";
        results.push((name, valid));
    }

    let total = results.len();
    let passed = results.iter().filter(|(_, v)| *v).count();
    let rate = passed as f64 / total as f64 * 100.0;

    // Write conformance report
    let mut md = String::from("# Session 09 — Kernel Conformance Results\n\n");
    md.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));
    md.push_str(&format!(
        "**Conformance rate:** {passed}/{total} ({rate:.1}%)\n\n"
    ));
    md.push_str("| Tool | Conformant |\n");
    md.push_str("|------|------------|\n");
    for (name, valid) in &results {
        md.push_str(&format!(
            "| {} | {} |\n",
            name,
            if *valid { "PASS" } else { "FAIL" }
        ));
    }

    let dir = artifacts_dir();
    std::fs::create_dir_all(&dir).expect("create artifacts dir");
    std::fs::write(dir.join("session-09-conformance-results.md"), &md)
        .expect("write conformance results");

    assert_eq!(
        rate,
        100.0,
        "Conformance rate {rate:.1}% < 100%. Failures: {:?}",
        results
            .iter()
            .filter(|(_, v)| !v)
            .map(|(n, _)| *n)
            .collect::<Vec<_>>()
    );
}
