//! GraphQL response parsing for X's internal API.
//!
//! All parsing uses `serde_json::Value` navigation rather than typed
//! deserialization because X's internal GraphQL format is undocumented
//! and changes frequently.

use crate::error::XApiError;
use crate::x_api::types::{PostedTweet, PublicMetrics, Tweet, User, UserMetrics};

/// Parse a single tweet from a GraphQL `tweet_results.result` node.
///
/// Handles `TweetWithVisibilityResults` wrapper (has `.tweet` sub-field)
/// and tombstones/unavailable results (returns `None`).
pub fn parse_tweet(result: &serde_json::Value) -> Option<Tweet> {
    // Handle TweetWithVisibilityResults wrapper
    let tweet_data = match result.get("__typename").and_then(|t| t.as_str()) {
        Some("TweetWithVisibilityResults") => result.get("tweet")?,
        Some("TweetTombstone") | Some("TweetUnavailable") => return None,
        _ => result,
    };

    let id = tweet_data.get("rest_id").and_then(|v| v.as_str())?;
    if id.is_empty() {
        return None;
    }

    let legacy = tweet_data.get("legacy")?;

    let text = legacy
        .get("full_text")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let author_id = legacy
        .get("user_id_str")
        .and_then(|v| v.as_str())
        .or_else(|| {
            tweet_data
                .get("core")
                .and_then(|c| c.get("user_results"))
                .and_then(|ur| ur.get("result"))
                .and_then(|r| r.get("rest_id"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or_default();

    let created_at = legacy
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let conversation_id = legacy
        .get("conversation_id_str")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let public_metrics = PublicMetrics {
        retweet_count: legacy
            .get("retweet_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        reply_count: legacy
            .get("reply_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        like_count: legacy
            .get("favorite_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        quote_count: legacy
            .get("quote_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        impression_count: tweet_data
            .get("views")
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        bookmark_count: legacy
            .get("bookmark_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    };

    Some(Tweet {
        id: id.to_string(),
        text: text.to_string(),
        author_id: author_id.to_string(),
        created_at: created_at.to_string(),
        public_metrics,
        conversation_id,
    })
}

/// Parse a user from a GraphQL `user_results.result` node.
pub fn parse_user(result: &serde_json::Value) -> Option<User> {
    let id = result.get("rest_id").and_then(|v| v.as_str())?;
    if id.is_empty() {
        return None;
    }

    let legacy = result.get("legacy")?;

    let username = legacy
        .get("screen_name")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let name = legacy
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let profile_image_url = legacy
        .get("profile_image_url_https")
        .and_then(|v| v.as_str())
        .map(|u| u.replace("_normal.", "_400x400."))
        .filter(|u| !u.is_empty());

    let description = legacy
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    let location = legacy
        .get("location")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    let url = legacy
        .get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    let public_metrics = UserMetrics {
        followers_count: legacy
            .get("followers_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        following_count: legacy
            .get("friends_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        tweet_count: legacy
            .get("statuses_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    };

    Some(User {
        id: id.to_string(),
        username: username.to_string(),
        name: name.to_string(),
        profile_image_url,
        description,
        location,
        url,
        public_metrics,
    })
}

/// Extract tweets and optional cursor from a timeline-style response.
///
/// `instructions_path` navigates from root to the `instructions` array,
/// e.g. `["data", "search_by_raw_query", "search_timeline", "timeline"]`.
pub fn parse_timeline(
    body: &serde_json::Value,
    instructions_path: &[&str],
) -> (Vec<Tweet>, Option<String>) {
    let instructions = navigate_to_array(body, instructions_path);

    let mut tweets = Vec::new();
    let mut cursor = None;

    for instruction in instructions {
        let inst_type = instruction.get("type").and_then(|v| v.as_str());

        match inst_type {
            Some("TimelineAddEntries") | Some("TimelineReplaceEntry") => {
                let entries = instruction
                    .get("entries")
                    .and_then(|e| e.as_array())
                    .cloned()
                    .unwrap_or_default();

                for entry in &entries {
                    let entry_id = entry
                        .get("entryId")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();

                    if entry_id.starts_with("cursor-bottom") || entry_id.starts_with("cursor-top") {
                        if entry_id.starts_with("cursor-bottom") {
                            cursor = entry
                                .get("content")
                                .and_then(|c| c.get("value"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                        continue;
                    }

                    // Single tweet entry
                    if let Some(tweet) = extract_tweet_from_entry(entry) {
                        tweets.push(tweet);
                    }

                    // Module entries (e.g., search results grouped in modules)
                    if let Some(items) = entry
                        .get("content")
                        .and_then(|c| c.get("items"))
                        .and_then(|i| i.as_array())
                    {
                        for item in items {
                            if let Some(tweet) = extract_tweet_from_module_item(item) {
                                tweets.push(tweet);
                            }
                        }
                    }
                }
            }
            _ => continue,
        }
    }

    (tweets, cursor)
}

/// Extract users and optional cursor from a user-list response (followers, following).
pub fn parse_user_list(
    body: &serde_json::Value,
    instructions_path: &[&str],
) -> (Vec<User>, Option<String>) {
    let instructions = navigate_to_array(body, instructions_path);

    let mut users = Vec::new();
    let mut cursor = None;

    for instruction in instructions {
        let inst_type = instruction.get("type").and_then(|v| v.as_str());

        if inst_type != Some("TimelineAddEntries") {
            continue;
        }

        let entries = instruction
            .get("entries")
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();

        for entry in &entries {
            let entry_id = entry
                .get("entryId")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            if entry_id.starts_with("cursor-bottom") {
                cursor = entry
                    .get("content")
                    .and_then(|c| c.get("value"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                continue;
            }
            if entry_id.starts_with("cursor-") {
                continue;
            }

            // User entry: content.itemContent.user_results.result
            if let Some(user_result) = entry
                .get("content")
                .and_then(|c| c.get("itemContent"))
                .and_then(|ic| ic.get("user_results"))
                .and_then(|ur| ur.get("result"))
            {
                if let Some(user) = parse_user(user_result) {
                    users.push(user);
                }
            }
        }
    }

    (users, cursor)
}

/// Parse the CreateTweet GraphQL response to extract the posted tweet.
pub fn parse_create_tweet_response(body: &serde_json::Value) -> Result<PostedTweet, XApiError> {
    check_graphql_errors(body)?;

    let result = body
        .get("data")
        .and_then(|d| d.get("create_tweet"))
        .and_then(|ct| ct.get("tweet_results"))
        .and_then(|tr| tr.get("result"))
        .ok_or_else(|| XApiError::ApiError {
            status: 0,
            message: format!(
                "unexpected CreateTweet response structure: {}",
                serde_json::to_string(body).unwrap_or_default()
            ),
        })?;

    let tweet_id = result
        .get("rest_id")
        .and_then(|id| id.as_str())
        .unwrap_or_default()
        .to_string();

    let text = result
        .get("legacy")
        .and_then(|l| l.get("full_text"))
        .and_then(|t| t.as_str())
        .unwrap_or_default()
        .to_string();

    if tweet_id.is_empty() {
        return Err(XApiError::ApiError {
            status: 0,
            message: "CreateTweet returned no tweet ID".to_string(),
        });
    }

    Ok(PostedTweet { id: tweet_id, text })
}

/// Check for GraphQL-level errors in a response body.
pub fn check_graphql_errors(body: &serde_json::Value) -> Result<(), XApiError> {
    if let Some(errors) = body.get("errors").and_then(|e| e.as_array()) {
        if let Some(first) = errors.first() {
            let msg = first
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown GraphQL error");
            return Err(XApiError::ApiError {
                status: 0,
                message: format!("GraphQL error: {msg}"),
            });
        }
    }
    Ok(())
}

// --- Helper functions ---

/// Navigate a JSON value by a path of keys to reach an array.
fn navigate_to_array(body: &serde_json::Value, path: &[&str]) -> Vec<serde_json::Value> {
    let mut current = body;
    for key in path {
        match current.get(key) {
            Some(v) => current = v,
            None => return Vec::new(),
        }
    }
    current
        .get("instructions")
        .and_then(|i| i.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Extract a tweet from a timeline entry.
fn extract_tweet_from_entry(entry: &serde_json::Value) -> Option<Tweet> {
    let result = entry
        .get("content")
        .and_then(|c| c.get("itemContent"))
        .and_then(|ic| ic.get("tweet_results"))
        .and_then(|tr| tr.get("result"))?;
    parse_tweet(result)
}

/// Extract a tweet from a module item (e.g., search result modules).
fn extract_tweet_from_module_item(item: &serde_json::Value) -> Option<Tweet> {
    let result = item
        .get("item")
        .and_then(|i| i.get("itemContent"))
        .and_then(|ic| ic.get("tweet_results"))
        .and_then(|tr| tr.get("result"))?;
    parse_tweet(result)
}
