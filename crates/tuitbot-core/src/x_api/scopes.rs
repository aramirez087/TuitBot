//! OAuth scope registry and diagnostics for X API features.
//!
//! Centralizes the required scopes and maps missing scopes to degraded
//! product capabilities for actionable diagnostics.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// OAuth 2.0 scopes required by Tuitbot.
pub const REQUIRED_SCOPES: &[&str] = &[
    "tweet.read",
    "tweet.write",
    "users.read",
    "follows.read",
    "follows.write",
    "like.read",
    "like.write",
    "bookmark.read",
    "bookmark.write",
    "dm.read",
    "dm.write",
    "offline.access",
];

/// Mapping between a feature and the scopes it requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureScopeMapping {
    /// Feature label shown to users.
    pub feature: &'static str,
    /// Short explanation of what the feature does.
    pub description: &'static str,
    /// Scopes required for this feature.
    pub required_scopes: &'static [&'static str],
}

/// Feature-to-scope registry used by diagnostics.
pub const FEATURE_SCOPE_MAP: &[FeatureScopeMapping] = &[
    FeatureScopeMapping {
        feature: "Search tweets",
        description: "Search recent tweets for discovery and targeting.",
        required_scopes: &["tweet.read", "users.read"],
    },
    FeatureScopeMapping {
        feature: "Post tweet/reply/thread",
        description: "Create tweets, replies, and thread posts.",
        required_scopes: &["tweet.read", "tweet.write", "users.read"],
    },
    FeatureScopeMapping {
        feature: "Like/unlike",
        description: "Like and unlike tweets on behalf of the account.",
        required_scopes: &["like.read", "like.write", "users.read"],
    },
    FeatureScopeMapping {
        feature: "Follow/unfollow",
        description: "Follow and unfollow users from the authenticated account.",
        required_scopes: &["follows.read", "follows.write", "users.read"],
    },
    FeatureScopeMapping {
        feature: "Read mentions",
        description: "Read @mentions for mention-reply workflows.",
        required_scopes: &["tweet.read", "users.read"],
    },
    FeatureScopeMapping {
        feature: "Bookmarks",
        description: "Bookmark and unbookmark tweets, read bookmarked tweets.",
        required_scopes: &["bookmark.read", "bookmark.write", "users.read"],
    },
    FeatureScopeMapping {
        feature: "Token refresh",
        description: "Refresh access tokens without re-authentication.",
        required_scopes: &["offline.access"],
    },
    FeatureScopeMapping {
        feature: "Read DMs",
        description: "Read direct message conversations and events.",
        required_scopes: &["dm.read", "users.read"],
    },
    FeatureScopeMapping {
        feature: "Send DMs",
        description: "Send direct messages and create group conversations.",
        required_scopes: &["dm.write", "dm.read", "users.read"],
    },
];

/// A degraded feature caused by missing scopes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DegradedFeature {
    /// Feature label.
    pub feature: String,
    /// Human-readable feature description.
    pub description: String,
    /// Missing scopes that degrade this feature.
    pub missing_scopes: Vec<String>,
}

/// Result of comparing granted OAuth scopes to required scopes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeAnalysis {
    /// Granted scopes from the token.
    pub granted: Vec<String>,
    /// Scopes required by Tuitbot.
    pub required: Vec<String>,
    /// Required scopes missing from the token.
    pub missing: Vec<String>,
    /// Granted scopes not required by Tuitbot.
    pub extra: Vec<String>,
    /// Features degraded due to missing scopes.
    pub degraded_features: Vec<DegradedFeature>,
    /// True when all required scopes are present.
    pub all_required_present: bool,
}

/// Compare granted OAuth scopes with required scopes.
pub fn analyze_scopes(granted: &[String]) -> ScopeAnalysis {
    let granted_set: BTreeSet<String> = granted
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    let required_set: BTreeSet<String> = REQUIRED_SCOPES.iter().map(|s| (*s).to_string()).collect();

    let missing: Vec<String> = required_set.difference(&granted_set).cloned().collect();
    let extra: Vec<String> = granted_set.difference(&required_set).cloned().collect();

    let degraded_features: Vec<DegradedFeature> = FEATURE_SCOPE_MAP
        .iter()
        .filter_map(|mapping| {
            let missing_scopes: Vec<String> = mapping
                .required_scopes
                .iter()
                .filter(|scope| !granted_set.contains(**scope))
                .map(|scope| (*scope).to_string())
                .collect();

            if missing_scopes.is_empty() {
                None
            } else {
                Some(DegradedFeature {
                    feature: mapping.feature.to_string(),
                    description: mapping.description.to_string(),
                    missing_scopes,
                })
            }
        })
        .collect();

    ScopeAnalysis {
        granted: granted_set.into_iter().collect(),
        required: REQUIRED_SCOPES.iter().map(|s| (*s).to_string()).collect(),
        missing: missing.clone(),
        extra,
        degraded_features,
        all_required_present: missing.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_required_scopes() -> Vec<String> {
        REQUIRED_SCOPES.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn full_scopes_have_no_degradation() {
        let analysis = analyze_scopes(&all_required_scopes());
        assert!(analysis.all_required_present);
        assert!(analysis.missing.is_empty());
        assert!(analysis.degraded_features.is_empty());
        assert!(analysis.extra.is_empty());
    }

    #[test]
    fn partial_scopes_report_degraded_features() {
        let mut scopes = all_required_scopes();
        scopes.retain(|scope| scope != "like.write");

        let analysis = analyze_scopes(&scopes);

        assert!(!analysis.all_required_present);
        assert_eq!(analysis.missing, vec!["like.write".to_string()]);

        let like_feature = analysis
            .degraded_features
            .iter()
            .find(|feature| feature.feature == "Like/unlike")
            .expect("like/unlike feature should be degraded");
        assert_eq!(like_feature.missing_scopes, vec!["like.write".to_string()]);
    }

    #[test]
    fn empty_scopes_degrade_all_features() {
        let analysis = analyze_scopes(&[]);

        assert!(!analysis.all_required_present);
        assert_eq!(analysis.missing.len(), REQUIRED_SCOPES.len());
        assert_eq!(analysis.degraded_features.len(), FEATURE_SCOPE_MAP.len());
        assert!(analysis.extra.is_empty());
    }

    #[test]
    fn extra_scopes_are_reported_without_error() {
        let mut scopes = all_required_scopes();
        scopes.push("mute.read".to_string());

        let analysis = analyze_scopes(&scopes);

        assert!(analysis.all_required_present);
        assert!(analysis.missing.is_empty());
        assert_eq!(analysis.extra, vec!["mute.read".to_string()]);
    }
}
