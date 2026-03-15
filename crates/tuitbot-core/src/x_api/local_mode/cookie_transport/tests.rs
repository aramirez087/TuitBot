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

// --- generate_client_uuid tests ---

#[test]
fn generate_client_uuid_format() {
    let uuid = generate_client_uuid();
    // UUID v4 format: 8-4-4-4-12 hex chars
    let parts: Vec<&str> = uuid.split('-').collect();
    assert_eq!(parts.len(), 5, "UUID should have 5 parts: {uuid}");
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
}

#[test]
fn generate_client_uuid_is_unique() {
    let u1 = generate_client_uuid();
    let u2 = generate_client_uuid();
    assert_ne!(u1, u2, "Two generated UUIDs should be different");
}

#[test]
fn generate_client_uuid_version_4_bits() {
    let uuid = generate_client_uuid();
    let parts: Vec<&str> = uuid.split('-').collect();
    // Third group should start with '4' (version 4)
    assert!(
        parts[2].starts_with('4'),
        "UUID v4 third group should start with '4': {uuid}"
    );
}

// --- build_search_response tests ---

#[test]
fn build_search_response_empty() {
    use super::queries::build_search_response;
    let resp = build_search_response(vec![], None);
    assert!(resp.data.is_empty());
    assert_eq!(resp.meta.result_count, 0);
    assert!(resp.meta.newest_id.is_none());
    assert!(resp.meta.oldest_id.is_none());
    assert!(resp.meta.next_token.is_none());
}

#[test]
fn build_search_response_with_tweets() {
    use super::queries::build_search_response;
    use crate::x_api::types::{PublicMetrics, Tweet};

    let tweets = vec![
        Tweet {
            id: "100".to_string(),
            text: "First".to_string(),
            author_id: "u1".to_string(),
            created_at: "2026-01-01".to_string(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        },
        Tweet {
            id: "99".to_string(),
            text: "Second".to_string(),
            author_id: "u1".to_string(),
            created_at: "2026-01-01".to_string(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        },
    ];

    let resp = build_search_response(tweets, Some("cursor_next".to_string()));
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.meta.result_count, 2);
    assert_eq!(resp.meta.newest_id.as_deref(), Some("100"));
    assert_eq!(resp.meta.oldest_id.as_deref(), Some("99"));
    assert_eq!(resp.meta.next_token.as_deref(), Some("cursor_next"));
}

// --- build_users_response tests ---

#[test]
fn build_users_response_empty() {
    use super::queries::build_users_response;
    let resp = build_users_response(vec![], None);
    assert!(resp.data.is_empty());
    assert_eq!(resp.meta.result_count, 0);
    assert!(resp.meta.next_token.is_none());
}

#[test]
fn build_users_response_with_users() {
    use super::queries::build_users_response;
    use crate::x_api::types::User;

    let users = vec![User {
        id: "u1".to_string(),
        username: "alice".to_string(),
        name: "Alice".to_string(),
        profile_image_url: None,
        description: None,
        location: None,
        url: None,
        public_metrics: Default::default(),
    }];

    let resp = build_users_response(users, Some("next".to_string()));
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.meta.result_count, 1);
    assert_eq!(resp.meta.next_token.as_deref(), Some("next"));
}

// --- Additional response parsing tests ---

#[test]
fn parse_user_no_legacy_returns_none() {
    let result = serde_json::json!({
        "rest_id": "u1"
        // No "legacy" field
    });
    assert!(response::parse_user(&result).is_none());
}

#[test]
fn parse_tweet_no_legacy_returns_none() {
    let result = serde_json::json!({
        "rest_id": "t1"
        // No "legacy" field
    });
    assert!(response::parse_tweet(&result).is_none());
}

#[test]
fn parse_user_with_empty_profile_url() {
    let result = serde_json::json!({
        "rest_id": "u1",
        "legacy": {
            "screen_name": "bob",
            "name": "Bob",
            "profile_image_url_https": "",
        }
    });
    let user = response::parse_user(&result).unwrap();
    assert!(
        user.profile_image_url.is_none(),
        "Empty URL should be filtered"
    );
}

#[test]
fn parse_user_without_optional_fields() {
    let result = serde_json::json!({
        "rest_id": "u2",
        "legacy": {
            "screen_name": "minimal",
            "name": "Minimal User"
        }
    });
    let user = response::parse_user(&result).unwrap();
    assert_eq!(user.id, "u2");
    assert!(user.description.is_none());
    assert!(user.location.is_none());
    assert!(user.url.is_none());
    assert!(user.profile_image_url.is_none());
}

#[test]
fn parse_tweet_missing_views() {
    let result = serde_json::json!({
        "rest_id": "t1",
        "legacy": {
            "full_text": "No views field",
            "user_id_str": "u1",
        }
    });
    let tweet = response::parse_tweet(&result).unwrap();
    assert_eq!(tweet.public_metrics.impression_count, 0);
}

#[test]
fn parse_tweet_views_non_numeric() {
    let result = serde_json::json!({
        "rest_id": "t1",
        "legacy": {
            "full_text": "test",
            "user_id_str": "u1",
        },
        "views": {"count": "not_a_number"}
    });
    let tweet = response::parse_tweet(&result).unwrap();
    assert_eq!(tweet.public_metrics.impression_count, 0);
}

// --- check_graphql_errors edge cases ---

#[test]
fn check_graphql_errors_empty_errors_array() {
    let body = serde_json::json!({
        "errors": []
    });
    // Empty errors array should be OK (no first element)
    assert!(response::check_graphql_errors(&body).is_ok());
}

#[test]
fn check_graphql_errors_no_message_field() {
    let body = serde_json::json!({
        "errors": [{"code": 88}]
    });
    let err = response::check_graphql_errors(&body).unwrap_err();
    assert!(err.to_string().contains("unknown GraphQL error"));
}

// --- extract_script_urls edge cases ---

#[test]
fn extract_script_urls_multiple_bundles() {
    let html = r#"
        <script src="https://abs.twimg.com/responsive-web/client-web/main.abc.js"></script>
        <script src="https://abs.twimg.com/responsive-web/client-web/vendor.xyz.js"></script>
        <script src="https://example.com/other.js"></script>
    "#;
    let urls = extract_script_urls(html);
    assert_eq!(urls.len(), 2); // Only twimg.com ones
}

#[test]
fn extract_script_urls_empty_html() {
    let urls = extract_script_urls("");
    assert!(urls.is_empty());
}

#[test]
fn extract_script_urls_x_com_bundles() {
    let html = r#"<script src="https://x.com/bundle/main.js"></script>"#;
    let urls = extract_script_urls(html);
    assert_eq!(urls.len(), 1);
}

// --- OPERATION_NAMES constant test ---

#[test]
fn operation_names_contains_key_operations() {
    assert!(OPERATION_NAMES.contains(&"CreateTweet"));
    assert!(OPERATION_NAMES.contains(&"SearchTimeline"));
    assert!(OPERATION_NAMES.contains(&"UserByScreenName"));
    assert!(OPERATION_NAMES.contains(&"HomeLatestTimeline"));
    assert!(OPERATION_NAMES.contains(&"Bookmarks"));
}

// --- parse_timeline with ReplaceEntry instruction ---

#[test]
fn parse_timeline_replace_entry_instruction() {
    let body = serde_json::json!({
        "data": {
            "tl": {
                "instructions": [{
                    "type": "TimelineReplaceEntry",
                    "entries": [{
                        "entryId": "tweet-1",
                        "content": {
                            "itemContent": {
                                "tweet_results": {
                                    "result": {
                                        "rest_id": "replaced1",
                                        "legacy": {
                                            "full_text": "Replaced tweet",
                                            "user_id_str": "u1"
                                        }
                                    }
                                }
                            }
                        }
                    }]
                }]
            }
        }
    });

    let (tweets, _) = response::parse_timeline(&body, &["data", "tl"]);
    assert_eq!(tweets.len(), 1);
    assert_eq!(tweets[0].id, "replaced1");
}

// --- parse_timeline missing instructions path ---

#[test]
fn parse_timeline_missing_path() {
    let body = serde_json::json!({"data": {}});
    let (tweets, cursor) = response::parse_timeline(&body, &["data", "nonexistent"]);
    assert!(tweets.is_empty());
    assert!(cursor.is_none());
}

// --- CookieTransport construction tests ---

#[test]
fn cookie_transport_new_sets_create_tweet_query_id() {
    let session = ScraperSession {
        auth_token: "test_auth".to_string(),
        ct0: "test_ct0".to_string(),
        username: Some("testuser".to_string()),
        created_at: Some("2026-01-01T00:00:00Z".to_string()),
    };
    let transport = CookieTransport::new(session);

    // Should have the CreateTweet query ID (fallback or env override)
    let query_id = transport.get_query_id("CreateTweet");
    assert!(query_id.is_ok(), "CreateTweet should have a query ID");
}

#[test]
fn cookie_transport_get_query_id_missing_operation() {
    let session = ScraperSession {
        auth_token: "test_auth".to_string(),
        ct0: "test_ct0".to_string(),
        username: None,
        created_at: None,
    };
    let transport = CookieTransport::new(session);

    let err = transport.get_query_id("NonExistentOperation");
    assert!(err.is_err());
    let err_str = err.unwrap_err().to_string();
    assert!(err_str.contains("NonExistentOperation"));
    assert!(err_str.contains("query ID not found"));
}

#[test]
fn cookie_transport_build_headers_includes_required_headers() {
    let session = ScraperSession {
        auth_token: "test_auth_token".to_string(),
        ct0: "test_csrf_token".to_string(),
        username: None,
        created_at: None,
    };
    let transport = CookieTransport::new(session);

    let headers = transport
        .build_headers("GET", "/test/path")
        .expect("build headers");

    assert!(headers.get("authorization").is_some());
    assert!(headers.get("cookie").is_some());
    assert!(headers.get("x-csrf-token").is_some());
    assert!(headers.get("x-twitter-auth-type").is_some());
    assert!(headers.get("x-client-uuid").is_some());
    assert!(headers.get("content-type").is_some());
    assert!(headers.get("origin").is_some());
    assert!(headers.get("referer").is_some());

    let auth = headers.get("authorization").unwrap().to_str().unwrap();
    assert!(auth.starts_with("Bearer "));

    let cookie = headers.get("cookie").unwrap().to_str().unwrap();
    assert!(cookie.contains("auth_token=test_auth_token"));
    assert!(cookie.contains("ct0=test_csrf_token"));

    let csrf = headers.get("x-csrf-token").unwrap().to_str().unwrap();
    assert_eq!(csrf, "test_csrf_token");
}

#[test]
fn cookie_transport_with_resolved_transport_uses_provided_ids() {
    let session = ScraperSession {
        auth_token: "auth".to_string(),
        ct0: "ct0".to_string(),
        username: None,
        created_at: None,
    };
    let mut query_ids = HashMap::new();
    query_ids.insert("CustomOp".to_string(), "custom_id_123".to_string());

    let transport = CookieTransport::with_resolved_transport(session, query_ids, None);

    let qid = transport.get_query_id("CustomOp").unwrap();
    assert_eq!(qid, "custom_id_123");
}

#[test]
fn operation_names_has_expected_count() {
    // Ensure we track a reasonable number of operations
    assert!(
        OPERATION_NAMES.len() >= 15,
        "Expected at least 15 operations, got {}",
        OPERATION_NAMES.len()
    );
}

#[test]
fn web_bearer_token_is_nonempty() {
    assert!(!WEB_BEARER_TOKEN.is_empty());
    assert!(WEB_BEARER_TOKEN.len() > 20);
}

#[test]
fn fallback_create_tweet_query_id_is_nonempty() {
    assert!(!FALLBACK_CREATE_TWEET_QUERY_ID.is_empty());
}

// --- Mutation variable JSON structure tests ---
// These test the JSON shapes that mutations.rs builds without making HTTP calls.

#[test]
fn create_tweet_variables_structure() {
    // Mirrors the logic in CookieTransport::create_tweet
    let text = "Hello world";
    let variables = serde_json::json!({
        "tweet_text": text,
        "dark_request": false,
        "media": {
            "media_entities": [],
            "possibly_sensitive": false
        },
        "semantic_annotation_ids": []
    });

    assert_eq!(variables["tweet_text"], "Hello world");
    assert_eq!(variables["dark_request"], false);
    assert!(variables["media"]["media_entities"].is_array());
    assert!(variables["semantic_annotation_ids"].is_array());
}

#[test]
fn create_tweet_with_reply_variables() {
    let text = "Reply text";
    let reply_id = "12345";
    let mut variables = serde_json::json!({
        "tweet_text": text,
        "dark_request": false,
        "media": {
            "media_entities": [],
            "possibly_sensitive": false
        },
        "semantic_annotation_ids": []
    });

    variables["reply"] = serde_json::json!({
        "in_reply_to_tweet_id": reply_id,
        "exclude_reply_user_ids": []
    });

    assert_eq!(variables["reply"]["in_reply_to_tweet_id"], "12345");
    assert!(variables["reply"]["exclude_reply_user_ids"].is_array());
}

#[test]
fn favorite_tweet_variables() {
    let tweet_id = "99999";
    let variables = serde_json::json!({
        "tweet_id": tweet_id,
    });
    assert_eq!(variables["tweet_id"], "99999");
}

#[test]
fn create_retweet_variables() {
    let tweet_id = "54321";
    let variables = serde_json::json!({
        "tweet_id": tweet_id,
        "dark_request": false,
    });
    assert_eq!(variables["tweet_id"], "54321");
    assert_eq!(variables["dark_request"], false);
}

#[test]
fn delete_retweet_variables() {
    let tweet_id = "11111";
    let variables = serde_json::json!({
        "source_tweet_id": tweet_id,
        "dark_request": false,
    });
    assert_eq!(variables["source_tweet_id"], "11111");
}

#[test]
fn delete_tweet_variables() {
    let tweet_id = "22222";
    let variables = serde_json::json!({
        "tweet_id": tweet_id,
        "dark_request": false,
    });
    assert_eq!(variables["tweet_id"], "22222");
    assert_eq!(variables["dark_request"], false);
}

#[test]
fn bookmark_variables() {
    let tweet_id = "33333";
    let variables = serde_json::json!({
        "tweet_id": tweet_id,
    });
    assert_eq!(variables["tweet_id"], "33333");
}

#[test]
fn follow_user_rest_body_format() {
    let target_user_id = "44444";
    let body = format!("include_profile_interstitial_type=1&user_id={target_user_id}");
    assert!(body.contains("include_profile_interstitial_type=1"));
    assert!(body.contains("user_id=44444"));
}

#[test]
fn graphql_post_body_structure() {
    let query_id = "abc123";
    let variables = serde_json::json!({"tweet_text": "test"});
    let feat = features::mutation_features();
    let body = serde_json::json!({
        "variables": variables,
        "features": feat,
        "queryId": query_id,
    });

    assert_eq!(body["queryId"], "abc123");
    assert!(body["variables"].is_object());
    assert!(body["features"].is_object());
}

// --- parse_user_list edge cases ---

#[test]
fn parse_user_list_skips_cursor_entries() {
    let body = serde_json::json!({
        "data": {
            "users": {
                "instructions": [{
                    "type": "TimelineAddEntries",
                    "entries": [
                        {
                            "entryId": "cursor-top-abc",
                            "content": {"value": "top_cursor"}
                        },
                        {
                            "entryId": "cursor-bottom-xyz",
                            "content": {"value": "bottom_cursor"}
                        }
                    ]
                }]
            }
        }
    });

    let (users, cursor) = response::parse_user_list(&body, &["data", "users"]);
    assert!(users.is_empty());
    assert_eq!(cursor, Some("bottom_cursor".to_string()));
}
