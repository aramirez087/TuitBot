//! Tests for the cookie transport module.

use super::*;

// --- Query ID extraction tests ---

#[test]
fn extract_query_id_before_operation_name() {
    let js = r#"e.exports={queryId:"uY34Pldm6W89yqswRmPMSQ",operationName:"CreateTweet",operationType:"mutation"}"#;
    assert_eq!(
        extract_query_id_for_operation(js, "CreateTweet"),
        Some("uY34Pldm6W89yqswRmPMSQ".to_string())
    );
}

#[test]
fn extract_query_id_after_operation_name() {
    let js = r#"{operationName:"CreateTweet",queryId:"abc123XYZ"}"#;
    assert_eq!(
        extract_query_id_for_operation(js, "CreateTweet"),
        Some("abc123XYZ".to_string())
    );
}

#[test]
fn extract_query_id_returns_none_for_missing_operation() {
    let js = r#"{queryId:"abc123",operationName:"DeleteTweet"}"#;
    assert_eq!(extract_query_id_for_operation(js, "CreateTweet"), None);
}

#[test]
fn extract_script_urls_finds_twimg_bundles() {
    let html =
        r#"<script src="https://abs.twimg.com/responsive-web/client-web/main.abc123.js"></script>"#;
    let urls = extract_script_urls(html);
    assert_eq!(urls.len(), 1);
    assert!(urls[0].contains("twimg.com"));
}

#[test]
fn extract_script_urls_ignores_non_js() {
    let html = r#"<script src="https://abs.twimg.com/data.json"></script>"#;
    let urls = extract_script_urls(html);
    assert!(urls.is_empty());
}

#[test]
fn extract_query_id_value_basic() {
    assert_eq!(
        extract_query_id_value(r#"queryId:"hello123""#),
        Some("hello123".to_string())
    );
}

#[test]
fn extract_query_id_value_rejects_empty() {
    assert_eq!(extract_query_id_value(r#"queryId:"""#), None);
}

#[test]
fn extract_multiple_operations_from_bundle() {
    let js = r#"
        e.exports={queryId:"aaa111",operationName:"CreateTweet",operationType:"mutation"}
        e.exports={queryId:"bbb222",operationName:"SearchTimeline",operationType:"query"}
        e.exports={queryId:"ccc333",operationName:"FavoriteTweet",operationType:"mutation"}
    "#;
    assert_eq!(
        extract_query_id_for_operation(js, "CreateTweet"),
        Some("aaa111".to_string())
    );
    assert_eq!(
        extract_query_id_for_operation(js, "SearchTimeline"),
        Some("bbb222".to_string())
    );
    assert_eq!(
        extract_query_id_for_operation(js, "FavoriteTweet"),
        Some("ccc333".to_string())
    );
    assert_eq!(extract_query_id_for_operation(js, "NonExistent"), None);
}

// --- Response parsing tests ---

#[test]
fn parse_tweet_basic() {
    let result = serde_json::json!({
        "rest_id": "123456",
        "legacy": {
            "full_text": "Hello world",
            "user_id_str": "user1",
            "created_at": "Mon Mar 10 12:00:00 +0000 2026",
            "retweet_count": 5,
            "reply_count": 2,
            "favorite_count": 10,
            "quote_count": 1,
            "bookmark_count": 0,
            "conversation_id_str": "123456"
        },
        "views": {"count": "500"}
    });
    let tweet = response::parse_tweet(&result).unwrap();
    assert_eq!(tweet.id, "123456");
    assert_eq!(tweet.text, "Hello world");
    assert_eq!(tweet.author_id, "user1");
    assert_eq!(tweet.public_metrics.like_count, 10);
    assert_eq!(tweet.public_metrics.impression_count, 500);
    assert_eq!(tweet.conversation_id, Some("123456".to_string()));
}

#[test]
fn parse_tweet_with_visibility_results() {
    let result = serde_json::json!({
        "__typename": "TweetWithVisibilityResults",
        "tweet": {
            "rest_id": "789",
            "legacy": {
                "full_text": "Visible tweet",
                "user_id_str": "user2",
            }
        }
    });
    let tweet = response::parse_tweet(&result).unwrap();
    assert_eq!(tweet.id, "789");
    assert_eq!(tweet.text, "Visible tweet");
}

#[test]
fn parse_tweet_tombstone_returns_none() {
    let result = serde_json::json!({
        "__typename": "TweetTombstone"
    });
    assert!(response::parse_tweet(&result).is_none());
}

#[test]
fn parse_tweet_unavailable_returns_none() {
    let result = serde_json::json!({
        "__typename": "TweetUnavailable"
    });
    assert!(response::parse_tweet(&result).is_none());
}

#[test]
fn parse_tweet_empty_id_returns_none() {
    let result = serde_json::json!({
        "rest_id": "",
        "legacy": {"full_text": "test"}
    });
    assert!(response::parse_tweet(&result).is_none());
}

#[test]
fn parse_tweet_author_from_core() {
    let result = serde_json::json!({
        "rest_id": "111",
        "legacy": {
            "full_text": "test",
        },
        "core": {
            "user_results": {
                "result": {
                    "rest_id": "author999"
                }
            }
        }
    });
    let tweet = response::parse_tweet(&result).unwrap();
    assert_eq!(tweet.author_id, "author999");
}

#[test]
fn parse_user_basic() {
    let result = serde_json::json!({
        "rest_id": "u1",
        "legacy": {
            "screen_name": "alice",
            "name": "Alice",
            "profile_image_url_https": "https://pbs.twimg.com/photo_normal.jpg",
            "description": "Hello!",
            "location": "NYC",
            "url": "https://t.co/abc",
            "followers_count": 1000,
            "friends_count": 500,
            "statuses_count": 5000,
        }
    });
    let user = response::parse_user(&result).unwrap();
    assert_eq!(user.id, "u1");
    assert_eq!(user.username, "alice");
    assert_eq!(user.name, "Alice");
    assert!(user
        .profile_image_url
        .as_ref()
        .unwrap()
        .contains("_400x400."));
    assert_eq!(user.description.as_deref(), Some("Hello!"));
    assert_eq!(user.location.as_deref(), Some("NYC"));
    assert_eq!(user.public_metrics.followers_count, 1000);
    assert_eq!(user.public_metrics.following_count, 500);
    assert_eq!(user.public_metrics.tweet_count, 5000);
}

#[test]
fn parse_user_empty_id_returns_none() {
    let result = serde_json::json!({
        "rest_id": "",
        "legacy": {"screen_name": "test"}
    });
    assert!(response::parse_user(&result).is_none());
}

#[test]
fn parse_create_tweet_response_success() {
    let body = serde_json::json!({
        "data": {
            "create_tweet": {
                "tweet_results": {
                    "result": {
                        "rest_id": "1234567890",
                        "legacy": {
                            "full_text": "Hello world"
                        }
                    }
                }
            }
        }
    });
    let result = response::parse_create_tweet_response(&body).unwrap();
    assert_eq!(result.id, "1234567890");
    assert_eq!(result.text, "Hello world");
}

#[test]
fn parse_create_tweet_response_graphql_error() {
    let body = serde_json::json!({
        "errors": [{"message": "Rate limit exceeded"}]
    });
    let err = response::parse_create_tweet_response(&body).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Rate limit exceeded"), "got: {msg}");
}

#[test]
fn parse_create_tweet_response_missing_result() {
    let body = serde_json::json!({"data": {}});
    let err = response::parse_create_tweet_response(&body).unwrap_err();
    assert!(err.to_string().contains("unexpected"), "got: {}", err);
}

#[test]
fn parse_create_tweet_response_empty_id() {
    let body = serde_json::json!({
        "data": {
            "create_tweet": {
                "tweet_results": {
                    "result": {
                        "rest_id": "",
                        "legacy": {"full_text": "hi"}
                    }
                }
            }
        }
    });
    let err = response::parse_create_tweet_response(&body).unwrap_err();
    assert!(err.to_string().contains("no tweet ID"));
}

// --- Timeline parsing tests ---

#[test]
fn parse_timeline_with_tweets_and_cursor() {
    let body = serde_json::json!({
        "data": {
            "search_by_raw_query": {
                "search_timeline": {
                    "timeline": {
                        "instructions": [{
                            "type": "TimelineAddEntries",
                            "entries": [
                                {
                                    "entryId": "tweet-1",
                                    "content": {
                                        "itemContent": {
                                            "tweet_results": {
                                                "result": {
                                                    "rest_id": "t1",
                                                    "legacy": {
                                                        "full_text": "Tweet 1",
                                                        "user_id_str": "u1"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                {
                                    "entryId": "cursor-bottom-abc123",
                                    "content": {
                                        "value": "cursor_next_page"
                                    }
                                }
                            ]
                        }]
                    }
                }
            }
        }
    });

    let (tweets, cursor) = response::parse_timeline(
        &body,
        &["data", "search_by_raw_query", "search_timeline", "timeline"],
    );
    assert_eq!(tweets.len(), 1);
    assert_eq!(tweets[0].id, "t1");
    assert_eq!(cursor, Some("cursor_next_page".to_string()));
}

#[test]
fn parse_timeline_empty() {
    let body = serde_json::json!({
        "data": {
            "search_by_raw_query": {
                "search_timeline": {
                    "timeline": {
                        "instructions": [{
                            "type": "TimelineAddEntries",
                            "entries": []
                        }]
                    }
                }
            }
        }
    });

    let (tweets, cursor) = response::parse_timeline(
        &body,
        &["data", "search_by_raw_query", "search_timeline", "timeline"],
    );
    assert!(tweets.is_empty());
    assert!(cursor.is_none());
}

#[test]
fn parse_timeline_skips_tombstones() {
    let body = serde_json::json!({
        "data": {
            "tl": {
                "instructions": [{
                    "type": "TimelineAddEntries",
                    "entries": [
                        {
                            "entryId": "tweet-1",
                            "content": {
                                "itemContent": {
                                    "tweet_results": {
                                        "result": {
                                            "__typename": "TweetTombstone"
                                        }
                                    }
                                }
                            }
                        },
                        {
                            "entryId": "tweet-2",
                            "content": {
                                "itemContent": {
                                    "tweet_results": {
                                        "result": {
                                            "rest_id": "t2",
                                            "legacy": {
                                                "full_text": "OK tweet",
                                                "user_id_str": "u1"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    ]
                }]
            }
        }
    });

    let (tweets, _) = response::parse_timeline(&body, &["data", "tl"]);
    assert_eq!(tweets.len(), 1);
    assert_eq!(tweets[0].id, "t2");
}

// --- User list parsing tests ---

#[test]
fn parse_user_list_basic() {
    let body = serde_json::json!({
        "data": {
            "user": {
                "result": {
                    "timeline": {
                        "timeline": {
                            "instructions": [{
                                "type": "TimelineAddEntries",
                                "entries": [
                                    {
                                        "entryId": "user-u1",
                                        "content": {
                                            "itemContent": {
                                                "user_results": {
                                                    "result": {
                                                        "rest_id": "u1",
                                                        "legacy": {
                                                            "screen_name": "alice",
                                                            "name": "Alice"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    {
                                        "entryId": "cursor-bottom-xyz",
                                        "content": {
                                            "value": "next_page"
                                        }
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
        response::parse_user_list(&body, &["data", "user", "result", "timeline", "timeline"]);
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].username, "alice");
    assert_eq!(cursor, Some("next_page".to_string()));
}

// --- Feature flag tests ---

#[test]
fn mutation_features_serializes() {
    let features = features::mutation_features();
    assert!(features.is_object());
    assert_eq!(
        features["tweet_awards_web_tipping_enabled"],
        serde_json::Value::Bool(false)
    );
}

#[test]
fn read_features_serializes() {
    let features = features::read_features();
    assert!(features.is_object());
    assert_eq!(
        features["view_counts_everywhere_api_enabled"],
        serde_json::Value::Bool(true)
    );
}

#[test]
fn user_features_serializes() {
    let features = features::user_features();
    assert!(features.is_object());
    assert_eq!(
        features["hidden_profile_subscriptions_enabled"],
        serde_json::Value::Bool(true)
    );
}

// --- Module-level search response builder tests ---

#[test]
fn parse_timeline_module_entries() {
    let body = serde_json::json!({
        "data": {
            "tl": {
                "instructions": [{
                    "type": "TimelineAddEntries",
                    "entries": [{
                        "entryId": "module-1",
                        "content": {
                            "items": [
                                {
                                    "item": {
                                        "itemContent": {
                                            "tweet_results": {
                                                "result": {
                                                    "rest_id": "m1",
                                                    "legacy": {
                                                        "full_text": "Module tweet",
                                                        "user_id_str": "u1"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            ]
                        }
                    }]
                }]
            }
        }
    });

    let (tweets, _) = response::parse_timeline(&body, &["data", "tl"]);
    assert_eq!(tweets.len(), 1);
    assert_eq!(tweets[0].id, "m1");
    assert_eq!(tweets[0].text, "Module tweet");
}

#[test]
fn check_graphql_errors_ok_when_none() {
    let body = serde_json::json!({"data": {}});
    assert!(response::check_graphql_errors(&body).is_ok());
}

#[test]
fn check_graphql_errors_returns_error() {
    let body = serde_json::json!({
        "errors": [{"message": "Something went wrong"}]
    });
    let err = response::check_graphql_errors(&body).unwrap_err();
    assert!(err.to_string().contains("Something went wrong"));
}
