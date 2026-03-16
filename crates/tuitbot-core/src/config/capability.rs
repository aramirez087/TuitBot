//! Capability tier model for progressive activation.
//!
//! Defines four tiers that represent what the user can do based on their
//! current configuration state. Tiers are computed — never stored — so
//! they always reflect the live config.

use serde::{Deserialize, Serialize};

use super::Config;

/// Progressive activation tiers, ordered from least to most capable.
///
/// Each tier is a strict superset of the previous one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityTier {
    /// No configuration exists yet.
    Unconfigured = 0,
    /// Business profile is filled — dashboard access, view settings.
    ProfileReady = 1,
    /// Profile + X credentials — discovery, search, scoring.
    ExplorationReady = 2,
    /// Exploration + valid LLM config — draft generation, reply composition.
    GenerationReady = 3,
    /// Generation + valid posting tokens — scheduled posting, autopilot.
    PostingReady = 4,
}

impl CapabilityTier {
    /// Human-readable label for this tier.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unconfigured => "Unconfigured",
            Self::ProfileReady => "Profile Ready",
            Self::ExplorationReady => "Exploration Ready",
            Self::GenerationReady => "Generation Ready",
            Self::PostingReady => "Posting Ready",
        }
    }

    /// Short description of what this tier unlocks.
    pub fn description(self) -> &'static str {
        match self {
            Self::Unconfigured => "Complete onboarding to get started",
            Self::ProfileReady => "Dashboard access and settings",
            Self::ExplorationReady => "Content discovery and scoring",
            Self::GenerationReady => "AI draft generation and composition",
            Self::PostingReady => "Scheduled posting and autopilot",
        }
    }

    /// Returns a list of what's missing to reach the next tier.
    pub fn missing_for_next(self, config: &Config, can_post: bool) -> Vec<String> {
        match self {
            Self::Unconfigured => {
                let mut missing = Vec::new();
                if config.business.product_name.is_empty() {
                    missing.push("Product/profile name".to_string());
                }
                if config.business.product_description.trim().is_empty() {
                    missing.push("Product/profile description".to_string());
                }
                if config.business.product_keywords.is_empty() {
                    missing.push("Product keywords".to_string());
                }
                if config.business.industry_topics.is_empty()
                    && config.business.product_keywords.is_empty()
                {
                    missing.push("Industry topics".to_string());
                }
                missing
            }
            Self::ProfileReady => {
                let mut missing = Vec::new();
                let backend = config.x_api.provider_backend.as_str();
                let is_x_api = backend.is_empty() || backend == "x_api";
                if is_x_api && config.x_api.client_id.trim().is_empty() {
                    missing.push("X API client ID".to_string());
                }
                missing
            }
            Self::ExplorationReady => {
                let mut missing = Vec::new();
                if config.llm.provider.is_empty() {
                    missing.push("LLM provider".to_string());
                } else if matches!(config.llm.provider.as_str(), "openai" | "anthropic")
                    && config.llm.api_key.as_ref().map_or(true, |k| k.is_empty())
                {
                    missing.push("LLM API key".to_string());
                }
                missing
            }
            Self::GenerationReady => {
                if !can_post {
                    vec!["Valid posting credentials (OAuth tokens or scraper session)".to_string()]
                } else {
                    vec![]
                }
            }
            Self::PostingReady => vec![],
        }
    }
}

/// Compute the capability tier from config state and posting ability.
///
/// This is a pure function with no side effects — call it freely.
pub fn compute_tier(config: &Config, can_post: bool) -> CapabilityTier {
    // Tier 1: business profile
    if config.business.product_name.is_empty()
        || config.business.product_description.trim().is_empty()
        || (config.business.product_keywords.is_empty()
            && config.business.competitor_keywords.is_empty())
    {
        return CapabilityTier::Unconfigured;
    }

    // Tier 2: X credentials
    let backend = config.x_api.provider_backend.as_str();
    let has_x = if backend == "scraper" {
        true // scraper mode doesn't need client_id
    } else {
        // x_api mode (default)
        !config.x_api.client_id.trim().is_empty()
    };

    if !has_x {
        return CapabilityTier::ProfileReady;
    }

    // Tier 3: LLM config
    let has_llm = if config.llm.provider.is_empty() {
        false
    } else if config.llm.provider == "ollama" {
        true // ollama doesn't need an API key
    } else {
        config.llm.api_key.as_ref().is_some_and(|k| !k.is_empty())
    };

    if !has_llm {
        return CapabilityTier::ExplorationReady;
    }

    // Tier 4: can post
    if !can_post {
        return CapabilityTier::GenerationReady;
    }

    CapabilityTier::PostingReady
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn minimal_profile_config() -> Config {
        let mut config = Config::default();
        config.business.product_name = "TestProduct".to_string();
        config.business.product_description = "A test product".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.business.industry_topics = vec!["testing".to_string()];
        config
    }

    #[test]
    fn test_unconfigured_tier() {
        let config = Config::default();
        assert_eq!(compute_tier(&config, false), CapabilityTier::Unconfigured);
    }

    #[test]
    fn test_profile_ready_tier() {
        let config = minimal_profile_config();
        assert_eq!(compute_tier(&config, false), CapabilityTier::ProfileReady);
    }

    #[test]
    fn test_exploration_ready_x_api() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc123".to_string();
        assert_eq!(
            compute_tier(&config, false),
            CapabilityTier::ExplorationReady
        );
    }

    #[test]
    fn test_exploration_ready_scraper() {
        let mut config = minimal_profile_config();
        config.x_api.provider_backend = "scraper".to_string();
        assert_eq!(
            compute_tier(&config, false),
            CapabilityTier::ExplorationReady
        );
    }

    #[test]
    fn test_generation_ready_cloud_provider() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc123".to_string();
        config.llm.provider = "openai".to_string();
        config.llm.api_key = Some("sk-test".to_string());
        assert_eq!(
            compute_tier(&config, false),
            CapabilityTier::GenerationReady
        );
    }

    #[test]
    fn test_generation_ready_ollama() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc123".to_string();
        config.llm.provider = "ollama".to_string();
        assert_eq!(
            compute_tier(&config, false),
            CapabilityTier::GenerationReady
        );
    }

    #[test]
    fn test_posting_ready() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc123".to_string();
        config.llm.provider = "anthropic".to_string();
        config.llm.api_key = Some("sk-ant-test".to_string());
        assert_eq!(compute_tier(&config, true), CapabilityTier::PostingReady);
    }

    #[test]
    fn test_tier_ordering() {
        assert!(CapabilityTier::Unconfigured < CapabilityTier::ProfileReady);
        assert!(CapabilityTier::ProfileReady < CapabilityTier::ExplorationReady);
        assert!(CapabilityTier::ExplorationReady < CapabilityTier::GenerationReady);
        assert!(CapabilityTier::GenerationReady < CapabilityTier::PostingReady);
    }

    #[test]
    fn test_missing_for_next_unconfigured() {
        let config = Config::default();
        let missing = CapabilityTier::Unconfigured.missing_for_next(&config, false);
        assert!(!missing.is_empty());
        assert!(missing.iter().any(|m| m.contains("name")));
    }

    #[test]
    fn test_missing_for_next_posting_ready() {
        let config = Config::default();
        let missing = CapabilityTier::PostingReady.missing_for_next(&config, true);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_cloud_provider_without_key_stays_exploration() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc123".to_string();
        config.llm.provider = "openai".to_string();
        // No API key
        assert_eq!(
            compute_tier(&config, false),
            CapabilityTier::ExplorationReady
        );
    }

    // -----------------------------------------------------------------------
    // Additional capability coverage tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_tier_labels_non_empty() {
        let tiers = [
            CapabilityTier::Unconfigured,
            CapabilityTier::ProfileReady,
            CapabilityTier::ExplorationReady,
            CapabilityTier::GenerationReady,
            CapabilityTier::PostingReady,
        ];
        for tier in tiers {
            assert!(!tier.label().is_empty());
            assert!(!tier.description().is_empty());
        }
    }

    #[test]
    fn test_tier_debug_and_clone() {
        let tier = CapabilityTier::GenerationReady;
        let cloned = tier;
        assert_eq!(tier, cloned);
        let debug = format!("{:?}", tier);
        assert!(debug.contains("GenerationReady"));
    }

    #[test]
    fn test_tier_serde_roundtrip() {
        let tier = CapabilityTier::PostingReady;
        let json = serde_json::to_string(&tier).expect("serialize");
        assert_eq!(json, "\"posting_ready\"");
        let deserialized: CapabilityTier = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, tier);
    }

    #[test]
    fn test_tier_serde_all_variants() {
        let expected = [
            (CapabilityTier::Unconfigured, "\"unconfigured\""),
            (CapabilityTier::ProfileReady, "\"profile_ready\""),
            (CapabilityTier::ExplorationReady, "\"exploration_ready\""),
            (CapabilityTier::GenerationReady, "\"generation_ready\""),
            (CapabilityTier::PostingReady, "\"posting_ready\""),
        ];
        for (tier, expected_json) in expected {
            let json = serde_json::to_string(&tier).expect("serialize");
            assert_eq!(json, expected_json, "mismatch for {:?}", tier);
        }
    }

    #[test]
    fn test_missing_for_next_profile_ready() {
        let config = minimal_profile_config();
        let missing = CapabilityTier::ProfileReady.missing_for_next(&config, false);
        assert!(!missing.is_empty());
        assert!(missing.iter().any(|m| m.contains("X API")));
    }

    #[test]
    fn test_missing_for_next_exploration_ready() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc".to_string();
        let missing = CapabilityTier::ExplorationReady.missing_for_next(&config, false);
        assert!(!missing.is_empty());
        assert!(missing.iter().any(|m| m.contains("LLM")));
    }

    #[test]
    fn test_missing_for_next_exploration_ready_with_provider_no_key() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc".to_string();
        config.llm.provider = "anthropic".to_string();
        // No API key
        let missing = CapabilityTier::ExplorationReady.missing_for_next(&config, false);
        assert!(!missing.is_empty());
        assert!(missing.iter().any(|m| m.contains("API key")));
    }

    #[test]
    fn test_missing_for_next_generation_ready_no_post() {
        let config = minimal_profile_config();
        let missing = CapabilityTier::GenerationReady.missing_for_next(&config, false);
        assert!(!missing.is_empty());
        assert!(missing.iter().any(|m| m.contains("posting")));
    }

    #[test]
    fn test_missing_for_next_generation_ready_can_post() {
        let config = minimal_profile_config();
        let missing = CapabilityTier::GenerationReady.missing_for_next(&config, true);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_unconfigured_empty_description() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_description = "   ".to_string(); // whitespace only
        config.business.product_keywords = vec!["kw".to_string()];
        assert_eq!(compute_tier(&config, false), CapabilityTier::Unconfigured);
    }

    #[test]
    fn test_competitor_keywords_count_for_profile() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_description = "A product".to_string();
        // No product_keywords, but has competitor_keywords
        config.business.competitor_keywords = vec!["rival".to_string()];
        assert_eq!(compute_tier(&config, false), CapabilityTier::ProfileReady);
    }

    #[test]
    fn test_unconfigured_missing_with_some_fields() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        // Missing description and keywords
        let missing = CapabilityTier::Unconfigured.missing_for_next(&config, false);
        assert!(missing.iter().any(|m| m.contains("description")));
        assert!(missing.iter().any(|m| m.contains("keywords")));
    }

    #[test]
    fn test_profile_ready_scraper_backend_no_client_id() {
        let mut config = minimal_profile_config();
        config.x_api.provider_backend = "scraper".to_string();
        // No client_id needed for scraper
        let missing = CapabilityTier::ProfileReady.missing_for_next(&config, false);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_cloud_provider_with_empty_key_stays_exploration() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc123".to_string();
        config.llm.provider = "anthropic".to_string();
        config.llm.api_key = Some("".to_string());
        assert_eq!(
            compute_tier(&config, false),
            CapabilityTier::ExplorationReady
        );
    }

    #[test]
    fn test_ollama_no_key_reaches_generation() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "abc123".to_string();
        config.llm.provider = "ollama".to_string();
        // No API key needed for ollama
        assert_eq!(
            compute_tier(&config, false),
            CapabilityTier::GenerationReady
        );
    }

    #[test]
    fn test_whitespace_client_id_stays_profile() {
        let mut config = minimal_profile_config();
        config.x_api.client_id = "   ".to_string();
        assert_eq!(compute_tier(&config, false), CapabilityTier::ProfileReady);
    }
}
