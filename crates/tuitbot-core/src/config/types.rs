//! Configuration section structs and their serde default functions.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// X API
// ---------------------------------------------------------------------------

/// X API credentials.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct XApiConfig {
    /// OAuth 2.0 client ID.
    #[serde(default)]
    pub client_id: String,

    /// OAuth 2.0 client secret (optional for public clients).
    #[serde(default)]
    pub client_secret: Option<String>,

    /// Provider backend: `"x_api"` (default) or `"scraper"`.
    #[serde(default)]
    pub provider_backend: String,

    /// Whether scraper backend is allowed to perform mutations.
    /// Only meaningful when `provider_backend = "scraper"`. Default: `false`.
    #[serde(default)]
    pub scraper_allow_mutations: bool,
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

/// Authentication mode and callback settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Auth mode: "manual" or "local_callback".
    #[serde(default = "default_auth_mode")]
    pub mode: String,

    /// Host for local callback server.
    #[serde(default = "default_callback_host")]
    pub callback_host: String,

    /// Port for local callback server.
    #[serde(default = "default_callback_port")]
    pub callback_port: u16,
}

// ---------------------------------------------------------------------------
// Business Profile
// ---------------------------------------------------------------------------

/// Business profile for content targeting and keyword matching.
///
/// Fields are grouped into two tiers:
///
/// **Quickstart fields** (required for a working config):
/// - `product_name`, `product_keywords`
///
/// **Optional context** (improve targeting but have sane defaults):
/// - `product_description`, `product_url`, `target_audience`,
///   `competitor_keywords`, `industry_topics`
///
/// **Enrichment fields** (shape voice/persona — unlocked via progressive setup):
/// - `brand_voice`, `reply_style`, `content_style`,
///   `persona_opinions`, `persona_experiences`, `content_pillars`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BusinessProfile {
    // -- Quickstart fields --
    /// Name of the user's product.
    #[serde(default)]
    pub product_name: String,

    /// Keywords for tweet discovery.
    #[serde(default)]
    pub product_keywords: Vec<String>,

    // -- Optional context --
    /// One-line description of the product.
    #[serde(default)]
    pub product_description: String,

    /// URL to the product website.
    #[serde(default)]
    pub product_url: Option<String>,

    /// Description of the target audience.
    #[serde(default)]
    pub target_audience: String,

    /// Competitor-related keywords for discovery.
    #[serde(default)]
    pub competitor_keywords: Vec<String>,

    /// Topics for content generation. Defaults to `product_keywords` when empty
    /// (see [`Self::effective_industry_topics`]).
    #[serde(default)]
    pub industry_topics: Vec<String>,

    // -- Enrichment fields --
    /// Brand voice / personality description for all generated content.
    #[serde(default)]
    pub brand_voice: Option<String>,

    /// Style guidelines specific to replies.
    #[serde(default)]
    pub reply_style: Option<String>,

    /// Style guidelines specific to original tweets and threads.
    #[serde(default)]
    pub content_style: Option<String>,

    /// Opinions the persona holds (used to add variety to generated content).
    #[serde(default)]
    pub persona_opinions: Vec<String>,

    /// Experiences the persona can reference (keeps content authentic).
    #[serde(default)]
    pub persona_experiences: Vec<String>,

    /// Core content pillars (broad themes the account focuses on).
    #[serde(default)]
    pub content_pillars: Vec<String>,
}

impl BusinessProfile {
    /// Create a quickstart profile with only the required fields.
    ///
    /// Copies `product_keywords` into `industry_topics` so content loops
    /// have topics to work with even without explicit configuration.
    pub fn quickstart(product_name: String, product_keywords: Vec<String>) -> Self {
        Self {
            product_name,
            industry_topics: product_keywords.clone(),
            product_keywords,
            ..Default::default()
        }
    }

    /// Returns the effective industry topics for content generation.
    ///
    /// If `industry_topics` is non-empty, returns it directly.
    /// Otherwise falls back to `product_keywords`, so quickstart users
    /// never need to configure topics separately.
    pub fn effective_industry_topics(&self) -> &[String] {
        if self.industry_topics.is_empty() {
            &self.product_keywords
        } else {
            &self.industry_topics
        }
    }

    /// Returns `true` if any enrichment field has been set.
    ///
    /// Enrichment fields are: `brand_voice`, `reply_style`, `content_style`,
    /// `persona_opinions`, `persona_experiences`, `content_pillars`.
    /// Used by progressive enrichment to decide whether to show setup hints.
    /// Returns the merged keyword set used for draft-context retrieval.
    ///
    /// Combines `product_keywords`, `competitor_keywords`, and the
    /// effective industry topics into a single owned `Vec<String>`.
    /// This is the single source of truth for keyword assembly across
    /// draft workflows, composer RAG resolution, and engagement scoring.
    pub fn draft_context_keywords(&self) -> Vec<String> {
        let mut keywords: Vec<String> = self.product_keywords.clone();
        keywords.extend(self.competitor_keywords.clone());
        keywords.extend(self.effective_industry_topics().to_vec());
        keywords
    }

    pub fn is_enriched(&self) -> bool {
        self.brand_voice.as_ref().is_some_and(|v| !v.is_empty())
            || self.reply_style.as_ref().is_some_and(|v| !v.is_empty())
            || self.content_style.as_ref().is_some_and(|v| !v.is_empty())
            || !self.persona_opinions.is_empty()
            || !self.persona_experiences.is_empty()
            || !self.content_pillars.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

/// Scoring engine weights and threshold.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoringConfig {
    /// Minimum score (0-100) to trigger a reply.
    #[serde(default = "default_threshold")]
    pub threshold: u32,

    /// Maximum points for keyword relevance.
    #[serde(default = "default_keyword_relevance_max")]
    pub keyword_relevance_max: f32,

    /// Maximum points for author follower count.
    #[serde(default = "default_follower_count_max")]
    pub follower_count_max: f32,

    /// Maximum points for tweet recency.
    #[serde(default = "default_recency_max")]
    pub recency_max: f32,

    /// Maximum points for engagement rate.
    #[serde(default = "default_engagement_rate_max")]
    pub engagement_rate_max: f32,

    /// Maximum points for reply count signal (fewer replies = higher score).
    #[serde(default = "default_reply_count_max")]
    pub reply_count_max: f32,

    /// Maximum points for content type signal (text-only originals score highest).
    #[serde(default = "default_content_type_max")]
    pub content_type_max: f32,
}

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

/// Safety limits for API actions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    /// Maximum replies per day.
    #[serde(default = "default_max_replies_per_day")]
    pub max_replies_per_day: u32,

    /// Maximum original tweets per day.
    #[serde(default = "default_max_tweets_per_day")]
    pub max_tweets_per_day: u32,

    /// Maximum threads per week.
    #[serde(default = "default_max_threads_per_week")]
    pub max_threads_per_week: u32,

    /// Minimum delay between actions in seconds.
    #[serde(default = "default_min_action_delay_seconds")]
    pub min_action_delay_seconds: u64,

    /// Maximum delay between actions in seconds.
    #[serde(default = "default_max_action_delay_seconds")]
    pub max_action_delay_seconds: u64,

    /// Maximum replies to the same author per day.
    #[serde(default = "default_max_replies_per_author_per_day")]
    pub max_replies_per_author_per_day: u32,

    /// Phrases that should never appear in generated replies.
    #[serde(default = "default_banned_phrases")]
    pub banned_phrases: Vec<String>,

    /// Fraction of replies that may mention the product (0.0 - 1.0).
    #[serde(default = "default_product_mention_ratio")]
    pub product_mention_ratio: f32,
}

// ---------------------------------------------------------------------------
// Intervals
// ---------------------------------------------------------------------------

/// Automation interval settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntervalsConfig {
    /// Seconds between mention checks.
    #[serde(default = "default_mentions_check_seconds")]
    pub mentions_check_seconds: u64,

    /// Seconds between discovery searches.
    #[serde(default = "default_discovery_search_seconds")]
    pub discovery_search_seconds: u64,

    /// Seconds for content post window.
    #[serde(default = "default_content_post_window_seconds")]
    pub content_post_window_seconds: u64,

    /// Seconds between thread posts.
    #[serde(default = "default_thread_interval_seconds")]
    pub thread_interval_seconds: u64,
}

// ---------------------------------------------------------------------------
// Targets
// ---------------------------------------------------------------------------

/// Target account monitoring configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetsConfig {
    /// Target account usernames to monitor (without @).
    #[serde(default)]
    pub accounts: Vec<String>,

    /// Maximum target account replies per day (separate from general limit).
    #[serde(default = "default_max_target_replies_per_day")]
    pub max_target_replies_per_day: u32,
}

fn default_max_target_replies_per_day() -> u32 {
    3
}

// ---------------------------------------------------------------------------
// LLM
// ---------------------------------------------------------------------------

/// LLM provider configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LlmConfig {
    /// LLM provider name: "openai", "anthropic", "ollama", or "groq".
    #[serde(default)]
    pub provider: String,

    /// API key for the LLM provider (not needed for ollama).
    #[serde(default)]
    pub api_key: Option<String>,

    /// Provider-specific model name.
    #[serde(default)]
    pub model: String,

    /// Override URL for custom endpoints.
    #[serde(default)]
    pub base_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

/// Data storage configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Path to the SQLite database file.
    #[serde(default = "default_db_path")]
    pub db_path: String,

    /// Number of days to retain data.
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

/// Server binding configuration for LAN access.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Host address to bind to. Use "0.0.0.0" for LAN access.
    #[serde(default = "default_server_host")]
    pub host: String,

    /// Port to listen on.
    #[serde(default = "default_server_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_host(),
            port: default_server_port(),
        }
    }
}

fn default_server_host() -> String {
    "127.0.0.1".to_string()
}
fn default_server_port() -> u16 {
    3001
}

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

/// Logging and observability settings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Seconds between periodic status summaries (0 = disabled).
    #[serde(default)]
    pub status_interval_seconds: u64,
}

// ---------------------------------------------------------------------------
// Serde default value functions
// ---------------------------------------------------------------------------

fn default_auth_mode() -> String {
    "manual".to_string()
}
fn default_callback_host() -> String {
    "127.0.0.1".to_string()
}
fn default_callback_port() -> u16 {
    8080
}
fn default_threshold() -> u32 {
    60
}
fn default_keyword_relevance_max() -> f32 {
    25.0
}
fn default_follower_count_max() -> f32 {
    15.0
}
fn default_recency_max() -> f32 {
    10.0
}
fn default_engagement_rate_max() -> f32 {
    15.0
}
fn default_reply_count_max() -> f32 {
    15.0
}
fn default_content_type_max() -> f32 {
    10.0
}
fn default_max_replies_per_day() -> u32 {
    5
}
fn default_max_tweets_per_day() -> u32 {
    6
}
fn default_max_threads_per_week() -> u32 {
    1
}
fn default_min_action_delay_seconds() -> u64 {
    45
}
fn default_max_action_delay_seconds() -> u64 {
    180
}
fn default_mentions_check_seconds() -> u64 {
    300
}
fn default_discovery_search_seconds() -> u64 {
    900
}
fn default_content_post_window_seconds() -> u64 {
    10800
}
fn default_thread_interval_seconds() -> u64 {
    604800
}
fn default_max_replies_per_author_per_day() -> u32 {
    1
}
fn default_banned_phrases() -> Vec<String> {
    vec![
        "check out".to_string(),
        "you should try".to_string(),
        "I recommend".to_string(),
        "link in bio".to_string(),
    ]
}
fn default_product_mention_ratio() -> f32 {
    0.2
}
fn default_db_path() -> String {
    "~/.tuitbot/tuitbot.db".to_string()
}
fn default_retention_days() -> u32 {
    90
}

// ---------------------------------------------------------------------------
// Content Sources
// ---------------------------------------------------------------------------

/// Content source configuration for the Watchtower.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ContentSourcesConfig {
    /// Configured content sources.
    #[serde(default)]
    pub sources: Vec<ContentSourceEntry>,
}

/// A single content source entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentSourceEntry {
    /// Source type: `"local_fs"` or `"google_drive"`.
    #[serde(default = "default_source_type")]
    pub source_type: String,

    /// Filesystem path (for local_fs sources). Supports ~ expansion.
    #[serde(default)]
    pub path: Option<String>,

    /// Google Drive folder ID (for google_drive sources).
    #[serde(default)]
    pub folder_id: Option<String>,

    /// Path to a Google service-account JSON key file (for google_drive sources).
    /// Legacy field -- new installs use `connection_id` with OAuth 2.0 instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account_key: Option<String>,

    /// Reference to a row in the `connections` table for remote sources.
    /// When set, the Watchtower uses the linked account's credentials
    /// instead of `service_account_key`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<i64>,

    /// Whether to watch for changes in real-time.
    ///
    /// **Deprecated:** Use `enabled` and `change_detection` instead.
    /// Kept for backward compatibility — when `enabled` is `None`, the
    /// value of `watch` is used as the fallback.
    #[serde(default = "default_watch")]
    pub watch: bool,

    /// Whether this source participates in ingestion at all.
    ///
    /// When `None`, falls back to `watch` for backward compatibility.
    /// When `Some(false)`, the source is completely skipped.
    #[serde(default)]
    pub enabled: Option<bool>,

    /// How changes are detected for this source.
    ///
    /// - `"auto"` (default) — local_fs: notify watcher + fallback poll;
    ///   google_drive: interval poll.
    /// - `"poll"` — poll only (useful when notify is unreliable, e.g. NFS).
    /// - `"none"` — initial scan only, no ongoing monitoring.
    #[serde(default = "default_change_detection")]
    pub change_detection: String,

    /// File patterns to include.
    #[serde(default = "default_file_patterns")]
    pub file_patterns: Vec<String>,

    /// Whether to write metadata back to source files.
    #[serde(default = "default_loop_back")]
    pub loop_back_enabled: bool,

    /// Polling interval in seconds for remote sources (default: 300 = 5 min).
    #[serde(default)]
    pub poll_interval_seconds: Option<u64>,
}

/// Valid values for `ContentSourceEntry::change_detection`.
pub const CHANGE_DETECTION_AUTO: &str = "auto";
pub const CHANGE_DETECTION_POLL: &str = "poll";
pub const CHANGE_DETECTION_NONE: &str = "none";

/// Minimum allowed poll interval in seconds.
pub const MIN_POLL_INTERVAL_SECONDS: u64 = 30;

impl ContentSourceEntry {
    /// Whether this source should participate in ingestion.
    ///
    /// Prefers `enabled` when explicitly set; otherwise falls back to
    /// the legacy `watch` field for backward compatibility.
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(self.watch)
    }

    /// The effective change detection mode for this source.
    ///
    /// Returns `"none"` when the source is disabled (short-circuit).
    pub fn effective_change_detection(&self) -> &str {
        if !self.is_enabled() {
            return CHANGE_DETECTION_NONE;
        }
        &self.change_detection
    }

    /// Whether this source uses poll-only change detection.
    pub fn is_poll_only(&self) -> bool {
        self.effective_change_detection() == CHANGE_DETECTION_POLL
    }

    /// Whether this source should only do an initial scan with no ongoing monitoring.
    pub fn is_scan_only(&self) -> bool {
        self.effective_change_detection() == CHANGE_DETECTION_NONE
    }
}

fn default_source_type() -> String {
    "local_fs".to_string()
}
fn default_watch() -> bool {
    true
}
fn default_change_detection() -> String {
    CHANGE_DETECTION_AUTO.to_string()
}
fn default_file_patterns() -> Vec<String> {
    vec!["*.md".to_string(), "*.txt".to_string()]
}
fn default_loop_back() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- XApiConfig ---

    #[test]
    fn x_api_config_default() {
        let cfg = XApiConfig::default();
        assert!(cfg.client_id.is_empty());
        assert!(cfg.client_secret.is_none());
        assert!(cfg.provider_backend.is_empty());
        assert!(!cfg.scraper_allow_mutations);
    }

    #[test]
    fn x_api_config_serde_roundtrip() {
        let cfg = XApiConfig {
            client_id: "my-client-id".into(),
            client_secret: Some("secret".into()),
            provider_backend: "x_api".into(),
            scraper_allow_mutations: true,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: XApiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.client_id, "my-client-id");
        assert_eq!(back.client_secret.as_deref(), Some("secret"));
        assert_eq!(back.provider_backend, "x_api");
        assert!(back.scraper_allow_mutations);
    }

    #[test]
    fn x_api_config_deserialize_empty() {
        let cfg: XApiConfig = serde_json::from_str("{}").unwrap();
        assert!(cfg.client_id.is_empty());
        assert!(cfg.client_secret.is_none());
        assert!(!cfg.scraper_allow_mutations);
    }

    // --- AuthConfig ---

    #[test]
    fn auth_config_serde_roundtrip() {
        let cfg = AuthConfig {
            mode: "local_callback".into(),
            callback_host: "0.0.0.0".into(),
            callback_port: 9090,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mode, "local_callback");
        assert_eq!(back.callback_host, "0.0.0.0");
        assert_eq!(back.callback_port, 9090);
    }

    #[test]
    fn auth_config_deserialize_defaults() {
        let cfg: AuthConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(cfg.mode, "manual");
        assert_eq!(cfg.callback_host, "127.0.0.1");
        assert_eq!(cfg.callback_port, 8080);
    }

    // --- BusinessProfile ---

    #[test]
    fn business_profile_default() {
        let bp = BusinessProfile::default();
        assert!(bp.product_name.is_empty());
        assert!(bp.product_keywords.is_empty());
        assert!(bp.product_description.is_empty());
        assert!(bp.product_url.is_none());
        assert!(bp.target_audience.is_empty());
        assert!(bp.competitor_keywords.is_empty());
        assert!(bp.industry_topics.is_empty());
        assert!(bp.brand_voice.is_none());
        assert!(bp.reply_style.is_none());
        assert!(bp.content_style.is_none());
        assert!(bp.persona_opinions.is_empty());
        assert!(bp.persona_experiences.is_empty());
        assert!(bp.content_pillars.is_empty());
    }

    #[test]
    fn business_profile_quickstart() {
        let bp = BusinessProfile::quickstart(
            "MyApp".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
        );
        assert_eq!(bp.product_name, "MyApp");
        assert_eq!(bp.product_keywords, vec!["rust", "cli"]);
        assert_eq!(bp.industry_topics, vec!["rust", "cli"]);
        assert!(bp.product_description.is_empty());
    }

    #[test]
    fn business_profile_effective_industry_topics_nonempty() {
        let bp = BusinessProfile {
            product_keywords: vec!["a".into()],
            industry_topics: vec!["b".into(), "c".into()],
            ..Default::default()
        };
        assert_eq!(bp.effective_industry_topics(), &["b", "c"]);
    }

    #[test]
    fn business_profile_effective_industry_topics_empty_falls_back() {
        let bp = BusinessProfile {
            product_keywords: vec!["fallback".into()],
            industry_topics: vec![],
            ..Default::default()
        };
        assert_eq!(bp.effective_industry_topics(), &["fallback"]);
    }

    #[test]
    fn business_profile_draft_context_keywords() {
        let bp = BusinessProfile {
            product_keywords: vec!["prod".into()],
            competitor_keywords: vec!["comp".into()],
            industry_topics: vec!["topic".into()],
            ..Default::default()
        };
        let kw = bp.draft_context_keywords();
        assert!(kw.contains(&"prod".to_string()));
        assert!(kw.contains(&"comp".to_string()));
        assert!(kw.contains(&"topic".to_string()));
    }

    #[test]
    fn business_profile_draft_context_keywords_dedup_with_fallback() {
        let bp = BusinessProfile {
            product_keywords: vec!["rust".into()],
            competitor_keywords: vec![],
            industry_topics: vec![], // falls back to product_keywords
            ..Default::default()
        };
        let kw = bp.draft_context_keywords();
        assert_eq!(kw.iter().filter(|k| *k == "rust").count(), 2);
    }

    #[test]
    fn business_profile_is_enriched_false_when_empty() {
        let bp = BusinessProfile::default();
        assert!(!bp.is_enriched());
    }

    #[test]
    fn business_profile_is_enriched_true_with_brand_voice() {
        let bp = BusinessProfile {
            brand_voice: Some("Friendly".into()),
            ..Default::default()
        };
        assert!(bp.is_enriched());
    }

    #[test]
    fn business_profile_is_enriched_true_with_reply_style() {
        let bp = BusinessProfile {
            reply_style: Some("Casual".into()),
            ..Default::default()
        };
        assert!(bp.is_enriched());
    }

    #[test]
    fn business_profile_is_enriched_true_with_content_style() {
        let bp = BusinessProfile {
            content_style: Some("Technical".into()),
            ..Default::default()
        };
        assert!(bp.is_enriched());
    }

    #[test]
    fn business_profile_is_enriched_true_with_opinions() {
        let bp = BusinessProfile {
            persona_opinions: vec!["Rust is great".into()],
            ..Default::default()
        };
        assert!(bp.is_enriched());
    }

    #[test]
    fn business_profile_is_enriched_true_with_experiences() {
        let bp = BusinessProfile {
            persona_experiences: vec!["Built CLI tools".into()],
            ..Default::default()
        };
        assert!(bp.is_enriched());
    }

    #[test]
    fn business_profile_is_enriched_true_with_pillars() {
        let bp = BusinessProfile {
            content_pillars: vec!["Developer productivity".into()],
            ..Default::default()
        };
        assert!(bp.is_enriched());
    }

    #[test]
    fn business_profile_is_enriched_false_with_empty_brand_voice() {
        let bp = BusinessProfile {
            brand_voice: Some(String::new()),
            ..Default::default()
        };
        assert!(!bp.is_enriched());
    }

    #[test]
    fn business_profile_serde_roundtrip() {
        let bp = BusinessProfile {
            product_name: "TestApp".into(),
            product_keywords: vec!["test".into(), "qa".into()],
            product_description: "A testing tool".into(),
            product_url: Some("https://test.com".into()),
            target_audience: "developers".into(),
            competitor_keywords: vec!["alt".into()],
            industry_topics: vec!["testing".into()],
            brand_voice: Some("Friendly".into()),
            reply_style: Some("Casual".into()),
            content_style: Some("Sharp".into()),
            persona_opinions: vec!["Testing first".into()],
            persona_experiences: vec!["5 years QA".into()],
            content_pillars: vec!["Quality".into()],
        };
        let json = serde_json::to_string(&bp).unwrap();
        let back: BusinessProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.product_name, "TestApp");
        assert_eq!(back.product_keywords.len(), 2);
        assert_eq!(back.product_url.as_deref(), Some("https://test.com"));
        assert_eq!(back.brand_voice.as_deref(), Some("Friendly"));
        assert_eq!(back.persona_opinions.len(), 1);
    }

    // --- ScoringConfig ---

    #[test]
    fn scoring_config_deserialize_defaults() {
        let cfg: ScoringConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(cfg.threshold, 60);
        assert!((cfg.keyword_relevance_max - 25.0).abs() < 0.001);
        assert!((cfg.follower_count_max - 15.0).abs() < 0.001);
        assert!((cfg.recency_max - 10.0).abs() < 0.001);
        assert!((cfg.engagement_rate_max - 15.0).abs() < 0.001);
        assert!((cfg.reply_count_max - 15.0).abs() < 0.001);
        assert!((cfg.content_type_max - 10.0).abs() < 0.001);
    }

    #[test]
    fn scoring_config_serde_roundtrip() {
        let cfg = ScoringConfig {
            threshold: 80,
            keyword_relevance_max: 30.0,
            follower_count_max: 20.0,
            recency_max: 15.0,
            engagement_rate_max: 20.0,
            reply_count_max: 10.0,
            content_type_max: 5.0,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ScoringConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.threshold, 80);
        assert!((back.keyword_relevance_max - 30.0).abs() < 0.001);
    }

    // --- LimitsConfig ---

    #[test]
    fn limits_config_deserialize_defaults() {
        let cfg: LimitsConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(cfg.max_replies_per_day, 5);
        assert_eq!(cfg.max_tweets_per_day, 6);
        assert_eq!(cfg.max_threads_per_week, 1);
        assert_eq!(cfg.min_action_delay_seconds, 45);
        assert_eq!(cfg.max_action_delay_seconds, 180);
        assert_eq!(cfg.max_replies_per_author_per_day, 1);
        assert!(!cfg.banned_phrases.is_empty());
        assert!((cfg.product_mention_ratio - 0.2).abs() < 0.001);
    }

    #[test]
    fn limits_config_serde_roundtrip() {
        let cfg = LimitsConfig {
            max_replies_per_day: 10,
            max_tweets_per_day: 8,
            max_threads_per_week: 3,
            min_action_delay_seconds: 60,
            max_action_delay_seconds: 300,
            max_replies_per_author_per_day: 2,
            banned_phrases: vec!["spam".into()],
            product_mention_ratio: 0.3,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: LimitsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_replies_per_day, 10);
        assert_eq!(back.max_tweets_per_day, 8);
        assert_eq!(back.banned_phrases, vec!["spam"]);
    }

    // --- IntervalsConfig ---

    #[test]
    fn intervals_config_deserialize_defaults() {
        let cfg: IntervalsConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(cfg.mentions_check_seconds, 300);
        assert_eq!(cfg.discovery_search_seconds, 900);
        assert_eq!(cfg.content_post_window_seconds, 10800);
        assert_eq!(cfg.thread_interval_seconds, 604800);
    }

    #[test]
    fn intervals_config_serde_roundtrip() {
        let cfg = IntervalsConfig {
            mentions_check_seconds: 120,
            discovery_search_seconds: 600,
            content_post_window_seconds: 7200,
            thread_interval_seconds: 86400,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: IntervalsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mentions_check_seconds, 120);
        assert_eq!(back.discovery_search_seconds, 600);
    }

    // --- TargetsConfig ---

    #[test]
    fn targets_config_default() {
        let cfg = TargetsConfig::default();
        assert!(cfg.accounts.is_empty());
        assert_eq!(cfg.max_target_replies_per_day, 0);
    }

    #[test]
    fn targets_config_serde_roundtrip() {
        let cfg = TargetsConfig {
            accounts: vec!["user1".into(), "user2".into()],
            max_target_replies_per_day: 5,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: TargetsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.accounts.len(), 2);
        assert_eq!(back.max_target_replies_per_day, 5);
    }

    #[test]
    fn targets_config_deserialize_defaults() {
        let cfg: TargetsConfig = serde_json::from_str("{}").unwrap();
        assert!(cfg.accounts.is_empty());
        assert_eq!(cfg.max_target_replies_per_day, 3);
    }

    // --- LlmConfig ---

    #[test]
    fn llm_config_default() {
        let cfg = LlmConfig::default();
        assert!(cfg.provider.is_empty());
        assert!(cfg.api_key.is_none());
        assert!(cfg.model.is_empty());
        assert!(cfg.base_url.is_none());
    }

    #[test]
    fn llm_config_serde_roundtrip() {
        let cfg = LlmConfig {
            provider: "anthropic".into(),
            api_key: Some("sk-test".into()),
            model: "claude-3-5-sonnet".into(),
            base_url: Some("https://api.anthropic.com".into()),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: LlmConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.provider, "anthropic");
        assert_eq!(back.api_key.as_deref(), Some("sk-test"));
        assert_eq!(back.model, "claude-3-5-sonnet");
        assert_eq!(back.base_url.as_deref(), Some("https://api.anthropic.com"));
    }

    // --- StorageConfig ---

    #[test]
    fn storage_config_deserialize_defaults() {
        let cfg: StorageConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(cfg.db_path, "~/.tuitbot/tuitbot.db");
        assert_eq!(cfg.retention_days, 90);
    }

    #[test]
    fn storage_config_serde_roundtrip() {
        let cfg = StorageConfig {
            db_path: "/custom/path.db".into(),
            retention_days: 30,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: StorageConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.db_path, "/custom/path.db");
        assert_eq!(back.retention_days, 30);
    }

    // --- ServerConfig ---

    #[test]
    fn server_config_default() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 3001);
    }

    #[test]
    fn server_config_serde_roundtrip() {
        let cfg = ServerConfig {
            host: "0.0.0.0".into(),
            port: 8080,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.host, "0.0.0.0");
        assert_eq!(back.port, 8080);
    }

    // --- LoggingConfig ---

    #[test]
    fn logging_config_default() {
        let cfg = LoggingConfig::default();
        assert_eq!(cfg.status_interval_seconds, 0);
    }

    #[test]
    fn logging_config_serde_roundtrip() {
        let cfg = LoggingConfig {
            status_interval_seconds: 60,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: LoggingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status_interval_seconds, 60);
    }

    // --- ContentSourcesConfig ---

    #[test]
    fn content_sources_config_default() {
        let cfg = ContentSourcesConfig::default();
        assert!(cfg.sources.is_empty());
    }

    #[test]
    fn content_sources_config_serde_roundtrip() {
        let cfg = ContentSourcesConfig {
            sources: vec![ContentSourceEntry {
                source_type: "local_fs".into(),
                path: Some("/notes".into()),
                folder_id: None,
                service_account_key: None,
                connection_id: None,
                watch: true,
                enabled: Some(true),
                change_detection: "auto".into(),
                file_patterns: vec!["*.md".into()],
                loop_back_enabled: false,
                poll_interval_seconds: None,
            }],
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ContentSourcesConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sources.len(), 1);
        assert_eq!(back.sources[0].source_type, "local_fs");
        assert_eq!(back.sources[0].path.as_deref(), Some("/notes"));
    }

    // --- ContentSourceEntry ---

    #[test]
    fn content_source_entry_is_enabled_prefers_enabled() {
        let entry = ContentSourceEntry {
            enabled: Some(false),
            watch: true,
            ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
        };
        assert!(!entry.is_enabled());
    }

    #[test]
    fn content_source_entry_is_enabled_fallback_to_watch() {
        let entry = ContentSourceEntry {
            enabled: None,
            watch: true,
            ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
        };
        assert!(entry.is_enabled());
    }

    #[test]
    fn content_source_entry_effective_change_detection_disabled() {
        let entry = ContentSourceEntry {
            enabled: Some(false),
            change_detection: "auto".into(),
            ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
        };
        assert_eq!(entry.effective_change_detection(), "none");
    }

    #[test]
    fn content_source_entry_effective_change_detection_poll() {
        let entry = ContentSourceEntry {
            enabled: Some(true),
            change_detection: "poll".into(),
            ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
        };
        assert_eq!(entry.effective_change_detection(), "poll");
        assert!(entry.is_poll_only());
        assert!(!entry.is_scan_only());
    }

    #[test]
    fn content_source_entry_is_scan_only() {
        let entry = ContentSourceEntry {
            enabled: Some(true),
            change_detection: "none".into(),
            ..serde_json::from_str::<ContentSourceEntry>("{}").unwrap()
        };
        assert!(entry.is_scan_only());
        assert!(!entry.is_poll_only());
    }

    #[test]
    fn content_source_entry_deserialize_defaults() {
        let entry: ContentSourceEntry = serde_json::from_str("{}").unwrap();
        assert_eq!(entry.source_type, "local_fs");
        assert!(entry.watch);
        assert!(entry.enabled.is_none());
        assert_eq!(entry.change_detection, "auto");
        assert_eq!(entry.file_patterns, vec!["*.md", "*.txt"]);
        assert!(entry.loop_back_enabled);
    }

    // --- DeploymentMode ---

    #[test]
    fn deployment_mode_default() {
        let mode = DeploymentMode::default();
        assert_eq!(mode, DeploymentMode::Desktop);
    }

    #[test]
    fn deployment_mode_display() {
        assert_eq!(DeploymentMode::Desktop.to_string(), "desktop");
        assert_eq!(DeploymentMode::SelfHost.to_string(), "self_host");
        assert_eq!(DeploymentMode::Cloud.to_string(), "cloud");
    }

    #[test]
    fn deployment_mode_serde_roundtrip() {
        for mode in &[
            DeploymentMode::Desktop,
            DeploymentMode::SelfHost,
            DeploymentMode::Cloud,
        ] {
            let json = serde_json::to_string(mode).unwrap();
            let back: DeploymentMode = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, mode);
        }
    }

    #[test]
    fn deployment_mode_capabilities_desktop() {
        let caps = DeploymentMode::Desktop.capabilities();
        assert!(caps.local_folder);
        assert!(caps.manual_local_path);
        assert!(caps.google_drive);
        assert!(caps.inline_ingest);
        assert!(caps.file_picker_native);
        assert_eq!(caps.preferred_source_default, "local_fs");
    }

    #[test]
    fn deployment_mode_capabilities_self_host() {
        let caps = DeploymentMode::SelfHost.capabilities();
        assert!(caps.local_folder);
        assert!(caps.manual_local_path);
        assert!(caps.google_drive);
        assert!(caps.inline_ingest);
        assert!(!caps.file_picker_native);
        assert_eq!(caps.preferred_source_default, "google_drive");
    }

    #[test]
    fn deployment_mode_capabilities_cloud() {
        let caps = DeploymentMode::Cloud.capabilities();
        assert!(!caps.local_folder);
        assert!(!caps.manual_local_path);
        assert!(caps.google_drive);
        assert!(caps.inline_ingest);
        assert!(!caps.file_picker_native);
        assert_eq!(caps.preferred_source_default, "google_drive");
    }

    #[test]
    fn deployment_mode_allows_source_type() {
        assert!(DeploymentMode::Desktop.allows_source_type("local_fs"));
        assert!(DeploymentMode::Desktop.allows_source_type("google_drive"));
        assert!(DeploymentMode::Desktop.allows_source_type("manual"));
        assert!(!DeploymentMode::Desktop.allows_source_type("unknown"));

        assert!(!DeploymentMode::Cloud.allows_source_type("local_fs"));
        assert!(DeploymentMode::Cloud.allows_source_type("google_drive"));
    }

    // --- DeploymentCapabilities ---

    #[test]
    fn deployment_capabilities_serde_roundtrip() {
        let caps = DeploymentCapabilities {
            local_folder: true,
            manual_local_path: true,
            google_drive: true,
            inline_ingest: false,
            file_picker_native: true,
            preferred_source_default: "local_fs".into(),
        };
        let json = serde_json::to_string(&caps).unwrap();
        let back: DeploymentCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(back, caps);
    }

    // --- ConnectorConfig ---

    #[test]
    fn connector_config_default() {
        let cfg = ConnectorConfig::default();
        assert!(cfg.google_drive.client_id.is_none());
        assert!(cfg.google_drive.client_secret.is_none());
        assert!(cfg.google_drive.redirect_uri.is_none());
    }

    #[test]
    fn connector_config_serde_roundtrip() {
        let cfg = ConnectorConfig {
            google_drive: GoogleDriveConnectorConfig {
                client_id: Some("gcp-client-id".into()),
                client_secret: Some("gcp-secret".into()),
                redirect_uri: Some("http://localhost:3001/callback".into()),
            },
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ConnectorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.google_drive.client_id.as_deref(),
            Some("gcp-client-id")
        );
        assert_eq!(
            back.google_drive.client_secret.as_deref(),
            Some("gcp-secret")
        );
    }

    // --- Constants ---

    #[test]
    fn change_detection_constants() {
        assert_eq!(CHANGE_DETECTION_AUTO, "auto");
        assert_eq!(CHANGE_DETECTION_POLL, "poll");
        assert_eq!(CHANGE_DETECTION_NONE, "none");
        assert_eq!(MIN_POLL_INTERVAL_SECONDS, 30);
    }
}

// ---------------------------------------------------------------------------
// Deployment Mode
// ---------------------------------------------------------------------------

/// Deployment environment controlling which features and source types are available.
///
/// - **Desktop**: Native Tauri app. Full local filesystem access + native file picker.
/// - **SelfHost**: Docker/VPS browser UI. Local filesystem access (server-side paths).
/// - **Cloud**: Managed cloud service. No local filesystem access.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    #[default]
    Desktop,
    SelfHost,
    Cloud,
}

impl std::fmt::Display for DeploymentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentMode::Desktop => write!(f, "desktop"),
            DeploymentMode::SelfHost => write!(f, "self_host"),
            DeploymentMode::Cloud => write!(f, "cloud"),
        }
    }
}

/// Capabilities available in the current deployment mode.
///
/// The frontend uses this to conditionally render source type options
/// and the backend uses it to validate source configurations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentCapabilities {
    /// Server can read from local filesystem paths.
    pub local_folder: bool,
    /// User can type a local path (browser text input, not native picker).
    pub manual_local_path: bool,
    /// Google Drive remote source is available.
    pub google_drive: bool,
    /// Direct content ingest via POST /api/ingest.
    pub inline_ingest: bool,
    /// Native file picker dialog (Tauri only).
    pub file_picker_native: bool,
    /// Preferred default source type for onboarding in this deployment mode.
    /// `"local_fs"` for Desktop, `"google_drive"` for SelfHost and Cloud.
    pub preferred_source_default: String,
}

// ---------------------------------------------------------------------------
// Connector Config
// ---------------------------------------------------------------------------

/// Application-level connector configuration for remote source OAuth flows.
///
/// These are *application credentials* (e.g. GCP OAuth client ID/secret),
/// not user credentials. They define which OAuth application the linking
/// flow uses. User credentials are stored in the `connections` table.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConnectorConfig {
    /// Google Drive connector settings.
    #[serde(default)]
    pub google_drive: GoogleDriveConnectorConfig,
}

/// Google Drive OAuth application credentials.
///
/// Self-hosted operators configure these once in `config.toml` or via
/// environment variables. Desktop installs can bundle embedded defaults
/// via env vars in the Tauri sidecar.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GoogleDriveConnectorConfig {
    /// GCP OAuth client ID for user-account Drive linking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// GCP OAuth client secret.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// Override redirect URI (default: http://localhost:3001/api/connectors/google-drive/callback).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

impl DeploymentMode {
    /// Returns the set of capabilities for this deployment mode.
    pub fn capabilities(&self) -> DeploymentCapabilities {
        match self {
            DeploymentMode::Desktop => DeploymentCapabilities {
                local_folder: true,
                manual_local_path: true,
                google_drive: true,
                inline_ingest: true,
                file_picker_native: true,
                preferred_source_default: "local_fs".to_string(),
            },
            DeploymentMode::SelfHost => DeploymentCapabilities {
                local_folder: true,
                manual_local_path: true,
                google_drive: true,
                inline_ingest: true,
                file_picker_native: false,
                preferred_source_default: "google_drive".to_string(),
            },
            DeploymentMode::Cloud => DeploymentCapabilities {
                local_folder: false,
                manual_local_path: false,
                google_drive: true,
                inline_ingest: true,
                file_picker_native: false,
                preferred_source_default: "google_drive".to_string(),
            },
        }
    }

    /// Returns `true` if the given source type is allowed in this mode.
    pub fn allows_source_type(&self, source_type: &str) -> bool {
        let caps = self.capabilities();
        match source_type {
            "local_fs" => caps.local_folder,
            "google_drive" => caps.google_drive,
            "manual" => caps.inline_ingest,
            _ => false,
        }
    }
}
