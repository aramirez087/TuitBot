//! Post dispatch: X API calls for approved queue items.
//!
//! Handles routing by action type (tweet / reply / thread),
//! media upload, and full thread reply-chain posting with persistence.

use crate::content::deserialize_blocks_from_content;
use crate::storage::{self, DbPool};
use crate::x_api::XApiClient;

/// Post a reply to a tweet via toolkit.
pub(super) async fn post_reply(
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
pub(super) async fn post_tweet(
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
pub(super) async fn upload_media(
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
pub(super) async fn post_thread_and_persist(
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
pub(super) async fn persist_and_propagate_thread(
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
            super::queue::execute_loopback_for_thread(
                pool,
                account_id,
                item,
                root_tweet_id,
                child_ids,
            )
            .await;
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
