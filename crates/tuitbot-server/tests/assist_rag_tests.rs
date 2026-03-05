//! Server-level regression tests for composer assist endpoints with automatic
//! vault context (RAG). Exercises the full HTTP path: request → handler →
//! resolver → generator → response.
//!
//! Covers success-with-context, success-without-context, no-generator, and
//! dual-context scenarios for all three composer assist handlers.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::{broadcast, Mutex, RwLock};
use tower::ServiceExt;

use tuitbot_core::config::BusinessProfile;
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::error::LlmError;
use tuitbot_core::llm::{GenerationParams, LlmProvider, LlmResponse, TokenUsage};
use tuitbot_core::storage;
use tuitbot_core::storage::watchtower;
use tuitbot_core::storage::DbPool;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::WsEvent;

const TEST_TOKEN: &str = "test-token-rag-abc";
const DEFAULT_ACCOUNT_ID: &str = "00000000-0000-0000-0000-000000000000";

// ============================================================================
// Mock LLM provider that captures system prompts
// ============================================================================

/// Mock LLM provider that captures every system prompt for assertion and
/// returns canned responses in order.
struct PromptCapturingProvider {
    responses: Vec<String>,
    call_count: std::sync::atomic::AtomicUsize,
    captured_prompts: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl PromptCapturingProvider {
    fn new(responses: Vec<String>, captured: Arc<tokio::sync::Mutex<Vec<String>>>) -> Self {
        Self {
            responses,
            call_count: std::sync::atomic::AtomicUsize::new(0),
            captured_prompts: captured,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for PromptCapturingProvider {
    fn name(&self) -> &str {
        "prompt_capturing_mock"
    }

    async fn complete(
        &self,
        system: &str,
        _user_message: &str,
        _params: &GenerationParams,
    ) -> Result<LlmResponse, LlmError> {
        // Capture the system prompt.
        self.captured_prompts.lock().await.push(system.to_string());

        let idx = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let text = self
            .responses
            .get(idx)
            .cloned()
            .unwrap_or_else(|| self.responses.last().cloned().unwrap_or_default());

        Ok(LlmResponse {
            text,
            usage: TokenUsage::default(),
            model: "mock".to_string(),
        })
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        Ok(())
    }
}

// ============================================================================
// Test helpers
// ============================================================================

fn test_business() -> BusinessProfile {
    BusinessProfile {
        product_name: "TestApp".to_string(),
        product_description: "A test application".to_string(),
        product_url: Some("https://testapp.com".to_string()),
        target_audience: "developers".to_string(),
        product_keywords: vec!["test".to_string()],
        competitor_keywords: vec![],
        industry_topics: vec!["testing".to_string()],
        brand_voice: None,
        reply_style: None,
        content_style: None,
        persona_opinions: vec![],
        persona_experiences: vec![],
        content_pillars: vec![],
    }
}

/// Valid 6-segment thread response that passes the generator's retry logic.
fn valid_thread_response() -> String {
    [
        "Hook: Testing makes everything better",
        "---",
        "Point 1: Unit tests catch bugs early",
        "---",
        "Point 2: Integration tests verify flow",
        "---",
        "Point 3: E2E tests build confidence",
        "---",
        "Point 4: Test coverage as a metric",
        "---",
        "Summary: Invest in testing today",
    ]
    .join("\n")
}

/// Seed vault data via the cold-start seeds path. After this,
/// `build_draft_context` should return a `DraftContext` with
/// `prompt_block` containing `"Relevant ideas"`.
async fn seed_vault_seeds(pool: &DbPool) {
    let source_id = watchtower::insert_source_context(pool, "local_fs", "{}")
        .await
        .expect("insert source context");

    watchtower::upsert_content_node(
        pool,
        source_id,
        "notes.md",
        "hash-seed-test-1",
        Some("Test Notes"),
        "Body text about growth strategies",
        None,
        None,
    )
    .await
    .expect("upsert content node");

    // Node ID is 1 (first row inserted into content_nodes).
    watchtower::insert_draft_seed_with_weight(
        pool,
        1,
        "Test hook about growth strategies for developers",
        Some("tip"),
        0.75,
    )
    .await
    .expect("insert draft seed");
}

/// Seed vault data via the ancestors path (winning tweets with performance
/// data). After this, `build_draft_context` should return a `DraftContext`
/// with `prompt_block` containing `"Winning patterns"`.
async fn seed_vault_ancestors(pool: &DbPool) {
    // Insert a tweet into original_tweets.
    sqlx::query(
        "INSERT INTO original_tweets (account_id, tweet_id, content, topic, status, created_at) \
         VALUES (?, ?, ?, ?, ?, datetime('now', '-3 days'))",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .bind("tw-ancestor-1")
    .bind("Testing makes everything better — here is my experience")
    .bind("test")
    .bind("sent")
    .execute(pool)
    .await
    .expect("insert original tweet");

    // Insert performance data using the proper analytics API.
    storage::analytics::upsert_tweet_performance(pool, "tw-ancestor-1", 50, 20, 10, 5000, 82.0)
        .await
        .expect("upsert tweet performance");
    storage::analytics::update_tweet_engagement_score(pool, "tw-ancestor-1", 0.9)
        .await
        .expect("update engagement score");
}

/// Build a test router with a content generator backed by the mock provider.
/// Returns `(Router, captured_prompts_handle)`.
///
/// If `seed_fn` is provided, it is called with the DB pool to populate vault data.
async fn build_test_router_with_generator(
    responses: Vec<String>,
    seed_fn: Option<
        fn(&DbPool) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>,
    >,
) -> (
    axum::Router,
    Arc<tokio::sync::Mutex<Vec<String>>>,
    tempfile::TempDir,
) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        "[business]\nproduct_name = \"TestApp\"\nproduct_keywords = [\"test\"]\n",
    )
    .expect("write config");

    let pool = storage::init_test_db().await.expect("init test db");

    if let Some(seeder) = seed_fn {
        seeder(&pool).await;
    }

    let captured = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
    let provider = PromptCapturingProvider::new(responses, Arc::clone(&captured));
    let generator = Arc::new(ContentGenerator::new(Box::new(provider), test_business()));

    let (event_tx, _) = broadcast::channel::<WsEvent>(16);
    let mut generators = HashMap::new();
    generators.insert(DEFAULT_ACCOUNT_ID.to_string(), generator);

    let state = Arc::new(AppState {
        db: pool,
        config_path,
        data_dir: dir.path().to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: RwLock::new(None),
        passphrase_hash_mtime: RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(HashMap::new()),
        content_generators: Mutex::new(generators),
        runtimes: Mutex::new(HashMap::new()),
        circuit_breaker: None,
        watchtower_cancel: None,
        content_sources: Default::default(),
        connector_config: Default::default(),
        deployment_mode: Default::default(),
        provider_backend: String::new(),
        pending_oauth: Mutex::new(HashMap::new()),
        token_managers: Mutex::new(HashMap::new()),
        x_client_id: String::new(),
    });

    let router = tuitbot_server::build_router(state);
    (router, captured, dir)
}

/// Build a test router without any content generator (for 400 tests).
async fn build_test_router_no_generator() -> (axum::Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        "[business]\nproduct_name = \"TestApp\"\nproduct_keywords = [\"test\"]\n",
    )
    .expect("write config");

    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = broadcast::channel::<WsEvent>(16);

    let state = Arc::new(AppState {
        db: pool,
        config_path,
        data_dir: dir.path().to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: RwLock::new(None),
        passphrase_hash_mtime: RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(HashMap::new()),
        content_generators: Mutex::new(HashMap::new()),
        runtimes: Mutex::new(HashMap::new()),
        circuit_breaker: None,
        watchtower_cancel: None,
        content_sources: Default::default(),
        connector_config: Default::default(),
        deployment_mode: Default::default(),
        provider_backend: String::new(),
        pending_oauth: Mutex::new(HashMap::new()),
        token_managers: Mutex::new(HashMap::new()),
        x_client_id: String::new(),
    });

    let router = tuitbot_server::build_router(state);
    (router, dir)
}

async fn post_json(
    router: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value =
        serde_json::from_slice(&bytes.to_bytes()).unwrap_or(serde_json::json!({}));

    (status, json)
}

/// Helper: check if any captured system prompt contains the given substring.
async fn any_prompt_contains(
    captured: &Arc<tokio::sync::Mutex<Vec<String>>>,
    needle: &str,
) -> bool {
    let prompts = captured.lock().await;
    prompts.iter().any(|p| p.contains(needle))
}

// ============================================================================
// Tests — Tweet endpoint
// ============================================================================

#[tokio::test]
async fn tweet_with_rag_context() {
    let (router, captured, _dir) = build_test_router_with_generator(
        vec!["A great tweet about testing strategies.".to_string()],
        Some(|pool| Box::pin(seed_vault_seeds(pool))),
    )
    .await;

    let (status, body) = post_json(
        router,
        "/api/assist/tweet",
        serde_json::json!({ "topic": "testing" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["content"].as_str().is_some());
    assert!(!body["content"].as_str().unwrap().is_empty());
    assert!(
        any_prompt_contains(&captured, "Relevant ideas").await,
        "System prompt should contain vault-derived seed context"
    );
}

#[tokio::test]
async fn tweet_without_rag_context() {
    let (router, captured, _dir) =
        build_test_router_with_generator(vec!["A tweet without vault context.".to_string()], None)
            .await;

    let (status, body) = post_json(
        router,
        "/api/assist/tweet",
        serde_json::json!({ "topic": "testing" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["content"].as_str().is_some());
    assert!(
        !any_prompt_contains(&captured, "Relevant ideas").await,
        "No vault context should appear when DB is empty"
    );
    assert!(
        !any_prompt_contains(&captured, "Winning patterns").await,
        "No ancestor context should appear when DB is empty"
    );
}

#[tokio::test]
async fn tweet_no_generator_returns_400() {
    let (router, _dir) = build_test_router_no_generator().await;

    let (status, body) = post_json(
        router,
        "/api/assist/tweet",
        serde_json::json!({ "topic": "testing" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("LLM not configured"),
        "Error should mention LLM configuration"
    );
}

// ============================================================================
// Tests — Thread endpoint
// ============================================================================

#[tokio::test]
async fn thread_with_rag_context() {
    let (router, captured, _dir) = build_test_router_with_generator(
        vec![valid_thread_response()],
        Some(|pool| Box::pin(seed_vault_seeds(pool))),
    )
    .await;

    let (status, body) = post_json(
        router,
        "/api/assist/thread",
        serde_json::json!({ "topic": "testing" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let tweets = body["tweets"].as_array().expect("tweets array");
    assert!(
        tweets.len() >= 5,
        "Thread should have at least 5 tweets, got {}",
        tweets.len()
    );
    assert!(
        any_prompt_contains(&captured, "Relevant ideas").await,
        "System prompt should contain vault-derived seed context"
    );
}

#[tokio::test]
async fn thread_without_rag_context() {
    let (router, captured, _dir) =
        build_test_router_with_generator(vec![valid_thread_response()], None).await;

    let (status, body) = post_json(
        router,
        "/api/assist/thread",
        serde_json::json!({ "topic": "testing" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let tweets = body["tweets"].as_array().expect("tweets array");
    assert!(tweets.len() >= 5);
    assert!(
        !any_prompt_contains(&captured, "Relevant ideas").await,
        "No vault context should appear when DB is empty"
    );
    assert!(
        !any_prompt_contains(&captured, "Winning patterns").await,
        "No ancestor context should appear when DB is empty"
    );
}

#[tokio::test]
async fn thread_no_generator_returns_400() {
    let (router, _dir) = build_test_router_no_generator().await;

    let (status, body) = post_json(
        router,
        "/api/assist/thread",
        serde_json::json!({ "topic": "testing" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap_or("")
        .contains("LLM not configured"),);
}

// ============================================================================
// Tests — Improve endpoint
// ============================================================================

#[tokio::test]
async fn improve_with_rag_context() {
    let (router, captured, _dir) = build_test_router_with_generator(
        vec!["An improved tweet with vault knowledge.".to_string()],
        Some(|pool| Box::pin(seed_vault_seeds(pool))),
    )
    .await;

    let (status, body) = post_json(
        router,
        "/api/assist/improve",
        serde_json::json!({ "draft": "Testing is important for code quality." }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["content"].as_str().is_some());
    assert!(!body["content"].as_str().unwrap().is_empty());
    assert!(
        any_prompt_contains(&captured, "Relevant ideas").await,
        "System prompt should contain vault-derived seed context"
    );
}

#[tokio::test]
async fn improve_without_rag_context() {
    let (router, captured, _dir) = build_test_router_with_generator(
        vec!["Improved tweet without vault context.".to_string()],
        None,
    )
    .await;

    let (status, body) = post_json(
        router,
        "/api/assist/improve",
        serde_json::json!({ "draft": "Testing is important." }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["content"].as_str().is_some());
    assert!(
        !any_prompt_contains(&captured, "Relevant ideas").await,
        "No vault context when DB is empty"
    );
    assert!(
        !any_prompt_contains(&captured, "Winning patterns").await,
        "No ancestor context when DB is empty"
    );
}

#[tokio::test]
async fn improve_no_generator_returns_400() {
    let (router, _dir) = build_test_router_no_generator().await;

    let (status, body) = post_json(
        router,
        "/api/assist/improve",
        serde_json::json!({ "draft": "Testing is important." }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap_or("")
        .contains("LLM not configured"),);
}

#[tokio::test]
async fn improve_dual_context() {
    let (router, captured, _dir) = build_test_router_with_generator(
        vec!["A punchy improved tweet with vault knowledge.".to_string()],
        Some(|pool| Box::pin(seed_vault_seeds(pool))),
    )
    .await;

    let (status, body) = post_json(
        router,
        "/api/assist/improve",
        serde_json::json!({
            "draft": "Testing matters for quality.",
            "context": "Be punchy"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["content"].as_str().is_some());

    // Both vault context and tone directive should appear in the prompt.
    assert!(
        any_prompt_contains(&captured, "Relevant ideas").await,
        "RAG context should be present"
    );
    assert!(
        any_prompt_contains(&captured, "Be punchy").await,
        "Tone directive should be present in system prompt"
    );
}

#[tokio::test]
async fn improve_tone_only_no_vault() {
    let (router, captured, _dir) =
        build_test_router_with_generator(vec!["A casual tweet about testing.".to_string()], None)
            .await;

    let (status, body) = post_json(
        router,
        "/api/assist/improve",
        serde_json::json!({
            "draft": "Testing is important.",
            "context": "Be casual"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["content"].as_str().is_some());
    assert!(
        any_prompt_contains(&captured, "Be casual").await,
        "Tone directive should appear even without vault context"
    );
    assert!(
        !any_prompt_contains(&captured, "Relevant ideas").await,
        "No vault context when DB is empty"
    );
}

// ============================================================================
// Tests — Ancestors path (Winning patterns header)
// ============================================================================

#[tokio::test]
async fn tweet_with_ancestors_context() {
    let (router, captured, _dir) = build_test_router_with_generator(
        vec!["A tweet grounded in winning patterns.".to_string()],
        Some(|pool| Box::pin(seed_vault_ancestors(pool))),
    )
    .await;

    let (status, body) = post_json(
        router,
        "/api/assist/tweet",
        serde_json::json!({ "topic": "testing" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["content"].as_str().is_some());
    assert!(
        any_prompt_contains(&captured, "Winning patterns").await,
        "System prompt should contain ancestor-derived winning patterns"
    );
}
