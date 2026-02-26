//! Rollback guidance matrix for mutation tools.
//!
//! Maps each mutation tool + its result to an advisory rollback action.
//! The rollback is metadata only — never auto-executed.

use serde::Serialize;
use serde_json::Value;

/// Advisory rollback information attached to mutation responses.
#[derive(Debug, Clone, Serialize)]
pub struct RollbackGuidance {
    /// Whether this mutation is reversible.
    pub reversible: bool,
    /// The tool to call to undo this mutation (if reversible).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub undo_tool: Option<String>,
    /// Parameters for the undo tool (if reversible).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub undo_params: Option<Value>,
    /// Human-readable note about reversal limitations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Generate rollback guidance for a mutation tool.
///
/// `result_data` is the parsed result JSON so we can extract IDs for undo params.
pub fn guidance_for(tool_name: &str, result_data: &Value) -> RollbackGuidance {
    match tool_name {
        "x_post_tweet" => {
            let tweet_id = result_data
                .get("id")
                .or_else(|| result_data.get("tweet_id"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            RollbackGuidance {
                reversible: !tweet_id.is_empty(),
                undo_tool: Some("x_delete_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: Some("Deleting removes the tweet permanently.".into()),
            }
        }
        "x_reply_to_tweet" => {
            let tweet_id = result_data
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            RollbackGuidance {
                reversible: !tweet_id.is_empty(),
                undo_tool: Some("x_delete_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: Some("Deleting the reply removes it from the conversation.".into()),
            }
        }
        "x_quote_tweet" => {
            let tweet_id = result_data
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            RollbackGuidance {
                reversible: !tweet_id.is_empty(),
                undo_tool: Some("x_delete_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: None,
            }
        }
        "x_delete_tweet" => RollbackGuidance {
            reversible: false,
            undo_tool: None,
            undo_params: None,
            note: Some("Deleted tweets cannot be restored.".into()),
        },
        "x_post_thread" => {
            let ids = result_data
                .get("thread_tweet_ids")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            RollbackGuidance {
                reversible: !ids.is_empty(),
                undo_tool: Some("x_delete_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_ids": ids})),
                note: Some(
                    "Delete each tweet individually. Start from last to preserve thread.".into(),
                ),
            }
        }
        "x_like_tweet" => {
            let tweet_id = extract_tweet_id(result_data);
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_unlike_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: None,
            }
        }
        "x_unlike_tweet" => {
            let tweet_id = extract_tweet_id(result_data);
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_like_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: None,
            }
        }
        "x_follow_user" => {
            let user_id = result_data
                .get("target_user_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_unfollow_user".into()),
                undo_params: Some(serde_json::json!({"target_user_id": user_id})),
                note: None,
            }
        }
        "x_unfollow_user" => {
            let user_id = result_data
                .get("target_user_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_follow_user".into()),
                undo_params: Some(serde_json::json!({"target_user_id": user_id})),
                note: None,
            }
        }
        "x_retweet" => {
            let tweet_id = extract_tweet_id(result_data);
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_unretweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: None,
            }
        }
        "x_unretweet" => {
            let tweet_id = extract_tweet_id(result_data);
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_retweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: None,
            }
        }
        "x_bookmark_tweet" => {
            let tweet_id = extract_tweet_id(result_data);
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_unbookmark_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: None,
            }
        }
        "x_unbookmark_tweet" => {
            let tweet_id = extract_tweet_id(result_data);
            RollbackGuidance {
                reversible: true,
                undo_tool: Some("x_bookmark_tweet".into()),
                undo_params: Some(serde_json::json!({"tweet_id": tweet_id})),
                note: None,
            }
        }
        "x_upload_media" => RollbackGuidance {
            reversible: false,
            undo_tool: None,
            undo_params: None,
            note: Some("Uploaded media expires after 24 hours if not attached to a tweet.".into()),
        },
        _ => RollbackGuidance {
            reversible: false,
            undo_tool: None,
            undo_params: None,
            note: Some(format!("No rollback guidance available for '{tool_name}'.")),
        },
    }
}

fn extract_tweet_id(data: &Value) -> String {
    data.get("tweet_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Serialize rollback guidance to JSON for storage.
pub fn guidance_to_json(guidance: &RollbackGuidance) -> Option<String> {
    serde_json::to_string(guidance).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_tweet_rollback() {
        let data = serde_json::json!({"id": "12345", "text": "hello"});
        let g = guidance_for("x_post_tweet", &data);
        assert!(g.reversible);
        assert_eq!(g.undo_tool.as_deref(), Some("x_delete_tweet"));
        assert_eq!(g.undo_params.as_ref().unwrap()["tweet_id"], "12345");
    }

    #[test]
    fn delete_tweet_not_reversible() {
        let g = guidance_for("x_delete_tweet", &Value::Null);
        assert!(!g.reversible);
        assert!(g.undo_tool.is_none());
    }

    #[test]
    fn like_unlike_symmetric() {
        let data = serde_json::json!({"tweet_id": "999"});
        let g_like = guidance_for("x_like_tweet", &data);
        assert!(g_like.reversible);
        assert_eq!(g_like.undo_tool.as_deref(), Some("x_unlike_tweet"));

        let g_unlike = guidance_for("x_unlike_tweet", &data);
        assert!(g_unlike.reversible);
        assert_eq!(g_unlike.undo_tool.as_deref(), Some("x_like_tweet"));
    }

    #[test]
    fn follow_unfollow_symmetric() {
        let data = serde_json::json!({"target_user_id": "42"});
        let g = guidance_for("x_follow_user", &data);
        assert!(g.reversible);
        assert_eq!(g.undo_tool.as_deref(), Some("x_unfollow_user"));
        assert_eq!(g.undo_params.as_ref().unwrap()["target_user_id"], "42");
    }

    #[test]
    fn thread_rollback_lists_ids() {
        let data = serde_json::json!({
            "thread_tweet_ids": ["1", "2", "3"],
            "tweet_count": 3
        });
        let g = guidance_for("x_post_thread", &data);
        assert!(g.reversible);
        let params = g.undo_params.unwrap();
        let ids = params["tweet_ids"].as_array().unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn unknown_tool_not_reversible() {
        let g = guidance_for("unknown_tool", &Value::Null);
        assert!(!g.reversible);
        assert!(g.note.as_deref().unwrap().contains("unknown_tool"));
    }
}
