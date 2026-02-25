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
use crate::x_api::XApiHttpClient;

/// Run the approval poster loop.
///
/// Polls the approval queue for approved items and posts them to X.
/// Uses randomized delay between `min_delay` and `max_delay` to appear human-like.
pub async fn run_approval_poster(
    pool: DbPool,
    x_client: Arc<XApiHttpClient>,
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
                    match upload_media(&x_client, &media_paths).await {
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
                            &x_client,
                            &item.target_tweet_id,
                            &item.generated_content,
                            &media_ids,
                        )
                        .await
                    }
                    _ => {
                        // tweet, thread_tweet, or reply with empty target
                        post_tweet(&x_client, &item.generated_content, &media_ids).await
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

/// Post a reply to a tweet.
async fn post_reply(
    client: &XApiHttpClient,
    tweet_id: &str,
    content: &str,
    media_ids: &[String],
) -> Result<String, String> {
    use crate::x_api::XApiClient;

    if media_ids.is_empty() {
        client
            .reply_to_tweet(content, tweet_id)
            .await
            .map(|posted| posted.id)
            .map_err(|e| e.to_string())
    } else {
        client
            .reply_to_tweet_with_media(content, tweet_id, media_ids)
            .await
            .map(|posted| posted.id)
            .map_err(|e| e.to_string())
    }
}

/// Post an original tweet.
async fn post_tweet(
    client: &XApiHttpClient,
    content: &str,
    media_ids: &[String],
) -> Result<String, String> {
    use crate::x_api::XApiClient;

    if media_ids.is_empty() {
        client
            .post_tweet(content)
            .await
            .map(|posted| posted.id)
            .map_err(|e| e.to_string())
    } else {
        client
            .post_tweet_with_media(content, media_ids)
            .await
            .map(|posted| posted.id)
            .map_err(|e| e.to_string())
    }
}

/// Upload local media files to X and return their media IDs.
async fn upload_media(
    client: &XApiHttpClient,
    media_paths: &[String],
) -> Result<Vec<String>, String> {
    use crate::x_api::XApiClient;

    use crate::x_api::types::{ImageFormat, MediaType};
    use std::path::Path;

    let mut media_ids = Vec::with_capacity(media_paths.len());
    for path in media_paths {
        let expanded = storage::expand_tilde(path);
        let data = tokio::fs::read(&expanded)
            .await
            .map_err(|e| format!("Failed to read media file {}: {}", path, e))?;

        // Guess media type from extension.
        let ext = Path::new(&expanded)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let media_type = match ext {
            "png" => MediaType::Image(ImageFormat::Png),
            "gif" => MediaType::Gif,
            "mp4" => MediaType::Video,
            "webp" => MediaType::Image(ImageFormat::Webp),
            _ => MediaType::Image(ImageFormat::Jpeg),
        };

        let media_id = client
            .upload_media(&data, media_type)
            .await
            .map_err(|e| format!("Failed to upload media {}: {}", path, e))?;
        media_ids.push(media_id.0);
    }
    Ok(media_ids)
}

/// Compute a randomized delay between `min` and `max`.
fn randomized_delay(min: Duration, max: Duration) -> Duration {
    if min >= max || (min.is_zero() && max.is_zero()) {
        return min;
    }
    let min_ms = min.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    Duration::from_millis(rand::thread_rng().gen_range(min_ms..=max_ms))
}
