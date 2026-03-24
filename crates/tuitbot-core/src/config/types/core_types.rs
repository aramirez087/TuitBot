//! Core server, auth, business profile, scoring, storage, and deployment configuration types.

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
    /// Privacy envelope label: `"local_first"`, `"user_controlled"`, or `"provider_controlled"`.
    #[serde(default)]
    pub privacy_envelope: String,
    /// Whether the Ghostwriter vault-to-compose pipeline runs entirely on the local machine.
    /// True only for Desktop mode (embedded server on 127.0.0.1).
    #[serde(default)]
    pub ghostwriter_local_only: bool,
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
                privacy_envelope: "local_first".to_string(),
                ghostwriter_local_only: true,
            },
            DeploymentMode::SelfHost => DeploymentCapabilities {
                local_folder: true,
                manual_local_path: true,
                google_drive: true,
                inline_ingest: true,
                file_picker_native: false,
                preferred_source_default: "google_drive".to_string(),
                privacy_envelope: "user_controlled".to_string(),
                ghostwriter_local_only: false,
            },
            DeploymentMode::Cloud => DeploymentCapabilities {
                local_folder: false,
                manual_local_path: false,
                google_drive: true,
                inline_ingest: true,
                file_picker_native: false,
                preferred_source_default: "google_drive".to_string(),
                privacy_envelope: "provider_controlled".to_string(),
                ghostwriter_local_only: false,
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

    /// Returns `true` only for Desktop mode where data never leaves the machine.
    ///
    /// Self-host has local filesystem access but data crosses a network boundary
    /// (browser → server), so it cannot claim local-first. Cloud is obviously
    /// not local-first.
    pub fn is_local_first(&self) -> bool {
        matches!(self, DeploymentMode::Desktop)
    }

    /// Returns the privacy envelope label for this deployment mode.
    ///
    /// - `"local_first"` — Desktop: data never leaves the machine.
    /// - `"user_controlled"` — Self-host: user controls the server, but data
    ///   may cross a network boundary.
    /// - `"provider_controlled"` — Cloud: data processed on provider infrastructure.
    pub fn privacy_envelope(&self) -> &'static str {
        match self {
            DeploymentMode::Desktop => "local_first",
            DeploymentMode::SelfHost => "user_controlled",
            DeploymentMode::Cloud => "provider_controlled",
        }
    }
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

// ---------------------------------------------------------------------------
// Default value functions
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

fn default_db_path() -> String {
    "~/.tuitbot/tuitbot.db".to_string()
}

fn default_retention_days() -> u32 {
    90
}

fn default_server_host() -> String {
    "127.0.0.1".to_string()
}

fn default_server_port() -> u16 {
    3001
}
