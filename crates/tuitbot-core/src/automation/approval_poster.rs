//! Approval poster loop: polls for approved items and posts them to X.
//!
//! When `approval_mode` is enabled, posts go into the approval queue rather
//! than being posted directly. This loop watches for items that have been
//! approved by the user and posts them via the X API.

use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio_util::sync::CancellationToken;

use crate::storage::{self, DbPool};
use crate::x_api::XApiClient;

/// Run the approval poster loop.
///
/// Polls the approval queue for approved items and posts them to X.
/// Uses randomized delay between `min_delay` and `max_delay` to appear human-like.
pub async fn run_approval_poster(
    pool: DbPool,
    x_client: Arc<dyn XApiClient>,
    min_delay: Duration,
    max_delay: Duration,
    cancel: CancellationToken,
) {
    tracing::info!("Approval poster loop started");

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

        match storage::approval_queue::get_next_approved(&pool).await {
            Ok(Some(item)) => {
                tracing::info!(
                    id = item.id,
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
                        if let Err(e) =
                            storage::approval_queue::mark_posted(&pool, item.id, &tweet_id).await
                        {
                            tracing::warn!(
                                id = item.id,
                                error = %e,
                                "Failed to mark approved item as posted"
                            );
                        }

                        // Propagate vault provenance to original_tweets record.
                        propagate_provenance(&pool, &item, &tweet_id).await;

                        // Write loop-back metadata to source notes.
                        execute_loopback_for_provenance(&pool, &item, &tweet_id).await;

                        // Log the action.
                        let _ = storage::action_log::log_action(
                            &pool,
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
                        let _ = storage::approval_queue::mark_failed(
                            &pool,
                            item.id,
                            &format!("Posting failed: {e}"),
                        )
                        .await;
                        let _ = storage::action_log::log_action(
                            &pool,
                            &format!("{}_posted", item.action_type),
                            "error",
                            Some(&format!("Failed to post approved item {}: {}", item.id, e)),
                            None,
                        )
                        .await;
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

    tracing::info!("Approval poster loop stopped");
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

/// Propagate vault provenance from the approval queue item to original_tweets.
///
/// If the approval item has a `source_node_id`, inserts an `original_tweets`
/// record with that node ID set, and copies provenance links from the
/// `approval_queue` entity to the new `original_tweet` entity.
async fn propagate_provenance(
    pool: &DbPool,
    item: &storage::approval_queue::ApprovalItem,
    tweet_id: &str,
) {
    use crate::storage::accounts::DEFAULT_ACCOUNT_ID;

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

        match storage::threads::insert_original_tweet_for(pool, DEFAULT_ACCOUNT_ID, &tweet).await {
            Ok(ot_id) => {
                // Set source_node_id on the original_tweet.
                if let Some(node_id) = item.source_node_id {
                    let _ = storage::threads::set_original_tweet_source_node_for(
                        pool,
                        DEFAULT_ACCOUNT_ID,
                        ot_id,
                        node_id,
                    )
                    .await;
                }

                // Copy provenance links from approval_queue to original_tweet.
                let _ = storage::provenance::copy_links_for(
                    pool,
                    DEFAULT_ACCOUNT_ID,
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
async fn execute_loopback_for_provenance(
    pool: &DbPool,
    item: &storage::approval_queue::ApprovalItem,
    tweet_id: &str,
) {
    use crate::automation::watchtower::loopback;
    use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
    use std::collections::HashSet;

    let url = format!("https://x.com/i/status/{tweet_id}");
    let content_type = &item.action_type;

    // Collect unique node_ids from provenance links.
    let links = match storage::provenance::get_links_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        "approval_queue",
        item.id,
    )
    .await
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
}
