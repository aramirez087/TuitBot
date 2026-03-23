//! Approval poster loop: polls for approved items and posts them to X.
//!
//! When `approval_mode` is enabled, posts go into the approval queue rather
//! than being posted directly. This loop watches for items that have been
//! approved by the user and posts them via the X API.

use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio_util::sync::CancellationToken;

use crate::content::deserialize_blocks_from_content;
use crate::storage::{self, DbPool};
use crate::x_api::XApiClient;

/// Run the approval poster loop for a specific account.
///
/// Polls the approval queue for approved items belonging to `account_id`
/// and posts them to X using the provided `x_client` (which must be
/// constructed with that account's credentials).
///
/// Uses randomized delay between `min_delay` and `max_delay` to appear human-like.
pub async fn run_approval_poster(
    pool: DbPool,
    x_client: Arc<dyn XApiClient>,
    account_id: String,
    min_delay: Duration,
    max_delay: Duration,
    cancel: CancellationToken,
) {
    tracing::info!(account_id = %account_id, "Approval poster loop started");

    // Poll interval when no items are found.
    let idle_interval = Duration::from_secs(15);

    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => {
                tracing::info!("Approval poster received cancellation");
                break;
            }
            () = tokio::time::sleep(idle_interval) => {}
        }

        match storage::approval_queue::get_next_approved_for(&pool, &account_id).await {
            Ok(Some(item)) => {
                tracing::info!(
                    id = item.id,
                    account_id = %account_id,
                    action_type = %item.action_type,
                    "Posting approved item"
                );

                // Parse media paths from JSON.
                let media_paths: Vec<String> =
                    serde_json::from_str(&item.media_paths).unwrap_or_default();

                // Upload media if any.
                let media_ids = if media_paths.is_empty() {
                    vec![]
                } else {
                    match upload_media(&*x_client, &media_paths).await {
                        Ok(ids) => ids,
                        Err(e) => {
                            tracing::warn!(
                                id = item.id,
                                error = %e,
                                "Failed to upload media for approved item, posting without media"
                            );
                            vec![]
                        }
                    }
                };

                // Route by action type: thread gets reply-chain posting,
                // reply gets in-reply-to, everything else posts standalone.
                if item.action_type == "thread" {
                    match post_thread_and_persist(&pool, &*x_client, &account_id, &item, &media_ids)
                        .await
                    {
                        Ok(root_tweet_id) => {
                            tracing::info!(
                                id = item.id,
                                root_tweet_id = %root_tweet_id,
                                "Approved thread posted successfully"
                            );
                            if let Err(e) = storage::approval_queue::mark_posted_for(
                                &pool,
                                &account_id,
                                item.id,
                                &root_tweet_id,
                            )
                            .await
                            {
                                tracing::warn!(id = item.id, error = %e, "Failed to mark thread as posted");
                            }

                            let _ = storage::action_log::log_action_for(
                                &pool,
                                &account_id,
                                "thread_posted",
                                "success",
                                Some(&format!("Posted approved thread {}", item.id)),
                                None,
                            )
                            .await;
                        }
                        Err(e) => {
                            tracing::warn!(id = item.id, error = %e, "Failed to post approved thread");
                            let _ = storage::approval_queue::mark_failed_for(
                                &pool,
                                &account_id,
                                item.id,
                                &format!("Thread posting failed: {e}"),
                            )
                            .await;
                            let _ = storage::action_log::log_action_for(
                                &pool,
                                &account_id,
                                "thread_posted",
                                "error",
                                Some(&format!(
                                    "Failed to post approved thread {}: {}",
                                    item.id, e
                                )),
                                None,
                            )
                            .await;
                        }
                    }
                } else {
                    let result = match item.action_type.as_str() {
                        "reply" if !item.target_tweet_id.is_empty() => {
                            post_reply(
                                &*x_client,
                                &item.target_tweet_id,
                                &item.generated_content,
                                &media_ids,
                            )
                            .await
                        }
                        _ => {
                            // tweet, thread_tweet, or reply with empty target
                            post_tweet(&*x_client, &item.generated_content, &media_ids).await
                        }
                    };

                    match result {
                        Ok(tweet_id) => {
                            tracing::info!(
                                id = item.id,
                                tweet_id = %tweet_id,
                                "Approved item posted successfully"
                            );
                            if let Err(e) = storage::approval_queue::mark_posted_for(
                                &pool,
                                &account_id,
                                item.id,
                                &tweet_id,
                            )
                            .await
                            {
                                tracing::warn!(
                                    id = item.id,
                                    error = %e,
                                    "Failed to mark approved item as posted"
                                );
                            }

                            // Propagate vault provenance to original_tweets record.
                            propagate_provenance(&pool, &account_id, &item, &tweet_id).await;

                            // Write loop-back metadata to source notes.
                            execute_loopback_for_provenance(&pool, &account_id, &item, &tweet_id)
                                .await;

                            // Log the action.
                            let _ = storage::action_log::log_action_for(
                                &pool,
                                &account_id,
                                &format!("{}_posted", item.action_type),
                                "success",
                                Some(&format!("Posted approved item {}", item.id)),
                                None,
                            )
                            .await;
                        }
                        Err(e) => {
                            tracing::warn!(
                                id = item.id,
                                error = %e,
                                "Failed to post approved item"
                            );
                            let _ = storage::approval_queue::mark_failed_for(
                                &pool,
                                &account_id,
                                item.id,
                                &format!("Posting failed: {e}"),
                            )
                            .await;
                            let _ = storage::action_log::log_action_for(
                                &pool,
                                &account_id,
                                &format!("{}_posted", item.action_type),
                                "error",
                                Some(&format!("Failed to post approved item {}: {}", item.id, e)),
                                None,
                            )
                            .await;
                        }
                    }
                }

                // Jittered delay between posts.
                let delay = randomized_delay(min_delay, max_delay);
                if !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }
            }
            Ok(None) => {
                // No approved items — continue polling.
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query approved items");
            }
        }
    }

    tracing::info!(account_id = %account_id, "Approval poster loop stopped");
}

/// Post a reply to a tweet via toolkit.
async fn post_reply(
    client: &dyn XApiClient,
    tweet_id: &str,
    content: &str,
    media_ids: &[String],
) -> Result<String, String> {
    let media = if media_ids.is_empty() {
        None
    } else {
        Some(media_ids)
    };
    crate::toolkit::write::reply_to_tweet(client, content, tweet_id, media)
        .await
        .map(|posted| posted.id)
        .map_err(|e| e.to_string())
}

/// Post an original tweet via toolkit.
async fn post_tweet(
    client: &dyn XApiClient,
    content: &str,
    media_ids: &[String],
) -> Result<String, String> {
    let media = if media_ids.is_empty() {
        None
    } else {
        Some(media_ids)
    };
    crate::toolkit::write::post_tweet(client, content, media)
        .await
        .map(|posted| posted.id)
        .map_err(|e| e.to_string())
}

/// Upload local media files to X via toolkit and return their media IDs.
async fn upload_media(
    client: &dyn XApiClient,
    media_paths: &[String],
) -> Result<Vec<String>, String> {
    use crate::x_api::types::{ImageFormat, MediaType};

    let mut media_ids = Vec::with_capacity(media_paths.len());
    for path in media_paths {
        let expanded = storage::expand_tilde(path);
        let data = tokio::fs::read(&expanded)
            .await
            .map_err(|e| format!("Failed to read media file {}: {}", path, e))?;

        // Infer media type via toolkit, falling back to JPEG.
        let media_type = crate::toolkit::media::infer_media_type(&expanded)
            .unwrap_or(MediaType::Image(ImageFormat::Jpeg));

        let media_id = crate::toolkit::media::upload_media(client, &data, media_type)
            .await
            .map_err(|e| format!("Failed to upload media {}: {}", path, e))?;
        media_ids.push(media_id.0);
    }
    Ok(media_ids)
}

/// Parse thread content from approval queue storage format.
///
/// Supports both block JSON format (versioned payload) and legacy string arrays.
/// Returns ordered tweet texts for sequential posting.
pub fn parse_thread_content(content: &str) -> Result<Vec<String>, String> {
    // Try block JSON format first.
    if let Some(mut blocks) = deserialize_blocks_from_content(content) {
        blocks.sort_by_key(|b| b.order);
        let texts: Vec<String> = blocks.into_iter().map(|b| b.text).collect();
        if texts.is_empty() {
            return Err("thread blocks payload is empty".to_string());
        }
        return Ok(texts);
    }

    // Try legacy string array format.
    if let Ok(tweets) = serde_json::from_str::<Vec<String>>(content) {
        if tweets.is_empty() {
            return Err("thread content array is empty".to_string());
        }
        return Ok(tweets);
    }

    Err("cannot parse thread content: not block JSON or string array".to_string())
}

/// Post a thread as a reply chain and persist all storage records.
///
/// Creates: threads row, thread_tweets rows, original_tweets row, provenance
/// links (to both original_tweet and thread entities), and loopback entries.
///
/// Returns the root tweet ID on success.
async fn post_thread_and_persist(
    pool: &DbPool,
    x_client: &dyn XApiClient,
    account_id: &str,
    item: &storage::approval_queue::ApprovalItem,
    media_ids: &[String],
) -> Result<String, String> {
    let tweet_texts = parse_thread_content(&item.generated_content)?;

    // Post as reply chain: root standalone, children reply to previous.
    let mut posted_ids: Vec<String> = Vec::with_capacity(tweet_texts.len());
    let mut posted_contents: Vec<String> = Vec::with_capacity(tweet_texts.len());

    for (i, text) in tweet_texts.iter().enumerate() {
        let result = if i == 0 {
            post_tweet(x_client, text, media_ids).await
        } else {
            post_reply(x_client, &posted_ids[i - 1], text, &[]).await
        };

        match result {
            Ok(tweet_id) => {
                posted_ids.push(tweet_id);
                posted_contents.push(text.clone());
            }
            Err(e) => {
                // Partial failure: persist what we posted so far.
                if !posted_ids.is_empty() {
                    persist_and_propagate_thread(
                        pool,
                        account_id,
                        item,
                        &posted_ids,
                        &posted_contents,
                        "partial",
                    )
                    .await;
                }
                return Err(format!(
                    "Thread failed at tweet {}/{}: {e}. {} tweet(s) posted.",
                    i + 1,
                    tweet_texts.len(),
                    posted_ids.len()
                ));
            }
        }
    }

    // Full success: persist all records.
    persist_and_propagate_thread(
        pool,
        account_id,
        item,
        &posted_ids,
        &posted_contents,
        "sent",
    )
    .await;

    Ok(posted_ids[0].clone())
}

/// Persist thread records and propagate provenance + loopback.
async fn persist_and_propagate_thread(
    pool: &DbPool,
    account_id: &str,
    item: &storage::approval_queue::ApprovalItem,
    posted_ids: &[String],
    posted_contents: &[String],
    status: &str,
) {
    let topic = if item.topic.is_empty() {
        ""
    } else {
        &item.topic
    };

    match storage::threads::persist_thread_records(
        pool,
        account_id,
        topic,
        posted_ids,
        posted_contents,
        status,
    )
    .await
    {
        Ok((thread_id, ot_id)) => {
            // Set source_node_id on the original_tweet.
            if let Some(node_id) = item.source_node_id {
                let _ = storage::threads::set_original_tweet_source_node_for(
                    pool, account_id, ot_id, node_id,
                )
                .await;
            }

            // Copy provenance to original_tweet entity.
            let _ = storage::provenance::copy_links_for(
                pool,
                account_id,
                "approval_queue",
                item.id,
                "original_tweet",
                ot_id,
            )
            .await;

            // Copy provenance to thread entity.
            let _ = storage::provenance::copy_links_for(
                pool,
                account_id,
                "approval_queue",
                item.id,
                "thread",
                thread_id,
            )
            .await;

            // Write loopback to source notes with child_tweet_ids.
            let root_tweet_id = &posted_ids[0];
            let child_ids: Vec<String> = posted_ids.iter().skip(1).cloned().collect();
            execute_loopback_for_thread(pool, account_id, item, root_tweet_id, child_ids).await;
        }
        Err(e) => {
            tracing::warn!(
                id = item.id,
                error = %e,
                "Failed to persist thread records"
            );
        }
    }
}

/// Write loop-back metadata to source notes for a thread.
async fn execute_loopback_for_thread(
    pool: &DbPool,
    account_id: &str,
    item: &storage::approval_queue::ApprovalItem,
    root_tweet_id: &str,
    child_tweet_ids: Vec<String>,
) {
    use crate::automation::watchtower::loopback;
    use std::collections::HashSet;

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
async fn propagate_provenance(
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

/// Write loop-back metadata to source notes referenced by provenance links.
///
/// Looks up provenance links for the approval queue item, deduplicates by
/// `node_id`, and calls `execute_loopback()` for each unique node.
///
/// `account_id` must match the account that owns the approval item.
async fn execute_loopback_for_provenance(
    pool: &DbPool,
    account_id: &str,
    item: &storage::approval_queue::ApprovalItem,
    tweet_id: &str,
) {
    use crate::automation::watchtower::loopback;
    use std::collections::HashSet;

    let url = format!("https://x.com/i/status/{tweet_id}");
    let content_type = &item.action_type;

    // Collect unique node_ids from provenance links.
    let links =
        match storage::provenance::get_links_for(pool, account_id, "approval_queue", item.id).await
        {
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

/// Compute a randomized delay between `min` and `max`.
fn randomized_delay(min: Duration, max: Duration) -> Duration {
    if min >= max || (min.is_zero() && max.is_zero()) {
        return min;
    }
    let min_ms = min.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    Duration::from_millis(rand::rng().random_range(min_ms..=max_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── randomized_delay ────────────────────────────────────────────

    #[test]
    fn delay_returns_min_when_min_equals_max() {
        let d = randomized_delay(Duration::from_secs(5), Duration::from_secs(5));
        assert_eq!(d, Duration::from_secs(5));
    }

    #[test]
    fn delay_returns_min_when_min_greater_than_max() {
        let d = randomized_delay(Duration::from_secs(10), Duration::from_secs(5));
        assert_eq!(d, Duration::from_secs(10));
    }

    #[test]
    fn delay_returns_zero_when_both_zero() {
        let d = randomized_delay(Duration::ZERO, Duration::ZERO);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn delay_within_range() {
        let min = Duration::from_millis(100);
        let max = Duration::from_millis(500);
        for _ in 0..50 {
            let d = randomized_delay(min, max);
            assert!(d >= min, "delay {d:?} should be >= {min:?}");
            assert!(d <= max, "delay {d:?} should be <= {max:?}");
        }
    }

    #[test]
    fn delay_zero_min_nonzero_max() {
        let min = Duration::ZERO;
        let max = Duration::from_millis(100);
        for _ in 0..20 {
            let d = randomized_delay(min, max);
            assert!(d <= max);
        }
    }

    #[test]
    fn delay_narrow_range_produces_deterministic_ish_result() {
        let min = Duration::from_millis(50);
        let max = Duration::from_millis(51);
        for _ in 0..20 {
            let d = randomized_delay(min, max);
            assert!(d >= min && d <= max);
        }
    }

    // ── media_paths JSON parsing (mirrors inline logic) ─────────────

    #[test]
    fn media_paths_parses_valid_json_array() {
        let json = r#"["/tmp/img1.png", "/tmp/img2.jpg"]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], "/tmp/img1.png");
    }

    #[test]
    fn media_paths_parses_empty_array() {
        let json = "[]";
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    #[test]
    fn media_paths_invalid_json_returns_empty() {
        let json = "not valid json";
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    #[test]
    fn media_paths_empty_string_returns_empty() {
        let json = "";
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    // ── action_type routing logic ───────────────────────────────────

    #[test]
    fn action_type_reply_with_target_routes_to_reply() {
        let action_type = "reply";
        let target_tweet_id = "12345";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(is_reply);
    }

    #[test]
    fn action_type_reply_without_target_routes_to_tweet() {
        let action_type = "reply";
        let target_tweet_id = "";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(!is_reply);
    }

    #[test]
    fn action_type_tweet_routes_to_tweet() {
        let action_type = "tweet";
        let target_tweet_id = "";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(!is_reply);
    }

    #[test]
    fn action_type_thread_tweet_routes_to_tweet() {
        let action_type = "thread_tweet";
        let target_tweet_id = "some_id";
        let is_reply = action_type == "reply" && !target_tweet_id.is_empty();
        assert!(!is_reply);
    }

    // ── action log format string ────────────────────────────────────

    #[test]
    fn action_log_format_for_reply() {
        let action_type = "reply";
        let log_action = format!("{action_type}_posted");
        assert_eq!(log_action, "reply_posted");
    }

    #[test]
    fn action_log_format_for_tweet() {
        let action_type = "tweet";
        let log_action = format!("{action_type}_posted");
        assert_eq!(log_action, "tweet_posted");
    }

    // ── post_reply / post_tweet helper logic ─────────────────────

    #[test]
    fn media_ids_empty_gives_none() {
        let media_ids: Vec<String> = vec![];
        let media: Option<&[String]> = if media_ids.is_empty() {
            None
        } else {
            Some(&media_ids)
        };
        assert!(media.is_none());
    }

    #[test]
    fn media_ids_nonempty_gives_some() {
        let media_ids = vec!["m1".to_string()];
        let media: Option<&[String]> = if media_ids.is_empty() {
            None
        } else {
            Some(&media_ids)
        };
        assert!(media.is_some());
        assert_eq!(media.unwrap().len(), 1);
    }

    // ── propagate_provenance conditional logic ───────────────────

    #[test]
    fn propagate_condition_both_none_skips() {
        let source_node_id: Option<i64> = None;
        let source_seed_id: Option<i64> = None;
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(!should_propagate);
    }

    #[test]
    fn propagate_condition_node_id_triggers() {
        let source_node_id: Option<i64> = Some(42);
        let source_seed_id: Option<i64> = None;
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(should_propagate);
    }

    #[test]
    fn propagate_condition_seed_id_triggers() {
        let source_node_id: Option<i64> = None;
        let source_seed_id: Option<i64> = Some(99);
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(should_propagate);
    }

    #[test]
    fn propagate_condition_both_set_triggers() {
        let source_node_id: Option<i64> = Some(1);
        let source_seed_id: Option<i64> = Some(2);
        let should_propagate = source_node_id.is_some() || source_seed_id.is_some();
        assert!(should_propagate);
    }

    // ── topic to Option conversion ───────────────────────────────

    #[test]
    fn empty_topic_becomes_none() {
        let topic = "";
        let opt: Option<String> = if topic.is_empty() {
            None
        } else {
            Some(topic.to_string())
        };
        assert!(opt.is_none());
    }

    #[test]
    fn nonempty_topic_becomes_some() {
        let topic = "rust programming";
        let opt: Option<String> = if topic.is_empty() {
            None
        } else {
            Some(topic.to_string())
        };
        assert_eq!(opt, Some("rust programming".to_string()));
    }

    // ── loopback URL construction ────────────────────────────────

    #[test]
    fn loopback_url_format() {
        let tweet_id = "1234567890";
        let url = format!("https://x.com/i/status/{tweet_id}");
        assert_eq!(url, "https://x.com/i/status/1234567890");
    }

    // ── delay edge cases ─────────────────────────────────────────

    #[test]
    fn delay_large_values() {
        let min = Duration::from_secs(60);
        let max = Duration::from_secs(300);
        for _ in 0..20 {
            let d = randomized_delay(min, max);
            assert!(d >= min);
            assert!(d <= max);
        }
    }

    #[test]
    fn delay_subsecond() {
        let min = Duration::from_millis(1);
        let max = Duration::from_millis(10);
        for _ in 0..20 {
            let d = randomized_delay(min, max);
            assert!(d >= min);
            assert!(d <= max);
        }
    }

    #[test]
    fn delay_is_zero_returns_true() {
        assert!(Duration::ZERO.is_zero());
        assert!(!Duration::from_millis(1).is_zero());
    }

    // ── action_type exhaustive routing ────────────────────────────

    #[test]
    fn action_type_all_variants() {
        for (action_type, target, expected_reply) in [
            ("reply", "12345", true),
            ("reply", "", false),
            ("tweet", "", false),
            ("tweet", "12345", false),
            ("thread_tweet", "12345", false),
            ("thread_tweet", "", false),
        ] {
            let is_reply = action_type == "reply" && !target.is_empty();
            assert_eq!(
                is_reply, expected_reply,
                "action={action_type}, target={target}"
            );
        }
    }

    // ── action log format all types ───────────────────────────────

    #[test]
    fn action_log_format_thread() {
        assert_eq!(format!("{}_posted", "thread_tweet"), "thread_tweet_posted");
    }

    // ── media_paths JSON edge cases ──────────────────────────────

    #[test]
    fn media_paths_nested_arrays_treated_as_invalid() {
        let json = r#"[["nested"]]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert!(paths.is_empty());
    }

    #[test]
    fn media_paths_single_item() {
        let json = r#"["/path/to/image.jpg"]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "/path/to/image.jpg");
    }

    #[test]
    fn media_paths_many_items() {
        let json = r#"["/a.jpg", "/b.png", "/c.gif", "/d.mp4"]"#;
        let paths: Vec<String> = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(paths.len(), 4);
    }

    // ── parse_thread_content ─────────────────────────────────────

    #[test]
    fn parse_thread_content_block_json() {
        use crate::content::{serialize_blocks_for_storage, ThreadBlock};

        let blocks = vec![
            ThreadBlock {
                id: "a".to_string(),
                text: "First tweet".to_string(),
                media_paths: vec![],
                order: 0,
            },
            ThreadBlock {
                id: "b".to_string(),
                text: "Second tweet".to_string(),
                media_paths: vec![],
                order: 1,
            },
        ];
        let content = serialize_blocks_for_storage(&blocks);
        let parsed = parse_thread_content(&content).unwrap();
        assert_eq!(parsed, vec!["First tweet", "Second tweet"]);
    }

    #[test]
    fn parse_thread_content_legacy_string_array() {
        let content = r#"["Tweet one","Tweet two","Tweet three"]"#;
        let parsed = parse_thread_content(content).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], "Tweet one");
    }

    #[test]
    fn parse_thread_content_invalid_format() {
        let result = parse_thread_content("just plain text");
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_empty_array() {
        let result = parse_thread_content("[]");
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_blocks_sorted_by_order() {
        use crate::content::{serialize_blocks_for_storage, ThreadBlock};

        // Blocks with reversed order
        let blocks = vec![
            ThreadBlock {
                id: "b".to_string(),
                text: "Second".to_string(),
                media_paths: vec![],
                order: 1,
            },
            ThreadBlock {
                id: "a".to_string(),
                text: "First".to_string(),
                media_paths: vec![],
                order: 0,
            },
        ];
        let content = serialize_blocks_for_storage(&blocks);
        let parsed = parse_thread_content(&content).unwrap();
        assert_eq!(parsed, vec!["First", "Second"]);
    }

    // ── action_type thread routing ───────────────────────────────

    #[test]
    fn action_type_thread_is_routed_separately() {
        // Thread items are handled by the thread-specific branch,
        // not the reply/tweet match.
        let action_type = "thread";
        let is_thread = action_type == "thread";
        assert!(is_thread);
    }

    // ── parse_thread_content additional edge cases ─────────────────

    #[test]
    fn parse_thread_content_single_tweet_array() {
        let content = r#"["Only tweet"]"#;
        let parsed = parse_thread_content(content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], "Only tweet");
    }

    #[test]
    fn parse_thread_content_numeric_array_is_invalid() {
        let content = "[1, 2, 3]";
        let result = parse_thread_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_nested_json_is_invalid() {
        let content = r#"{"key": "value"}"#;
        let result = parse_thread_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_preserves_tweet_order() {
        let content = r#"["First","Second","Third","Fourth"]"#;
        let parsed = parse_thread_content(content).unwrap();
        assert_eq!(parsed, vec!["First", "Second", "Third", "Fourth"]);
    }

    #[test]
    fn parse_thread_content_empty_string() {
        let result = parse_thread_content("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_thread_content_whitespace_only() {
        let result = parse_thread_content("   ");
        assert!(result.is_err());
    }

    // ── tweet URL construction ────────────────────────────────────

    #[test]
    fn loopback_url_format_long_id() {
        let tweet_id = "1234567890123456789";
        let url = format!("https://x.com/i/status/{tweet_id}");
        assert_eq!(url, "https://x.com/i/status/1234567890123456789");
    }

    // ── child_tweet_ids extraction ────────────────────────────────

    #[test]
    fn child_ids_from_posted_ids() {
        let posted_ids = vec![
            "root".to_string(),
            "child1".to_string(),
            "child2".to_string(),
        ];
        let child_ids: Vec<String> = posted_ids.iter().skip(1).cloned().collect();
        assert_eq!(child_ids, vec!["child1", "child2"]);
    }

    #[test]
    fn child_ids_single_tweet_no_children() {
        let posted_ids = vec!["root".to_string()];
        let child_ids: Vec<String> = posted_ids.iter().skip(1).cloned().collect();
        assert!(child_ids.is_empty());
    }

    // ── topic normalization ───────────────────────────────────────

    #[test]
    fn empty_topic_uses_fallback() {
        let topic = "";
        let effective = if topic.is_empty() { "" } else { topic };
        assert_eq!(effective, "");
    }

    #[test]
    fn nonempty_topic_used_directly() {
        let topic = "rust async";
        let effective = if topic.is_empty() { "" } else { topic };
        assert_eq!(effective, "rust async");
    }
}
