//! Approval poster loop: polls for approved items and posts them to X.
//!
//! When `approval_mode` is enabled, posts go into the approval queue rather
//! than being posted directly. This loop watches for items that have been
//! approved by the user and posts them via the X API.

mod poster;
mod queue;
mod tests;

pub use poster::parse_thread_content;

use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

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
                    match poster::upload_media(&*x_client, &media_paths).await {
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
                    match poster::post_thread_and_persist(&pool, &*x_client, &account_id, &item, &media_ids)
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
                            poster::post_reply(
                                &*x_client,
                                &item.target_tweet_id,
                                &item.generated_content,
                                &media_ids,
                            )
                            .await
                        }
                        _ => {
                            // tweet, thread_tweet, or reply with empty target
                            poster::post_tweet(&*x_client, &item.generated_content, &media_ids).await
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
                            queue::propagate_provenance(&pool, &account_id, &item, &tweet_id).await;

                            // Write loop-back metadata to source notes.
                            queue::execute_loopback_for_provenance(&pool, &account_id, &item, &tweet_id)
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
                let delay = queue::randomized_delay(min_delay, max_delay);
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
