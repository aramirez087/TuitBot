//! Contract tests: verify every tool returns a valid ToolResponse envelope.
//!
//! Each test calls the tool function with `init_test_db()` and asserts the
//! output parses as JSON with `success`, `data`, and optionally `error`/`meta`.

#[cfg(test)]
mod tests {
    use tuitbot_core::config::Config;
    use tuitbot_core::storage;

    use crate::contract::error_code::ErrorCode;
    use crate::tools::response::ToolResponse;

    fn test_config() -> Config {
        let mut config = Config::default();
        config.business.product_keywords = vec!["rust".to_string()];
        config.business.industry_topics = vec!["software".to_string()];
        config
    }

    /// Parse JSON and verify it has the ToolResponse envelope shape.
    fn assert_envelope(json: &str, context: &str) {
        let parsed: serde_json::Value =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("{context}: invalid JSON: {e}"));
        assert!(
            parsed.get("success").is_some(),
            "{context}: missing 'success' key"
        );
        assert!(
            parsed.get("data").is_some() || parsed.get("error").is_some(),
            "{context}: must have 'data' or 'error'"
        );
        // Verify it roundtrips as ToolResponse
        let _: ToolResponse = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("{context}: doesn't deserialize as ToolResponse: {e}"));
    }

    fn assert_success(json: &str, context: &str) {
        assert_envelope(json, context);
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(
            parsed["success"].as_bool().unwrap_or(false),
            "{context}: expected success=true"
        );
    }

    fn assert_has_meta(json: &str, context: &str) {
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(
            parsed.get("meta").is_some(),
            "{context}: expected meta to be present"
        );
    }

    // ── rate_limits ──

    #[tokio::test]
    async fn contract_get_rate_limits() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::rate_limits::get_rate_limits(&pool, &config).await;
        assert_success(&json, "get_rate_limits");
        assert_has_meta(&json, "get_rate_limits");
    }

    // ── actions ──

    #[tokio::test]
    async fn contract_get_action_log() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::actions::get_action_log(&pool, 24, None, &config).await;
        assert_success(&json, "get_action_log");
        assert_has_meta(&json, "get_action_log");
    }

    #[tokio::test]
    async fn contract_get_action_counts() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::actions::get_action_counts(&pool, 24, &config).await;
        assert_success(&json, "get_action_counts");
        assert_has_meta(&json, "get_action_counts");
    }

    // ── replies ──

    #[tokio::test]
    async fn contract_get_recent_replies() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::replies::get_recent_replies(&pool, 24, &config).await;
        assert_success(&json, "get_recent_replies");
        assert_has_meta(&json, "get_recent_replies");
    }

    #[tokio::test]
    async fn contract_get_reply_count_today() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::replies::get_reply_count_today(&pool, &config).await;
        assert_success(&json, "get_reply_count_today");
        assert_has_meta(&json, "get_reply_count_today");
    }

    // ── targets ──

    #[tokio::test]
    async fn contract_list_target_accounts() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::targets::list_target_accounts(&pool, &config).await;
        assert_success(&json, "list_target_accounts");
        assert_has_meta(&json, "list_target_accounts");
    }

    // ── discovery ──

    #[tokio::test]
    async fn contract_list_unreplied_tweets() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json =
            crate::tools::workflow::discovery::list_unreplied_tweets(&pool, 0.0, &config).await;
        assert_success(&json, "list_unreplied_tweets");
        assert_has_meta(&json, "list_unreplied_tweets");
    }

    #[tokio::test]
    async fn contract_list_unreplied_tweets_with_limit() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::discovery::list_unreplied_tweets_with_limit(
            &pool, 0.0, 10, &config,
        )
        .await;
        assert_success(&json, "list_unreplied_tweets_with_limit");
        assert_has_meta(&json, "list_unreplied_tweets_with_limit");
    }

    // ── scoring ──

    #[tokio::test]
    async fn contract_score_tweet() {
        let config = test_config();
        let input = crate::tools::scoring::ScoreTweetInput {
            text: "Rust is great for async programming",
            author_username: "dev",
            author_followers: 1000,
            likes: 5,
            retweets: 2,
            replies: 1,
            created_at: "2026-02-24T12:00:00Z",
        };
        let json = crate::tools::scoring::score_tweet(&config, &input);
        assert_success(&json, "score_tweet");
        assert_has_meta(&json, "score_tweet");
    }

    // ── config ──

    #[tokio::test]
    async fn contract_get_config() {
        let config = test_config();
        let json = crate::tools::config::get_config(&config);
        assert_success(&json, "get_config");
        assert_has_meta(&json, "get_config");
    }

    #[tokio::test]
    async fn contract_validate_config() {
        let config = test_config();
        let json = crate::tools::config::validate_config(&config);
        assert_success(&json, "validate_config");
        assert_has_meta(&json, "validate_config");
    }

    // ── analytics ──

    #[tokio::test]
    async fn contract_get_follower_trend() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::analytics::get_follower_trend(&pool, 7, &config).await;
        assert_success(&json, "get_follower_trend");
        assert_has_meta(&json, "get_follower_trend");
    }

    #[tokio::test]
    async fn contract_get_top_topics() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::analytics::get_top_topics(&pool, 10, &config).await;
        assert_success(&json, "get_top_topics");
        assert_has_meta(&json, "get_top_topics");
    }

    // ── approval ──

    #[tokio::test]
    async fn contract_list_pending() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::approval::list_pending(&pool, &config).await;
        assert_success(&json, "list_pending");
        assert_has_meta(&json, "list_pending");
    }

    #[tokio::test]
    async fn contract_get_pending_count() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::approval::get_pending_count(&pool, &config).await;
        assert_success(&json, "get_pending_count");
        assert_has_meta(&json, "get_pending_count");
    }

    #[tokio::test]
    async fn contract_approve_all_empty() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::workflow::approval::approve_all(&pool, &config, true).await;
        assert_success(&json, "approve_all");
        assert_has_meta(&json, "approve_all");
    }

    // ── typed error constructors ──

    #[tokio::test]
    async fn contract_llm_not_configured_produces_envelope() {
        let json = ToolResponse::llm_not_configured().to_json();
        assert_envelope(&json, "llm_not_configured");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed["success"].as_bool().unwrap());
        assert_eq!(parsed["error"]["code"], "llm_not_configured");
        assert!(!parsed["error"]["retryable"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn contract_x_not_configured_produces_envelope() {
        let json = ToolResponse::x_not_configured().to_json();
        assert_envelope(&json, "x_not_configured");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed["success"].as_bool().unwrap());
        assert_eq!(parsed["error"]["code"], "x_not_configured");
        assert!(!parsed["error"]["retryable"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn contract_db_error_produces_envelope() {
        let json = ToolResponse::db_error("connection refused").to_json();
        assert_envelope(&json, "db_error");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed["success"].as_bool().unwrap());
        assert_eq!(parsed["error"]["code"], "db_error");
        assert!(parsed["error"]["retryable"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn contract_validation_error_produces_envelope() {
        let json = ToolResponse::validation_error("bad input").to_json();
        assert_envelope(&json, "validation_error");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed["success"].as_bool().unwrap());
        assert_eq!(parsed["error"]["code"], "validation_error");
        assert!(!parsed["error"]["retryable"].as_bool().unwrap());
    }

    // ══════════════════════════════════════════════════════════════════════
    // Session 05: ErrorCode exhaustiveness tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn error_code_all_variants_roundtrip() {
        for &code in ErrorCode::ALL {
            let json = serde_json::to_string(&code)
                .unwrap_or_else(|e| panic!("serialize {code:?} failed: {e}"));
            let back: ErrorCode = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("deserialize {code:?} from {json} failed: {e}"));
            assert_eq!(back, code, "roundtrip mismatch for {code:?}");
        }
    }

    #[test]
    fn error_code_is_retryable_consistent() {
        let retryable = [
            ErrorCode::XRateLimited,
            ErrorCode::XNetworkError,
            ErrorCode::XApiError,
            ErrorCode::DbError,
            ErrorCode::LlmError,
            ErrorCode::ThreadPartialFailure,
            ErrorCode::PolicyError,
        ];
        for &code in ErrorCode::ALL {
            let expected = retryable.contains(&code);
            assert_eq!(
                code.is_retryable(),
                expected,
                "{code:?} retryable mismatch: expected {expected}"
            );
        }
    }

    #[test]
    fn error_code_display_matches_json() {
        for &code in ErrorCode::ALL {
            let display = code.to_string();
            let json = serde_json::to_string(&code).unwrap();
            assert_eq!(
                format!("\"{display}\""),
                json,
                "Display/serde mismatch for {code:?}"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Session 05: Error path validation
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn every_error_code_produces_valid_envelope() {
        for &code in ErrorCode::ALL {
            let resp = ToolResponse::error(code, format!("test error for {code}"));
            let json = resp.to_json();
            assert_envelope(&json, &format!("ErrorCode::{code:?}"));

            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert!(!parsed["success"].as_bool().unwrap());
            assert_eq!(parsed["error"]["code"].as_str().unwrap(), code.as_str());
            assert_eq!(
                parsed["error"]["retryable"].as_bool().unwrap(),
                code.is_retryable(),
                "retryable flag mismatch for {code:?}"
            );
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Session 05: API profile field isolation
    // ══════════════════════════════════════════════════════════════════════

    /// Verify that kernel tools (API profile) do NOT include workflow fields.
    fn assert_no_workflow_fields(json: &str, context: &str) {
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        if let Some(meta) = parsed.get("meta") {
            assert!(
                meta.get("mode").is_none(),
                "{context}: 'mode' should be absent in API profile"
            );
            assert!(
                meta.get("approval_mode").is_none(),
                "{context}: 'approval_mode' should be absent in API profile"
            );
        }
    }

    #[test]
    fn kernel_tweet_too_long_no_workflow() {
        let start = std::time::Instant::now();
        let text = "a".repeat(300);
        if let Some(json) = crate::kernel::utils::check_tweet_length(&text, start) {
            assert_no_workflow_fields(&json, "check_tweet_length");
        }
    }

    #[test]
    fn kernel_invalid_input_no_workflow() {
        let json = ToolResponse::error(
            ErrorCode::InvalidInput,
            "Thread must contain at least one tweet.",
        )
        .to_json();
        assert_no_workflow_fields(&json, "invalid_input");
    }

    #[test]
    fn kernel_media_error_no_workflow() {
        let json = ToolResponse::error(ErrorCode::UnsupportedMediaType, "bad ext").to_json();
        assert_no_workflow_fields(&json, "unsupported_media_type");
    }

    /// Meta without workflow: verify workflow fields are absent.
    #[test]
    fn meta_without_workflow_has_no_mode() {
        use crate::contract::envelope::ToolMeta;
        let meta = ToolMeta::new(42);
        let resp = ToolResponse::success(1).with_meta(meta);
        let json = resp.to_json();
        assert_no_workflow_fields(&json, "meta_without_workflow");
        // But elapsed_ms and tool_version should be there.
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["meta"]["elapsed_ms"], 42);
        assert_eq!(parsed["meta"]["tool_version"], "1.0");
    }

    /// Meta with workflow: verify workflow fields are present and flattened.
    #[test]
    fn meta_with_workflow_has_mode_flattened() {
        use crate::contract::envelope::ToolMeta;
        let meta = ToolMeta::new(99).with_workflow("autopilot", true);
        let resp = ToolResponse::success(1).with_meta(meta);
        let json = resp.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["meta"]["mode"], "autopilot");
        assert_eq!(parsed["meta"]["approval_mode"], true);
        assert_eq!(parsed["meta"]["elapsed_ms"], 99);
    }

    // ── telemetry ──

    #[tokio::test]
    async fn contract_get_mcp_tool_metrics() {
        let pool = storage::init_test_db().await.unwrap();
        let json = crate::tools::workflow::telemetry::get_mcp_tool_metrics(&pool, 24).await;
        assert_success(&json, "get_mcp_tool_metrics");
        assert_has_meta(&json, "get_mcp_tool_metrics");
    }

    #[tokio::test]
    async fn contract_get_mcp_error_breakdown() {
        let pool = storage::init_test_db().await.unwrap();
        let json = crate::tools::workflow::telemetry::get_mcp_error_breakdown(&pool, 24).await;
        assert_success(&json, "get_mcp_error_breakdown");
        assert_has_meta(&json, "get_mcp_error_breakdown");
    }

    // ── context ──

    #[tokio::test]
    async fn contract_topic_performance_snapshot() {
        let pool = storage::init_test_db().await.unwrap();
        let json = crate::tools::workflow::context::topic_performance_snapshot(&pool, 30).await;
        assert_success(&json, "topic_performance_snapshot");
        assert_has_meta(&json, "topic_performance_snapshot");
    }
}
