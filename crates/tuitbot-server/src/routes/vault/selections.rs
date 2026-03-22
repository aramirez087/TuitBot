//! Vault selection endpoints for receiving Ghostwriter selections from the
//! Obsidian plugin and retrieving stored selections.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use tuitbot_core::config::DeploymentMode;
use tuitbot_core::context::{graph_expansion, retrieval};
use tuitbot_core::storage::vault_selections;

use crate::account::AccountContext;
use crate::state::AppState;
use crate::ws::{AccountWsEvent, WsEvent};

// ---------------------------------------------------------------------------
// POST /api/vault/send-selection
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SendSelectionRequest {
    pub vault_name: String,
    pub file_path: String,
    pub selected_text: String,
    #[serde(default)]
    pub heading_context: Option<String>,
    #[serde(default)]
    pub selection_start_line: i64,
    #[serde(default)]
    pub selection_end_line: i64,
    #[serde(default)]
    pub note_title: Option<String>,
    #[serde(default)]
    pub frontmatter_tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct SendSelectionResponse {
    pub status: String,
    pub session_id: String,
    pub composer_url: String,
}

#[derive(Serialize)]
struct SelectionError {
    error: String,
    message: String,
}

fn error_response(status: StatusCode, error: &str, message: &str) -> Response {
    (
        status,
        Json(SelectionError {
            error: error.to_string(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

pub async fn send_selection(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<SendSelectionRequest>,
) -> Response {
    // Cloud mode privacy gate — vault selections must not be accepted in
    // cloud deployments because the raw text never leaves the user's device.
    if state.deployment_mode == DeploymentMode::Cloud {
        return error_response(
            StatusCode::FORBIDDEN,
            "cloud_mode_prohibited",
            "Vault selections are not available in cloud mode",
        );
    }

    // Validate vault_name
    if body.vault_name.is_empty() || body.vault_name.len() > 255 {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            "vault_name must be 1-255 characters",
        );
    }

    // Validate file_path
    if body.file_path.is_empty() || !body.file_path.ends_with(".md") {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            "file_path must be non-empty and end in .md",
        );
    }

    // Validate selected_text
    if body.selected_text.trim().is_empty() {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            "selected_text must not be empty",
        );
    }

    if body.selected_text.len() > 10_000 {
        return error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "selected_text exceeds 10000 character limit",
        );
    }

    // Validate line range
    if body.selection_end_line < body.selection_start_line {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            "selection_end_line must be >= selection_start_line",
        );
    }

    // Rate limit: 10 per minute per account
    match vault_selections::count_recent_for(&state.db, &ctx.account_id, 60).await {
        Ok(count) if count >= 10 => {
            return error_response(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Rate limit exceeded. Try again in 6 seconds.",
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to check selection rate limit");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Failed to check rate limit",
            );
        }
        _ => {}
    }

    // Generate session_id and expires_at
    let session_id = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(30))
        .unwrap_or_else(chrono::Utc::now)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    // Resolve block identity (best-effort)
    let (resolved_node_id, resolved_chunk_id) = retrieval::resolve_selection_identity(
        &state.db,
        &ctx.account_id,
        &body.file_path,
        body.heading_context.as_deref(),
    )
    .await
    .unwrap_or((None, None));

    // Serialize frontmatter_tags to JSON string
    let tags_json = body
        .frontmatter_tags
        .as_ref()
        .and_then(|tags| serde_json::to_string(tags).ok());

    // Insert
    let insert_result = vault_selections::insert_selection(
        &state.db,
        &ctx.account_id,
        &session_id,
        &body.vault_name,
        &body.file_path,
        &body.selected_text,
        body.heading_context.as_deref(),
        body.selection_start_line,
        body.selection_end_line,
        body.note_title.as_deref(),
        tags_json.as_deref(),
        resolved_node_id,
        resolved_chunk_id,
        &expires_at,
    )
    .await;

    if let Err(e) = insert_result {
        tracing::error!(error = %e, "failed to store vault selection");
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Failed to store selection",
        );
    }

    // Emit WebSocket event (best-effort, ignore send errors)
    let _ = state.event_tx.send(AccountWsEvent {
        account_id: ctx.account_id,
        event: WsEvent::SelectionReceived {
            session_id: session_id.clone(),
        },
    });

    let composer_url = format!("/compose?selection={session_id}");

    (
        StatusCode::OK,
        Json(SendSelectionResponse {
            status: "received".to_string(),
            session_id,
            composer_url,
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// GET /api/vault/selection/{session_id}
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct GetSelectionResponse {
    pub session_id: String,
    pub vault_name: String,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_context: Option<String>,
    pub selection_start_line: i64,
    pub selection_end_line: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_node_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_chunk_id: Option<i64>,
    pub privacy_envelope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_neighbors: Option<Vec<super::NeighborItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_state: Option<String>,
}

pub async fn get_selection(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Path(session_id): Path<String>,
) -> Response {
    let selection =
        match vault_selections::get_selection_by_session(&state.db, &ctx.account_id, &session_id)
            .await
        {
            Ok(Some(sel)) => sel,
            Ok(None) => {
                return error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "Selection not found or expired",
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to get vault selection");
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Failed to retrieve selection",
                );
            }
        };

    let is_cloud = state.deployment_mode == DeploymentMode::Cloud;

    // Cloud mode: omit selected_text (privacy invariant)
    let selected_text = if is_cloud {
        None
    } else {
        Some(selection.selected_text)
    };

    // Parse frontmatter_tags from JSON string
    let frontmatter_tags: Option<Vec<String>> = selection
        .frontmatter_tags
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    // Auto-expand graph neighbors when a resolved node exists.
    let (graph_neighbors, graph_state) = if let Some(node_id) = selection.resolved_node_id {
        let result = crate::routes::rag_helpers::resolve_graph_suggestions(
            &state,
            &ctx.account_id,
            node_id,
            graph_expansion::DEFAULT_MAX_NEIGHBORS,
        )
        .await;
        let state_str = serde_json::to_value(&result.graph_state)
            .ok()
            .and_then(|v| v.as_str().map(String::from));
        let items: Vec<super::NeighborItem> = result
            .neighbors
            .into_iter()
            .map(|n| super::NeighborItem::from_graph_neighbor(n, is_cloud))
            .collect();
        (Some(items), state_str)
    } else {
        (None, None)
    };

    (
        StatusCode::OK,
        Json(GetSelectionResponse {
            session_id: selection.session_id,
            vault_name: selection.vault_name,
            file_path: selection.file_path,
            selected_text,
            heading_context: selection.heading_context,
            selection_start_line: selection.selection_start_line,
            selection_end_line: selection.selection_end_line,
            note_title: selection.note_title,
            frontmatter_tags,
            resolved_node_id: selection.resolved_node_id,
            resolved_chunk_id: selection.resolved_chunk_id,
            privacy_envelope: state.deployment_mode.privacy_envelope().to_string(),
            graph_neighbors,
            graph_state,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_selection_request_deserialization() {
        let json = r##"{
            "vault_name": "marketing",
            "file_path": "notes/test.md",
            "selected_text": "Hello world",
            "heading_context": "# Title > ## Section",
            "selection_start_line": 10,
            "selection_end_line": 15,
            "note_title": "Test",
            "frontmatter_tags": ["rust", "async"]
        }"##;
        let req: SendSelectionRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.vault_name, "marketing");
        assert_eq!(req.file_path, "notes/test.md");
        assert_eq!(req.selected_text, "Hello world");
        assert_eq!(req.heading_context.as_deref(), Some("# Title > ## Section"));
        assert_eq!(req.selection_start_line, 10);
        assert_eq!(req.selection_end_line, 15);
        assert_eq!(req.note_title.as_deref(), Some("Test"));
        assert_eq!(
            req.frontmatter_tags,
            Some(vec!["rust".to_string(), "async".to_string()])
        );
    }

    #[test]
    fn send_selection_request_defaults() {
        let json = r#"{
            "vault_name": "v",
            "file_path": "n.md",
            "selected_text": "text"
        }"#;
        let req: SendSelectionRequest = serde_json::from_str(json).expect("deserialize");
        assert!(req.heading_context.is_none());
        assert_eq!(req.selection_start_line, 0);
        assert_eq!(req.selection_end_line, 0);
        assert!(req.note_title.is_none());
        assert!(req.frontmatter_tags.is_none());
    }

    #[test]
    fn send_selection_response_serialization() {
        let resp = SendSelectionResponse {
            status: "received".to_string(),
            session_id: "abc-123".to_string(),
            composer_url: "/compose?selection=abc-123".to_string(),
        };
        let json = serde_json::to_value(&resp).expect("serialize");
        assert_eq!(json["status"], "received");
        assert_eq!(json["session_id"], "abc-123");
        assert_eq!(json["composer_url"], "/compose?selection=abc-123");
    }

    #[test]
    fn get_selection_response_omits_none_fields() {
        let resp = GetSelectionResponse {
            session_id: "s".to_string(),
            vault_name: "v".to_string(),
            file_path: "f.md".to_string(),
            selected_text: None,
            heading_context: None,
            selection_start_line: 0,
            selection_end_line: 0,
            note_title: None,
            frontmatter_tags: None,
            resolved_node_id: None,
            resolved_chunk_id: None,
            privacy_envelope: "local_first".to_string(),
            graph_neighbors: None,
            graph_state: None,
        };
        let json = serde_json::to_value(&resp).expect("serialize");
        assert!(json.get("selected_text").is_none());
        assert!(json.get("heading_context").is_none());
        assert!(json.get("note_title").is_none());
        assert!(json.get("frontmatter_tags").is_none());
        assert!(json.get("resolved_node_id").is_none());
        assert!(json.get("resolved_chunk_id").is_none());
        assert!(json.get("graph_neighbors").is_none());
        assert!(json.get("graph_state").is_none());
        assert_eq!(json["privacy_envelope"], "local_first");
    }

    #[test]
    fn get_selection_response_includes_privacy_envelope() {
        let resp = GetSelectionResponse {
            session_id: "s".to_string(),
            vault_name: "v".to_string(),
            file_path: "f.md".to_string(),
            selected_text: Some("text".to_string()),
            heading_context: None,
            selection_start_line: 0,
            selection_end_line: 0,
            note_title: None,
            frontmatter_tags: None,
            resolved_node_id: None,
            resolved_chunk_id: None,
            privacy_envelope: "user_controlled".to_string(),
            graph_neighbors: None,
            graph_state: None,
        };
        let json = serde_json::to_value(&resp).expect("serialize");
        assert_eq!(json["privacy_envelope"], "user_controlled");
        assert_eq!(json["selected_text"], "text");
    }
}
