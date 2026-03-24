//! User and profile types for X API v2.

use serde::{Deserialize, Serialize};

/// An X API user profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID.
    pub id: String,
    /// @username handle (without the @).
    pub username: String,
    /// Display name.
    pub name: String,
    /// Profile image URL.
    #[serde(default)]
    pub profile_image_url: Option<String>,
    /// Bio / profile description.
    #[serde(default)]
    pub description: Option<String>,
    /// Self-reported location string.
    #[serde(default)]
    pub location: Option<String>,
    /// Profile URL (often a t.co shortened link).
    #[serde(default)]
    pub url: Option<String>,
    /// User engagement metrics.
    #[serde(default)]
    pub public_metrics: UserMetrics,
}

/// Public metrics for a user profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserMetrics {
    /// Number of followers.
    #[serde(default)]
    pub followers_count: u64,
    /// Number of accounts being followed.
    #[serde(default)]
    pub following_count: u64,
    /// Total number of tweets posted.
    #[serde(default)]
    pub tweet_count: u64,
}

/// Wrapper for user/me responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    /// The user data.
    pub data: User,
}

/// Response from endpoints returning a list of users (followers, following, batch lookup).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    /// List of users.
    #[serde(default)]
    pub data: Vec<User>,
    /// Pagination and result metadata.
    pub meta: UsersMeta,
}

/// Metadata from a users list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersMeta {
    /// Number of users returned in this response.
    #[serde(default)]
    pub result_count: u32,
    /// Pagination token for fetching the next page.
    #[serde(default)]
    pub next_token: Option<String>,
}

/// Request body for following a user via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUserRequest {
    /// The target user ID to follow.
    pub target_user_id: String,
}
