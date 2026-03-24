//! Configuration section structs and their serde default functions.
//!
//! Split into submodules by domain:
//! - `core_types`: X API, auth, business profile, scoring, storage, server, logging, deployment
//! - `policy_types`: rate limits, intervals, targets, content sources
//! - `llm_types`: LLM and embedding provider config

mod core_types;
mod llm_types;
mod policy_types;

#[cfg(test)]
mod tests;

pub use core_types::{
    AuthConfig, BusinessProfile, ConnectorConfig, DeploymentCapabilities, DeploymentMode,
    GoogleDriveConnectorConfig, LoggingConfig, ScoringConfig, ServerConfig, StorageConfig,
    XApiConfig,
};
pub use llm_types::{EmbeddingConfig, LlmConfig};
pub use policy_types::{
    ContentSourceEntry, ContentSourcesConfig, IntervalsConfig, LimitsConfig, TargetsConfig,
    CHANGE_DETECTION_AUTO, CHANGE_DETECTION_NONE, CHANGE_DETECTION_POLL, MIN_POLL_INTERVAL_SECONDS,
};
