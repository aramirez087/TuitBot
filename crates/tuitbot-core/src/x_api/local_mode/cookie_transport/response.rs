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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── parse_tweet ──────────────────────────────────────────────

    #[test]
    fn parse_tweet_with_metrics() {
        let result = json!({
            "rest_id": "42",
            "legacy": {
                "full_text": "Hello",
                "user_id_str": "99",
                "created_at": "Mon Jan 01 00:00:00 +0000 2026",
                "retweet_count": 10,
                "favorite_count": 20,
                "reply_count": 3,
                "quote_count": 1,
                "bookmark_count": 5,
                "conversation_id_str": "42"
            },
            "views": {"count": "500"}
        });
        let t = parse_tweet(&result).unwrap();
        assert_eq!(t.id, "42");
        assert_eq!(t.text, "Hello");
        assert_eq!(t.author_id, "99");
        assert_eq!(t.public_metrics.retweet_count, 10);
        assert_eq!(t.public_metrics.like_count, 20);
        assert_eq!(t.public_metrics.reply_count, 3);
        assert_eq!(t.public_metrics.quote_count, 1);
        assert_eq!(t.public_metrics.bookmark_count, 5);
        assert_eq!(t.public_metrics.impression_count, 500);
        assert_eq!(t.conversation_id, Some("42".to_string()));
    }

    #[test]
    fn parse_tweet_missing_views() {
        let result = json!({
            "rest_id": "1",
            "legacy": {
                "full_text": "test",
                "user_id_str": "2",
                "created_at": ""
            }
        });
        let t = parse_tweet(&result).unwrap();
        assert_eq!(t.public_metrics.impression_count, 0);
    }

    #[test]
    fn parse_tweet_visibility_results_wrapper() {
        let result = json!({
            "__typename": "TweetWithVisibilityResults",
            "tweet": {
                "rest_id": "wrapped_1",
                "legacy": {
                    "full_text": "wrapped tweet",
                    "user_id_str": "55"
                }
            }
        });
        let t = parse_tweet(&result).unwrap();
        assert_eq!(t.id, "wrapped_1");
        assert_eq!(t.text, "wrapped tweet");
    }

    #[test]
    fn parse_tweet_author_from_core() {
        let result = json!({
            "rest_id": "77",
            "legacy": {
                "full_text": "text"
            },
            "core": {
                "user_results": {
                    "result": {
                        "rest_id": "author_88"
                    }
                }
            }
        });
        let t = parse_tweet(&result).unwrap();
        assert_eq!(t.author_id, "author_88");
    }

    #[test]
    fn parse_tweet_missing_legacy_returns_none() {
        let result = json!({"rest_id": "1"});
        assert!(parse_tweet(&result).is_none());
    }

    // ── parse_user ───────────────────────────────────────────────

    #[test]
    fn parse_user_full() {
        let result = json!({
            "__typename": "User",
            "rest_id": "100",
            "legacy": {
                "screen_name": "alice",
                "name": "Alice W",
                "profile_image_url_https": "https://pbs.twimg.com/img_normal.jpg",
                "description": "A developer",
                "location": "NYC",
                "url": "https://t.co/abc",
                "followers_count": 1000,
                "friends_count": 500,
                "statuses_count": 5000
            }
        });
        let u = parse_user(&result).unwrap();
        assert_eq!(u.id, "100");
        assert_eq!(u.username, "alice");
        assert_eq!(u.name, "Alice W");
        // _normal should be replaced with _400x400
        assert!(u.profile_image_url.as_deref().unwrap().contains("_400x400"));
        assert_eq!(u.description.as_deref(), Some("A developer"));
        assert_eq!(u.location.as_deref(), Some("NYC"));
        assert_eq!(u.public_metrics.followers_count, 1000);
        assert_eq!(u.public_metrics.following_count, 500);
        assert_eq!(u.public_metrics.tweet_count, 5000);
    }

    #[test]
    fn parse_user_empty_rest_id_returns_none() {
        let result = json!({
            "rest_id": "",
            "legacy": {"screen_name": "test", "name": "Test"}
        });
        assert!(parse_user(&result).is_none());
    }

    #[test]
    fn parse_user_empty_description_filtered() {
        let result = json!({
            "rest_id": "1",
            "legacy": {
                "screen_name": "u",
                "name": "U",
                "description": "",
                "location": ""
            }
        });
        let u = parse_user(&result).unwrap();
        assert!(u.description.is_none());
        assert!(u.location.is_none());
    }

    // ── check_graphql_errors ─────────────────────────────────────

    #[test]
    fn check_errors_none_present() {
        let body = json!({"data": {}});
        assert!(check_graphql_errors(&body).is_ok());
    }

    #[test]
    fn check_errors_empty_array() {
        let body = json!({"errors": [], "data": {}});
        assert!(check_graphql_errors(&body).is_ok());
    }

    #[test]
    fn check_errors_with_message() {
        let body = json!({"errors": [{"message": "Rate limit"}]});
        let err = check_graphql_errors(&body).unwrap_err();
        assert!(format!("{err}").contains("Rate limit"));
    }

    #[test]
    fn check_errors_no_message_field() {
        let body = json!({"errors": [{"code": 88}]});
        let err = check_graphql_errors(&body).unwrap_err();
        assert!(format!("{err}").contains("unknown GraphQL error"));
    }

    // ── parse_create_tweet_response ──────────────────────────────

    #[test]
    fn parse_create_tweet_success() {
        let body = json!({
            "data": {
                "create_tweet": {
                    "tweet_results": {
                        "result": {
                            "rest_id": "new_tweet_123",
                            "legacy": {
                                "full_text": "My new tweet"
                            }
                        }
                    }
                }
            }
        });
        let posted = parse_create_tweet_response(&body).unwrap();
        assert_eq!(posted.id, "new_tweet_123");
        assert_eq!(posted.text, "My new tweet");
    }

    #[test]
    fn parse_create_tweet_missing_result() {
        let body = json!({"data": {}});
        assert!(parse_create_tweet_response(&body).is_err());
    }

    #[test]
    fn parse_create_tweet_empty_id() {
        let body = json!({
            "data": {
                "create_tweet": {
                    "tweet_results": {
                        "result": {
                            "rest_id": "",
                            "legacy": {"full_text": "text"}
                        }
                    }
                }
            }
        });
        assert!(parse_create_tweet_response(&body).is_err());
    }

    #[test]
    fn parse_create_tweet_with_errors() {
        let body = json!({
            "errors": [{"message": "Duplicate tweet"}]
        });
        assert!(parse_create_tweet_response(&body).is_err());
    }

    // ── navigate_to_array ────────────────────────────────────────

    #[test]
    fn navigate_to_array_valid_path() {
        let body = json!({
            "data": {
                "search": {
                    "instructions": [
                        {"type": "TimelineAddEntries", "entries": []}
                    ]
                }
            }
        });
        let result = navigate_to_array(&body, &["data", "search"]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn navigate_to_array_invalid_path() {
        let body = json!({"data": {}});
        let result = navigate_to_array(&body, &["data", "missing"]);
        assert!(result.is_empty());
    }

    #[test]
    fn navigate_to_array_no_instructions_key() {
        let body = json!({"data": {"search": {"other": "value"}}});
        let result = navigate_to_array(&body, &["data", "search"]);
        assert!(result.is_empty());
    }

    // ── parse_timeline with entries ──────────────────────────────

    #[test]
    fn parse_timeline_with_tweets_and_cursor() {
        let body = json!({
            "data": {
                "search": {
                    "instructions": [{
                        "type": "TimelineAddEntries",
                        "entries": [
                            {
                                "entryId": "tweet-1",
                                "content": {
                                    "itemContent": {
                                        "tweet_results": {
                                            "result": {
                                                "rest_id": "111",
                                                "legacy": {
                                                    "full_text": "Hello",
                                                    "user_id_str": "9"
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            {
                                "entryId": "cursor-bottom-2",
                                "content": {"value": "next_cursor_abc"}
                            }
                        ]
                    }]
                }
            }
        });
        let (tweets, cursor) = parse_timeline(&body, &["data", "search"]);
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0].id, "111");
        assert_eq!(cursor, Some("next_cursor_abc".to_string()));
    }

    #[test]
    fn parse_timeline_cursor_top_ignored() {
        let body = json!({
            "data": {
                "tl": {
                    "instructions": [{
                        "type": "TimelineAddEntries",
                        "entries": [
                            {"entryId": "cursor-top-1", "content": {"value": "top_val"}}
                        ]
                    }]
                }
            }
        });
        let (tweets, cursor) = parse_timeline(&body, &["data", "tl"]);
        assert!(tweets.is_empty());
        assert!(cursor.is_none()); // only cursor-bottom sets cursor
    }

    #[test]
    fn parse_timeline_module_items() {
        let body = json!({
            "data": {
                "s": {
                    "instructions": [{
                        "type": "TimelineAddEntries",
                        "entries": [{
                            "entryId": "search-module-1",
                            "content": {
                                "items": [{
                                    "item": {
                                        "itemContent": {
                                            "tweet_results": {
                                                "result": {
                                                    "rest_id": "mod_tweet_1",
                                                    "legacy": {
                                                        "full_text": "module tweet",
                                                        "user_id_str": "3"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }]
                            }
                        }]
                    }]
                }
            }
        });
        let (tweets, _cursor) = parse_timeline(&body, &["data", "s"]);
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0].id, "mod_tweet_1");
    }

    // ── parse_user_list ──────────────────────────────────────────

    #[test]
    fn parse_user_list_with_users_and_cursor() {
        let body = json!({
            "data": {
                "user": {
                    "result": {
                        "timeline": {
                            "timeline": {
                                "instructions": [{
                                    "type": "TimelineAddEntries",
                                    "entries": [
                                        {
                                            "entryId": "user-1",
                                            "content": {
                                                "itemContent": {
                                                    "user_results": {
                                                        "result": {
                                                            "rest_id": "u1",
                                                            "legacy": {
                                                                "screen_name": "bob",
                                                                "name": "Bob"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        {
                                            "entryId": "cursor-bottom-x",
                                            "content": {"value": "next_page"}
                                        },
                                        {
                                            "entryId": "cursor-top-x",
                                            "content": {"value": "top_page"}
                                        }
                                    ]
                                }]
                            }
                        }
                    }
                }
            }
        });
        let (users, cursor) =
            parse_user_list(&body, &["data", "user", "result", "timeline", "timeline"]);
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "bob");
        assert_eq!(cursor, Some("next_page".to_string()));
    }

    // ── extract_tweet_from_entry ─────────────────────────────────

    #[test]
    fn extract_tweet_from_entry_valid() {
        let entry = json!({
            "content": {
                "itemContent": {
                    "tweet_results": {
                        "result": {
                            "rest_id": "e1",
                            "legacy": {"full_text": "entry tweet", "user_id_str": "1"}
                        }
                    }
                }
            }
        });
        let t = extract_tweet_from_entry(&entry).unwrap();
        assert_eq!(t.id, "e1");
    }

    #[test]
    fn extract_tweet_from_entry_missing_content() {
        let entry = json!({});
        assert!(extract_tweet_from_entry(&entry).is_none());
    }

    // ── extract_tweet_from_module_item ────────────────────────────

    #[test]
    fn extract_tweet_from_module_item_valid() {
        let item = json!({
            "item": {
                "itemContent": {
                    "tweet_results": {
                        "result": {
                            "rest_id": "m1",
                            "legacy": {"full_text": "module", "user_id_str": "2"}
                        }
                    }
                }
            }
        });
        let t = extract_tweet_from_module_item(&item).unwrap();
        assert_eq!(t.id, "m1");
    }

    #[test]
    fn extract_tweet_from_module_item_missing() {
        let item = json!({"item": {}});
        assert!(extract_tweet_from_module_item(&item).is_none());
    }
}
