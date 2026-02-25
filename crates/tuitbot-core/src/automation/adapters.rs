//! Adapter implementations bridging port traits to real dependencies.
//!
//! Each adapter struct wraps one or more concrete dependencies (X API client,
//! content generator, scoring engine, safety guard, database pool, posting queue)
//! and implements the port traits defined in [`loop_helpers`], [`analytics_loop`],
//! [`target_loop`], [`thread_loop`], [`posting_queue`], and [`status_reporter`].

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use tokio::sync::mpsc;

use crate::content::ContentGenerator;
use crate::error::{LlmError, XApiError};
use crate::safety::SafetyGuard;
use crate::scoring::{self, ScoringEngine, TweetData};
use crate::storage::{self, DbPool};
use crate::x_api::{SearchResponse, XApiClient, XApiHttpClient};

use super::analytics_loop::{AnalyticsError, AnalyticsStorage, EngagementFetcher, ProfileFetcher};
use super::loop_helpers::{
    ContentLoopError, ContentSafety, ContentStorage, LoopError, LoopStorage, LoopTweet,
    MentionsFetcher, PostSender, ReplyGenerator, SafetyChecker, ScoreResult, ThreadPoster,
    TopicScorer, TweetGenerator, TweetScorer, TweetSearcher,
};
use super::posting_queue::{ApprovalQueue, PostAction, PostExecutor};
use super::status_reporter::{ActionCounts, StatusQuerier};
use super::target_loop::{TargetStorage, TargetTweetFetcher, TargetUserManager};
use super::thread_loop::ThreadGenerator;

// ============================================================================
// Helper functions
// ============================================================================

/// Convert an X API `SearchResponse` to a `Vec<LoopTweet>`.
///
/// Joins tweet data with user data from the `includes` expansion to populate
/// author username and follower count.
fn search_response_to_loop_tweets(response: SearchResponse) -> Vec<LoopTweet> {
    let users: HashMap<&str, _> = response
        .includes
        .as_ref()
        .map(|inc| inc.users.iter().map(|u| (u.id.as_str(), u)).collect())
        .unwrap_or_default();

    response
        .data
        .into_iter()
        .map(|tweet| {
            let user = users.get(tweet.author_id.as_str());
            LoopTweet {
                id: tweet.id,
                text: tweet.text,
                author_id: tweet.author_id,
                author_username: user.map(|u| u.username.clone()).unwrap_or_default(),
                author_followers: user.map(|u| u.public_metrics.followers_count).unwrap_or(0),
                created_at: tweet.created_at,
                likes: tweet.public_metrics.like_count,
                retweets: tweet.public_metrics.retweet_count,
                replies: tweet.public_metrics.reply_count,
            }
        })
        .collect()
}

/// Map `XApiError` to `LoopError`.
fn xapi_to_loop_error(e: XApiError) -> LoopError {
    match e {
        XApiError::RateLimited { retry_after } => LoopError::RateLimited { retry_after },
        XApiError::AuthExpired => LoopError::AuthExpired,
        XApiError::Network { source } => LoopError::NetworkError(source.to_string()),
        other => LoopError::Other(other.to_string()),
    }
}

/// Map `XApiError` to `ContentLoopError`.
fn xapi_to_content_error(e: XApiError) -> ContentLoopError {
    match e {
        XApiError::RateLimited { retry_after } => ContentLoopError::PostFailed(format!(
            "rate limited{}",
            retry_after
                .map(|s| format!(", retry after {s}s"))
                .unwrap_or_default()
        )),
        XApiError::Network { source } => ContentLoopError::NetworkError(source.to_string()),
        other => ContentLoopError::PostFailed(other.to_string()),
    }
}

/// Map `XApiError` to `AnalyticsError`.
fn xapi_to_analytics_error(e: XApiError) -> AnalyticsError {
    AnalyticsError::ApiError(e.to_string())
}

/// Map `LlmError` to `LoopError`.
fn llm_to_loop_error(e: LlmError) -> LoopError {
    LoopError::LlmFailure(e.to_string())
}

/// Map `LlmError` to `ContentLoopError`.
fn llm_to_content_error(e: LlmError) -> ContentLoopError {
    ContentLoopError::LlmFailure(e.to_string())
}

/// Map `sqlx::Error` to `ContentLoopError`.
fn sqlx_to_content_error(e: sqlx::Error) -> ContentLoopError {
    ContentLoopError::StorageError(e.to_string())
}

/// Map `StorageError` to `LoopError`.
fn storage_to_loop_error(e: crate::error::StorageError) -> LoopError {
    LoopError::StorageError(e.to_string())
}

/// Parse a datetime string into `DateTime<Utc>`.
///
/// Tries RFC-3339 first, then `%Y-%m-%d %H:%M:%S` (SQLite `datetime()` format),
/// then `%Y-%m-%dT%H:%M:%SZ`.
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(naive.and_utc());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(naive.and_utc());
    }
    None
}

// ============================================================================
// X API adapters
// ============================================================================

/// Adapts `XApiHttpClient` to the `TweetSearcher` port trait.
pub struct XApiSearchAdapter {
    client: Arc<XApiHttpClient>,
}

impl XApiSearchAdapter {
    pub fn new(client: Arc<XApiHttpClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl TweetSearcher for XApiSearchAdapter {
    async fn search_tweets(&self, query: &str) -> Result<Vec<LoopTweet>, LoopError> {
        let response = self
            .client
            .search_tweets(query, 20, None, None)
            .await
            .map_err(xapi_to_loop_error)?;
        Ok(search_response_to_loop_tweets(response))
    }
}

/// Adapts `XApiHttpClient` to the `MentionsFetcher` port trait.
pub struct XApiMentionsAdapter {
    client: Arc<XApiHttpClient>,
    own_user_id: String,
}

impl XApiMentionsAdapter {
    pub fn new(client: Arc<XApiHttpClient>, own_user_id: String) -> Self {
        Self {
            client,
            own_user_id,
        }
    }
}

#[async_trait::async_trait]
impl MentionsFetcher for XApiMentionsAdapter {
    async fn get_mentions(&self, since_id: Option<&str>) -> Result<Vec<LoopTweet>, LoopError> {
        let response = self
            .client
            .get_mentions(&self.own_user_id, since_id, None)
            .await
            .map_err(xapi_to_loop_error)?;
        Ok(search_response_to_loop_tweets(response))
    }
}

/// Adapts `XApiHttpClient` to `TargetTweetFetcher` and `TargetUserManager`.
pub struct XApiTargetAdapter {
    client: Arc<XApiHttpClient>,
}

impl XApiTargetAdapter {
    pub fn new(client: Arc<XApiHttpClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl TargetTweetFetcher for XApiTargetAdapter {
    async fn fetch_user_tweets(&self, user_id: &str) -> Result<Vec<LoopTweet>, LoopError> {
        let response = self
            .client
            .get_user_tweets(user_id, 10, None)
            .await
            .map_err(xapi_to_loop_error)?;
        Ok(search_response_to_loop_tweets(response))
    }
}

#[async_trait::async_trait]
impl TargetUserManager for XApiTargetAdapter {
    async fn lookup_user(&self, username: &str) -> Result<(String, String), LoopError> {
        let user = self
            .client
            .get_user_by_username(username)
            .await
            .map_err(xapi_to_loop_error)?;
        Ok((user.id, user.username))
    }
}

/// Adapts `XApiHttpClient` to `ProfileFetcher` and `EngagementFetcher`.
pub struct XApiProfileAdapter {
    client: Arc<XApiHttpClient>,
}

impl XApiProfileAdapter {
    pub fn new(client: Arc<XApiHttpClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl ProfileFetcher for XApiProfileAdapter {
    async fn get_profile_metrics(
        &self,
    ) -> Result<super::analytics_loop::ProfileMetrics, AnalyticsError> {
        let user = self
            .client
            .get_me()
            .await
            .map_err(xapi_to_analytics_error)?;
        Ok(super::analytics_loop::ProfileMetrics {
            follower_count: user.public_metrics.followers_count as i64,
            following_count: user.public_metrics.following_count as i64,
            tweet_count: user.public_metrics.tweet_count as i64,
        })
    }
}

#[async_trait::async_trait]
impl EngagementFetcher for XApiProfileAdapter {
    async fn get_tweet_metrics(
        &self,
        tweet_id: &str,
    ) -> Result<super::analytics_loop::TweetMetrics, AnalyticsError> {
        let tweet = self
            .client
            .get_tweet(tweet_id)
            .await
            .map_err(xapi_to_analytics_error)?;
        Ok(super::analytics_loop::TweetMetrics {
            likes: tweet.public_metrics.like_count as i64,
            retweets: tweet.public_metrics.retweet_count as i64,
            replies: tweet.public_metrics.reply_count as i64,
            impressions: tweet.public_metrics.impression_count as i64,
        })
    }
}

/// Adapts `XApiHttpClient` to `PostExecutor` (for the posting queue).
pub struct XApiPostExecutorAdapter {
    client: Arc<XApiHttpClient>,
}

impl XApiPostExecutorAdapter {
    pub fn new(client: Arc<XApiHttpClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl PostExecutor for XApiPostExecutorAdapter {
    async fn execute_reply(
        &self,
        tweet_id: &str,
        content: &str,
        media_ids: &[String],
    ) -> Result<String, String> {
        if media_ids.is_empty() {
            self.client
                .reply_to_tweet(content, tweet_id)
                .await
                .map(|posted| posted.id)
                .map_err(|e| e.to_string())
        } else {
            self.client
                .reply_to_tweet_with_media(content, tweet_id, media_ids)
                .await
                .map(|posted| posted.id)
                .map_err(|e| e.to_string())
        }
    }

    async fn execute_tweet(&self, content: &str, media_ids: &[String]) -> Result<String, String> {
        if media_ids.is_empty() {
            self.client
                .post_tweet(content)
                .await
                .map(|posted| posted.id)
                .map_err(|e| e.to_string())
        } else {
            self.client
                .post_tweet_with_media(content, media_ids)
                .await
                .map(|posted| posted.id)
                .map_err(|e| e.to_string())
        }
    }
}

/// Adapts `XApiHttpClient` to `ThreadPoster` (for direct thread posting).
pub struct XApiThreadPosterAdapter {
    client: Arc<XApiHttpClient>,
}

impl XApiThreadPosterAdapter {
    pub fn new(client: Arc<XApiHttpClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl ThreadPoster for XApiThreadPosterAdapter {
    async fn post_tweet(&self, content: &str) -> Result<String, ContentLoopError> {
        self.client
            .post_tweet(content)
            .await
            .map(|posted| posted.id)
            .map_err(xapi_to_content_error)
    }

    async fn reply_to_tweet(
        &self,
        in_reply_to: &str,
        content: &str,
    ) -> Result<String, ContentLoopError> {
        self.client
            .reply_to_tweet(content, in_reply_to)
            .await
            .map(|posted| posted.id)
            .map_err(xapi_to_content_error)
    }
}

// ============================================================================
// LLM adapters
// ============================================================================

/// Record LLM usage to the database (fire-and-forget).
async fn record_usage(
    pool: &DbPool,
    generation_type: &str,
    provider: &str,
    model: &str,
    input_tokens: u32,
    output_tokens: u32,
) {
    let pricing = crate::llm::pricing::lookup(provider, model);
    let cost = pricing.compute_cost(input_tokens, output_tokens);
    if let Err(e) = storage::llm_usage::insert_llm_usage(
        pool,
        generation_type,
        provider,
        model,
        input_tokens,
        output_tokens,
        cost,
    )
    .await
    {
        tracing::warn!(error = %e, "Failed to record LLM usage");
    }
}

/// Adapts `ContentGenerator` to the `ReplyGenerator` port trait.
pub struct LlmReplyAdapter {
    generator: Arc<ContentGenerator>,
    pool: DbPool,
}

impl LlmReplyAdapter {
    pub fn new(generator: Arc<ContentGenerator>, pool: DbPool) -> Self {
        Self { generator, pool }
    }
}

#[async_trait::async_trait]
impl ReplyGenerator for LlmReplyAdapter {
    async fn generate_reply(
        &self,
        tweet_text: &str,
        author: &str,
        mention_product: bool,
    ) -> Result<String, LoopError> {
        let output = self
            .generator
            .generate_reply(tweet_text, author, mention_product)
            .await
            .map_err(llm_to_loop_error)?;
        record_usage(
            &self.pool,
            "reply",
            &output.provider,
            &output.model,
            output.usage.input_tokens,
            output.usage.output_tokens,
        )
        .await;
        Ok(output.text)
    }
}

/// Adapts `ContentGenerator` to the `TweetGenerator` port trait.
pub struct LlmTweetAdapter {
    generator: Arc<ContentGenerator>,
    pool: DbPool,
}

impl LlmTweetAdapter {
    pub fn new(generator: Arc<ContentGenerator>, pool: DbPool) -> Self {
        Self { generator, pool }
    }
}

#[async_trait::async_trait]
impl TweetGenerator for LlmTweetAdapter {
    async fn generate_tweet(&self, topic: &str) -> Result<String, ContentLoopError> {
        let output = self
            .generator
            .generate_tweet(topic)
            .await
            .map_err(llm_to_content_error)?;
        record_usage(
            &self.pool,
            "tweet",
            &output.provider,
            &output.model,
            output.usage.input_tokens,
            output.usage.output_tokens,
        )
        .await;
        Ok(output.text)
    }
}

/// Adapts `ContentGenerator` to the `ThreadGenerator` port trait.
pub struct LlmThreadAdapter {
    generator: Arc<ContentGenerator>,
    pool: DbPool,
}

impl LlmThreadAdapter {
    pub fn new(generator: Arc<ContentGenerator>, pool: DbPool) -> Self {
        Self { generator, pool }
    }
}

#[async_trait::async_trait]
impl ThreadGenerator for LlmThreadAdapter {
    async fn generate_thread(
        &self,
        topic: &str,
        _count: Option<usize>,
    ) -> Result<Vec<String>, ContentLoopError> {
        let output = self
            .generator
            .generate_thread(topic)
            .await
            .map_err(llm_to_content_error)?;
        record_usage(
            &self.pool,
            "thread",
            &output.provider,
            &output.model,
            output.usage.input_tokens,
            output.usage.output_tokens,
        )
        .await;
        Ok(output.tweets)
    }
}

// ============================================================================
// Scoring adapter
// ============================================================================

/// Adapts `ScoringEngine` to the `TweetScorer` port trait.
pub struct ScoringAdapter {
    engine: Arc<ScoringEngine>,
}

impl ScoringAdapter {
    pub fn new(engine: Arc<ScoringEngine>) -> Self {
        Self { engine }
    }
}

impl TweetScorer for ScoringAdapter {
    fn score(&self, tweet: &LoopTweet) -> ScoreResult {
        let data = TweetData {
            text: tweet.text.clone(),
            created_at: tweet.created_at.clone(),
            likes: tweet.likes,
            retweets: tweet.retweets,
            replies: tweet.replies,
            author_username: tweet.author_username.clone(),
            author_followers: tweet.author_followers,
            has_media: false,
            is_quote_tweet: false,
        };

        let score = self.engine.score_tweet(&data);
        let matched_keywords = scoring::find_matched_keywords(&tweet.text, self.engine.keywords());

        ScoreResult {
            total: score.total,
            meets_threshold: score.meets_threshold,
            matched_keywords,
        }
    }
}

// ============================================================================
// Safety adapters
// ============================================================================

/// Adapts `SafetyGuard` to the `SafetyChecker` port trait.
pub struct SafetyAdapter {
    guard: Arc<SafetyGuard>,
    pool: DbPool,
}

impl SafetyAdapter {
    pub fn new(guard: Arc<SafetyGuard>, pool: DbPool) -> Self {
        Self { guard, pool }
    }
}

#[async_trait::async_trait]
impl SafetyChecker for SafetyAdapter {
    async fn can_reply(&self) -> bool {
        match self.guard.can_reply_to("__check__", None).await {
            Ok(Ok(())) => true,
            Ok(Err(reason)) => {
                tracing::debug!(reason = %reason, "Safety check denied reply");
                false
            }
            Err(e) => {
                tracing::warn!(error = %e, "Safety check error, denying reply");
                false
            }
        }
    }

    async fn has_replied_to(&self, tweet_id: &str) -> bool {
        match self.guard.dedup_checker().has_replied_to(tweet_id).await {
            Ok(replied) => replied,
            Err(e) => {
                tracing::warn!(error = %e, "Dedup check error, assuming already replied");
                true
            }
        }
    }

    async fn record_reply(&self, tweet_id: &str, reply_content: &str) -> Result<(), LoopError> {
        // Insert a reply record for dedup tracking.
        let reply = storage::replies::ReplySent {
            id: 0,
            target_tweet_id: tweet_id.to_string(),
            reply_tweet_id: None,
            reply_content: reply_content.to_string(),
            llm_provider: None,
            llm_model: None,
            created_at: Utc::now().to_rfc3339(),
            status: "pending".to_string(),
            error_message: None,
        };
        storage::replies::insert_reply(&self.pool, &reply)
            .await
            .map_err(storage_to_loop_error)?;

        // Increment rate limit counter.
        self.guard
            .record_reply()
            .await
            .map_err(storage_to_loop_error)?;

        Ok(())
    }
}

/// Adapts `SafetyGuard` to the `ContentSafety` port trait.
pub struct ContentSafetyAdapter {
    guard: Arc<SafetyGuard>,
}

impl ContentSafetyAdapter {
    pub fn new(guard: Arc<SafetyGuard>) -> Self {
        Self { guard }
    }
}

#[async_trait::async_trait]
impl ContentSafety for ContentSafetyAdapter {
    async fn can_post_tweet(&self) -> bool {
        match self.guard.can_post_tweet().await {
            Ok(Ok(())) => true,
            Ok(Err(reason)) => {
                tracing::debug!(reason = %reason, "Safety check denied tweet");
                false
            }
            Err(e) => {
                tracing::warn!(error = %e, "Safety check error, denying tweet");
                false
            }
        }
    }

    async fn can_post_thread(&self) -> bool {
        match self.guard.can_post_thread().await {
            Ok(Ok(())) => true,
            Ok(Err(reason)) => {
                tracing::debug!(reason = %reason, "Safety check denied thread");
                false
            }
            Err(e) => {
                tracing::warn!(error = %e, "Safety check error, denying thread");
                false
            }
        }
    }
}

// ============================================================================
// Storage adapters
// ============================================================================

/// Adapts `DbPool` to the `LoopStorage` port trait.
///
/// Provides cursor persistence (via the `cursors` table), tweet dedup,
/// discovered tweet recording, and action logging.
pub struct StorageAdapter {
    pool: DbPool,
}

impl StorageAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LoopStorage for StorageAdapter {
    async fn get_cursor(&self, key: &str) -> Result<Option<String>, LoopError> {
        storage::cursors::get_cursor(&self.pool, key)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn set_cursor(&self, key: &str, value: &str) -> Result<(), LoopError> {
        storage::cursors::set_cursor(&self.pool, key, value)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError> {
        storage::tweets::tweet_exists(&self.pool, tweet_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn store_discovered_tweet(
        &self,
        tweet: &LoopTweet,
        score: f32,
        keyword: &str,
    ) -> Result<(), LoopError> {
        let discovered = storage::tweets::DiscoveredTweet {
            id: tweet.id.clone(),
            author_id: tweet.author_id.clone(),
            author_username: tweet.author_username.clone(),
            content: tweet.text.clone(),
            like_count: tweet.likes as i64,
            retweet_count: tweet.retweets as i64,
            reply_count: tweet.replies as i64,
            impression_count: None,
            relevance_score: Some(score as f64),
            matched_keyword: Some(keyword.to_string()),
            discovered_at: Utc::now().to_rfc3339(),
            replied_to: 0,
        };
        storage::tweets::insert_discovered_tweet(&self.pool, &discovered)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), LoopError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(storage_to_loop_error)
    }
}

/// Adapts `DbPool` + posting queue to the `ContentStorage` port trait.
pub struct ContentStorageAdapter {
    pool: DbPool,
    post_tx: mpsc::Sender<PostAction>,
}

impl ContentStorageAdapter {
    pub fn new(pool: DbPool, post_tx: mpsc::Sender<PostAction>) -> Self {
        Self { pool, post_tx }
    }
}

#[async_trait::async_trait]
impl ContentStorage for ContentStorageAdapter {
    async fn last_tweet_time(&self) -> Result<Option<DateTime<Utc>>, ContentLoopError> {
        let time_str = storage::threads::get_last_original_tweet_time(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(time_str.and_then(|s| parse_datetime(&s)))
    }

    async fn todays_tweet_times(&self) -> Result<Vec<DateTime<Utc>>, ContentLoopError> {
        let time_strs = storage::threads::get_todays_tweet_times(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(time_strs.iter().filter_map(|s| parse_datetime(s)).collect())
    }

    async fn last_thread_time(&self) -> Result<Option<DateTime<Utc>>, ContentLoopError> {
        let time_str = storage::threads::get_last_thread_time(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(time_str.and_then(|s| parse_datetime(&s)))
    }

    async fn post_tweet(&self, topic: &str, content: &str) -> Result<(), ContentLoopError> {
        // Send to the posting queue and await result.
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        self.post_tx
            .send(PostAction::Tweet {
                content: content.to_string(),
                media_ids: vec![],
                result_tx: Some(result_tx),
            })
            .await
            .map_err(|e| ContentLoopError::PostFailed(e.to_string()))?;

        let tweet_id = result_rx
            .await
            .map_err(|e| ContentLoopError::PostFailed(e.to_string()))?
            .map_err(ContentLoopError::PostFailed)?;

        // Record in the database.
        let original = storage::threads::OriginalTweet {
            id: 0,
            tweet_id: Some(tweet_id),
            content: content.to_string(),
            topic: Some(topic.to_string()),
            llm_provider: None,
            created_at: Utc::now().to_rfc3339(),
            status: "sent".to_string(),
            error_message: None,
        };
        storage::threads::insert_original_tweet(&self.pool, &original)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        // Increment rate limit.
        storage::rate_limits::increment_rate_limit(&self.pool, "tweet")
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn create_thread(
        &self,
        topic: &str,
        tweet_count: usize,
    ) -> Result<String, ContentLoopError> {
        let thread = storage::threads::Thread {
            id: 0,
            topic: topic.to_string(),
            tweet_count: tweet_count as i64,
            root_tweet_id: None,
            created_at: Utc::now().to_rfc3339(),
            status: "pending".to_string(),
        };
        let id = storage::threads::insert_thread(&self.pool, &thread)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(id.to_string())
    }

    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: &str,
        tweet_count: usize,
        root_tweet_id: Option<&str>,
    ) -> Result<(), ContentLoopError> {
        let id: i64 = thread_id
            .parse()
            .map_err(|_| ContentLoopError::StorageError("invalid thread_id".to_string()))?;

        sqlx::query(
            "UPDATE threads SET status = ?1, tweet_count = ?2, root_tweet_id = ?3 WHERE id = ?4",
        )
        .bind(status)
        .bind(tweet_count as i64)
        .bind(root_tweet_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_content_error)?;

        // If the thread was fully posted, increment the rate limit.
        if status == "sent" {
            storage::rate_limits::increment_rate_limit(&self.pool, "thread")
                .await
                .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        }

        Ok(())
    }

    async fn store_thread_tweet(
        &self,
        thread_id: &str,
        position: usize,
        tweet_id: &str,
        content: &str,
    ) -> Result<(), ContentLoopError> {
        let tid: i64 = thread_id
            .parse()
            .map_err(|_| ContentLoopError::StorageError("invalid thread_id".to_string()))?;

        sqlx::query(
            "INSERT INTO thread_tweets (thread_id, position, tweet_id, content, created_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        )
        .bind(tid)
        .bind(position as i64)
        .bind(tweet_id)
        .bind(content)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_content_error)?;

        Ok(())
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), ContentLoopError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))
    }

    async fn next_scheduled_item(&self) -> Result<Option<(i64, String, String)>, ContentLoopError> {
        let items = storage::scheduled_content::get_due_items(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        Ok(items
            .into_iter()
            .next()
            .map(|item| (item.id, item.content_type, item.content)))
    }

    async fn mark_scheduled_posted(
        &self,
        id: i64,
        tweet_id: Option<&str>,
    ) -> Result<(), ContentLoopError> {
        storage::scheduled_content::update_status(&self.pool, id, "posted", tweet_id)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))
    }
}

/// Adapts `DbPool` to the `TargetStorage` port trait.
pub struct TargetStorageAdapter {
    pool: DbPool,
}

impl TargetStorageAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TargetStorage for TargetStorageAdapter {
    async fn upsert_target_account(
        &self,
        account_id: &str,
        username: &str,
    ) -> Result<(), LoopError> {
        storage::target_accounts::upsert_target_account(&self.pool, account_id, username)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn target_tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError> {
        storage::target_accounts::target_tweet_exists(&self.pool, tweet_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn store_target_tweet(
        &self,
        tweet_id: &str,
        account_id: &str,
        content: &str,
        created_at: &str,
        reply_count: i64,
        like_count: i64,
        relevance_score: f64,
    ) -> Result<(), LoopError> {
        storage::target_accounts::store_target_tweet(
            &self.pool,
            tweet_id,
            account_id,
            content,
            created_at,
            reply_count,
            like_count,
            relevance_score,
        )
        .await
        .map_err(storage_to_loop_error)
    }

    async fn mark_target_tweet_replied(&self, tweet_id: &str) -> Result<(), LoopError> {
        storage::target_accounts::mark_target_tweet_replied(&self.pool, tweet_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn record_target_reply(&self, account_id: &str) -> Result<(), LoopError> {
        storage::target_accounts::record_target_reply(&self.pool, account_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn count_target_replies_today(&self) -> Result<i64, LoopError> {
        storage::target_accounts::count_target_replies_today(&self.pool)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), LoopError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(storage_to_loop_error)
    }
}

/// Adapts `DbPool` to the `AnalyticsStorage` port trait.
pub struct AnalyticsStorageAdapter {
    pool: DbPool,
}

impl AnalyticsStorageAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AnalyticsStorage for AnalyticsStorageAdapter {
    async fn store_follower_snapshot(
        &self,
        followers: i64,
        following: i64,
        tweets: i64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::upsert_follower_snapshot(&self.pool, followers, following, tweets)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn get_yesterday_followers(&self) -> Result<Option<i64>, AnalyticsError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT follower_count FROM follower_snapshots
             WHERE snapshot_date < date('now')
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        Ok(row.map(|(c,)| c))
    }

    async fn get_replies_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT rs.reply_tweet_id FROM replies_sent rs
             WHERE rs.status = 'sent'
               AND rs.reply_tweet_id IS NOT NULL
               AND rs.created_at >= datetime('now', '-25 hours')
               AND rs.created_at <= datetime('now', '-23 hours')
               AND NOT EXISTS (
                   SELECT 1 FROM reply_performance rp WHERE rp.reply_id = rs.reply_tweet_id
               )",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn get_tweets_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT ot.tweet_id FROM original_tweets ot
             WHERE ot.status = 'sent'
               AND ot.tweet_id IS NOT NULL
               AND ot.created_at >= datetime('now', '-25 hours')
               AND ot.created_at <= datetime('now', '-23 hours')
               AND NOT EXISTS (
                   SELECT 1 FROM tweet_performance tp WHERE tp.tweet_id = ot.tweet_id
               )",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn store_reply_performance(
        &self,
        reply_id: &str,
        likes: i64,
        replies: i64,
        impressions: i64,
        score: f64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::upsert_reply_performance(
            &self.pool,
            reply_id,
            likes,
            replies,
            impressions,
            score,
        )
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn store_tweet_performance(
        &self,
        tweet_id: &str,
        likes: i64,
        retweets: i64,
        replies: i64,
        impressions: i64,
        score: f64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::upsert_tweet_performance(
            &self.pool,
            tweet_id,
            likes,
            retweets,
            replies,
            impressions,
            score,
        )
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn update_content_score(
        &self,
        topic: &str,
        format: &str,
        score: f64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::update_content_score(&self.pool, topic, format, score)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), AnalyticsError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }
}

/// Adapts `DbPool` to the `TopicScorer` port trait.
pub struct TopicScorerAdapter {
    pool: DbPool,
}

impl TopicScorerAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TopicScorer for TopicScorerAdapter {
    async fn get_top_topics(&self, limit: u32) -> Result<Vec<String>, ContentLoopError> {
        let scores = storage::analytics::get_top_topics(&self.pool, limit)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(scores.into_iter().map(|cs| cs.topic).collect())
    }
}

// ============================================================================
// Posting queue adapters
// ============================================================================

/// Adapts `mpsc::Sender<PostAction>` to the `PostSender` port trait.
pub struct PostSenderAdapter {
    tx: mpsc::Sender<PostAction>,
}

impl PostSenderAdapter {
    pub fn new(tx: mpsc::Sender<PostAction>) -> Self {
        Self { tx }
    }
}

#[async_trait::async_trait]
impl PostSender for PostSenderAdapter {
    async fn send_reply(&self, tweet_id: &str, content: &str) -> Result<(), LoopError> {
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(PostAction::Reply {
                tweet_id: tweet_id.to_string(),
                content: content.to_string(),
                media_ids: vec![],
                result_tx: Some(result_tx),
            })
            .await
            .map_err(|e| LoopError::Other(format!("posting queue send failed: {e}")))?;

        result_rx
            .await
            .map_err(|e| LoopError::Other(format!("posting queue result recv failed: {e}")))?
            .map_err(|e| LoopError::Other(format!("post action failed: {e}")))?;

        Ok(())
    }
}

/// Adapts `DbPool` to the `ApprovalQueue` port trait.
pub struct ApprovalQueueAdapter {
    pool: DbPool,
}

impl ApprovalQueueAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApprovalQueue for ApprovalQueueAdapter {
    async fn queue_reply(
        &self,
        tweet_id: &str,
        content: &str,
        media_paths: &[String],
    ) -> Result<i64, String> {
        let media_json = serde_json::to_string(media_paths).unwrap_or_else(|_| "[]".to_string());
        storage::approval_queue::enqueue(
            &self.pool,
            "reply",
            tweet_id,
            "", // target_author not available here
            content,
            "",  // topic
            "",  // archetype
            0.0, // score
            &media_json,
        )
        .await
        .map_err(|e| e.to_string())
    }

    async fn queue_tweet(&self, content: &str, media_paths: &[String]) -> Result<i64, String> {
        let media_json = serde_json::to_string(media_paths).unwrap_or_else(|_| "[]".to_string());
        storage::approval_queue::enqueue(
            &self.pool,
            "tweet",
            "", // no target tweet
            "", // no target author
            content,
            "",  // topic
            "",  // archetype
            0.0, // score
            &media_json,
        )
        .await
        .map_err(|e| e.to_string())
    }
}

// ============================================================================
// Status reporter adapter
// ============================================================================

/// Adapts `DbPool` to the `StatusQuerier` port trait.
pub struct StatusQuerierAdapter {
    pool: DbPool,
}

impl StatusQuerierAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StatusQuerier for StatusQuerierAdapter {
    async fn query_action_counts_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<ActionCounts, String> {
        let since_str = since.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let counts = storage::action_log::get_action_counts_since(&self.pool, &since_str)
            .await
            .map_err(|e| e.to_string())?;

        Ok(ActionCounts {
            tweets_scored: *counts.get("tweet_scored").unwrap_or(&0) as u64,
            replies_sent: *counts.get("reply_sent").unwrap_or(&0) as u64,
            tweets_posted: *counts.get("tweet_posted").unwrap_or(&0) as u64,
            threads_posted: *counts.get("thread_posted").unwrap_or(&0) as u64,
        })
    }
}
