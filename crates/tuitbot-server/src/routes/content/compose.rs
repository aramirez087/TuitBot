//! Compose endpoints for tweets, threads, and unified compose.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::content::{
    serialize_blocks_for_storage, tweet_weighted_len, validate_thread_blocks, ThreadBlock,
    MAX_TWEET_CHARS,
};
use tuitbot_core::storage::{action_log, approval_queue, scheduled_content};
use tuitbot_core::x_api::{XApiClient, XApiHttpClient};

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::{AccountWsEvent, WsEvent};

use super::read_approval_mode;

/// A single thread block in an API request payload.
#[derive(Debug, Deserialize)]
pub struct ThreadBlockRequest {
    /// Client-generated stable UUID.
    pub id: String,
    /// Tweet text content.
    pub text: String,
    /// Per-block media file paths.
    #[serde(default)]
    pub media_paths: Vec<String>,
    /// Zero-based ordering index.
    pub order: u32,
}

impl ThreadBlockRequest {
    /// Convert to the core domain type.
    pub(crate) fn into_core(self) -> ThreadBlock {
        ThreadBlock {
            id: self.id,
            text: self.text,
            media_paths: self.media_paths,
            order: self.order,
        }
    }
}

/// Request body for composing a manual tweet.
#[derive(Deserialize)]
pub struct ComposeTweetRequest {
    /// The tweet text.
    pub text: String,
    /// Optional ISO 8601 timestamp to schedule the tweet.
    pub scheduled_for: Option<String>,
}

/// `POST /api/content/tweets` — compose and queue a manual tweet.
pub async fn compose_tweet(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<ComposeTweetRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let text = body.text.trim();
    if text.is_empty() {
        return Err(ApiError::BadRequest("text is required".to_string()));
    }

    // Check if approval mode is enabled.
    let approval_mode = read_approval_mode(&state, &ctx.account_id).await?;

    if approval_mode {
        let id = approval_queue::enqueue_for(
            &state.db,
            &ctx.account_id,
            "tweet",
            "", // no target tweet
            "", // no target author
            text,
            "", // no topic
            "", // no archetype
            0.0,
            "[]",
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ApprovalQueued {
                id,
                action_type: "tweet".to_string(),
                content: text.to_string(),
                media_paths: vec![],
            },
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else {
        // Without X API client in AppState, we can only acknowledge the intent.
        Ok(Json(json!({
            "status": "accepted",
            "text": text,
            "scheduled_for": body.scheduled_for,
        })))
    }
}

/// Request body for composing a manual thread.
#[derive(Deserialize)]
pub struct ComposeThreadRequest {
    /// The tweets forming the thread.
    pub tweets: Vec<String>,
    /// Optional ISO 8601 timestamp to schedule the thread.
    pub scheduled_for: Option<String>,
}

/// `POST /api/content/threads` — compose and queue a manual thread.
pub async fn compose_thread(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<ComposeThreadRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    if body.tweets.is_empty() {
        return Err(ApiError::BadRequest(
            "tweets array must not be empty".to_string(),
        ));
    }

    let approval_mode = read_approval_mode(&state, &ctx.account_id).await?;
    let combined = body.tweets.join("\n---\n");

    if approval_mode {
        let id = approval_queue::enqueue_for(
            &state.db,
            &ctx.account_id,
            "thread",
            "",
            "",
            &combined,
            "",
            "",
            0.0,
            "[]",
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ApprovalQueued {
                id,
                action_type: "thread".to_string(),
                content: combined,
                media_paths: vec![],
            },
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else {
        Ok(Json(json!({
            "status": "accepted",
            "tweet_count": body.tweets.len(),
            "scheduled_for": body.scheduled_for,
        })))
    }
}

/// Request body for the unified compose endpoint.
#[derive(Deserialize)]
pub struct ComposeRequest {
    /// Content type: "tweet" or "thread".
    pub content_type: String,
    /// Content text (string for tweet, JSON array string for thread).
    pub content: String,
    /// Optional ISO 8601 timestamp to schedule the content.
    pub scheduled_for: Option<String>,
    /// Optional local media file paths to attach (top-level, used for tweets).
    #[serde(default)]
    pub media_paths: Option<Vec<String>>,
    /// Optional structured thread blocks. Takes precedence over `content` for threads.
    #[serde(default)]
    pub blocks: Option<Vec<ThreadBlockRequest>>,
}

/// `POST /api/content/compose` — compose manual content (tweet or thread).
pub async fn compose(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(mut body): Json<ComposeRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    let blocks = body.blocks.take();

    match body.content_type.as_str() {
        "tweet" => compose_tweet_flow(&state, &ctx, &body).await,
        "thread" => {
            if let Some(blocks) = blocks {
                compose_thread_blocks_flow(&state, &ctx, &body, blocks).await
            } else {
                compose_thread_legacy_flow(&state, &ctx, &body).await
            }
        }
        _ => Err(ApiError::BadRequest(
            "content_type must be 'tweet' or 'thread'".to_string(),
        )),
    }
}

/// Handle tweet compose via the unified endpoint.
async fn compose_tweet_flow(
    state: &AppState,
    ctx: &AccountContext,
    body: &ComposeRequest,
) -> Result<Json<Value>, ApiError> {
    let content = body.content.trim().to_string();
    if content.is_empty() {
        return Err(ApiError::BadRequest("content is required".to_string()));
    }
    if tweet_weighted_len(&content) > MAX_TWEET_CHARS {
        return Err(ApiError::BadRequest(
            "tweet content must not exceed 280 characters".to_string(),
        ));
    }

    persist_content(state, ctx, body, &content).await
}

/// Handle legacy thread compose (content as JSON array of strings).
async fn compose_thread_legacy_flow(
    state: &AppState,
    ctx: &AccountContext,
    body: &ComposeRequest,
) -> Result<Json<Value>, ApiError> {
    let content = body.content.trim().to_string();
    if content.is_empty() {
        return Err(ApiError::BadRequest("content is required".to_string()));
    }

    let tweets: Vec<String> = serde_json::from_str(&content).map_err(|_| {
        ApiError::BadRequest("thread content must be a JSON array of strings".to_string())
    })?;

    if tweets.is_empty() {
        return Err(ApiError::BadRequest(
            "thread must contain at least one tweet".to_string(),
        ));
    }

    for (i, tweet) in tweets.iter().enumerate() {
        if tweet_weighted_len(tweet) > MAX_TWEET_CHARS {
            return Err(ApiError::BadRequest(format!(
                "tweet {} exceeds 280 characters",
                i + 1
            )));
        }
    }

    persist_content(state, ctx, body, &content).await
}

/// Handle structured thread blocks compose.
async fn compose_thread_blocks_flow(
    state: &AppState,
    ctx: &AccountContext,
    body: &ComposeRequest,
    block_requests: Vec<ThreadBlockRequest>,
) -> Result<Json<Value>, ApiError> {
    let core_blocks: Vec<ThreadBlock> = block_requests.into_iter().map(|b| b.into_core()).collect();

    validate_thread_blocks(&core_blocks).map_err(|e| ApiError::BadRequest(e.api_message()))?;

    let block_ids: Vec<String> = {
        let mut sorted = core_blocks.clone();
        sorted.sort_by_key(|b| b.order);
        sorted.iter().map(|b| b.id.clone()).collect()
    };

    let content = serialize_blocks_for_storage(&core_blocks);

    // Collect per-block media into a flat list for approval queue storage.
    let all_media: Vec<String> = {
        let mut sorted = core_blocks.clone();
        sorted.sort_by_key(|b| b.order);
        sorted.iter().flat_map(|b| b.media_paths.clone()).collect()
    };

    let approval_mode = read_approval_mode(state, &ctx.account_id).await?;

    if approval_mode {
        let media_json = serde_json::to_string(&all_media).unwrap_or_else(|_| "[]".to_string());
        let id = approval_queue::enqueue_for(
            &state.db,
            &ctx.account_id,
            "thread",
            "",
            "",
            &content,
            "",
            "",
            0.0,
            &media_json,
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ApprovalQueued {
                id,
                action_type: "thread".to_string(),
                content: content.clone(),
                media_paths: all_media,
            },
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
            "block_ids": block_ids,
        })))
    } else {
        let scheduled_for = body
            .scheduled_for
            .clone()
            .or_else(|| Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()));

        let id = scheduled_content::insert_for(
            &state.db,
            &ctx.account_id,
            "thread",
            &content,
            scheduled_for.as_deref(),
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ContentScheduled {
                id,
                content_type: "thread".to_string(),
                scheduled_for: scheduled_for.clone(),
            },
        });

        Ok(Json(json!({
            "status": "scheduled",
            "id": id,
            "block_ids": block_ids,
        })))
    }
}

/// Persist content via approval queue, scheduled content, or post directly.
async fn persist_content(
    state: &AppState,
    ctx: &AccountContext,
    body: &ComposeRequest,
    content: &str,
) -> Result<Json<Value>, ApiError> {
    let approval_mode = read_approval_mode(state, &ctx.account_id).await?;

    if approval_mode {
        let media_paths = body.media_paths.as_deref().unwrap_or(&[]);
        let media_json = serde_json::to_string(media_paths).unwrap_or_else(|_| "[]".to_string());
        let id = approval_queue::enqueue_for(
            &state.db,
            &ctx.account_id,
            &body.content_type,
            "",
            "",
            content,
            "",
            "",
            0.0,
            &media_json,
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ApprovalQueued {
                id,
                action_type: body.content_type.clone(),
                content: content.to_string(),
                media_paths: media_paths.to_vec(),
            },
        });

        Ok(Json(json!({
            "status": "queued_for_approval",
            "id": id,
        })))
    } else if body.scheduled_for.is_some() {
        // User explicitly chose a future time — save to calendar.
        let id = scheduled_content::insert_for(
            &state.db,
            &ctx.account_id,
            &body.content_type,
            content,
            body.scheduled_for.as_deref(),
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ContentScheduled {
                id,
                content_type: body.content_type.clone(),
                scheduled_for: body.scheduled_for.clone(),
            },
        });

        Ok(Json(json!({
            "status": "scheduled",
            "id": id,
        })))
    } else {
        // Immediate publish — try posting via X API directly.
        // If not configured for direct posting, save to calendar instead.
        let can_post = super::can_post_for(state, &ctx.account_id).await;
        if !can_post {
            let scheduled_for = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            let id = scheduled_content::insert_for(
                &state.db,
                &ctx.account_id,
                &body.content_type,
                content,
                Some(&scheduled_for),
            )
            .await?;

            let _ = state.event_tx.send(AccountWsEvent {
                account_id: ctx.account_id.clone(),
                event: WsEvent::ContentScheduled {
                    id,
                    content_type: body.content_type.clone(),
                    scheduled_for: Some(scheduled_for),
                },
            });

            return Ok(Json(json!({
                "status": "scheduled",
                "id": id,
            })));
        }

        try_post_now(state, ctx, &body.content_type, content).await
    }
}

/// Attempt to post a tweet directly via X API or cookie-auth transport.
async fn try_post_now(
    state: &AppState,
    ctx: &AccountContext,
    content_type: &str,
    content: &str,
) -> Result<Json<Value>, ApiError> {
    let config = super::read_effective_config(state, &ctx.account_id).await?;

    let posted = match config.x_api.provider_backend.as_str() {
        "scraper" => {
            // Use cookie-auth transport via LocalModeXClient with account-scoped data dir.
            let account_data =
                tuitbot_core::storage::accounts::account_data_dir(&state.data_dir, &ctx.account_id);
            let client = tuitbot_core::x_api::LocalModeXClient::with_session(
                config.x_api.scraper_allow_mutations,
                &account_data,
            );
            client
                .post_tweet(content)
                .await
                .map_err(|e| ApiError::Internal(format!("Failed to post tweet: {e}")))?
        }
        "x_api" => {
            // Use official X API with OAuth tokens.
            let token_path = tuitbot_core::storage::accounts::account_token_path(
                &state.data_dir,
                &ctx.account_id,
            );

            let access_token = state
                .get_x_access_token(&token_path, &ctx.account_id)
                .await
                .map_err(|e| {
                    ApiError::BadRequest(format!(
                        "X API authentication failed — re-link your account in Settings. ({e})"
                    ))
                })?;

            let client = XApiHttpClient::new(access_token);
            client
                .post_tweet(content)
                .await
                .map_err(|e| ApiError::Internal(format!("Failed to post tweet: {e}")))?
        }
        _ => {
            return Err(ApiError::BadRequest(
                "Direct posting requires X API credentials or a browser session. \
                 Configure in Settings → X API."
                    .to_string(),
            ));
        }
    };

    // Log the successful post.
    let metadata = json!({
        "tweet_id": posted.id,
        "content_type": content_type,
        "source": "compose",
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "tweet_posted",
        "success",
        Some(&format!("Posted tweet {}", posted.id)),
        Some(&metadata.to_string()),
    )
    .await;

    Ok(Json(json!({
        "status": "posted",
        "tweet_id": posted.id,
    })))
}
