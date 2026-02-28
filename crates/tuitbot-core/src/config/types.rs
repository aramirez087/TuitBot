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
    /// LLM provider name: "openai", "anthropic", or "ollama".
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
    #[serde(default)]
    pub service_account_key: Option<String>,

    /// Whether to watch for changes in real-time.
    #[serde(default = "default_watch")]
    pub watch: bool,

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

fn default_source_type() -> String {
    "local_fs".to_string()
}
fn default_watch() -> bool {
    true
}
fn default_file_patterns() -> Vec<String> {
    vec!["*.md".to_string(), "*.txt".to_string()]
}
fn default_loop_back() -> bool {
    true
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
            },
            DeploymentMode::SelfHost => DeploymentCapabilities {
                local_folder: true,
                manual_local_path: true,
                google_drive: true,
                inline_ingest: true,
                file_picker_native: false,
            },
            DeploymentMode::Cloud => DeploymentCapabilities {
                local_folder: false,
                manual_local_path: false,
                google_drive: true,
                inline_ingest: true,
                file_picker_native: false,
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
