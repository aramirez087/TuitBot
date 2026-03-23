use super::*;
use std::fs;

fn sample_entry() -> LoopBackEntry {
    LoopBackEntry {
        tweet_id: "1234567890".to_string(),
        url: "https://x.com/user/status/1234567890".to_string(),
        published_at: "2026-02-28T14:30:00Z".to_string(),
        content_type: "tweet".to_string(),
        status: None,
        thread_url: None,
        child_tweet_ids: None,
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    }
}

#[test]
fn split_no_front_matter() {
    let content = "Just a plain note.\n";
    let (yaml, body) = split_front_matter(content);
    assert!(yaml.is_none());
    assert_eq!(body, content);
}

#[test]
fn split_with_front_matter() {
    let content = "---\ntitle: Hello\n---\nBody text here.\n";
    let (yaml, body) = split_front_matter(content);
    assert_eq!(yaml.unwrap(), "title: Hello");
    assert_eq!(body, "Body text here.\n");
}

#[test]
fn split_no_closing_delimiter() {
    let content = "---\ntitle: Hello\nNo closing.\n";
    let (yaml, body) = split_front_matter(content);
    assert!(yaml.is_none());
    assert_eq!(body, content);
}

#[test]
fn parse_tuitbot_entries() {
    let content = "---\ntuitbot:\n  - tweet_id: \"123\"\n    url: \"https://x.com/u/status/123\"\n    published_at: \"2026-01-01T00:00:00Z\"\n    type: tweet\n---\nBody.\n";
    let entries = parse_tuitbot_metadata(content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tweet_id, "123");
}

#[test]
fn parse_no_tuitbot_key() {
    let content = "---\ntitle: Hello\n---\nBody.\n";
    let entries = parse_tuitbot_metadata(content);
    assert!(entries.is_empty());
}

#[test]
fn loopback_write_new_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "This is my note.\n").unwrap();

    let entry = sample_entry();
    let modified = write_metadata_to_file(&path, &entry).unwrap();
    assert!(modified);

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("---\n"));
    assert!(content.contains("tweet_id"));
    assert!(content.contains("1234567890"));
    assert!(content.contains("This is my note."));
}

#[test]
fn loopback_write_existing_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "---\ntitle: My Note\n---\nBody here.\n").unwrap();

    let entry = sample_entry();
    let modified = write_metadata_to_file(&path, &entry).unwrap();
    assert!(modified);

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("title"));
    assert!(content.contains("My Note"));
    assert!(content.contains("tweet_id"));
    assert!(content.contains("Body here."));
}

#[test]
fn loopback_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "My note.\n").unwrap();

    let entry = sample_entry();
    let first = write_metadata_to_file(&path, &entry).unwrap();
    assert!(first);

    let second = write_metadata_to_file(&path, &entry).unwrap();
    assert!(!second);

    let content = fs::read_to_string(&path).unwrap();
    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
}

#[test]
fn loopback_multiple_tweets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "My note.\n").unwrap();

    let entry_a = sample_entry();
    write_metadata_to_file(&path, &entry_a).unwrap();

    let entry_b = LoopBackEntry {
        tweet_id: "9876543210".to_string(),
        url: "https://x.com/user/status/9876543210".to_string(),
        published_at: "2026-03-01T10:00:00Z".to_string(),
        content_type: "thread".to_string(),
        status: None,
        thread_url: None,
        child_tweet_ids: None,
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };
    write_metadata_to_file(&path, &entry_b).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].tweet_id, "1234567890");
    assert_eq!(entries[1].tweet_id, "9876543210");
}

#[test]
fn thread_entry_serializes_child_tweet_ids() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "Thread note.\n").unwrap();

    let entry = LoopBackEntry {
        tweet_id: "root_001".to_string(),
        url: "https://x.com/i/status/root_001".to_string(),
        published_at: "2026-03-22T10:00:00Z".to_string(),
        content_type: "thread".to_string(),
        status: Some("posted".to_string()),
        thread_url: Some("https://x.com/i/status/root_001".to_string()),
        child_tweet_ids: Some(vec!["child_002".to_string(), "child_003".to_string()]),
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };
    let modified = write_metadata_to_file(&path, &entry).unwrap();
    assert!(modified);

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("child_tweet_ids"));
    assert!(content.contains("child_002"));
    assert!(content.contains("child_003"));
    assert!(content.contains("thread_url"));
}

#[test]
fn thread_entry_without_child_ids_omits_field() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "Note.\n").unwrap();

    let entry = LoopBackEntry {
        tweet_id: "t_100".to_string(),
        url: "https://x.com/i/status/t_100".to_string(),
        published_at: "2026-03-22T10:00:00Z".to_string(),
        content_type: "thread".to_string(),
        status: Some("posted".to_string()),
        thread_url: None,
        child_tweet_ids: None,
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };
    let modified = write_metadata_to_file(&path, &entry).unwrap();
    assert!(modified);

    let content = fs::read_to_string(&path).unwrap();
    assert!(!content.contains("child_tweet_ids"));
}

#[test]
fn thread_entry_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "Roundtrip note.\n").unwrap();

    let entry = LoopBackEntry {
        tweet_id: "rt_001".to_string(),
        url: "https://x.com/i/status/rt_001".to_string(),
        published_at: "2026-03-22T12:00:00Z".to_string(),
        content_type: "thread".to_string(),
        status: Some("posted".to_string()),
        thread_url: Some("https://x.com/i/status/rt_001".to_string()),
        child_tweet_ids: Some(vec!["rt_002".to_string()]),
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };
    write_metadata_to_file(&path, &entry).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tweet_id, "rt_001");
    assert_eq!(entries[0].content_type, "thread");
    assert_eq!(
        entries[0].thread_url.as_deref(),
        Some("https://x.com/i/status/rt_001")
    );
    assert_eq!(entries[0].child_tweet_ids, Some(vec!["rt_002".to_string()]));
}

#[test]
fn thread_entry_idempotent_by_root_tweet_id() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "Idem note.\n").unwrap();

    let entry = LoopBackEntry {
        tweet_id: "idem_root".to_string(),
        url: "https://x.com/i/status/idem_root".to_string(),
        published_at: "2026-03-22T12:00:00Z".to_string(),
        content_type: "thread".to_string(),
        status: Some("posted".to_string()),
        thread_url: Some("https://x.com/i/status/idem_root".to_string()),
        child_tweet_ids: Some(vec!["idem_child".to_string()]),
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };

    let first = write_metadata_to_file(&path, &entry).unwrap();
    assert!(first);
    let second = write_metadata_to_file(&path, &entry).unwrap();
    assert!(!second);

    let content = fs::read_to_string(&path).unwrap();
    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
}

// --- Analytics fields tests ---

#[test]
fn loopback_entry_without_analytics_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "Analytics test.\n").unwrap();

    let entry = sample_entry();
    write_metadata_to_file(&path, &entry).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    // Analytics fields should be omitted when None
    assert!(!content.contains("impressions"));
    assert!(!content.contains("engagement_rate"));
    assert!(!content.contains("performance_score"));
    assert!(!content.contains("synced_at"));

    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].impressions.is_none());
    assert!(entries[0].engagement_rate.is_none());
}

#[test]
fn loopback_entry_with_analytics_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "Analytics roundtrip.\n").unwrap();

    let entry = LoopBackEntry {
        tweet_id: "analytics_001".to_string(),
        url: "https://x.com/i/status/analytics_001".to_string(),
        published_at: "2026-03-22T14:00:00Z".to_string(),
        content_type: "tweet".to_string(),
        status: Some("posted".to_string()),
        thread_url: None,
        child_tweet_ids: None,
        impressions: Some(4820),
        likes: Some(47),
        retweets: Some(12),
        replies: Some(8),
        engagement_rate: Some(1.39),
        performance_score: Some(72.5),
        synced_at: Some("2026-03-23T02:00:00Z".to_string()),
    };
    write_metadata_to_file(&path, &entry).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("impressions: 4820"));
    assert!(content.contains("likes: 47"));
    assert!(content.contains("synced_at:"));

    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].impressions, Some(4820));
    assert_eq!(entries[0].likes, Some(47));
    assert_eq!(entries[0].retweets, Some(12));
    assert_eq!(entries[0].replies, Some(8));
    assert_eq!(entries[0].performance_score, Some(72.5));
}

// --- Sync tests ---

use sync::{
    aggregate_thread_metrics, recompute_summaries, update_entry_analytics, EntryAnalytics,
    PerformancePercentiles, TweetPerformanceRow, UpdateResult,
};

fn default_percentiles() -> PerformancePercentiles {
    PerformancePercentiles {
        p50_impressions: 1000,
        p90_impressions: 10000,
        has_sufficient_data: true,
    }
}

#[test]
fn update_analytics_single_tweet() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");

    // Create a note with an existing entry (no analytics)
    let entry = sample_entry();
    fs::write(&path, "My note.\n").unwrap();
    write_metadata_to_file(&path, &entry).unwrap();

    let analytics = EntryAnalytics {
        impressions: 5000,
        likes: 50,
        retweets: 15,
        replies: 10,
        engagement_rate: Some(1.5),
        performance_score: Some(75.0),
        synced_at: "2026-03-23T02:00:00Z".to_string(),
    };

    let result =
        update_entry_analytics(&path, "1234567890", &analytics, &default_percentiles()).unwrap();
    assert_eq!(result, UpdateResult::Updated);

    // Verify the analytics were written
    let content = fs::read_to_string(&path).unwrap();
    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].impressions, Some(5000));
    assert_eq!(entries[0].likes, Some(50));
    assert_eq!(entries[0].retweets, Some(15));
    assert_eq!(entries[0].replies, Some(10));
    assert_eq!(entries[0].performance_score, Some(75.0));
    assert_eq!(
        entries[0].synced_at.as_deref(),
        Some("2026-03-23T02:00:00Z")
    );
}

#[test]
fn update_analytics_preserves_other_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");

    // Write a note with custom frontmatter + tuitbot entry
    fs::write(
        &path,
        "---\ntitle: My Great Note\ntags:\n  - rust\n  - programming\n---\nBody content.\n",
    )
    .unwrap();
    let entry = sample_entry();
    write_metadata_to_file(&path, &entry).unwrap();

    let analytics = EntryAnalytics {
        impressions: 3000,
        likes: 30,
        retweets: 8,
        replies: 5,
        engagement_rate: Some(1.43),
        performance_score: Some(65.0),
        synced_at: "2026-03-23T02:00:00Z".to_string(),
    };

    let result =
        update_entry_analytics(&path, "1234567890", &analytics, &default_percentiles()).unwrap();
    assert_eq!(result, UpdateResult::Updated);

    let content = fs::read_to_string(&path).unwrap();
    // User's frontmatter preserved
    assert!(content.contains("title:"));
    assert!(content.contains("My Great Note"));
    assert!(content.contains("Body content."));
    // Analytics written
    assert!(content.contains("impressions: 3000"));
}

#[test]
fn update_analytics_entry_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "---\ntitle: Empty\n---\nBody.\n").unwrap();

    let analytics = EntryAnalytics {
        impressions: 1000,
        likes: 10,
        retweets: 2,
        replies: 1,
        engagement_rate: Some(1.3),
        performance_score: Some(50.0),
        synced_at: "2026-03-23T02:00:00Z".to_string(),
    };

    let result =
        update_entry_analytics(&path, "nonexistent", &analytics, &default_percentiles()).unwrap();
    assert_eq!(result, UpdateResult::EntryNotFound);
}

#[test]
fn update_analytics_file_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.md");

    let analytics = EntryAnalytics {
        impressions: 1000,
        likes: 10,
        retweets: 2,
        replies: 1,
        engagement_rate: None,
        performance_score: None,
        synced_at: "2026-03-23T02:00:00Z".to_string(),
    };

    let result =
        update_entry_analytics(&path, "tweet_1", &analytics, &default_percentiles()).unwrap();
    assert_eq!(result, UpdateResult::FileNotFound);
}

#[test]
fn update_analytics_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "My note.\n").unwrap();

    let entry = sample_entry();
    write_metadata_to_file(&path, &entry).unwrap();

    let analytics = EntryAnalytics {
        impressions: 5000,
        likes: 50,
        retweets: 15,
        replies: 10,
        engagement_rate: Some(1.5),
        performance_score: Some(75.0),
        synced_at: "2026-03-23T02:00:00Z".to_string(),
    };

    // First sync
    update_entry_analytics(&path, "1234567890", &analytics, &default_percentiles()).unwrap();

    // Second sync — same data
    let result =
        update_entry_analytics(&path, "1234567890", &analytics, &default_percentiles()).unwrap();
    assert_eq!(result, UpdateResult::Updated);

    // Still exactly one entry
    let content = fs::read_to_string(&path).unwrap();
    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].impressions, Some(5000));
}

#[test]
fn recompute_summaries_best_post() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![
            LoopBackEntry {
                tweet_id: "a".to_string(),
                url: "https://x.com/i/status/a".to_string(),
                published_at: "2026-03-20T10:00:00Z".to_string(),
                content_type: "tweet".to_string(),
                status: None,
                thread_url: None,
                child_tweet_ids: None,
                impressions: Some(1200),
                likes: Some(15),
                retweets: Some(3),
                replies: Some(2),
                engagement_rate: Some(1.67),
                performance_score: Some(45.2),
                synced_at: Some("2026-03-23T02:00:00Z".to_string()),
            },
            LoopBackEntry {
                tweet_id: "b".to_string(),
                url: "https://x.com/i/status/b".to_string(),
                published_at: "2026-03-22T16:00:00Z".to_string(),
                content_type: "tweet".to_string(),
                status: None,
                thread_url: None,
                child_tweet_ids: None,
                impressions: Some(23100),
                likes: Some(580),
                retweets: Some(145),
                replies: Some(67),
                engagement_rate: Some(3.43),
                performance_score: Some(94.7),
                synced_at: Some("2026-03-23T02:00:00Z".to_string()),
            },
        ],
        other: serde_yaml::Mapping::new(),
    };

    recompute_summaries(&mut fm, &default_percentiles());

    let perf = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_social_performance".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(perf, "high");

    let best_impressions = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_best_post_impressions".to_string(),
        ))
        .and_then(|v| v.as_i64())
        .unwrap();
    assert_eq!(best_impressions, 23100);

    let best_url = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_best_post_url".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(best_url, "https://x.com/i/status/b");
}

#[test]
fn recompute_summaries_tie_break_by_date() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![
            LoopBackEntry {
                tweet_id: "old".to_string(),
                url: "https://x.com/i/status/old".to_string(),
                published_at: "2026-03-20T10:00:00Z".to_string(),
                content_type: "tweet".to_string(),
                status: None,
                thread_url: None,
                child_tweet_ids: None,
                impressions: Some(5000),
                likes: None,
                retweets: None,
                replies: None,
                engagement_rate: None,
                performance_score: None,
                synced_at: Some("2026-03-23T02:00:00Z".to_string()),
            },
            LoopBackEntry {
                tweet_id: "new".to_string(),
                url: "https://x.com/i/status/new".to_string(),
                published_at: "2026-03-22T16:00:00Z".to_string(),
                content_type: "tweet".to_string(),
                status: None,
                thread_url: None,
                child_tweet_ids: None,
                impressions: Some(5000),
                likes: None,
                retweets: None,
                replies: None,
                engagement_rate: None,
                performance_score: None,
                synced_at: Some("2026-03-23T02:00:00Z".to_string()),
            },
        ],
        other: serde_yaml::Mapping::new(),
    };

    recompute_summaries(&mut fm, &default_percentiles());

    let best_url = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_best_post_url".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(best_url, "https://x.com/i/status/new");
}

#[test]
fn recompute_summaries_no_impressions() {
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![sample_entry()],
        other: serde_yaml::Mapping::new(),
    };

    recompute_summaries(&mut fm, &default_percentiles());

    let perf = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_social_performance".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(perf, "none");

    // Other summary keys should be absent
    assert!(fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_best_post_impressions".to_string()
        ))
        .is_none());
}

#[test]
fn recompute_summaries_performance_tiers() {
    let p = PerformancePercentiles {
        p50_impressions: 1000,
        p90_impressions: 10000,
        has_sufficient_data: true,
    };

    // Low: below p50
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![LoopBackEntry {
            impressions: Some(500),
            ..sample_entry()
        }],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &p);
    let tier = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_social_performance".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(tier, "low");

    // Medium: at p50 but below p90
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![LoopBackEntry {
            impressions: Some(1000),
            ..sample_entry()
        }],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &p);
    let tier = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_social_performance".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(tier, "medium");

    // High: at or above p90
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![LoopBackEntry {
            impressions: Some(10000),
            ..sample_entry()
        }],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &p);
    let tier = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_social_performance".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(tier, "high");

    // None: insufficient data
    let p_insufficient = PerformancePercentiles {
        p50_impressions: 1000,
        p90_impressions: 10000,
        has_sufficient_data: false,
    };
    let mut fm = TuitbotFrontMatter {
        tuitbot: vec![LoopBackEntry {
            impressions: Some(50000),
            ..sample_entry()
        }],
        other: serde_yaml::Mapping::new(),
    };
    recompute_summaries(&mut fm, &p_insufficient);
    let tier = fm
        .other
        .get(serde_yaml::Value::String(
            "tuitbot_social_performance".to_string(),
        ))
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(tier, "none");
}

#[test]
fn aggregate_thread_metrics_empty() {
    let result = aggregate_thread_metrics(&[]);
    assert!(result.is_none());
}

#[test]
fn aggregate_thread_metrics_weighted_score() {
    // Match the example from forge-thread-contract.md
    let performances = vec![
        TweetPerformanceRow {
            tweet_id: "root".to_string(),
            likes_received: 84,
            retweets_received: 21,
            replies_received: 15,
            impressions: 5200,
            performance_score: 78.2,
        },
        TweetPerformanceRow {
            tweet_id: "child_1".to_string(),
            likes_received: 45,
            retweets_received: 10,
            replies_received: 8,
            impressions: 3100,
            performance_score: 65.0,
        },
        TweetPerformanceRow {
            tweet_id: "child_2".to_string(),
            likes_received: 38,
            retweets_received: 7,
            replies_received: 12,
            impressions: 2800,
            performance_score: 60.1,
        },
        TweetPerformanceRow {
            tweet_id: "child_3".to_string(),
            likes_received: 22,
            retweets_received: 4,
            replies_received: 5,
            impressions: 1900,
            performance_score: 48.5,
        },
    ];

    let analytics = aggregate_thread_metrics(&performances).unwrap();
    assert_eq!(analytics.impressions, 13000);
    assert_eq!(analytics.likes, 189);
    assert_eq!(analytics.retweets, 42);
    assert_eq!(analytics.replies, 40);

    // engagement_rate = (189 + 42 + 40) / 13000 * 100 ≈ 2.08
    let er = analytics.engagement_rate.unwrap();
    assert!((er - 2.08).abs() < 0.01, "engagement_rate was {er}");

    // performance_score = impression-weighted average
    // (78.2*5200 + 65.0*3100 + 60.1*2800 + 48.5*1900) / 13000 ≈ 66.81
    let ps = analytics.performance_score.unwrap();
    assert!((ps - 66.81).abs() < 0.1, "performance_score was {ps}");
}

#[test]
fn aggregate_thread_metrics_partial_children() {
    // Only root has metrics
    let performances = vec![TweetPerformanceRow {
        tweet_id: "root".to_string(),
        likes_received: 84,
        retweets_received: 21,
        replies_received: 15,
        impressions: 5200,
        performance_score: 78.2,
    }];

    let analytics = aggregate_thread_metrics(&performances).unwrap();
    assert_eq!(analytics.impressions, 5200);
    assert_eq!(analytics.likes, 84);
    assert_eq!(analytics.performance_score, Some(78.2));
}

#[test]
fn update_analytics_thread_aggregation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "Thread note.\n").unwrap();

    // Write a thread entry
    let entry = LoopBackEntry {
        tweet_id: "thread_root".to_string(),
        url: "https://x.com/i/status/thread_root".to_string(),
        published_at: "2026-03-22T10:00:00Z".to_string(),
        content_type: "thread".to_string(),
        status: Some("posted".to_string()),
        thread_url: Some("https://x.com/i/status/thread_root".to_string()),
        child_tweet_ids: Some(vec!["child_1".to_string(), "child_2".to_string()]),
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };
    write_metadata_to_file(&path, &entry).unwrap();

    // Update with aggregated thread analytics
    let analytics = EntryAnalytics {
        impressions: 13000,
        likes: 189,
        retweets: 42,
        replies: 40,
        engagement_rate: Some(2.08),
        performance_score: Some(66.2),
        synced_at: "2026-03-23T02:00:00Z".to_string(),
    };

    let result =
        update_entry_analytics(&path, "thread_root", &analytics, &default_percentiles()).unwrap();
    assert_eq!(result, UpdateResult::Updated);

    let content = fs::read_to_string(&path).unwrap();
    let entries = parse_tuitbot_metadata(&content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].impressions, Some(13000));
    assert_eq!(entries[0].likes, Some(189));
    // child_tweet_ids preserved
    assert_eq!(
        entries[0].child_tweet_ids,
        Some(vec!["child_1".to_string(), "child_2".to_string()])
    );
    // Summary should be set
    assert!(content.contains("tuitbot_social_performance"));
    assert!(content.contains("tuitbot_best_post_impressions"));
}

#[test]
fn aggregate_thread_zero_impressions() {
    let performances = vec![
        TweetPerformanceRow {
            tweet_id: "root".to_string(),
            likes_received: 0,
            retweets_received: 0,
            replies_received: 0,
            impressions: 0,
            performance_score: 0.0,
        },
        TweetPerformanceRow {
            tweet_id: "child_1".to_string(),
            likes_received: 0,
            retweets_received: 0,
            replies_received: 0,
            impressions: 0,
            performance_score: 0.0,
        },
    ];

    let analytics = aggregate_thread_metrics(&performances).unwrap();
    assert_eq!(analytics.impressions, 0);
    assert!(analytics.engagement_rate.is_none());
    assert!(analytics.performance_score.is_none());
}

#[test]
fn aggregate_thread_mixed_zero_and_positive_impressions() {
    let performances = vec![
        TweetPerformanceRow {
            tweet_id: "root".to_string(),
            likes_received: 10,
            retweets_received: 3,
            replies_received: 2,
            impressions: 500,
            performance_score: 80.0,
        },
        TweetPerformanceRow {
            tweet_id: "child_1".to_string(),
            likes_received: 0,
            retweets_received: 0,
            replies_received: 0,
            impressions: 0,
            performance_score: 0.0,
        },
    ];

    let analytics = aggregate_thread_metrics(&performances).unwrap();
    assert_eq!(analytics.impressions, 500);
    assert_eq!(analytics.likes, 10);
    // engagement_rate = (10+3+2)/500*100 = 3.0
    let er = analytics.engagement_rate.unwrap();
    assert!((er - 3.0).abs() < 0.01, "engagement_rate was {er}");
    // Only root has impressions, so weighted avg = root's score
    let ps = analytics.performance_score.unwrap();
    assert!((ps - 80.0).abs() < 0.01, "performance_score was {ps}");
}

#[test]
fn aggregate_thread_single_child_only() {
    let performances = vec![TweetPerformanceRow {
        tweet_id: "child_1".to_string(),
        likes_received: 5,
        retweets_received: 1,
        replies_received: 2,
        impressions: 200,
        performance_score: 45.0,
    }];

    let analytics = aggregate_thread_metrics(&performances).unwrap();
    assert_eq!(analytics.impressions, 200);
    assert_eq!(analytics.likes, 5);
    assert!(analytics.engagement_rate.is_some());
    assert_eq!(analytics.performance_score, Some(45.0));
}

#[test]
fn update_analytics_no_frontmatter_returns_entry_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plain.md");
    fs::write(&path, "Just plain text, no frontmatter.\n").unwrap();

    let analytics = EntryAnalytics {
        impressions: 1000,
        likes: 10,
        retweets: 5,
        replies: 3,
        engagement_rate: Some(1.8),
        performance_score: Some(55.0),
        synced_at: "2026-03-22T12:00:00Z".to_string(),
    };

    let result =
        update_entry_analytics(&path, "tweet_999", &analytics, &default_percentiles()).unwrap();
    assert_eq!(result, UpdateResult::EntryNotFound);
}
