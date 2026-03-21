//! POST /api/assist/hooks — generate 5 differentiated hook options.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use tuitbot_core::content::generator::HookOption;
use tuitbot_core::context::retrieval::VaultCitation;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::routes::rag_helpers::{resolve_composer_rag_context, resolve_selection_rag_context};
use crate::state::AppState;

use super::get_generator;

// ---------------------------------------------------------------------------
// Request / Response DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistHooksRequest {
    pub topic: String,
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct AssistHooksResponse {
    pub hooks: Vec<HookOptionDto>,
    pub topic: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

#[derive(Serialize)]
pub struct HookOptionDto {
    pub style: String,
    pub text: String,
    pub char_count: usize,
    pub confidence: String,
}

impl From<HookOption> for HookOptionDto {
    fn from(h: HookOption) -> Self {
        Self {
            style: h.style,
            text: h.text,
            char_count: h.char_count,
            confidence: h.confidence,
        }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn assist_hooks(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<AssistHooksRequest>,
) -> Result<Json<AssistHooksResponse>, ApiError> {
    let gen = get_generator(&state, &ctx.account_id).await?;

    // Resolve RAG context: selection session > selected node IDs > none
    let (prompt_block, selected_text, citations) = if let Some(ref sid) = body.session_id {
        let sel_ctx = resolve_selection_rag_context(&state, &ctx.account_id, sid).await;
        match sel_ctx {
            Some(sc) => {
                let pb = sc.draft_context.as_ref().map(|c| c.prompt_block.clone());
                let cites = sc
                    .draft_context
                    .as_ref()
                    .map(|c| c.vault_citations.clone())
                    .unwrap_or_default();
                (pb, sc.selected_text, cites)
            }
            None => (None, None, vec![]),
        }
    } else {
        let node_ids = body.selected_node_ids.as_deref();
        let rag = resolve_composer_rag_context(&state, &ctx.account_id, node_ids).await;
        let pb = rag.as_ref().map(|c| c.prompt_block.clone());
        let cites = rag
            .as_ref()
            .map(|c| c.vault_citations.clone())
            .unwrap_or_default();
        (pb, None, cites)
    };

    // Build the combined RAG context string
    let rag_context = match (&prompt_block, &selected_text) {
        (Some(pb), Some(st)) => Some(format!("{pb}\n\nSelected text from vault:\n{st}")),
        (Some(pb), None) => Some(pb.clone()),
        (None, Some(st)) => Some(format!("Selected text from vault:\n{st}")),
        (None, None) => None,
    };

    let output = gen
        .generate_hooks(&body.topic, rag_context.as_deref())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let hooks = output.hooks.into_iter().map(HookOptionDto::from).collect();

    Ok(Json(AssistHooksResponse {
        hooks,
        topic: body.topic,
        vault_citations: citations,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_request_deserializes_minimal() {
        let json = r#"{"topic": "Rust testing"}"#;
        let req: AssistHooksRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.topic, "Rust testing");
        assert!(req.selected_node_ids.is_none());
        assert!(req.session_id.is_none());
    }

    #[test]
    fn hooks_request_deserializes_full() {
        let json = r#"{"topic": "Rust", "selected_node_ids": [1, 2], "session_id": "abc-123"}"#;
        let req: AssistHooksRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.topic, "Rust");
        assert_eq!(req.selected_node_ids.unwrap(), vec![1, 2]);
        assert_eq!(req.session_id.unwrap(), "abc-123");
    }

    #[test]
    fn hooks_response_omits_empty_citations() {
        let resp = AssistHooksResponse {
            hooks: vec![HookOptionDto {
                style: "question".to_string(),
                text: "Is testing overrated?".to_string(),
                char_count: 20,
                confidence: "high".to_string(),
            }],
            topic: "testing".to_string(),
            vault_citations: vec![],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(!json.contains("vault_citations"));
        assert!(json.contains("\"style\":\"question\""));
    }

    #[test]
    fn hook_option_dto_from_core() {
        let core = HookOption {
            style: "tip".to_string(),
            text: "Use cargo nextest".to_string(),
            char_count: 17,
            confidence: "high".to_string(),
        };
        let dto: HookOptionDto = core.into();
        assert_eq!(dto.style, "tip");
        assert_eq!(dto.text, "Use cargo nextest");
        assert_eq!(dto.char_count, 17);
        assert_eq!(dto.confidence, "high");
    }
}
