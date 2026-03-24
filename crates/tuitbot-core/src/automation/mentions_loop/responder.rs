//! LLM reply generation, safety checks, and PostSender dispatch.

use super::{truncate, MentionResult, MentionsLoop};
use crate::automation::loop_helpers::{LoopStorage, LoopTweet};
use std::sync::Arc;

impl MentionsLoop {
    /// Process a single mention: safety check, generate reply, post.
    pub(crate) async fn process_mention(
        &self,
        mention: &LoopTweet,
        storage: &Arc<dyn LoopStorage>,
    ) -> MentionResult {
        // Check if already replied
        if self.safety.has_replied_to(&mention.id).await {
            tracing::debug!(tweet_id = %mention.id, "Already replied to mention, skipping");
            return MentionResult::Skipped {
                tweet_id: mention.id.clone(),
                reason: "already replied".to_string(),
            };
        }

        // Check rate limits
        if !self.safety.can_reply().await {
            tracing::warn!(tweet_id = %mention.id, "Reply rate limit reached, skipping");
            return MentionResult::Skipped {
                tweet_id: mention.id.clone(),
                reason: "rate limited".to_string(),
            };
        }

        // Generate reply with vault context (always mention product for direct mentions)
        let reply_output = match self
            .generator
            .generate_reply_with_rag(&mention.text, &mention.author_username, true)
            .await
        {
            Ok(output) => output,
            Err(e) => {
                tracing::error!(
                    tweet_id = %mention.id,
                    error = %e,
                    "Failed to generate reply for mention"
                );
                return MentionResult::Failed {
                    tweet_id: mention.id.clone(),
                    error: e.to_string(),
                };
            }
        };
        let reply_text = reply_output.text;

        tracing::info!(
            author = %mention.author_username,
            "Replied to mention from @{}",
            mention.author_username,
        );

        if self.dry_run {
            tracing::info!(
                "DRY RUN: Would reply to mention {} by @{}: \"{}\"",
                mention.id,
                mention.author_username,
                reply_text
            );
        } else {
            // Send to posting queue
            if let Err(e) = self.poster.send_reply(&mention.id, &reply_text).await {
                tracing::error!(
                    tweet_id = %mention.id,
                    error = %e,
                    "Failed to send reply to posting queue"
                );
                return MentionResult::Failed {
                    tweet_id: mention.id.clone(),
                    error: e.to_string(),
                };
            }

            // Record the reply
            if let Err(e) = self.safety.record_reply(&mention.id, &reply_text).await {
                tracing::warn!(
                    tweet_id = %mention.id,
                    error = %e,
                    "Failed to record reply (post may have been sent)"
                );
            }
        }

        // Log to action log (even dry-run records discovered tweets)
        let _ = storage
            .log_action(
                "mention_reply",
                if self.dry_run { "dry_run" } else { "success" },
                &format!(
                    "Reply to @{}: {}",
                    mention.author_username,
                    truncate(&reply_text, 50)
                ),
            )
            .await;

        MentionResult::Replied {
            tweet_id: mention.id.clone(),
            author: mention.author_username.clone(),
            reply_text,
        }
    }
}
