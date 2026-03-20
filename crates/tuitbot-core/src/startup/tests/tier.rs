//! Tests for ApiTier and TierCapabilities.

use crate::startup::config::{ApiTier, TierCapabilities};

// ============================================================================
// ApiTier
// ============================================================================

#[test]
fn api_tier_display() {
    assert_eq!(ApiTier::Free.to_string(), "Free");
    assert_eq!(ApiTier::Basic.to_string(), "Basic");
    assert_eq!(ApiTier::Pro.to_string(), "Pro");
}

// ============================================================================
// TierCapabilities
// ============================================================================

#[test]
fn free_tier_capabilities() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    assert!(!caps.mentions);
    assert!(!caps.discovery);
    assert!(caps.posting);
    assert!(!caps.search);
}

#[test]
fn basic_tier_capabilities() {
    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    assert!(caps.mentions);
    assert!(caps.discovery);
    assert!(caps.posting);
    assert!(caps.search);
}

#[test]
fn pro_tier_capabilities() {
    let caps = TierCapabilities::for_tier(ApiTier::Pro);
    assert!(caps.mentions);
    assert!(caps.discovery);
    assert!(caps.posting);
    assert!(caps.search);
}

#[test]
fn free_tier_enabled_loops() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let loops = caps.enabled_loop_names();
    assert_eq!(loops, vec!["content", "threads"]);
}

#[test]
fn basic_tier_enabled_loops() {
    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    let loops = caps.enabled_loop_names();
    assert_eq!(loops, vec!["mentions", "discovery", "content", "threads"]);
}

#[test]
fn tier_capabilities_format_status() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let status = caps.format_status();
    assert!(status.contains("Mentions: DISABLED"));
    assert!(status.contains("Discovery: DISABLED"));

    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    let status = caps.format_status();
    assert!(status.contains("Mentions: enabled"));
    assert!(status.contains("Discovery: enabled"));
}
