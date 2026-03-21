//! Target loop integration and edge case tests.

use super::*;

// --- AuthExpired-aware mock ---

struct MockAuthExpiredUserManager {
    lookup_count: AtomicU32,
    error: LoopError,
}

impl MockAuthExpiredUserManager {
    fn auth_expired() -> Self {
        Self {
            lookup_count: AtomicU32::new(0),
            error: LoopError::AuthExpired,
        }
    }
}

#[async_trait::async_trait]
impl TargetUserManager for MockAuthExpiredUserManager {
    async fn lookup_user(&self, _username: &str) -> Result<(String, String), LoopError> {
        self.lookup_count.fetch_add(1, Ordering::SeqCst);
        Err(match &self.error {
            LoopError::AuthExpired => LoopError::AuthExpired,
            LoopError::Other(msg) => LoopError::Other(msg.clone()),
            _ => unreachable!(),
        })
    }
}

/// A user manager where the first lookup fails and the second succeeds.
struct MockPartialFailUserManager {
    call_count: AtomicU32,
}

#[async_trait::async_trait]
impl TargetUserManager for MockPartialFailUserManager {
    async fn lookup_user(&self, username: &str) -> Result<(String, String), LoopError> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        if n == 0 {
            Err(LoopError::Other("transient failure".to_string()))
        } else {
            Ok((format!("uid_{username}"), username.to_string()))
        }
    }
}

#[tokio::test]
async fn auth_expired_stops_iteration() {
    let user_mgr = Arc::new(MockAuthExpiredUserManager::auth_expired());
    let storage = Arc::new(MockTargetStorage::new());
    let poster = Arc::new(MockPoster::new());

    let mut config = default_config();
    config.accounts = vec![
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
    ];

    let target_loop = TargetLoop::new(
        Arc::new(MockFetcher { tweets: vec![] }),
        user_mgr.clone(),
        Arc::new(MockGenerator {
            reply: "Great!".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        storage,
        poster,
        config,
    );

    let result = target_loop.run_iteration().await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LoopError::AuthExpired));
    // Only one lookup should have been attempted — the loop exits early.
    assert_eq!(user_mgr.lookup_count.load(Ordering::SeqCst), 1);
}

#[test]
fn target_loop_config_debug() {
    let config = TargetLoopConfig {
        accounts: vec!["alice".to_string(), "bob".to_string()],
        max_target_replies_per_day: 5,
        dry_run: false,
    };
    let debug = format!("{config:?}");
    assert!(debug.contains("alice"));
    assert!(debug.contains("5"));
}

#[test]
fn target_loop_config_clone() {
    let config = default_config();
    let clone = config.clone();
    assert_eq!(clone.accounts, config.accounts);
    assert_eq!(
        clone.max_target_replies_per_day,
        config.max_target_replies_per_day
    );
    assert_eq!(clone.dry_run, config.dry_run);
}

#[test]
fn target_result_debug_all_variants() {
    let r = TargetResult::Replied {
        tweet_id: "t1".to_string(),
        account: "alice".to_string(),
        reply_text: "hi".to_string(),
    };
    assert!(format!("{r:?}").contains("Replied"));

    let r = TargetResult::Skipped {
        tweet_id: "t2".to_string(),
        reason: "dup".to_string(),
    };
    assert!(format!("{r:?}").contains("Skipped"));

    let r = TargetResult::Failed {
        tweet_id: "t3".to_string(),
        error: "oops".to_string(),
    };
    assert!(format!("{r:?}").contains("Failed"));
}

#[test]
fn truncate_exact_boundary() {
    assert_eq!(truncate("hello", 5), "hello");
}

#[test]
fn truncate_empty() {
    assert_eq!(truncate("", 10), "");
}

#[test]
fn truncate_zero() {
    assert_eq!(truncate("hello", 0), "...");
}

#[test]
fn truncate_one_char() {
    assert_eq!(truncate("hello", 1), "h...");
}

#[test]
fn target_loop_config_default_values() {
    let config = TargetLoopConfig {
        accounts: vec![],
        max_target_replies_per_day: 0,
        dry_run: true,
    };
    assert!(config.accounts.is_empty());
    assert_eq!(config.max_target_replies_per_day, 0);
    assert!(config.dry_run);
}

#[tokio::test]
async fn replies_only_to_first_tweet_per_account() {
    // TargetLoop replies to only one tweet per account per iteration.
    let tweets = vec![
        test_tweet("tw1", "alice"),
        test_tweet("tw2", "alice"),
        test_tweet("tw3", "alice"),
    ];
    let storage = Arc::new(MockTargetStorage::new());
    let (target_loop, poster) = build_loop(tweets, default_config(), storage);

    let results = target_loop.run_iteration().await.expect("iteration");
    // Only 1 reply per account — first tweet gets replied, loop breaks
    let replied = results
        .iter()
        .filter(|r| matches!(r, TargetResult::Replied { .. }))
        .count();
    assert_eq!(replied, 1);
    assert_eq!(poster.sent_count(), 1);
}

#[tokio::test]
async fn skips_when_safety_cant_reply() {
    let tweets = vec![test_tweet("tw1", "alice")];
    let storage = Arc::new(MockTargetStorage::new());
    let poster = Arc::new(MockPoster::new());
    let user_mgr = Arc::new(MockUserManager {
        users: vec![(
            "alice".to_string(),
            "uid_alice".to_string(),
            "alice".to_string(),
        )],
    });

    let target_loop = TargetLoop::new(
        Arc::new(MockFetcher { tweets }),
        user_mgr,
        Arc::new(MockGenerator {
            reply: "Great!".to_string(),
        }),
        Arc::new(MockSafety::new(false)), // can_reply = false
        storage,
        poster.clone(),
        default_config(),
    );

    let results = target_loop.run_iteration().await.expect("iteration");
    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0],
        TargetResult::Skipped { reason, .. } if reason == "rate limited"
    ));
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn skips_when_already_replied() {
    let tweets = vec![test_tweet("tw1", "alice")];
    let storage = Arc::new(MockTargetStorage::new());
    let poster = Arc::new(MockPoster::new());
    let safety = Arc::new(MockSafety::new(true));
    // Pre-mark tw1 as replied
    safety.record_reply("tw1", "already replied").await.unwrap();

    let user_mgr = Arc::new(MockUserManager {
        users: vec![(
            "alice".to_string(),
            "uid_alice".to_string(),
            "alice".to_string(),
        )],
    });

    let target_loop = TargetLoop::new(
        Arc::new(MockFetcher { tweets }),
        user_mgr,
        Arc::new(MockGenerator {
            reply: "Great!".to_string(),
        }),
        safety,
        storage,
        poster.clone(),
        default_config(),
    );

    let results = target_loop.run_iteration().await.expect("iteration");
    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0],
        TargetResult::Skipped { reason, .. } if reason == "already replied"
    ));
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn no_tweets_returns_empty_results() {
    let storage = Arc::new(MockTargetStorage::new());
    let (target_loop, poster) = build_loop(vec![], default_config(), storage);

    let results = target_loop.run_iteration().await.expect("iteration");
    assert!(results.is_empty());
    assert_eq!(poster.sent_count(), 0);
}

#[tokio::test]
async fn non_auth_error_continues_iteration() {
    let user_mgr = Arc::new(MockPartialFailUserManager {
        call_count: AtomicU32::new(0),
    });
    let storage = Arc::new(MockTargetStorage::new());
    let poster = Arc::new(MockPoster::new());

    let mut config = default_config();
    config.accounts = vec!["alice".to_string(), "bob".to_string()];

    let target_loop = TargetLoop::new(
        Arc::new(MockFetcher {
            tweets: vec![test_tweet("tw1", "bob")],
        }),
        user_mgr.clone(),
        Arc::new(MockGenerator {
            reply: "Nice!".to_string(),
        }),
        Arc::new(MockSafety::new(true)),
        storage,
        poster.clone(),
        config,
    );

    let results = target_loop.run_iteration().await.expect("should succeed");
    // First account fails with Other, second succeeds — both should be attempted.
    assert_eq!(user_mgr.call_count.load(Ordering::SeqCst), 2);
    // Second account produces a reply.
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], TargetResult::Replied { .. }));
    assert_eq!(poster.sent_count(), 1);
}
