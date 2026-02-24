//! WebSocket hub for real-time event streaming.
//!
//! Provides a `/api/ws` endpoint that streams server events to dashboard clients
//! via a `tokio::sync::broadcast` channel.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::state::AppState;

/// Events pushed to WebSocket clients.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    /// An automation action was performed (reply, tweet, thread, etc.).
    ActionPerformed {
        action_type: String,
        target: String,
        content: String,
        timestamp: String,
    },
    /// A new item was queued for approval.
    ApprovalQueued {
        id: i64,
        action_type: String,
        content: String,
    },
    /// An approval item's status was updated (approved, rejected, edited).
    ApprovalUpdated {
        id: i64,
        status: String,
        action_type: String,
    },
    /// Follower count changed.
    FollowerUpdate { count: i64, change: i64 },
    /// Automation runtime status changed.
    RuntimeStatus {
        running: bool,
        active_loops: Vec<String>,
    },
    /// A tweet was discovered and scored by the discovery loop.
    TweetDiscovered {
        tweet_id: String,
        author: String,
        score: f64,
        timestamp: String,
    },
    /// An action was skipped (rate limited, below threshold, safety filter).
    ActionSkipped {
        action_type: String,
        reason: String,
        timestamp: String,
    },
    /// A new content item was scheduled via the composer.
    ContentScheduled {
        id: i64,
        content_type: String,
        scheduled_for: Option<String>,
    },
    /// An error occurred.
    Error { message: String },
}

/// Query parameters for WebSocket authentication.
#[derive(Deserialize)]
pub struct WsQuery {
    /// API token passed as a query parameter.
    pub token: String,
}

/// `GET /api/ws?token=...` — WebSocket upgrade with token auth.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsQuery>,
) -> Response {
    // Authenticate via query parameter.
    if params.token != state.api_token {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({"error": "unauthorized"})),
        )
            .into_response();
    }

    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

/// Handle a single WebSocket connection.
///
/// Subscribes to the broadcast channel and forwards events as JSON text frames.
async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.event_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(event) => {
                let json = match serde_json::to_string(&event) {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to serialize WsEvent");
                        continue;
                    }
                };
                if socket.send(Message::Text(json.into())).await.is_err() {
                    // Client disconnected.
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                tracing::warn!(count, "WebSocket client lagged, events dropped");
                let error_event = WsEvent::Error {
                    message: format!("{count} events dropped due to slow consumer"),
                };
                if let Ok(json) = serde_json::to_string(&error_event) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}
