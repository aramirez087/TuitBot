use serde_json::Value;

use super::helpers::validate_schema;
use super::{ScenarioResult, StepResult};
use crate::kernel::read;
use crate::tools::test_mocks::ErrorProvider;

// ErrorProvider in test_mocks returns Other for get_tweet. Scenario F
// needs AuthExpired for get_tweet, so we use a local variant.
struct AuthErrorProvider;

#[async_trait::async_trait]
impl crate::provider::SocialReadProvider for AuthErrorProvider {
    async fn get_tweet(
        &self,
        _tid: &str,
    ) -> Result<tuitbot_core::x_api::types::Tweet, crate::contract::ProviderError> {
        Err(crate::contract::ProviderError::AuthExpired)
    }

    async fn get_user_by_username(
        &self,
        _u: &str,
    ) -> Result<tuitbot_core::x_api::types::User, crate::contract::ProviderError> {
        Err(crate::contract::ProviderError::AuthExpired)
    }

    async fn search_tweets(
        &self,
        _q: &str,
        _max: u32,
        _since: Option<&str>,
        _pt: Option<&str>,
    ) -> Result<tuitbot_core::x_api::types::SearchResponse, crate::contract::ProviderError> {
        Err(crate::contract::ProviderError::RateLimited {
            retry_after: Some(60),
        })
    }

    async fn get_me(
        &self,
    ) -> Result<tuitbot_core::x_api::types::User, crate::contract::ProviderError> {
        Err(crate::contract::ProviderError::AuthExpired)
    }
}

pub async fn run_scenario_f() -> ScenarioResult {
    let mut steps = Vec::new();

    // Step 1: search_tweets rate limited (uses shared ErrorProvider)
    let start = std::time::Instant::now();
    let json = read::search_tweets(&ErrorProvider, "test", 10, None, None).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let code = parsed["error"]["code"].as_str().map(String::from);
    let retryable = parsed["error"]["retryable"].as_bool().unwrap_or(false);
    let retry_after = parsed["error"]["retry_after_ms"].as_u64();

    let correct_rate_limit =
        code.as_deref() == Some("x_rate_limited") && retryable && retry_after == Some(60000);

    steps.push(StepResult {
        tool_name: "search_tweets_rate_limited".to_string(),
        latency_ms: elapsed,
        success: correct_rate_limit,
        response_valid: valid,
        error_code: code,
    });

    // Step 2: get_tweet auth expired (uses local AuthErrorProvider)
    let start = std::time::Instant::now();
    let json = read::get_tweet(&AuthErrorProvider, "t1").await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let code = parsed["error"]["code"].as_str().map(String::from);
    let retryable = parsed["error"]["retryable"].as_bool().unwrap_or(true);

    let correct_auth = code.as_deref() == Some("x_auth_expired") && !retryable;

    steps.push(StepResult {
        tool_name: "get_tweet_auth_expired".to_string(),
        latency_ms: elapsed,
        success: correct_auth,
        response_valid: valid,
        error_code: code,
    });

    let total = steps.iter().map(|s| s.latency_ms).sum();
    ScenarioResult {
        scenario: "F".to_string(),
        description: "Rate-limited and auth error behavior validation".to_string(),
        total_latency_ms: total,
        success: steps.iter().all(|s| s.success),
        schema_valid: steps.iter().all(|s| s.response_valid),
        steps,
    }
}

#[tokio::test]
async fn scenario_f_error_behavior() {
    let result = run_scenario_f().await;
    assert!(result.success, "Scenario F failed");
    assert!(result.schema_valid, "Scenario F schema validation failed");
}
