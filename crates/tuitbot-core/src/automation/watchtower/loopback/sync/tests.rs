use super::*;
use crate::automation::watchtower::loopback::{LoopBackEntry, TuitbotFrontMatter};

fn sample_percentiles(sufficient: bool) -> PerformancePercentiles {
    PerformancePercentiles {
        p50_impressions: 500,
        p90_impressions: 2000,
        has_sufficient_data: sufficient,
    }
}

fn entry(tweet_id: &str, impressions: Option<i64>, synced: Option<&str>) -> LoopBackEntry {
    LoopBackEntry {
        tweet_id: tweet_id.to_string(),
        url: format!("https://x.com/u/status/{tweet_id}"),
        published_at: "2026-03-01T10:00:00Z".to_string(),
        content_type: "tweet".to_string(),
        status: None,
        thread_url: None,
        child_tweet_ids: None,
        impressions,
        likes: impressions.map(|i| i / 10),
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: synced.map(String::from),
    }
}

fn sample_analytics(impressions: i64) -> EntryAnalytics {
    EntryAnalytics {
        impressions,
        likes: impressions / 10,
        retweets: impressions / 20,
        replies: impressions / 50,
        engagement_rate: Some(1.8),
        performance_score: Some(80.0),
        synced_at: "2026-03-01T12:00:00Z".into(),
    }
}

// -- aggregate_thread_metrics ---------------------------------------------------

#[test]
fn aggregate_empty_returns_none() {
    assert!(aggregate_thread_metrics(&[]).is_none());
}

#[test]
fn aggregate_single_tweet() {
    let rows = vec![TweetPerformanceRow {
        tweet_id: "t1".into(),
        likes_received: 10,
        retweets_received: 5,
        replies_received: 3,
        impressions: 1000,
        performance_score: 80.0,
    }];
    let a = aggregate_thread_metrics(&rows).unwrap();
    assert_eq!(a.impressions, 1000);
    assert_eq!(a.likes, 10);
    assert_eq!(a.retweets, 5);
    assert_eq!(a.replies, 3);
    let rate = a.engagement_rate.unwrap();
    assert!((rate - 1.8).abs() < 0.01); // (10+5+3)/1000*100
    assert!((a.performance_score.unwrap() - 80.0).abs() < 0.01);
}

#[test]
fn aggregate_multiple_tweets_weighted_score() {
    let rows = vec![
        TweetPerformanceRow {
            tweet_id: "t1".into(),
            likes_received: 10,
            retweets_received: 2,
            replies_received: 1,
            impressions: 500,
            performance_score: 60.0,
        },
        TweetPerformanceRow {
            tweet_id: "t2".into(),
            likes_received: 20,
            retweets_received: 8,
            replies_received: 4,
            impressions: 1500,
            performance_score: 90.0,
        },
    ];
    let a = aggregate_thread_metrics(&rows).unwrap();
    assert_eq!(a.impressions, 2000);
    assert_eq!(a.likes, 30);
    assert_eq!(a.retweets, 10);
    assert_eq!(a.replies, 5);
    // Engagement rate: (30+10+5)/2000*100 = 2.25
    assert!((a.engagement_rate.unwrap() - 2.25).abs() < 0.01);
    // Weighted avg: (60*500 + 90*1500) / 2000 = 82.5
    assert!((a.performance_score.unwrap() - 82.5).abs() < 0.01);
}

#[test]
fn aggregate_zero_impressions_yields_none_rates() {
    let rows = vec![TweetPerformanceRow {
        tweet_id: "t1".into(),
        likes_received: 0,
        retweets_received: 0,
        replies_received: 0,
        impressions: 0,
        performance_score: 0.0,
    }];
    let a = aggregate_thread_metrics(&rows).unwrap();
    assert_eq!(a.impressions, 0);
    assert!(a.engagement_rate.is_none());
    assert!(a.performance_score.is_none());
}

#[test]
fn aggregate_mixed_zero_and_nonzero_impressions() {
    let rows = vec![
        TweetPerformanceRow {
            tweet_id: "t1".into(),
            likes_received: 5,
            retweets_received: 1,
            replies_received: 0,
            impressions: 0,
            performance_score: 10.0,
        },
        TweetPerformanceRow {
            tweet_id: "t2".into(),
            likes_received: 20,
            retweets_received: 5,
            replies_received: 2,
            impressions: 1000,
            performance_score: 70.0,
        },
    ];
    let a = aggregate_thread_metrics(&rows).unwrap();
    assert_eq!(a.impressions, 1000);
    assert_eq!(a.likes, 25);
    // Only t2 has impressions, so weighted score = 70.0
    assert!((a.performance_score.unwrap() - 70.0).abs() < 0.01);
}

#[test]
fn aggregate_synced_at_is_populated() {
    let rows = vec![TweetPerformanceRow {
        tweet_id: "t1".into(),
        likes_received: 1,
        retweets_received: 0,
        replies_received: 0,
        impressions: 100,
        performance_score: 50.0,
    }];
    let a = aggregate_thread_metrics(&rows).unwrap();
    // synced_at should be a valid ISO 8601 timestamp
    assert!(a.synced_at.contains('T'));
    assert!(a.synced_at.ends_with('Z'));
}

// -- recompute_summaries --------------------------------------------------------

#[test]
fn recompute_no_entries_sets_none() {
    let mut fm = TuitbotFrontMatter::default();
    recompute_summaries(&mut fm, &sample_percentiles(true));
    let key = serde_yaml::Value::String("tuitbot_social_performance".into());
    assert_eq!(fm.other.get(&key).unwrap().as_str().unwrap(), "none");
}

#[test]
fn recompute_high_tier() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![entry("t1", Some(3000), Some("2026-03-01T12:00:00Z"))],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &sample_percentiles(true));
    let key = serde_yaml::Value::String("tuitbot_social_performance".into());
    assert_eq!(fm.other.get(&key).unwrap().as_str().unwrap(), "high");
    let imp_key = serde_yaml::Value::String("tuitbot_best_post_impressions".into());
    assert_eq!(fm.other.get(&imp_key).unwrap().as_i64().unwrap(), 3000);
}

#[test]
fn recompute_medium_tier() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![entry("t1", Some(800), None)],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &sample_percentiles(true));
    let key = serde_yaml::Value::String("tuitbot_social_performance".into());
    assert_eq!(fm.other.get(&key).unwrap().as_str().unwrap(), "medium");
}

#[test]
fn recompute_low_tier() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![entry("t1", Some(100), None)],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &sample_percentiles(true));
    let key = serde_yaml::Value::String("tuitbot_social_performance".into());
    assert_eq!(fm.other.get(&key).unwrap().as_str().unwrap(), "low");
}

#[test]
fn recompute_insufficient_data_forces_none_tier() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![entry("t1", Some(5000), None)],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &sample_percentiles(false));
    let key = serde_yaml::Value::String("tuitbot_social_performance".into());
    assert_eq!(fm.other.get(&key).unwrap().as_str().unwrap(), "none");
}

#[test]
fn recompute_picks_best_by_impressions() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![
            entry("t1", Some(100), None),
            entry("t2", Some(5000), None),
            entry("t3", Some(200), None),
        ],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &sample_percentiles(true));
    let url_key = serde_yaml::Value::String("tuitbot_best_post_url".into());
    let url = fm.other.get(&url_key).unwrap().as_str().unwrap();
    assert!(url.contains("t2"));
}

#[test]
fn recompute_last_synced_at_picks_latest() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![
            entry("t1", Some(100), Some("2026-03-01T10:00:00Z")),
            entry("t2", Some(200), Some("2026-03-02T10:00:00Z")),
        ],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &sample_percentiles(true));
    let key = serde_yaml::Value::String("tuitbot_last_synced_at".into());
    assert_eq!(
        fm.other.get(&key).unwrap().as_str().unwrap(),
        "2026-03-02T10:00:00Z"
    );
}

#[test]
fn recompute_clears_stale_summary_keys() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![],
        other: serde_yaml::Mapping::new(),
    };
    let key = |s: &str| serde_yaml::Value::String(s.into());
    let val = |s: &str| serde_yaml::Value::String(s.into());
    fm.other
        .insert(key("tuitbot_social_performance"), val("high"));
    fm.other
        .insert(key("tuitbot_best_post_impressions"), val("9999"));

    recompute_summaries(&mut fm, &sample_percentiles(true));

    // Old values should be cleared, replaced with "none"
    assert_eq!(
        fm.other
            .get(&key("tuitbot_social_performance"))
            .unwrap()
            .as_str()
            .unwrap(),
        "none"
    );
    // best_post_impressions should be removed since no entries have impressions
    assert!(fm
        .other
        .get(&key("tuitbot_best_post_impressions"))
        .is_none());
}

#[test]
fn recompute_entries_without_impressions_ignored() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![
            entry("t1", None, None),      // no impressions
            entry("t2", Some(800), None), // has impressions
        ],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &sample_percentiles(true));
    let url_key = serde_yaml::Value::String("tuitbot_best_post_url".into());
    let url = fm.other.get(&url_key).unwrap().as_str().unwrap();
    assert!(url.contains("t2"));
}

// -- update_entry_analytics (file I/O) ------------------------------------------

#[test]
fn update_entry_file_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.md");
    let result = update_entry_analytics(
        &path,
        "t1",
        &sample_analytics(100),
        &sample_percentiles(true),
    )
    .unwrap();
    assert_eq!(result, UpdateResult::FileNotFound);
}

#[test]
fn update_entry_not_found_in_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    let content = "---\ntuitbot:\n  - tweet_id: \"t1\"\n    url: \"https://x.com/u/status/t1\"\n    published_at: \"2026-03-01T10:00:00Z\"\n    type: tweet\n---\nBody.\n";
    std::fs::write(&path, content).unwrap();
    let result = update_entry_analytics(
        &path,
        "t999",
        &sample_analytics(100),
        &sample_percentiles(true),
    )
    .unwrap();
    assert_eq!(result, UpdateResult::EntryNotFound);
}

#[test]
fn update_entry_no_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plain.md");
    std::fs::write(&path, "Just a plain note.\n").unwrap();
    let result = update_entry_analytics(
        &path,
        "t1",
        &sample_analytics(100),
        &sample_percentiles(true),
    )
    .unwrap();
    assert_eq!(result, UpdateResult::EntryNotFound);
}

#[test]
fn update_entry_writes_analytics_and_summaries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    let content = "---\ntuitbot:\n  - tweet_id: \"t1\"\n    url: \"https://x.com/u/status/t1\"\n    published_at: \"2026-03-01T10:00:00Z\"\n    type: tweet\n---\nBody text here.\n";
    std::fs::write(&path, content).unwrap();

    let result = update_entry_analytics(
        &path,
        "t1",
        &sample_analytics(3000),
        &sample_percentiles(true),
    )
    .unwrap();
    assert_eq!(result, UpdateResult::Updated);

    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("impressions"));
    assert!(updated.contains("3000"));
    assert!(updated.contains("tuitbot_social_performance"));
    // Body should be preserved
    assert!(updated.contains("Body text here."));
}

#[test]
fn update_entry_preserves_other_entries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("multi.md");
    let content = "---\ntuitbot:\n  - tweet_id: \"t1\"\n    url: \"https://x.com/u/status/t1\"\n    published_at: \"2026-03-01T10:00:00Z\"\n    type: tweet\n  - tweet_id: \"t2\"\n    url: \"https://x.com/u/status/t2\"\n    published_at: \"2026-03-02T10:00:00Z\"\n    type: tweet\n---\nBody.\n";
    std::fs::write(&path, content).unwrap();

    let result = update_entry_analytics(
        &path,
        "t1",
        &sample_analytics(500),
        &sample_percentiles(true),
    )
    .unwrap();
    assert_eq!(result, UpdateResult::Updated);

    let updated = std::fs::read_to_string(&path).unwrap();
    // Both entries should still be present
    assert!(updated.contains("t1"));
    assert!(updated.contains("t2"));
}
