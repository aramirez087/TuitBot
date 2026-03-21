//! WebSocket hub for real-time event streaming.
//!
//! Provides a `/api/ws` endpoint that streams server events to dashboard clients
//! via a `tokio::sync::broadcast` channel.
//!
//! Supports two authentication methods:
//! - Query parameter: `?token=<api_token>` (Tauri/API clients)
//! - Session cookie: `tuitbot_session=<token>` (web/LAN clients)

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tuitbot_core::auth::session;

use crate::state::AppState;

/// Wrapper that tags every [`WsEvent`] with the originating account.
///
/// Serializes flat thanks to `#[serde(flatten)]`, so the JSON looks like:
/// `{ "account_id": "...", "type": "ApprovalQueued", ... }`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountWsEvent {
    pub account_id: String,
    #[serde(flatten)]
    pub event: WsEvent,
}

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
        #[serde(default)]
        media_paths: Vec<String>,
    },
    /// An approval item's status was updated (approved, rejected, edited).
    ApprovalUpdated {
        id: i64,
        status: String,
        action_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        actor: Option<String>,
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
    /// Circuit breaker state changed.
    CircuitBreakerTripped {
        state: String,
        error_count: u32,
        cooldown_remaining_seconds: u64,
        timestamp: String,
    },
    /// A Ghostwriter selection was received from the Obsidian plugin.
    SelectionReceived { session_id: String },
    /// An error occurred.
    Error { message: String },
}

/// Query parameters for WebSocket authentication.
#[derive(Deserialize)]
pub struct WsQuery {
    /// API token passed as a query parameter (optional — cookie auth is fallback).
    pub token: Option<String>,
}

/// Extract the session cookie value from headers (exported for tests).
fn extract_session_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix("tuitbot_session=").map(|v| v.to_string())
            })
        })
}

/// `GET /api/ws` — WebSocket upgrade with token or cookie auth.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<WsQuery>,
) -> Response {
    // Strategy 1: Bearer token via query parameter
    if let Some(ref token) = params.token {
        if token == &state.api_token {
            return ws.on_upgrade(move |socket| handle_ws(socket, state));
        }
    }

    // Strategy 2: Session cookie
    if let Some(session_token) = extract_session_cookie(&headers) {
        if let Ok(Some(_)) = session::validate_session(&state.db, &session_token).await {
            return ws.on_upgrade(move |socket| handle_ws(socket, state));
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({"error": "unauthorized"})),
    )
        .into_response()
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
                let error_event = AccountWsEvent {
                    account_id: String::new(),
                    event: WsEvent::Error {
                        message: format!("{count} events dropped due to slow consumer"),
                    },
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- WsEvent serialization (tagged enum) ---

    #[test]
    fn action_performed_serializes_with_type_tag() {
        let event = WsEvent::ActionPerformed {
            action_type: "reply".into(),
            target: "@user".into(),
            content: "Hello!".into(),
            timestamp: "2026-03-15T12:00:00Z".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "ActionPerformed");
        assert_eq!(json["action_type"], "reply");
        assert_eq!(json["target"], "@user");
    }

    #[test]
    fn approval_queued_serializes() {
        let event = WsEvent::ApprovalQueued {
            id: 42,
            action_type: "tweet".into(),
            content: "Draft tweet".into(),
            media_paths: vec!["img.png".into()],
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "ApprovalQueued");
        assert_eq!(json["id"], 42);
        assert_eq!(json["media_paths"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn approval_updated_serializes_with_optional_actor() {
        let event = WsEvent::ApprovalUpdated {
            id: 1,
            status: "approved".into(),
            action_type: "tweet".into(),
            actor: Some("admin".into()),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["actor"], "admin");

        let event_no_actor = WsEvent::ApprovalUpdated {
            id: 1,
            status: "rejected".into(),
            action_type: "tweet".into(),
            actor: None,
        };
        let json2 = serde_json::to_value(&event_no_actor).unwrap();
        assert!(
            json2.get("actor").is_none(),
            "actor should be skipped when None"
        );
    }

    #[test]
    fn follower_update_serializes() {
        let event = WsEvent::FollowerUpdate {
            count: 1500,
            change: 25,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "FollowerUpdate");
        assert_eq!(json["count"], 1500);
        assert_eq!(json["change"], 25);
    }

    #[test]
    fn runtime_status_serializes() {
        let event = WsEvent::RuntimeStatus {
            running: true,
            active_loops: vec!["mentions".into(), "discovery".into()],
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "RuntimeStatus");
        assert_eq!(json["running"], true);
        assert_eq!(json["active_loops"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn tweet_discovered_serializes() {
        let event = WsEvent::TweetDiscovered {
            tweet_id: "123456".into(),
            author: "user1".into(),
            score: 0.95,
            timestamp: "2026-03-15T12:00:00Z".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "TweetDiscovered");
        assert_eq!(json["score"], 0.95);
    }

    #[test]
    fn action_skipped_serializes() {
        let event = WsEvent::ActionSkipped {
            action_type: "reply".into(),
            reason: "rate limited".into(),
            timestamp: "2026-03-15T12:00:00Z".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "ActionSkipped");
        assert_eq!(json["reason"], "rate limited");
    }

    #[test]
    fn content_scheduled_serializes() {
        let event = WsEvent::ContentScheduled {
            id: 7,
            content_type: "tweet".into(),
            scheduled_for: Some("2026-03-16T09:00:00Z".into()),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "ContentScheduled");
        assert_eq!(json["id"], 7);
        assert!(json["scheduled_for"].is_string());
    }

    #[test]
    fn circuit_breaker_tripped_serializes() {
        let event = WsEvent::CircuitBreakerTripped {
            state: "open".into(),
            error_count: 5,
            cooldown_remaining_seconds: 120,
            timestamp: "2026-03-15T12:00:00Z".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "CircuitBreakerTripped");
        assert_eq!(json["error_count"], 5);
        assert_eq!(json["cooldown_remaining_seconds"], 120);
    }

    #[test]
    fn error_event_serializes() {
        let event = WsEvent::Error {
            message: "something broke".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "Error");
        assert_eq!(json["message"], "something broke");
    }

    // --- AccountWsEvent flattening ---

    #[test]
    fn account_ws_event_flattens_correctly() {
        let event = AccountWsEvent {
            account_id: "acct-123".into(),
            event: WsEvent::FollowerUpdate {
                count: 100,
                change: 5,
            },
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["account_id"], "acct-123");
        assert_eq!(json["type"], "FollowerUpdate");
        assert_eq!(json["count"], 100);
    }

    #[test]
    fn account_ws_event_roundtrip() {
        let original = AccountWsEvent {
            account_id: "acct-456".into(),
            event: WsEvent::Error {
                message: "test error".into(),
            },
        };
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: AccountWsEvent = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.account_id, "acct-456");
        match deserialized.event {
            WsEvent::Error { message } => assert_eq!(message, "test error"),
            _ => panic!("expected Error variant"),
        }
    }

    // --- extract_session_cookie ---

    #[test]
    fn extract_session_cookie_present() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "other=foo; tuitbot_session=abc123; another=bar"
                .parse()
                .unwrap(),
        );
        let result = extract_session_cookie(&headers);
        assert_eq!(result.as_deref(), Some("abc123"));
    }

    #[test]
    fn extract_session_cookie_not_present() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "other=foo; another=bar".parse().unwrap());
        let result = extract_session_cookie(&headers);
        assert!(result.is_none());
    }

    #[test]
    fn extract_session_cookie_no_cookie_header() {
        let headers = HeaderMap::new();
        let result = extract_session_cookie(&headers);
        assert!(result.is_none());
    }

    // --- WsEvent deserialization ---

    #[test]
    fn ws_event_deserializes_from_json() {
        let json = r#"{"type":"Error","message":"test"}"#;
        let event: WsEvent = serde_json::from_str(json).unwrap();
        match event {
            WsEvent::Error { message } => assert_eq!(message, "test"),
            _ => panic!("expected Error variant"),
        }
    }

    #[test]
    fn all_event_variants_serialize_without_panic() {
        let events: Vec<WsEvent> = vec![
            WsEvent::ActionPerformed {
                action_type: "reply".into(),
                target: "t".into(),
                content: "c".into(),
                timestamp: "ts".into(),
            },
            WsEvent::ApprovalQueued {
                id: 1,
                action_type: "tweet".into(),
                content: "c".into(),
                media_paths: vec![],
            },
            WsEvent::ApprovalUpdated {
                id: 1,
                status: "s".into(),
                action_type: "a".into(),
                actor: None,
            },
            WsEvent::FollowerUpdate {
                count: 0,
                change: 0,
            },
            WsEvent::RuntimeStatus {
                running: false,
                active_loops: vec![],
            },
            WsEvent::TweetDiscovered {
                tweet_id: "t".into(),
                author: "a".into(),
                score: 0.0,
                timestamp: "ts".into(),
            },
            WsEvent::ActionSkipped {
                action_type: "a".into(),
                reason: "r".into(),
                timestamp: "ts".into(),
            },
            WsEvent::ContentScheduled {
                id: 1,
                content_type: "tweet".into(),
                scheduled_for: None,
            },
            WsEvent::CircuitBreakerTripped {
                state: "open".into(),
                error_count: 0,
                cooldown_remaining_seconds: 0,
                timestamp: "ts".into(),
            },
            WsEvent::SelectionReceived {
                session_id: "sess-1".into(),
            },
            WsEvent::Error {
                message: "err".into(),
            },
        ];
        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            assert!(!json.is_empty());
            // Round-trip
            let _: WsEvent = serde_json::from_str(&json).unwrap();
        }
    }
}
