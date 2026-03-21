//! Private compose flows: tweet/thread orchestration, persistence, and X API helpers.

use axum::Json;
use serde_json::{json, Value};
use tuitbot_core::content::{
    serialize_blocks_for_storage, tweet_weighted_len, validate_thread_blocks, ThreadBlock,
    MAX_TWEET_CHARS,
};
use tuitbot_core::storage::{action_log, approval_queue, provenance, scheduled_content};
use tuitbot_core::x_api::{XApiClient, XApiHttpClient};

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;
use crate::ws::{AccountWsEvent, WsEvent};

use super::super::read_approval_mode;
use super::{build_provenance_input, ComposeRequest, ThreadBlockRequest};

pub(super) async fn compose_tweet_flow(
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
pub(super) async fn compose_thread_legacy_flow(
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
pub(super) async fn compose_thread_blocks_flow(
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

        // Attach provenance links to the scheduled thread.
        if let Some(refs) = body.provenance.as_deref() {
            if !refs.is_empty() {
                let _ = provenance::insert_links_for(
                    &state.db,
                    &ctx.account_id,
                    "scheduled_content",
                    id,
                    refs,
                )
                .await;
            }
        }

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
        let can_post = super::super::can_post_for(state, &ctx.account_id).await;
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

            // Attach provenance links to the fallback-scheduled thread.
            if let Some(refs) = body.provenance.as_deref() {
                if !refs.is_empty() {
                    let _ = provenance::insert_links_for(
                        &state.db,
                        &ctx.account_id,
                        "scheduled_content",
                        id,
                        refs,
                    )
                    .await;
                }
            }

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

        // Attach provenance links to the scheduled content if present.
        if let Some(refs) = body.provenance.as_deref() {
            if !refs.is_empty() {
                let _ = provenance::insert_links_for(
                    &state.db,
                    &ctx.account_id,
                    "scheduled_content",
                    id,
                    refs,
                )
                .await;
            }
        }

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
        let can_post = super::super::can_post_for(state, &ctx.account_id).await;
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

            // Attach provenance links to the fallback-scheduled content.
            if let Some(refs) = body.provenance.as_deref() {
                if !refs.is_empty() {
                    let _ = provenance::insert_links_for(
                        &state.db,
                        &ctx.account_id,
                        "scheduled_content",
                        id,
                        refs,
                    )
                    .await;
                }
            }

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
    let config = super::super::read_effective_config(state, &ctx.account_id).await?;

    match config.x_api.provider_backend.as_str() {
        "scraper" => {
            let account_data =
                tuitbot_core::storage::accounts::account_data_dir(&state.data_dir, &ctx.account_id);
            // Use the shared health handle from AppState so each request's outcome
            // aggregates into the tracker that /health reads, rather than being discarded.
            let client = if let Some(ref health) = state.scraper_health {
                tuitbot_core::x_api::LocalModeXClient::with_session_and_health(
                    config.x_api.scraper_allow_mutations,
                    &account_data,
                    health.clone(),
                )
                .await
            } else {
                tuitbot_core::x_api::LocalModeXClient::with_session(
                    config.x_api.scraper_allow_mutations,
                    &account_data,
                )
                .await
            };
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
