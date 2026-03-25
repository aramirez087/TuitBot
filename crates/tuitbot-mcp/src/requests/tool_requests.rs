//! Composite tool and universal X API request types.

use schemars::JsonSchema;
use serde::Deserialize;

// --- Composite Tools ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindReplyOpportunitiesRequest {
    /// Search query (defaults to product keywords joined with OR).
    pub query: Option<String>,
    /// Minimum score to include (defaults to scoring threshold from config).
    pub min_score: Option<f64>,
    /// Maximum number of results (default: 10).
    pub limit: Option<u32>,
    /// Only return tweets newer than this tweet ID.
    pub since_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DraftRepliesRequest {
    /// Tweet IDs of previously discovered candidates.
    pub candidate_ids: Vec<String>,
    /// Override the reply archetype (e.g., "agree_and_expand", "ask_question").
    pub archetype: Option<String>,
    /// Whether to potentially mention the product (default: false).
    pub mention_product: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProposeAndQueueRepliesRequest {
    /// Items to propose as replies.
    pub items: Vec<ProposeItem>,
    /// Whether to potentially mention the product in auto-generated replies (default: false).
    pub mention_product: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProposeItem {
    /// The tweet ID to reply to.
    pub candidate_id: String,
    /// Pre-drafted reply text. If omitted, generates one via LLM.
    pub pre_drafted_text: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateThreadPlanRequest {
    /// Topic for the thread.
    pub topic: String,
    /// Objective for the thread (e.g., "establish expertise", "drive traffic").
    pub objective: Option<String>,
    /// Target audience description.
    pub target_audience: Option<String>,
    /// Thread structure override (e.g., "transformation", "framework", "mistakes", "analysis").
    pub structure: Option<String>,
}

// --- Universal X API Request Tools ---

/// Key-value pair for query parameters and headers.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct KeyValue {
    /// Parameter key.
    pub key: String,
    /// Parameter value.
    pub value: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XGetRequest {
    /// API path (e.g., "/2/tweets/123"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
    /// Auto-paginate by following next_token (default: false). Only for GET.
    #[serde(default)]
    pub auto_paginate: bool,
    /// Maximum pages to fetch when auto_paginate is true (default: 10, max: 10).
    pub max_pages: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XPostRequest {
    /// API path (e.g., "/2/tweets"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// JSON request body as a string.
    pub body: Option<String>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XPutRequest {
    /// API path (e.g., "/2/lists/123"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// JSON request body as a string.
    pub body: Option<String>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XDeleteRequest {
    /// API path (e.g., "/2/tweets/123"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Composite Tools ---

    #[test]
    fn find_reply_opportunities_request_deser() {
        let json = r#"{"query": "rust", "min_score": 60.0, "limit": 5}"#;
        let req: FindReplyOpportunitiesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.query.as_deref(), Some("rust"));
        assert_eq!(req.limit, Some(5));
    }

    #[test]
    fn draft_replies_request_deser() {
        let json = r#"{"candidate_ids": ["c1", "c2"], "archetype": "agree_and_expand", "mention_product": false}"#;
        let req: DraftRepliesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.candidate_ids.len(), 2);
        assert_eq!(req.archetype.as_deref(), Some("agree_and_expand"));
    }

    #[test]
    fn propose_and_queue_replies_request_deser() {
        let json = r#"{"items": [{"candidate_id": "c1", "pre_drafted_text": "draft"}], "mention_product": true}"#;
        let req: ProposeAndQueueRepliesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.items.len(), 1);
        assert_eq!(req.items[0].candidate_id, "c1");
        assert_eq!(req.items[0].pre_drafted_text.as_deref(), Some("draft"));
    }

    #[test]
    fn propose_item_without_pre_drafted_text() {
        let json = r#"{"candidate_id": "c2"}"#;
        let item: ProposeItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.candidate_id, "c2");
        assert!(item.pre_drafted_text.is_none());
    }

    #[test]
    fn generate_thread_plan_request_deser() {
        let json = r#"{"topic": "testing", "objective": "educate", "structure": "framework"}"#;
        let req: GenerateThreadPlanRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.topic, "testing");
        assert_eq!(req.objective.as_deref(), Some("educate"));
        assert_eq!(req.structure.as_deref(), Some("framework"));
        assert!(req.target_audience.is_none());
    }

    // --- Universal X API Request ---

    #[test]
    fn key_value_deser() {
        let json = r#"{"key": "k", "value": "v"}"#;
        let kv: KeyValue = serde_json::from_str(json).unwrap();
        assert_eq!(kv.key, "k");
        assert_eq!(kv.value, "v");
    }

    #[test]
    fn x_get_request_deser() {
        let json = r#"{"path": "/2/tweets/123", "auto_paginate": true, "max_pages": 5}"#;
        let req: XGetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets/123");
        assert!(req.auto_paginate);
        assert_eq!(req.max_pages, Some(5));
        assert!(req.host.is_none());
    }

    #[test]
    fn x_get_request_auto_paginate_default() {
        let json = r#"{"path": "/2/tweets"}"#;
        let req: XGetRequest = serde_json::from_str(json).unwrap();
        assert!(!req.auto_paginate);
    }

    #[test]
    fn x_post_request_deser() {
        let json = r#"{"path": "/2/tweets", "body": "{\"text\":\"hi\"}"}"#;
        let req: XPostRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets");
        assert!(req.body.is_some());
    }

    #[test]
    fn x_put_request_deser() {
        let json = r#"{"path": "/2/lists/123", "host": "api.x.com"}"#;
        let req: XPutRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/lists/123");
        assert_eq!(req.host.as_deref(), Some("api.x.com"));
    }

    #[test]
    fn x_delete_request_deser() {
        let json = r#"{"path": "/2/tweets/123"}"#;
        let req: XDeleteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets/123");
    }

    // --- Debug impls ---

    #[test]
    fn key_value_debug() {
        let kv = KeyValue {
            key: "k".to_string(),
            value: "v".to_string(),
        };
        let debug = format!("{kv:?}");
        assert!(debug.contains("k"));
        assert!(debug.contains("v"));
    }

    #[test]
    fn key_value_clone() {
        let kv = KeyValue {
            key: "k".to_string(),
            value: "v".to_string(),
        };
        let kv2 = kv.clone();
        assert_eq!(kv2.key, "k");
        assert_eq!(kv2.value, "v");
    }

    #[test]
    fn propose_item_debug_and_clone() {
        let item = ProposeItem {
            candidate_id: "c1".to_string(),
            pre_drafted_text: Some("draft".to_string()),
        };
        let debug = format!("{item:?}");
        assert!(debug.contains("c1"));
        let clone = item.clone();
        assert_eq!(clone.candidate_id, "c1");
    }

    // --- Schema generation ---

    #[test]
    fn key_value_schema() {
        let schema = schemars::schema_for!(KeyValue);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }

    #[test]
    fn x_get_request_schema() {
        let schema = schemars::schema_for!(XGetRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
        assert!(json.contains("auto_paginate"));
    }

    #[test]
    fn find_reply_opportunities_request_schema() {
        let schema = schemars::schema_for!(FindReplyOpportunitiesRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("query"));
        assert!(json.contains("min_score"));
    }

    #[test]
    fn propose_and_queue_replies_request_schema() {
        let schema = schemars::schema_for!(ProposeAndQueueRepliesRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("items"));
    }

    #[test]
    fn generate_thread_plan_request_schema() {
        let schema = schemars::schema_for!(GenerateThreadPlanRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("topic"));
        assert!(json.contains("objective"));
    }

    #[test]
    fn x_post_request_schema() {
        let schema = schemars::schema_for!(XPostRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
        assert!(json.contains("body"));
    }

    #[test]
    fn x_put_request_schema() {
        let schema = schemars::schema_for!(XPutRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
    }

    #[test]
    fn x_delete_request_schema() {
        let schema = schemars::schema_for!(XDeleteRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
    }

    // --- X request types with all fields ---

    #[test]
    fn x_get_request_with_query_and_headers() {
        let json = r#"{"path": "/2/tweets", "host": "api.x.com", "query": [{"key": "q", "value": "rust"}], "headers": [{"key": "Accept", "value": "application/json"}], "auto_paginate": false}"#;
        let req: XGetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.query.as_ref().unwrap().len(), 1);
        assert_eq!(req.headers.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn x_post_request_with_all_fields() {
        let json = r#"{"path": "/2/tweets", "host": "api.x.com", "query": [{"key": "a", "value": "1"}], "body": "{}", "headers": [{"key": "X-Custom", "value": "val"}]}"#;
        let req: XPostRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets");
        assert!(req.query.is_some());
        assert!(req.body.is_some());
        assert!(req.headers.is_some());
    }

    #[test]
    fn x_put_request_with_all_fields() {
        let json =
            r#"{"path": "/2/lists/1", "body": "{\"name\":\"test\"}", "query": [], "headers": []}"#;
        let req: XPutRequest = serde_json::from_str(json).unwrap();
        assert!(req.query.unwrap().is_empty());
        assert!(req.headers.unwrap().is_empty());
    }

    #[test]
    fn x_delete_request_with_all_fields() {
        let json =
            r#"{"path": "/2/tweets/99", "host": "api.x.com", "query": null, "headers": null}"#;
        let req: XDeleteRequest = serde_json::from_str(json).unwrap();
        assert!(req.query.is_none());
        assert!(req.headers.is_none());
    }

    // --- Debug formatting ---

    #[test]
    fn tool_request_types_debug() {
        let _ = format!(
            "{:?}",
            DraftRepliesRequest {
                candidate_ids: vec!["c1".to_string()],
                archetype: None,
                mention_product: None,
            }
        );
    }
}
