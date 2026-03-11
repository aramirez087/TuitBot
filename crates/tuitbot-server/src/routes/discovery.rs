//! Discovery feed endpoints for browsing scored tweets and composing replies.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::context::retrieval::VaultCitation;
use tuitbot_core::storage::approval_queue::{self, ProvenanceInput};
use tuitbot_core::storage::provenance::ProvenanceRef;
use tuitbot_core::storage::{self};

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::routes::rag_helpers::resolve_composer_rag_context;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn get_generator(
    state: &AppState,
    account_id: &str,
) -> Result<Arc<ContentGenerator>, ApiError> {
    state
        .get_or_create_content_generator(account_id)
        .await
        .map_err(ApiError::BadRequest)
}

// ---------------------------------------------------------------------------
// GET /api/discovery/feed
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    pub max_score: Option<f64>,
    pub keyword: Option<String>,
    #[serde(default = "default_feed_limit")]
    pub limit: u32,
}

fn default_min_score() -> f64 {
    50.0
}
fn default_feed_limit() -> u32 {
    20
}

#[derive(Serialize)]
pub struct DiscoveryTweet {
    pub id: String,
    pub author_username: String,
    pub content: String,
    pub relevance_score: f64,
    pub matched_keyword: Option<String>,
    pub like_count: i64,
    pub retweet_count: i64,
    pub reply_count: i64,
    pub replied_to: bool,
    pub discovered_at: String,
}

pub async fn feed(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Query(q): Query<FeedQuery>,
) -> Result<Json<Vec<DiscoveryTweet>>, ApiError> {
    let rows = storage::tweets::get_discovery_feed_filtered_for(
        &state.db,
        &ctx.account_id,
        q.min_score,
        q.max_score,
        q.keyword.as_deref(),
        q.limit,
    )
    .await?;

    let tweets = rows
        .into_iter()
        .map(|t| DiscoveryTweet {
            id: t.id,
            author_username: t.author_username,
            content: t.content,
            relevance_score: t.relevance_score.unwrap_or(0.0),
            matched_keyword: t.matched_keyword,
            like_count: t.like_count,
            retweet_count: t.retweet_count,
            reply_count: t.reply_count,
            replied_to: t.replied_to != 0,
            discovered_at: t.discovered_at,
        })
        .collect();

    Ok(Json(tweets))
}

// ---------------------------------------------------------------------------
// GET /api/discovery/keywords
// ---------------------------------------------------------------------------

pub async fn keywords(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
) -> Result<Json<Vec<String>>, ApiError> {
    let kws = storage::tweets::get_distinct_keywords_for(&state.db, &ctx.account_id).await?;
    Ok(Json(kws))
}

// ---------------------------------------------------------------------------
// POST /api/discovery/{tweet_id}/compose-reply
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ComposeReplyRequest {
    #[serde(default)]
    pub mention_product: bool,
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
}

#[derive(Serialize)]
pub struct ComposeReplyResponse {
    pub content: String,
    pub tweet_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

pub async fn compose_reply(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(tweet_id): Path<String>,
    Json(body): Json<ComposeReplyRequest>,
) -> Result<Json<ComposeReplyResponse>, ApiError> {
    let gen = get_generator(&state, &ctx.account_id).await?;

    // Fetch the tweet content from discovered_tweets.
    let tweet = storage::tweets::get_tweet_by_id_for(&state.db, &ctx.account_id, &tweet_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Tweet {tweet_id} not found in discovered tweets"))
        })?;

    let node_ids = body.selected_node_ids.as_deref();
    let rag_context = resolve_composer_rag_context(&state, &ctx.account_id, node_ids).await;

    let prompt_block = rag_context.as_ref().map(|c| c.prompt_block.as_str());
    let citations = rag_context
        .as_ref()
        .map(|c| c.vault_citations.clone())
        .unwrap_or_default();

    let output = gen
        .generate_reply_with_context(
            &tweet.content,
            &tweet.author_username,
            body.mention_product,
            None,
            prompt_block,
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ComposeReplyResponse {
        content: output.text,
        tweet_id,
        vault_citations: citations,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/discovery/{tweet_id}/queue-reply
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct QueueReplyRequest {
    pub content: String,
    #[serde(default)]
    pub provenance: Option<Vec<ProvenanceRef>>,
}

pub async fn queue_reply(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(tweet_id): Path<String>,
    Json(body): Json<QueueReplyRequest>,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;

    // Block posting unless the backend can actually post for this account.
    crate::routes::content::require_post_capable(&state, &ctx.account_id).await?;

    if body.content.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "content must not be empty".to_string(),
        ));
    }

    // Look up author from discovered_tweets.
    let target_author = storage::tweets::get_tweet_by_id_for(&state.db, &ctx.account_id, &tweet_id)
        .await?
        .map(|t| t.author_username)
        .unwrap_or_default();

    // Build provenance input when citations are provided.
    let provenance_input = body.provenance.as_ref().map(|refs| ProvenanceInput {
        source_node_id: refs.first().and_then(|r| r.node_id),
        source_seed_id: None,
        source_chunks_json: "[]".to_string(),
        refs: refs.clone(),
    });

    let queue_id = approval_queue::enqueue_with_provenance_for(
        &state.db,
        &ctx.account_id,
        "reply",
        &tweet_id,
        &target_author,
        &body.content,
        "",  // topic
        "",  // archetype
        0.0, // score
        "[]",
        None, // reason
        None, // detected_risks
        provenance_input.as_ref(),
        None, // no scheduling intent — discovery replies post immediately
    )
    .await?;

    // Auto-approve for immediate posting.
    approval_queue::update_status_for(&state.db, &ctx.account_id, queue_id, "approved").await?;

    Ok(Json(json!({
        "approval_queue_id": queue_id,
        "tweet_id": tweet_id,
        "status": "queued_for_posting"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_reply_request_provenance_is_optional() {
        let json = r#"{"content": "Great reply!"}"#;
        let req: QueueReplyRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.content, "Great reply!");
        assert!(req.provenance.is_none());
    }

    #[test]
    fn queue_reply_request_with_provenance() {
        let json = r#"{
            "content": "Thanks!",
            "provenance": [{"node_id": 1, "chunk_id": 2, "source_path": "notes/foo.md"}]
        }"#;
        let req: QueueReplyRequest = serde_json::from_str(json).expect("deserialize");
        let refs = req.provenance.unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].node_id, Some(1));
    }

    #[test]
    fn compose_reply_request_selected_node_ids_optional() {
        let json = r#"{"mention_product": true}"#;
        let req: ComposeReplyRequest = serde_json::from_str(json).expect("deserialize");
        assert!(req.mention_product);
        assert!(req.selected_node_ids.is_none());
    }

    #[test]
    fn compose_reply_response_omits_empty_citations() {
        let resp = ComposeReplyResponse {
            content: "Nice!".to_string(),
            tweet_id: "123".to_string(),
            vault_citations: vec![],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(!json.contains("vault_citations"));
    }
}
