//! Static X API endpoint definitions — the spec pack.
//!
//! Each `EndpointDef` maps one X API v2 (or v1.1) endpoint to an MCP tool.
//! Only *new* endpoints are defined here — the 29 curated Layer 1 tools
//! already exist in `manifest.rs` and are not duplicated.
//!
//! ## Module layout
//!
//! | Module          | Contents                                              |
//! |-----------------|-------------------------------------------------------|
//! | `shared`        | Profile shorthands, error sets, common params         |
//! | `tweets`        | Batch Lookups, Tweet Metadata, Pin Management (9 eps) |
//! | `lists`         | Lists section (15 eps)                                |
//! | `users`         | Mutes, Blocks (6 eps)                                 |
//! | `spaces`        | Spaces, Direct Messages (14 eps)                      |
//! | `ads`           | Ads/Campaign — Admin-only (16 eps)                    |
//! | `compliance`    | Compliance, Stream Rules (7 eps)                      |

mod ads;
mod ads_line_items;
mod compliance;
mod lists;
mod shared;
mod spaces;
mod tweets;
mod users;

use std::sync::LazyLock;

use crate::spec::params::EndpointDef;

/// All X API endpoint definitions in the spec pack.
///
/// Assembled from domain sub-modules in declaration order.
/// Sorted alphabetically within each section by `tool_name`.
/// Only new endpoints — existing curated tools are not duplicated.
pub static SPEC_ENDPOINTS: LazyLock<Vec<EndpointDef>> = LazyLock::new(|| {
    let mut v: Vec<EndpointDef> = Vec::new();
    v.extend_from_slice(tweets::TWEET_ENDPOINTS);
    v.extend_from_slice(lists::LIST_ENDPOINTS);
    v.extend_from_slice(users::USER_ENDPOINTS);
    v.extend_from_slice(spaces::SPACE_ENDPOINTS);
    v.extend_from_slice(ads::ADS_ACCOUNT_ENDPOINTS);
    v.extend_from_slice(ads_line_items::ADS_LINE_ITEM_ENDPOINTS);
    v.extend_from_slice(compliance::COMPLIANCE_ENDPOINTS);
    v
});
