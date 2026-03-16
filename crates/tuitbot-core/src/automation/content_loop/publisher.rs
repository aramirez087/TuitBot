//! Scheduled-content posting — single tweets and multi-tweet thread chains.
//!
//! Implements the `try_post_scheduled` and `post_scheduled_thread` methods
//! on [`ContentLoop`].

use super::{ContentLoop, ContentResult};

impl ContentLoop {
    /// Check for scheduled content due for posting and post it if found.
    ///
    /// Returns `Some(ContentResult)` if a scheduled item was handled,
    /// `None` if no scheduled items are due.
    pub(super) async fn try_post_scheduled(&self) -> Option<ContentResult> {
        match self.storage.next_scheduled_item().await {
            Ok(Some((id, content_type, content))) => {
                tracing::info!(
                    id = id,
                    content_type = %content_type,
                    "Posting scheduled content"
                );

                let preview = &content[..content.len().min(80)];

                if self.dry_run {
                    tracing::info!(
                        "DRY RUN: Would post scheduled {} (id={}): \"{}\"",
                        content_type,
                        id,
                        preview
                    );
                    let _ = self
                        .storage
                        .log_action(
                            &content_type,
                            "dry_run",
                            &format!("Scheduled id={id}: {preview}"),
                        )
                        .await;
                } else if content_type == "thread" {
                    // Post thread as a reply chain.
                    match self.post_scheduled_thread(id, &content).await {
                        Ok(()) => {}
                        Err(e) => {
                            return Some(ContentResult::Failed {
                                error: format!("Scheduled thread failed: {e}"),
                            });
                        }
                    }
                } else if let Err(e) = self.storage.post_tweet("scheduled", &content).await {
                    tracing::error!(error = %e, "Failed to post scheduled content");
                    return Some(ContentResult::Failed {
                        error: format!("Scheduled post failed: {e}"),
                    });
                } else {
                    let _ = self.storage.mark_scheduled_posted(id, None).await;
                    let _ = self
                        .storage
                        .log_action(
                            &content_type,
                            "success",
                            &format!("Scheduled id={id}: {preview}"),
                        )
                        .await;
                }

                Some(ContentResult::Posted {
                    topic: format!("scheduled:{id}"),
                    content,
                })
            }
            Ok(None) => None,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to check scheduled content");
                None
            }
        }
    }

    /// Post a scheduled thread as a reply chain using the `ThreadPoster`.
    async fn post_scheduled_thread(&self, id: i64, content: &str) -> Result<(), String> {
        let poster = self.thread_poster.as_ref().ok_or_else(|| {
            "No thread poster configured — cannot post scheduled threads".to_string()
        })?;

        // Parse blocks from stored content (versioned JSON or legacy string array).
        let tweets: Vec<String> =
            if let Some(blocks) = crate::content::deserialize_blocks_from_content(content) {
                let mut sorted = blocks;
                sorted.sort_by_key(|b| b.order);
                sorted.into_iter().map(|b| b.text).collect()
            } else if let Ok(arr) = serde_json::from_str::<Vec<String>>(content) {
                arr
            } else {
                return Err(format!("Cannot parse thread content for scheduled id={id}"));
            };

        if tweets.is_empty() {
            return Err(format!("Scheduled thread id={id} has no tweets"));
        }

        // Post first tweet, then reply chain.
        let mut prev_id: Option<String> = None;
        for (i, text) in tweets.iter().enumerate() {
            let result = if let Some(ref reply_to) = prev_id {
                poster.reply_to_tweet(reply_to, text).await
            } else {
                poster.post_tweet(text).await
            };

            match result {
                Ok(tweet_id) => prev_id = Some(tweet_id),
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        index = i,
                        "Scheduled thread id={id} failed at tweet {}/{}",
                        i + 1,
                        tweets.len()
                    );
                    return Err(format!(
                        "Thread failed at tweet {}/{}: {e}",
                        i + 1,
                        tweets.len()
                    ));
                }
            }
        }

        let _ = self
            .storage
            .mark_scheduled_posted(id, prev_id.as_deref())
            .await;
        let _ = self
            .storage
            .log_action(
                "thread",
                "success",
                &format!("Scheduled thread id={id}: {} tweets posted", tweets.len()),
            )
            .await;

        Ok(())
    }
}
