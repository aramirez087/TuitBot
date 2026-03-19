use super::*;
use crate::automation::analytics_loop::AnalyticsStorage;
use crate::automation::loop_helpers::{ContentStorage, LoopStorage, LoopTweet, TopicScorer};
use crate::automation::posting_queue::PostAction;
use crate::automation::target_loop::TargetStorage;
use crate::storage::init_test_db;
use tokio::sync::mpsc;

fn test_post_channel() -> (mpsc::Sender<PostAction>, mpsc::Receiver<PostAction>) {
    tokio::sync::mpsc::channel(16)
}

fn sample_loop_tweet() -> LoopTweet {
    LoopTweet {
        id: "t123".to_string(),
        text: "hello".to_string(),
        author_id: "a1".to_string(),
        author_username: "user".to_string(),
        author_followers: 100,
        created_at: chrono::Utc::now().to_rfc3339(),
        likes: 5,
        retweets: 1,
        replies: 0,
    }
}

// ============================================================================
// StorageAdapter (LoopStorage)
// ============================================================================

#[tokio::test]
async fn storage_adapter_get_set_cursor() {
    let pool = init_test_db().await.expect("init db");
    let adapter = StorageAdapter::new(pool);

    let val = adapter.get_cursor("mentions_since").await.unwrap();
    assert!(val.is_none());

    adapter.set_cursor("mentions_since", "12345").await.unwrap();
    let val = adapter.get_cursor("mentions_since").await.unwrap();
    assert_eq!(val.as_deref(), Some("12345"));
}

#[tokio::test]
async fn storage_adapter_tweet_exists() {
    let pool = init_test_db().await.expect("init db");
    let adapter = StorageAdapter::new(pool);

    let tweet = sample_loop_tweet();
    adapter
        .store_discovered_tweet(&tweet, 75.0, "rust")
        .await
        .unwrap();

    let exists = adapter.tweet_exists("t123").await.unwrap();
    assert!(exists);

    let missing = adapter.tweet_exists("nonexistent").await.unwrap();
    assert!(!missing);
}

#[tokio::test]
async fn storage_adapter_store_discovered_tweet() {
    let pool = init_test_db().await.expect("init db");
    let adapter = StorageAdapter::new(pool);

    let tweet = sample_loop_tweet();
    let result = adapter.store_discovered_tweet(&tweet, 80.0, "async").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn storage_adapter_log_action() {
    let pool = init_test_db().await.expect("init db");
    let adapter = StorageAdapter::new(pool);

    let result = adapter
        .log_action("discovery", "success", "Found 3 tweets")
        .await;
    assert!(result.is_ok());
}

// ============================================================================
// ContentStorageAdapter (ContentStorage)
// ============================================================================

#[tokio::test]
async fn content_storage_last_tweet_time_none() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool, tx);

    let time = adapter.last_tweet_time().await.unwrap();
    assert!(time.is_none());
}

#[tokio::test]
async fn content_storage_last_tweet_time_some() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();

    use crate::storage::threads::{insert_original_tweet, OriginalTweet};
    let ot = OriginalTweet {
        id: 0,
        tweet_id: Some("tw1".to_string()),
        content: "test".to_string(),
        topic: Some("Rust".to_string()),
        llm_provider: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        status: "sent".to_string(),
        error_message: None,
    };
    insert_original_tweet(&pool, &ot).await.unwrap();

    let adapter = ContentStorageAdapter::new(pool, tx);
    let time = adapter.last_tweet_time().await.unwrap();
    assert!(time.is_some());
}

#[tokio::test]
async fn content_storage_last_thread_time() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool, tx);

    let time = adapter.last_thread_time().await.unwrap();
    assert!(time.is_none());
}

#[tokio::test]
async fn content_storage_todays_tweet_times_empty() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool, tx);

    let times = adapter.todays_tweet_times().await.unwrap();
    assert!(times.is_empty());
}

#[tokio::test]
async fn content_storage_create_thread_and_update_status() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool, tx);

    let thread_id = adapter.create_thread("Rust async", 3).await.unwrap();
    assert!(!thread_id.is_empty());

    let result = adapter
        .update_thread_status(&thread_id, "posting", 3, None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn content_storage_store_thread_tweet() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool, tx);

    let thread_id = adapter.create_thread("Rust errors", 2).await.unwrap();
    let result = adapter
        .store_thread_tweet(&thread_id, 0, "tweet_001", "First tweet in thread")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn content_storage_log_action() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool, tx);

    let result = adapter
        .log_action("content", "success", "Posted tweet")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn content_storage_next_scheduled_item_none() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool, tx);

    let item = adapter.next_scheduled_item().await.unwrap();
    assert!(item.is_none());
}

#[tokio::test]
async fn content_storage_mark_scheduled_posted() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();

    // Insert a scheduled item that is due.
    sqlx::query(
        "INSERT INTO scheduled_content \
         (content_type, content, scheduled_for, status, created_at, updated_at, \
          qa_report, qa_hard_flags, qa_soft_flags, qa_recommendations, qa_score, source) \
         VALUES ('tweet', 'test content', datetime('now', '-1 hour'), 'scheduled', \
                 datetime('now'), datetime('now'), '[]', '[]', '[]', '[]', 0.0, 'manual')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let adapter = ContentStorageAdapter::new(pool, tx);

    let item = adapter.next_scheduled_item().await.unwrap();
    assert!(item.is_some());
    let (id, content_type, _content) = item.unwrap();
    assert_eq!(content_type, "tweet");

    let result = adapter.mark_scheduled_posted(id, Some("tw_posted")).await;
    assert!(result.is_ok());
}

// ============================================================================
// TargetStorageAdapter (TargetStorage)
// ============================================================================

#[tokio::test]
async fn target_storage_upsert_and_exists() {
    let pool = init_test_db().await.expect("init db");
    let adapter = TargetStorageAdapter::new(pool);

    adapter
        .upsert_target_account("acc1", "alice")
        .await
        .unwrap();
    adapter
        .store_target_tweet(
            "tw1",
            "acc1",
            "Hello world",
            "2026-01-01T00:00:00Z",
            3,
            10,
            0.8,
        )
        .await
        .unwrap();

    let exists = adapter.target_tweet_exists("tw1").await.unwrap();
    assert!(exists);

    let missing = adapter.target_tweet_exists("tw999").await.unwrap();
    assert!(!missing);
}

#[tokio::test]
async fn target_storage_mark_replied() {
    let pool = init_test_db().await.expect("init db");
    let adapter = TargetStorageAdapter::new(pool);

    adapter
        .upsert_target_account("acc1", "alice")
        .await
        .unwrap();
    adapter
        .store_target_tweet("tw1", "acc1", "Hello", "2026-01-01T00:00:00Z", 0, 5, 0.5)
        .await
        .unwrap();

    let result = adapter.mark_target_tweet_replied("tw1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn target_storage_count_replies_today() {
    let pool = init_test_db().await.expect("init db");
    let adapter = TargetStorageAdapter::new(pool);

    adapter
        .upsert_target_account("acc1", "alice")
        .await
        .unwrap();

    let count = adapter.count_target_replies_today().await.unwrap();
    assert_eq!(count, 0);

    // Store a target tweet and mark it as replied to increment today's count.
    adapter
        .store_target_tweet(
            "tw_count",
            "acc1",
            "Hello",
            "2026-01-01T00:00:00Z",
            0,
            5,
            0.5,
        )
        .await
        .unwrap();
    adapter.mark_target_tweet_replied("tw_count").await.unwrap();
    adapter.record_target_reply("acc1").await.unwrap();

    let count = adapter.count_target_replies_today().await.unwrap();
    assert_eq!(count, 1);
}

// ============================================================================
// AnalyticsStorageAdapter (AnalyticsStorage)
// ============================================================================

#[tokio::test]
async fn analytics_storage_store_follower_snapshot() {
    let pool = init_test_db().await.expect("init db");
    let adapter = AnalyticsStorageAdapter::new(pool);

    let result = adapter.store_follower_snapshot(1000, 200, 500).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn analytics_storage_store_reply_performance() {
    let pool = init_test_db().await.expect("init db");
    let adapter = AnalyticsStorageAdapter::new(pool);

    let result = adapter
        .store_reply_performance("reply1", 10, 5, 1000, 67.0)
        .await;
    assert!(result.is_ok());
}

// ============================================================================
// TopicScorerAdapter (TopicScorer)
// ============================================================================

#[tokio::test]
async fn topic_scorer_empty_db() {
    let pool = init_test_db().await.expect("init db");
    let adapter = TopicScorerAdapter::new(pool);

    let topics = adapter.get_top_topics(5).await.unwrap();
    assert!(topics.is_empty());
}

// ============================================================================
// Dead-Letter + Approval Queue Integration (Task C2)
// ============================================================================

#[tokio::test]
async fn content_storage_mark_failed_permanent_creates_approval_queue_entry() {
    let pool = init_test_db().await.expect("init db");
    let (tx, _rx) = test_post_channel();
    let adapter = ContentStorageAdapter::new(pool.clone(), tx);

    // Create a thread that will fail
    let thread_id = adapter.create_thread("Rust reliability", 2).await.unwrap();
    let content = "This is a tweet that will fail to post";
    adapter
        .store_thread_tweet(&thread_id, 0, "mock_id_1", content)
        .await
        .unwrap();

    // Mark as permanently failed
    let error_msg = "X API error: 503 Service Unavailable after 3 retries";
    let result = adapter.mark_failed_permanent(&thread_id, error_msg).await;
    assert!(result.is_ok(), "mark_failed_permanent should succeed");

    // Verify thread is marked as failed
    let thread_row: (String, String) =
        sqlx::query_as("SELECT status, failure_kind FROM threads WHERE id = ?1")
            .bind(thread_id.parse::<i64>().unwrap())
            .fetch_one(&pool)
            .await
            .expect("thread should exist");
    assert_eq!(thread_row.0, "failed");
    assert_eq!(thread_row.1, "permanent");

    // Verify approval queue entry was created
    let queue_row: (String, String, String) = sqlx::query_as(
        "SELECT action_type, status, reason FROM approval_queue WHERE action_type = 'failed_post_recovery' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("approval queue entry should exist");

    assert_eq!(queue_row.0, "failed_post_recovery");
    assert_eq!(queue_row.1, "pending");
    assert!(
        queue_row.2.contains(&thread_id),
        "reason should contain thread_id"
    );
    assert!(
        queue_row.2.contains("retries=0"),
        "reason should contain retry count"
    );
}
