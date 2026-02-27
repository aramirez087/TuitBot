//! Request family classification for universal X API requests.
//!
//! Classifies each request by API domain (public, DM, Ads, enterprise admin,
//! media upload) based on the target host and path. Used for audit enrichment,
//! policy categorization, and reporting.

use serde::Serialize;

/// Semantic classification of a universal request by API domain.
///
/// Used for policy categorization, audit enrichment, and reporting.
/// Derived from the target host and path of a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestFamily {
    /// Standard X API v2 public endpoints (tweets, users, lists, etc.).
    PublicApi,
    /// Direct Message endpoints (`/2/dm_conversations`, `/2/dm_events`).
    DirectMessage,
    /// Ads/Campaign API on `ads-api.x.com`.
    Ads,
    /// Enterprise admin/compliance endpoints (`/2/compliance`, `/2/usage`).
    EnterpriseAdmin,
    /// Media upload endpoints on `upload.x.com` or `upload.twitter.com`.
    MediaUpload,
}

impl std::fmt::Display for RequestFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PublicApi => write!(f, "public_api"),
            Self::DirectMessage => write!(f, "direct_message"),
            Self::Ads => write!(f, "ads"),
            Self::EnterpriseAdmin => write!(f, "enterprise_admin"),
            Self::MediaUpload => write!(f, "media_upload"),
        }
    }
}

/// Classify a request into its semantic family based on host and path.
pub(crate) fn classify_request_family(host: Option<&str>, path: &str) -> RequestFamily {
    let effective_host = host.unwrap_or("api.x.com").to_ascii_lowercase();

    if effective_host == "ads-api.x.com" {
        return RequestFamily::Ads;
    }
    if effective_host == "upload.x.com" || effective_host == "upload.twitter.com" {
        return RequestFamily::MediaUpload;
    }

    // api.x.com path-based classification
    let lower_path = path.to_ascii_lowercase();
    if lower_path.starts_with("/2/dm_conversations") || lower_path.starts_with("/2/dm_events") {
        return RequestFamily::DirectMessage;
    }
    if lower_path.starts_with("/2/compliance") || lower_path.starts_with("/2/usage") {
        return RequestFamily::EnterpriseAdmin;
    }

    RequestFamily::PublicApi
}
