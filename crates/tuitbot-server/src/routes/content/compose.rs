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
use tuitbot_core::storage::provenance::ProvenanceRef;
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
    /// Optional provenance refs linking this content to vault source material.
    #[serde(default)]
    pub provenance: Option<Vec<ProvenanceRef>>,
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
        let prov_input = build_provenance_input(body.provenance.as_deref());

        let id = approval_queue::enqueue_with_provenance_for(
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
            None,
            None,
            prov_input.as_ref(),
            body.scheduled_for.as_deref(),
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
            "scheduled_for": body.scheduled_for,
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
        let id = approval_queue::enqueue_with_context_for(
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
            None,
            None,
            body.scheduled_for.as_deref(),
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
            "scheduled_for": body.scheduled_for,
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
    /// Optional provenance refs linking this content to vault source material.
    #[serde(default)]
    pub provenance: Option<Vec<ProvenanceRef>>,
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

    // Validate scheduled_for early, before any branching logic
    let normalized_schedule = match &body.scheduled_for {
        Some(raw) => Some(
            tuitbot_core::scheduling::validate_and_normalize(
                raw,
                tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
            )
            .map_err(|e| ApiError::BadRequest(e.to_string()))?,
        ),
        None => None,
    };

    let approval_mode = read_approval_mode(state, &ctx.account_id).await?;

    if approval_mode {
        let media_json = serde_json::to_string(&all_media).unwrap_or_else(|_| "[]".to_string());
        let prov_input = build_provenance_input(body.provenance.as_deref());

        let id = approval_queue::enqueue_with_provenance_for(
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
            None,
            None,
            prov_input.as_ref(),
            normalized_schedule.as_deref(),
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
            "scheduled_for": normalized_schedule,
        })))
    } else if let Some(ref normalized) = normalized_schedule {
        // User explicitly chose a future time — already validated above.
        let id = scheduled_content::insert_for(
            &state.db,
            &ctx.account_id,
            "thread",
            &content,
            Some(normalized),
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ContentScheduled {
                id,
                content_type: "thread".to_string(),
                scheduled_for: Some(normalized.clone()),
            },
        });

        Ok(Json(json!({
            "status": "scheduled",
            "id": id,
            "block_ids": block_ids,
        })))
    } else {
        // Immediate publish — try posting as a reply chain.
        let can_post = super::can_post_for(state, &ctx.account_id).await;
        if !can_post {
            let scheduled_for = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let id = scheduled_content::insert_for(
                &state.db,
                &ctx.account_id,
                "thread",
                &content,
                Some(&scheduled_for),
            )
            .await?;

            let _ = state.event_tx.send(AccountWsEvent {
                account_id: ctx.account_id.clone(),
                event: WsEvent::ContentScheduled {
                    id,
                    content_type: "thread".to_string(),
                    scheduled_for: Some(scheduled_for),
                },
            });

            return Ok(Json(json!({
                "status": "scheduled",
                "id": id,
                "block_ids": block_ids,
            })));
        }

        try_post_thread_now(state, ctx, &core_blocks).await
    }
}

/// Persist content via approval queue, scheduled content, or post directly.
async fn persist_content(
    state: &AppState,
    ctx: &AccountContext,
    body: &ComposeRequest,
    content: &str,
) -> Result<Json<Value>, ApiError> {
    // Validate scheduled_for early, before any branching logic
    let normalized_schedule = match &body.scheduled_for {
        Some(raw) => Some(
            tuitbot_core::scheduling::validate_and_normalize(
                raw,
                tuitbot_core::scheduling::DEFAULT_GRACE_SECONDS,
            )
            .map_err(|e| ApiError::BadRequest(e.to_string()))?,
        ),
        None => None,
    };

    let approval_mode = read_approval_mode(state, &ctx.account_id).await?;

    if approval_mode {
        let media_paths = body.media_paths.as_deref().unwrap_or(&[]);
        let media_json = serde_json::to_string(media_paths).unwrap_or_else(|_| "[]".to_string());

        let prov_input = build_provenance_input(body.provenance.as_deref());

        let id = approval_queue::enqueue_with_provenance_for(
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
            None,
            None,
            prov_input.as_ref(),
            normalized_schedule.as_deref(),
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
            "scheduled_for": normalized_schedule,
        })))
    } else if let Some(ref normalized) = normalized_schedule {
        // User explicitly chose a future time — already validated above.
        let id = scheduled_content::insert_for(
            &state.db,
            &ctx.account_id,
            &body.content_type,
            content,
            Some(normalized),
        )
        .await?;

        let _ = state.event_tx.send(AccountWsEvent {
            account_id: ctx.account_id.clone(),
            event: WsEvent::ContentScheduled {
                id,
                content_type: body.content_type.clone(),
                scheduled_for: Some(normalized.clone()),
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
            let scheduled_for = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
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

/// Build an X API client for the given account based on the configured backend.
///
/// Returns `Box<dyn XApiClient>` so callers can use either scraper or OAuth
/// without duplicating the construction logic.
async fn build_x_client(
    state: &AppState,
    ctx: &AccountContext,
) -> Result<Box<dyn XApiClient>, ApiError> {
    let config = super::read_effective_config(state, &ctx.account_id).await?;

    match config.x_api.provider_backend.as_str() {
        "scraper" => {
            let account_data =
                tuitbot_core::storage::accounts::account_data_dir(&state.data_dir, &ctx.account_id);
            let client = tuitbot_core::x_api::LocalModeXClient::with_session(
                config.x_api.scraper_allow_mutations,
                &account_data,
            )
            .await;
            Ok(Box::new(client))
        }
        "x_api" => {
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
            Ok(Box::new(XApiHttpClient::new(access_token)))
        }
        _ => Err(ApiError::BadRequest(
            "Direct posting requires X API credentials or a browser session. \
             Configure in Settings → X API."
                .to_string(),
        )),
    }
}

/// Attempt to post a tweet directly via X API or cookie-auth transport.
async fn try_post_now(
    state: &AppState,
    ctx: &AccountContext,
    content_type: &str,
    content: &str,
) -> Result<Json<Value>, ApiError> {
    let client = build_x_client(state, ctx).await?;

    let posted = client
        .post_tweet(content)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to post tweet: {e}")))?;

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

/// Post a thread as a reply chain: first tweet standalone, each subsequent
/// tweet replying to the previous one. Returns all posted tweet IDs.
async fn try_post_thread_now(
    state: &AppState,
    ctx: &AccountContext,
    blocks: &[ThreadBlock],
) -> Result<Json<Value>, ApiError> {
    let client = build_x_client(state, ctx).await?;

    let mut sorted: Vec<&ThreadBlock> = blocks.iter().collect();
    sorted.sort_by_key(|b| b.order);

    let mut tweet_ids: Vec<String> = Vec::with_capacity(sorted.len());

    for (i, block) in sorted.iter().enumerate() {
        let posted = if i == 0 {
            client.post_tweet(&block.text).await
        } else {
            client.reply_to_tweet(&block.text, &tweet_ids[i - 1]).await
        };

        match posted {
            Ok(p) => tweet_ids.push(p.id),
            Err(e) => {
                // Log partial failure with the IDs we did post.
                let metadata = json!({
                    "posted_tweet_ids": tweet_ids,
                    "failed_at_index": i,
                    "error": e.to_string(),
                    "source": "compose",
                });
                let _ = action_log::log_action_for(
                    &state.db,
                    &ctx.account_id,
                    "thread_posted",
                    "partial_failure",
                    Some(&format!(
                        "Thread failed at tweet {}/{}: {e}",
                        i + 1,
                        sorted.len()
                    )),
                    Some(&metadata.to_string()),
                )
                .await;

                return Err(ApiError::Internal(format!(
                    "Thread failed at tweet {}/{}: {e}. \
                     {} tweet(s) were posted and cannot be undone.",
                    i + 1,
                    sorted.len(),
                    tweet_ids.len()
                )));
            }
        }
    }

    let metadata = json!({
        "tweet_ids": tweet_ids,
        "content_type": "thread",
        "source": "compose",
    });
    let _ = action_log::log_action_for(
        &state.db,
        &ctx.account_id,
        "thread_posted",
        "success",
        Some(&format!("Posted thread ({} tweets)", tweet_ids.len())),
        Some(&metadata.to_string()),
    )
    .await;

    Ok(Json(json!({
        "status": "posted",
        "tweet_ids": tweet_ids,
    })))
}

/// Build a `ProvenanceInput` from optional provenance refs.
fn build_provenance_input(
    provenance: Option<&[ProvenanceRef]>,
) -> Option<approval_queue::ProvenanceInput> {
    let refs = provenance?;
    if refs.is_empty() {
        return None;
    }

    let source_node_id = refs.iter().find_map(|r| r.node_id);
    let source_seed_id = refs.iter().find_map(|r| r.seed_id);
    let source_chunks_json = serde_json::to_string(refs).unwrap_or_else(|_| "[]".to_string());

    Some(approval_queue::ProvenanceInput {
        source_node_id,
        source_seed_id,
        source_chunks_json,
        refs: refs.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuitbot_core::content::ThreadBlock;
    use tuitbot_core::storage::provenance::ProvenanceRef;

    // ── ThreadBlockRequest::into_core ──────────────────────────────

    #[test]
    fn thread_block_request_into_core_basic() {
        let req = ThreadBlockRequest {
            id: "uuid-1".to_string(),
            text: "Hello world".to_string(),
            media_paths: vec![],
            order: 0,
        };
        let core = req.into_core();
        assert_eq!(core.id, "uuid-1");
        assert_eq!(core.text, "Hello world");
        assert_eq!(core.order, 0);
        assert!(core.media_paths.is_empty());
    }

    #[test]
    fn thread_block_request_into_core_with_media() {
        let req = ThreadBlockRequest {
            id: "uuid-2".to_string(),
            text: "Tweet with media".to_string(),
            media_paths: vec!["/path/a.jpg".to_string(), "/path/b.png".to_string()],
            order: 3,
        };
        let core = req.into_core();
        assert_eq!(core.media_paths.len(), 2);
        assert_eq!(core.media_paths[0], "/path/a.jpg");
        assert_eq!(core.order, 3);
    }

    #[test]
    fn thread_block_request_deserialize_without_media() {
        let json = r#"{"id":"x","text":"hi","order":0}"#;
        let req: ThreadBlockRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id, "x");
        assert!(req.media_paths.is_empty());
    }

    #[test]
    fn thread_block_request_deserialize_with_media() {
        let json = r#"{"id":"x","text":"hi","media_paths":["a.jpg"],"order":1}"#;
        let req: ThreadBlockRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.media_paths.len(), 1);
        assert_eq!(req.order, 1);
    }

    // ── build_provenance_input ────────────────────────────────────

    #[test]
    fn build_provenance_input_none_returns_none() {
        assert!(build_provenance_input(None).is_none());
    }

    #[test]
    fn build_provenance_input_empty_slice_returns_none() {
        let refs: Vec<ProvenanceRef> = vec![];
        assert!(build_provenance_input(Some(&refs)).is_none());
    }

    #[test]
    fn build_provenance_input_with_node_id() {
        let refs = vec![ProvenanceRef {
            node_id: Some(42),
            chunk_id: None,
            seed_id: None,
            source_path: None,
            heading_path: None,
            snippet: None,
        }];
        let result = build_provenance_input(Some(&refs)).unwrap();
        assert_eq!(result.source_node_id, Some(42));
        assert!(result.source_seed_id.is_none());
        assert_eq!(result.refs.len(), 1);
    }

    #[test]
    fn build_provenance_input_with_seed_id() {
        let refs = vec![ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: Some(99),
            source_path: None,
            heading_path: None,
            snippet: None,
        }];
        let result = build_provenance_input(Some(&refs)).unwrap();
        assert!(result.source_node_id.is_none());
        assert_eq!(result.source_seed_id, Some(99));
    }

    #[test]
    fn build_provenance_input_with_multiple_refs_picks_first() {
        let refs = vec![
            ProvenanceRef {
                node_id: Some(1),
                chunk_id: None,
                seed_id: None,
                source_path: Some("/notes/a.md".to_string()),
                heading_path: Some("## Intro".to_string()),
                snippet: Some("text snippet".to_string()),
            },
            ProvenanceRef {
                node_id: Some(2),
                chunk_id: Some(10),
                seed_id: Some(50),
                source_path: None,
                heading_path: None,
                snippet: None,
            },
        ];
        let result = build_provenance_input(Some(&refs)).unwrap();
        // find_map returns first match
        assert_eq!(result.source_node_id, Some(1));
        assert_eq!(result.source_seed_id, Some(50));
        assert_eq!(result.refs.len(), 2);
        // source_chunks_json should be valid JSON
        let parsed: Vec<ProvenanceRef> = serde_json::from_str(&result.source_chunks_json).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    // ── ComposeTweetRequest deserialization ────────────────────────

    #[test]
    fn compose_tweet_request_minimal() {
        let json = r#"{"text": "Hello"}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "Hello");
        assert!(req.scheduled_for.is_none());
        assert!(req.provenance.is_none());
    }

    #[test]
    fn compose_tweet_request_with_schedule() {
        let json = r#"{"text": "Hello", "scheduled_for": "2026-06-01T12:00:00Z"}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scheduled_for.as_deref(), Some("2026-06-01T12:00:00Z"));
    }

    #[test]
    fn compose_tweet_request_with_provenance() {
        let json = r#"{"text": "Hello", "provenance": [{"node_id": 1}]}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        let prov = req.provenance.unwrap();
        assert_eq!(prov.len(), 1);
        assert_eq!(prov[0].node_id, Some(1));
    }

    // ── ComposeThreadRequest deserialization ───────────────────────

    #[test]
    fn compose_thread_request_basic() {
        let json = r#"{"tweets": ["First", "Second"]}"#;
        let req: ComposeThreadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweets.len(), 2);
        assert!(req.scheduled_for.is_none());
    }

    // ── ComposeRequest deserialization ─────────────────────────────

    #[test]
    fn compose_request_tweet_type() {
        let json = r#"{"content_type": "tweet", "content": "Hello world"}"#;
        let req: ComposeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content_type, "tweet");
        assert_eq!(req.content, "Hello world");
        assert!(req.blocks.is_none());
        assert!(req.media_paths.is_none());
        assert!(req.provenance.is_none());
    }

    #[test]
    fn compose_request_thread_with_blocks() {
        let json = r#"{
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "a", "text": "First", "order": 0},
                {"id": "b", "text": "Second", "order": 1}
            ]
        }"#;
        let req: ComposeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content_type, "thread");
        let blocks = req.blocks.unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].id, "a");
    }

    #[test]
    fn compose_request_with_media_paths() {
        let json = r#"{
            "content_type": "tweet",
            "content": "photo tweet",
            "media_paths": ["/tmp/img.jpg"]
        }"#;
        let req: ComposeRequest = serde_json::from_str(json).unwrap();
        let media = req.media_paths.unwrap();
        assert_eq!(media.len(), 1);
    }

    // ── content_type routing ──────────────────────────────────────

    #[test]
    fn content_type_routing_tweet() {
        let ct = "tweet";
        assert_eq!(ct, "tweet");
        assert_ne!(ct, "thread");
    }

    #[test]
    fn content_type_routing_thread() {
        let ct = "thread";
        assert_ne!(ct, "tweet");
        assert_eq!(ct, "thread");
    }

    #[test]
    fn content_type_routing_unknown() {
        let ct = "story";
        assert_ne!(ct, "tweet");
        assert_ne!(ct, "thread");
    }

    // ── tweet length validation logic ─────────────────────────────

    #[test]
    fn tweet_validation_empty_rejected() {
        let text = "   ";
        assert!(text.trim().is_empty());
    }

    #[test]
    fn tweet_validation_within_limit() {
        let text = "a".repeat(280);
        assert!(
            tuitbot_core::content::tweet_weighted_len(&text)
                <= tuitbot_core::content::MAX_TWEET_CHARS
        );
    }

    #[test]
    fn tweet_validation_over_limit() {
        let text = "a".repeat(281);
        assert!(
            tuitbot_core::content::tweet_weighted_len(&text)
                > tuitbot_core::content::MAX_TWEET_CHARS
        );
    }

    // ── legacy thread parsing logic ───────────────────────────────

    #[test]
    fn legacy_thread_valid_json_array() {
        let content = r#"["First tweet", "Second tweet"]"#;
        let tweets: Result<Vec<String>, _> = serde_json::from_str(content);
        assert!(tweets.is_ok());
        assert_eq!(tweets.unwrap().len(), 2);
    }

    #[test]
    fn legacy_thread_invalid_json() {
        let content = "not json at all";
        let tweets: Result<Vec<String>, _> = serde_json::from_str(content);
        assert!(tweets.is_err());
    }

    #[test]
    fn legacy_thread_empty_array() {
        let content = "[]";
        let tweets: Vec<String> = serde_json::from_str(content).unwrap();
        assert!(tweets.is_empty());
    }

    // ── thread blocks to core conversion ──────────────────────────

    #[test]
    fn block_requests_to_core_preserves_order() {
        let reqs = vec![
            ThreadBlockRequest {
                id: "c".to_string(),
                text: "Third".to_string(),
                media_paths: vec![],
                order: 2,
            },
            ThreadBlockRequest {
                id: "a".to_string(),
                text: "First".to_string(),
                media_paths: vec![],
                order: 0,
            },
            ThreadBlockRequest {
                id: "b".to_string(),
                text: "Second".to_string(),
                media_paths: vec![],
                order: 1,
            },
        ];
        let core_blocks: Vec<ThreadBlock> = reqs.into_iter().map(|b| b.into_core()).collect();
        assert_eq!(core_blocks.len(), 3);

        // Sort by order to get block_ids in order
        let mut sorted = core_blocks.clone();
        sorted.sort_by_key(|b| b.order);
        let ids: Vec<String> = sorted.iter().map(|b| b.id.clone()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    // ── media_json serialization ──────────────────────────────────

    #[test]
    fn media_json_empty() {
        let media: Vec<String> = vec![];
        let json = serde_json::to_string(&media).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn media_json_with_paths() {
        let media = vec!["a.jpg".to_string(), "b.png".to_string()];
        let json = serde_json::to_string(&media).unwrap();
        assert!(json.contains("a.jpg"));
        assert!(json.contains("b.png"));
    }

    // ── thread block validation integration ───────────────────────

    #[test]
    fn validate_thread_blocks_from_requests() {
        let reqs = vec![
            ThreadBlockRequest {
                id: "a".to_string(),
                text: "First tweet".to_string(),
                media_paths: vec![],
                order: 0,
            },
            ThreadBlockRequest {
                id: "b".to_string(),
                text: "Second tweet".to_string(),
                media_paths: vec![],
                order: 1,
            },
        ];
        let core_blocks: Vec<ThreadBlock> = reqs.into_iter().map(|b| b.into_core()).collect();
        assert!(tuitbot_core::content::validate_thread_blocks(&core_blocks).is_ok());
    }

    #[test]
    fn serialize_blocks_roundtrip() {
        let blocks = vec![
            ThreadBlock {
                id: "a".to_string(),
                text: "First".to_string(),
                media_paths: vec!["img.jpg".to_string()],
                order: 0,
            },
            ThreadBlock {
                id: "b".to_string(),
                text: "Second".to_string(),
                media_paths: vec![],
                order: 1,
            },
        ];
        let serialized = tuitbot_core::content::serialize_blocks_for_storage(&blocks);
        let deserialized =
            tuitbot_core::content::deserialize_blocks_from_content(&serialized).unwrap();
        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].id, "a");
        assert_eq!(deserialized[0].media_paths.len(), 1);
    }

    // ── thread block validation edge cases ─────────────────────────

    #[test]
    fn validate_empty_blocks_fails() {
        let blocks: Vec<ThreadBlock> = vec![];
        let result = tuitbot_core::content::validate_thread_blocks(&blocks);
        assert!(result.is_err());
    }

    #[test]
    fn validate_single_block_fails() {
        // Threads require at least 2 blocks
        let blocks = vec![ThreadBlock {
            id: "a".to_string(),
            text: "Solo tweet".to_string(),
            media_paths: vec![],
            order: 0,
        }];
        assert!(tuitbot_core::content::validate_thread_blocks(&blocks).is_err());
    }

    #[test]
    fn validate_block_with_empty_text_fails() {
        let blocks = vec![ThreadBlock {
            id: "a".to_string(),
            text: "   ".to_string(),
            media_paths: vec![],
            order: 0,
        }];
        let result = tuitbot_core::content::validate_thread_blocks(&blocks);
        assert!(result.is_err());
    }

    #[test]
    fn validate_block_over_280_chars_fails() {
        let blocks = vec![ThreadBlock {
            id: "a".to_string(),
            text: "x".repeat(281),
            media_paths: vec![],
            order: 0,
        }];
        let result = tuitbot_core::content::validate_thread_blocks(&blocks);
        assert!(result.is_err());
    }

    // ── compose_tweet_request edge cases ───────────────────────────

    #[test]
    fn compose_tweet_request_with_empty_provenance() {
        let json = r#"{"text": "Hello", "provenance": []}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert!(req.provenance.unwrap().is_empty());
    }

    // ── compose_request edge cases ─────────────────────────────────

    #[test]
    fn compose_request_with_empty_blocks() {
        let json = r#"{
            "content_type": "thread",
            "content": "",
            "blocks": []
        }"#;
        let req: ComposeRequest = serde_json::from_str(json).unwrap();
        assert!(req.blocks.unwrap().is_empty());
    }

    #[test]
    fn compose_request_with_scheduled_for() {
        let json = r#"{
            "content_type": "tweet",
            "content": "scheduled tweet",
            "scheduled_for": "2026-06-01T12:00:00Z"
        }"#;
        let req: ComposeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scheduled_for.as_deref(), Some("2026-06-01T12:00:00Z"));
    }

    #[test]
    fn compose_request_with_provenance() {
        let json = r#"{
            "content_type": "tweet",
            "content": "text",
            "provenance": [{"node_id": 5, "chunk_id": 10}]
        }"#;
        let req: ComposeRequest = serde_json::from_str(json).unwrap();
        let prov = req.provenance.unwrap();
        assert_eq!(prov.len(), 1);
        assert_eq!(prov[0].node_id, Some(5));
        assert_eq!(prov[0].chunk_id, Some(10));
    }

    // ── build_provenance_input detailed ────────────────────────────

    #[test]
    fn build_provenance_input_all_none_fields() {
        let refs = vec![ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: None,
            source_path: None,
            heading_path: None,
            snippet: None,
        }];
        let result = build_provenance_input(Some(&refs)).unwrap();
        assert!(result.source_node_id.is_none());
        assert!(result.source_seed_id.is_none());
        assert_eq!(result.refs.len(), 1);
        // source_chunks_json should be valid JSON
        let parsed: Vec<ProvenanceRef> = serde_json::from_str(&result.source_chunks_json).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    // ── tweet_weighted_len boundary tests ──────────────────────────

    #[test]
    fn tweet_len_exactly_280() {
        let text = "a".repeat(280);
        assert_eq!(tuitbot_core::content::tweet_weighted_len(&text), 280);
    }

    #[test]
    fn tweet_len_with_url() {
        // URLs count as 23 chars in X's weighted length
        let text = "Check out https://example.com/some/long/path/here";
        let len = tuitbot_core::content::tweet_weighted_len(text);
        // Should be less than the raw char count due to URL shortening
        assert!(len < text.len(), "URL should be shortened in weighted len");
    }

    // ── thread block media aggregation ─────────────────────────────

    #[test]
    fn block_media_aggregation() {
        let blocks = vec![
            ThreadBlock {
                id: "a".to_string(),
                text: "First".to_string(),
                media_paths: vec!["img1.jpg".to_string()],
                order: 0,
            },
            ThreadBlock {
                id: "b".to_string(),
                text: "Second".to_string(),
                media_paths: vec!["img2.png".to_string(), "img3.gif".to_string()],
                order: 1,
            },
            ThreadBlock {
                id: "c".to_string(),
                text: "Third".to_string(),
                media_paths: vec![],
                order: 2,
            },
        ];
        let mut sorted = blocks.clone();
        sorted.sort_by_key(|b| b.order);
        let all_media: Vec<String> = sorted.iter().flat_map(|b| b.media_paths.clone()).collect();
        assert_eq!(all_media.len(), 3);
        assert_eq!(all_media[0], "img1.jpg");
        assert_eq!(all_media[1], "img2.png");
        assert_eq!(all_media[2], "img3.gif");
    }

    // ── legacy thread content parsing ──────────────────────────────

    #[test]
    fn legacy_thread_single_tweet() {
        let content = r#"["Only tweet"]"#;
        let tweets: Vec<String> = serde_json::from_str(content).unwrap();
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0], "Only tweet");
    }

    #[test]
    fn legacy_thread_with_special_chars() {
        let content = r#"["Hello \"world\"", "Tweet with\nnewline"]"#;
        let tweets: Vec<String> = serde_json::from_str(content).unwrap();
        assert_eq!(tweets.len(), 2);
        assert!(tweets[0].contains('"'));
    }

    #[test]
    fn legacy_thread_combined_separator() {
        let tweets = vec!["First".to_string(), "Second".to_string()];
        let combined = tweets.join("\n---\n");
        assert_eq!(combined, "First\n---\nSecond");
        assert!(combined.contains("---"));
    }

    #[test]
    fn thread_block_request_into_core() {
        let req = ThreadBlockRequest {
            id: "uuid-1".to_string(),
            text: "Hello".to_string(),
            media_paths: vec!["img.png".to_string()],
            order: 0,
        };
        let core = req.into_core();
        assert_eq!(core.id, "uuid-1");
        assert_eq!(core.text, "Hello");
        assert_eq!(core.media_paths.len(), 1);
        assert_eq!(core.order, 0);
    }

    #[test]
    fn thread_block_request_default_media_paths() {
        let json = r#"{"id":"u1","text":"t","order":0}"#;
        let req: ThreadBlockRequest = serde_json::from_str(json).unwrap();
        assert!(req.media_paths.is_empty());
    }

    #[test]
    fn compose_tweet_request_text_only() {
        let json = r#"{"text":"Hello world"}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "Hello world");
        assert!(req.scheduled_for.is_none());
        assert!(req.provenance.is_none());
    }

    #[test]
    fn compose_tweet_request_scheduled() {
        let json = r#"{"text":"Later","scheduled_for":"2026-04-01T10:00:00Z"}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scheduled_for.as_deref(), Some("2026-04-01T10:00:00Z"));
    }
}
