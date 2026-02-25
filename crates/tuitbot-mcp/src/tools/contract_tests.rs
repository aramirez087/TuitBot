//! Contract tests: verify every tool returns a valid ToolResponse envelope.
//!
//! Each test calls the tool function with `init_test_db()` and asserts the
//! output parses as JSON with `success`, `data`, and optionally `error`/`meta`.

#[cfg(test)]
mod tests {
    use tuitbot_core::config::Config;
    use tuitbot_core::storage;

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
        let json = crate::tools::rate_limits::get_rate_limits(&pool, &config).await;
        assert_success(&json, "get_rate_limits");
        assert_has_meta(&json, "get_rate_limits");
    }

    // ── actions ──

    #[tokio::test]
    async fn contract_get_action_log() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::actions::get_action_log(&pool, 24, None, &config).await;
        assert_success(&json, "get_action_log");
        assert_has_meta(&json, "get_action_log");
    }

    #[tokio::test]
    async fn contract_get_action_counts() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::actions::get_action_counts(&pool, 24, &config).await;
        assert_success(&json, "get_action_counts");
        assert_has_meta(&json, "get_action_counts");
    }

    // ── replies ──

    #[tokio::test]
    async fn contract_get_recent_replies() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::replies::get_recent_replies(&pool, 24, &config).await;
        assert_success(&json, "get_recent_replies");
        assert_has_meta(&json, "get_recent_replies");
    }

    #[tokio::test]
    async fn contract_get_reply_count_today() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::replies::get_reply_count_today(&pool, &config).await;
        assert_success(&json, "get_reply_count_today");
        assert_has_meta(&json, "get_reply_count_today");
    }

    // ── targets ──

    #[tokio::test]
    async fn contract_list_target_accounts() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::targets::list_target_accounts(&pool, &config).await;
        assert_success(&json, "list_target_accounts");
        assert_has_meta(&json, "list_target_accounts");
    }

    // ── discovery ──

    #[tokio::test]
    async fn contract_list_unreplied_tweets() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::discovery::list_unreplied_tweets(&pool, 0.0, &config).await;
        assert_success(&json, "list_unreplied_tweets");
        assert_has_meta(&json, "list_unreplied_tweets");
    }

    #[tokio::test]
    async fn contract_list_unreplied_tweets_with_limit() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json =
            crate::tools::discovery::list_unreplied_tweets_with_limit(&pool, 0.0, 10, &config)
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
        let json = crate::tools::analytics::get_follower_trend(&pool, 7, &config).await;
        assert_success(&json, "get_follower_trend");
        assert_has_meta(&json, "get_follower_trend");
    }

    #[tokio::test]
    async fn contract_get_top_topics() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::analytics::get_top_topics(&pool, 10, &config).await;
        assert_success(&json, "get_top_topics");
        assert_has_meta(&json, "get_top_topics");
    }

    // ── approval ──

    #[tokio::test]
    async fn contract_list_pending() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::approval::list_pending(&pool, &config).await;
        assert_success(&json, "list_pending");
        assert_has_meta(&json, "list_pending");
    }

    #[tokio::test]
    async fn contract_get_pending_count() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::approval::get_pending_count(&pool, &config).await;
        assert_success(&json, "get_pending_count");
        assert_has_meta(&json, "get_pending_count");
    }

    #[tokio::test]
    async fn contract_approve_all_empty() {
        let pool = storage::init_test_db().await.unwrap();
        let config = test_config();
        let json = crate::tools::approval::approve_all(&pool, &config).await;
        assert_success(&json, "approve_all");
        assert_has_meta(&json, "approve_all");
    }

    // ── not-configured error paths ──

    #[tokio::test]
    async fn contract_not_configured_produces_envelope() {
        let json = ToolResponse::not_configured("llm").to_json();
        assert_envelope(&json, "not_configured(llm)");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed["success"].as_bool().unwrap());
        assert_eq!(parsed["error"]["code"], "llm_not_configured");
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
}
