//! Publish step: execute a reply or tweet through the toolkit layer.
//!
//! Thin wrapper that routes all X API writes through `toolkit::write`.
//! Callers that need policy gating should check policy before calling this step.

use crate::toolkit;
use crate::x_api::XApiClient;

use super::WorkflowError;

/// Output from a publish operation.
#[derive(Debug, Clone)]
pub struct PublishOutput {
    /// The ID of the posted tweet.
    pub tweet_id: String,
    /// The text that was posted.
    pub text: String,
}

/// Publish a reply to a tweet via toolkit.
pub async fn reply(
    x_client: &dyn XApiClient,
    text: &str,
    in_reply_to_id: &str,
) -> Result<PublishOutput, WorkflowError> {
    let posted = toolkit::write::reply_to_tweet(x_client, text, in_reply_to_id, None).await?;
    Ok(PublishOutput {
        tweet_id: posted.id,
        text: posted.text,
    })
}

/// Publish an original tweet via toolkit.
pub async fn tweet(x_client: &dyn XApiClient, text: &str) -> Result<PublishOutput, WorkflowError> {
    let posted = toolkit::write::post_tweet(x_client, text, None).await?;
    Ok(PublishOutput {
        tweet_id: posted.id,
        text: posted.text,
    })
}

/// Publish a thread via toolkit.
pub async fn thread(
    x_client: &dyn XApiClient,
    tweets: &[String],
) -> Result<Vec<String>, WorkflowError> {
    let ids = toolkit::write::post_thread(x_client, tweets, None).await?;
    Ok(ids)
}
