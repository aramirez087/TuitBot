//! Queue management and loopback metadata writing.
//!
//! Handles random delays between posts, provenance tracking,
//! and loopback metadata updates to source vault notes.

use std::collections::HashSet;

use crate::storage::{self, DbPool};

/// Compute a randomized delay between `min` and `max`.
pub(super) fn randomized_delay(min: std::time::Duration, max: std::time::Duration) -> std::time::Duration {
    use rand::Rng;
    
    if min >= max || (min.is_zero() && max.is_zero()) {
        return min;
    }
    let min_ms = min.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    std::time::Duration::from_millis(rand::rng().random_range(min_ms..=max_ms))
}

/// Write loop-back metadata to source notes referenced by provenance links.
///
/// Looks up provenance links for the approval queue item, deduplicates by
/// `node_id`, and calls `execute_loopback()` for each unique node.
///
/// `account_id` must match the account that owns the approval item.
pub(super) async fn execute_loopback_for_provenance(
    pool: &DbPool,
    account_id: &str,
    item: &storage::approval_queue::ApprovalItem,
    tweet_id: &str,
) {
    use crate::automation::watchtower::loopback;

    let url = format!("https://x.com/i/status/{tweet_id}");
    let content_type = &item.action_type;

    // Collect unique node_ids from provenance links.
    let links = match storage::provenance::get_links_for(pool, account_id, "approval_queue", item.id).await {
        Ok(l) => l,
        Err(e) => {
            tracing::debug!(id = item.id, error = %e, "No provenance links for loopback");
            return;
        }
    };

    let mut seen = HashSet::new();
    for link in &links {
        if let Some(node_id) = link.node_id {
            if seen.insert(node_id) {
                let result =
                    loopback::execute_loopback(pool, node_id, tweet_id, &url, content_type).await;
                match &result {
                    loopback::LoopBackResult::Written => {
                        tracing::info!(
                            node_id,
                            tweet_id,
                            "Loopback: wrote metadata to source note"
                        );
                    }
                    loopback::LoopBackResult::AlreadyPresent => {
                        tracing::debug!(node_id, tweet_id, "Loopback: already present");
                    }
                    loopback::LoopBackResult::SourceNotWritable(reason) => {
                        tracing::debug!(node_id, reason, "Loopback: source not writable, skipping");
                    }
                    loopback::LoopBackResult::NodeNotFound => {
                        tracing::debug!(node_id, "Loopback: node not found");
                    }
                    loopback::LoopBackResult::FileNotFound => {
                        tracing::debug!(node_id, "Loopback: file not found on disk");
                    }
                }
            }
        }
    }
}

/// Write loop-back metadata to source notes for a thread.
pub(super) async fn execute_loopback_for_thread(
    pool: &DbPool,
    account_id: &str,
    item: &storage::approval_queue::ApprovalItem,
    root_tweet_id: &str,
    child_tweet_ids: Vec<String>,
) {
    use crate::automation::watchtower::loopback;

    let url = format!("https://x.com/i/status/{root_tweet_id}");

    let links = match storage::provenance::get_links_for(
        pool,
        account_id,
        "approval_queue",
        item.id,
    )
    .await
    {
        Ok(l) => l,
        Err(e) => {
            tracing::debug!(id = item.id, error = %e, "No provenance links for thread loopback");
            return;
        }
    };

    let mut seen = HashSet::new();
    for link in &links {
        if let Some(node_id) = link.node_id {
            if seen.insert(node_id) {
                let result = loopback::execute_loopback_thread(
                    pool,
                    node_id,
                    root_tweet_id,
                    &url,
                    child_tweet_ids.clone(),
                )
                .await;
                match &result {
                    loopback::LoopBackResult::Written => {
                        tracing::info!(node_id, root_tweet_id, "Loopback: wrote thread metadata");
                    }
                    loopback::LoopBackResult::AlreadyPresent => {
                        tracing::debug!(node_id, root_tweet_id, "Loopback: thread already present");
                    }
                    other => {
                        tracing::debug!(node_id, result = ?other, "Loopback: thread write skipped");
                    }
                }
            }
        }
    }
}

/// Propagate vault provenance from the approval queue item to original_tweets.
///
/// If the approval item has a `source_node_id`, inserts an `original_tweets`
/// record with that node ID set, and copies provenance links from the
/// `approval_queue` entity to the new `original_tweet` entity.
///
/// `account_id` must match the account that owns the approval item.
pub(super) async fn propagate_provenance(
    pool: &DbPool,
    account_id: &str,
    item: &storage::approval_queue::ApprovalItem,
    tweet_id: &str,
) {
    // Insert an original_tweets record for provenance tracking.
    if item.source_node_id.is_some() || item.source_seed_id.is_some() {
        let tweet = storage::threads::OriginalTweet {
            id: 0,
            tweet_id: Some(tweet_id.to_string()),
            content: item.generated_content.clone(),
            topic: if item.topic.is_empty() {
                None
            } else {
                Some(item.topic.clone())
            },
            llm_provider: None,
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            status: "sent".to_string(),
            error_message: None,
        };

        match storage::threads::insert_original_tweet_for(pool, account_id, &tweet).await {
            Ok(ot_id) => {
                // Set source_node_id on the original_tweet.
                if let Some(node_id) = item.source_node_id {
                    let _ = storage::threads::set_original_tweet_source_node_for(
                        pool, account_id, ot_id, node_id,
                    )
                    .await;
                }

                // Copy provenance links from approval_queue to original_tweet.
                let _ = storage::provenance::copy_links_for(
                    pool,
                    account_id,
                    "approval_queue",
                    item.id,
                    "original_tweet",
                    ot_id,
                )
                .await;
            }
            Err(e) => {
                tracing::warn!(
                    id = item.id,
                    error = %e,
                    "Failed to insert original_tweet for provenance tracking"
                );
            }
        }
    }
}
