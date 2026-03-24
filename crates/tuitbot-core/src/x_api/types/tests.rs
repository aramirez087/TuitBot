//! Comprehensive tests for X API types.

use super::super::*;
use std::collections::HashMap;

#[test]
fn deserialize_tweet() {
    let json = r#"{"id":"1234567890","text":"Hello world","author_id":"987654321","created_at":"2026-02-21T12:00:00.000Z","public_metrics":{"retweet_count":5,"reply_count":2,"like_count":10,"quote_count":1,"impression_count":500,"bookmark_count":3},"conversation_id":"1234567890"}"#;
    let tweet: Tweet = serde_json::from_str(json).unwrap();
    assert_eq!(tweet.id, "1234567890");
    assert_eq!(tweet.text, "Hello world");
    assert_eq!(tweet.public_metrics.like_count, 10);
}

#[test]
fn deserialize_tweet_missing_optional() {
    let json = r#"{"id":"123","text":"Hello","author_id":"456"}"#;
    let tweet: Tweet = serde_json::from_str(json).unwrap();
    assert_eq!(tweet.public_metrics.like_count, 0);
    assert!(tweet.conversation_id.is_none());
}

#[test]
fn deserialize_search_response() {
    let json = r#"{"data":[{"id":"1","text":"Tweet 1","author_id":"a1"}],"includes":{"users":[{"id":"a1","username":"user1","name":"User One"}]},"meta":{"newest_id":"1","oldest_id":"1","result_count":1,"next_token":"abc123"}}"#;
    let resp: SearchResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.meta.result_count, 1);
    assert_eq!(resp.meta.next_token, Some("abc123".to_string()));
}

#[test]
fn deserialize_search_response_empty() {
    let json = r#"{"meta":{"result_count":0}}"#;
    let resp: SearchResponse = serde_json::from_str(json).unwrap();
    assert!(resp.data.is_empty());
    assert_eq!(resp.meta.result_count, 0);
}

#[test]
fn serialize_post_tweet_request_minimal() {
    let req = PostTweetRequest {
        text: "Hello!".to_string(),
        reply: None,
        media: None,
        quote_tweet_id: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("reply"));
    assert!(!json.contains("media"));
}

#[test]
fn serialize_post_tweet_request_full() {
    let req = PostTweetRequest {
        text: "test".to_string(),
        reply: Some(ReplyTo {
            in_reply_to_tweet_id: "123".to_string(),
        }),
        media: Some(MediaPayload {
            media_ids: vec!["m1".to_string()],
        }),
        quote_tweet_id: Some("qt1".to_string()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("in_reply_to_tweet_id"));
    assert!(json.contains("media_ids"));
    assert!(json.contains("quote_tweet_id"));
}

#[test]
fn media_type_properties_jpeg() {
    let jpeg = MediaType::Image(ImageFormat::Jpeg);
    assert_eq!(jpeg.mime_type(), "image/jpeg");
    assert_eq!(jpeg.max_size(), 5 * 1024 * 1024);
    assert_eq!(jpeg.media_category(), "tweet_image");
    assert!(!jpeg.requires_chunked(1024));
}

#[test]
fn media_type_properties_gif() {
    let gif = MediaType::Gif;
    assert_eq!(gif.mime_type(), "image/gif");
    assert_eq!(gif.max_size(), 15 * 1024 * 1024);
    assert!(gif.requires_chunked(1024));
}

#[test]
fn media_type_properties_video() {
    let video = MediaType::Video;
    assert_eq!(video.mime_type(), "video/mp4");
    assert_eq!(video.max_size(), 512 * 1024 * 1024);
    assert!(video.requires_chunked(1024));
}

#[test]
fn deserialize_post_tweet_response() {
    let json = r#"{"data":{"id":"111","text":"My tweet"}}"#;
    let resp: PostTweetResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data.id, "111");
    assert_eq!(resp.data.text, "My tweet");
}

#[test]
fn deserialize_error_response() {
    let json = r#"{"detail":"Too Many Requests","title":"Too Many Requests","type":"about:blank","status":429}"#;
    let err: XApiErrorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(err.detail, Some("Too Many Requests".to_string()));
    assert_eq!(err.status, Some(429));
}

#[test]
fn deserialize_users_response() {
    let json = r#"{"data":[{"id":"u1","username":"alice","name":"Alice"},{"id":"u2","username":"bob","name":"Bob"}],"meta":{"result_count":2,"next_token":"page2"}}"#;
    let resp: UsersResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.data[0].username, "alice");
    assert_eq!(resp.meta.result_count, 2);
}

#[test]
fn deserialize_users_response_empty() {
    let json = r#"{"meta":{"result_count":0}}"#;
    let resp: UsersResponse = serde_json::from_str(json).unwrap();
    assert!(resp.data.is_empty());
}

#[test]
fn action_result_data_aliases() {
    let tests = vec![
        (r#"{"liked":true}"#, true),
        (r#"{"following":false}"#, false),
        (r#"{"retweeted":true}"#, true),
        (r#"{"bookmarked":false}"#, false),
    ];
    for (json, expected) in tests {
        let data: ActionResultData = serde_json::from_str(json).unwrap();
        assert_eq!(data.result, expected);
    }
}

#[test]
fn deserialize_user_response() {
    let json = r#"{"data":{"id":"123","username":"testuser","name":"Test User"}}"#;
    let resp: UserResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data.username, "testuser");
}

#[test]
fn deserialize_user_with_profile() {
    let json = r#"{"data":{"id":"123","username":"testuser","name":"Test User","description":"dev","location":"NYC","url":"https://example.com"}}"#;
    let resp: UserResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data.description.as_deref(), Some("dev"));
    assert_eq!(resp.data.location.as_deref(), Some("NYC"));
}

#[test]
fn tweet_serde_roundtrip() {
    let json = r#"{"id":"99","text":"test","author_id":"7","public_metrics":{"like_count":3,"bookmark_count":6}}"#;
    let tweet: Tweet = serde_json::from_str(json).unwrap();
    assert_eq!(tweet.public_metrics.like_count, 3);
    let back = serde_json::to_string(&tweet).unwrap();
    let re: Tweet = serde_json::from_str(&back).unwrap();
    assert_eq!(re.text, "test");
}

#[test]
fn public_metrics_default() {
    let pm = PublicMetrics::default();
    assert_eq!(pm.retweet_count, 0);
    assert_eq!(pm.like_count, 0);
}

#[test]
fn user_metrics_default() {
    let um = UserMetrics::default();
    assert_eq!(um.followers_count, 0);
    assert_eq!(um.tweet_count, 0);
}

#[test]
fn user_metrics_serde_roundtrip() {
    let um = UserMetrics {
        followers_count: 100,
        following_count: 50,
        tweet_count: 200,
    };
    let json = serde_json::to_string(&um).unwrap();
    let back: UserMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(back.followers_count, 100);
}

#[test]
fn includes_serde_roundtrip() {
    let inc = Includes {
        users: vec![User {
            id: "1".into(),
            username: "u".into(),
            name: "N".into(),
            profile_image_url: None,
            description: None,
            location: None,
            url: None,
            public_metrics: UserMetrics::default(),
        }],
    };
    let json = serde_json::to_string(&inc).unwrap();
    let back: Includes = serde_json::from_str(&json).unwrap();
    assert_eq!(back.users.len(), 1);
}

#[test]
fn user_serde_roundtrip() {
    let user = User {
        id: "42".into(),
        username: "alice".into(),
        name: "Alice".into(),
        profile_image_url: Some("https://img.example.com/a.jpg".into()),
        description: Some("dev".into()),
        location: Some("NYC".into()),
        url: Some("https://example.com".into()),
        public_metrics: UserMetrics {
            followers_count: 10,
            following_count: 5,
            tweet_count: 30,
        },
    };
    let json = serde_json::to_string(&user).unwrap();
    let back: User = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "42");
}

#[test]
fn search_meta_serde_roundtrip() {
    let meta = SearchMeta {
        newest_id: Some("n1".into()),
        oldest_id: Some("o1".into()),
        result_count: 5,
        next_token: Some("tok".into()),
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: SearchMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.result_count, 5);
}

#[test]
fn media_payload_serde_roundtrip() {
    let mp = MediaPayload {
        media_ids: vec!["a".into(), "b".into()],
    };
    let json = serde_json::to_string(&mp).unwrap();
    let back: MediaPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(back.media_ids.len(), 2);
}

#[test]
fn posted_tweet_serde_roundtrip() {
    let pt = PostedTweet {
        id: "100".into(),
        text: "posted".into(),
    };
    let json = serde_json::to_string(&pt).unwrap();
    let back: PostedTweet = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "100");
}

#[test]
fn raw_api_response_serde() {
    let raw = RawApiResponse {
        status: 200,
        headers: HashMap::from([("x-rate".into(), "100".into())]),
        body: r#"{"ok":true}"#.into(),
        rate_limit: None,
    };
    let json = serde_json::to_string(&raw).unwrap();
    let back: RawApiResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, 200);
}

#[test]
fn single_tweet_response_serde() {
    let json = r#"{"data":{"id":"1","text":"hi","author_id":"2"}}"#;
    let resp: SingleTweetResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data.id, "1");
    assert!(resp.includes.is_none());
}

#[test]
fn users_meta_serde() {
    let meta = UsersMeta {
        result_count: 3,
        next_token: Some("nt".into()),
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: UsersMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.result_count, 3);
}

#[test]
fn like_tweet_request_serde() {
    let req = LikeTweetRequest {
        tweet_id: "t1".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: LikeTweetRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tweet_id, "t1");
}

#[test]
fn follow_user_request_serde() {
    let req = FollowUserRequest {
        target_user_id: "u1".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: FollowUserRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.target_user_id, "u1");
}

#[test]
fn action_result_response_serde() {
    let resp = ActionResultResponse {
        data: ActionResultData { result: true },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ActionResultResponse = serde_json::from_str(&json).unwrap();
    assert!(back.data.result);
}

#[test]
fn bookmark_tweet_request_serde() {
    let req = BookmarkTweetRequest {
        tweet_id: "b1".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: BookmarkTweetRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tweet_id, "b1");
}

#[test]
fn retweet_request_serde() {
    let req = RetweetRequest {
        tweet_id: "rt1".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: RetweetRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tweet_id, "rt1");
}

#[test]
fn delete_tweet_response_serde() {
    let resp = DeleteTweetResponse {
        data: DeleteTweetData { deleted: true },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: DeleteTweetResponse = serde_json::from_str(&json).unwrap();
    assert!(back.data.deleted);
}

#[test]
fn media_type_image_formats() {
    assert_eq!(MediaType::Image(ImageFormat::Png).mime_type(), "image/png");
    assert_eq!(
        MediaType::Image(ImageFormat::Webp).mime_type(),
        "image/webp"
    );
}

#[test]
fn media_id_clone_eq() {
    let id = MediaId("mid1".into());
    let id2 = id.clone();
    assert_eq!(id, id2);
}

#[test]
fn mention_response_is_search_response() {
    let json = r#"{"data":[],"meta":{"result_count":0}}"#;
    let resp: MentionResponse = serde_json::from_str(json).unwrap();
    assert!(resp.data.is_empty());
}

#[test]
fn image_format_eq() {
    assert_eq!(ImageFormat::Jpeg, ImageFormat::Jpeg);
    assert_ne!(ImageFormat::Jpeg, ImageFormat::Png);
}

#[test]
fn media_type_eq_copy() {
    let a = MediaType::Image(ImageFormat::Jpeg);
    let b = a;
    assert_eq!(a, b);
    assert_ne!(a, MediaType::Gif);
}

#[test]
fn media_type_chunked_boundary() {
    let jpeg = MediaType::Image(ImageFormat::Jpeg);
    assert!(!jpeg.requires_chunked(5 * 1024 * 1024));
    assert!(jpeg.requires_chunked(5 * 1024 * 1024 + 1));
}

#[test]
fn rate_limit_info_construction() {
    let info = RateLimitInfo {
        remaining: Some(100),
        reset_at: Some(1700000000),
    };
    assert_eq!(info.remaining, Some(100));
}

#[test]
fn post_tweet_request_full_roundtrip() {
    let req = PostTweetRequest {
        text: "full test".into(),
        reply: Some(ReplyTo {
            in_reply_to_tweet_id: "123".into(),
        }),
        media: Some(MediaPayload {
            media_ids: vec!["m1".into()],
        }),
        quote_tweet_id: Some("qt1".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: PostTweetRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.text, "full test");
}

#[test]
fn user_response_serde_roundtrip() {
    let resp = UserResponse {
        data: User {
            id: "u1".into(),
            username: "test".into(),
            name: "Test".into(),
            profile_image_url: None,
            description: None,
            location: None,
            url: None,
            public_metrics: UserMetrics::default(),
        },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: UserResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.data.username, "test");
}

#[test]
fn users_response_serde_roundtrip() {
    let resp = UsersResponse {
        data: vec![],
        meta: UsersMeta {
            result_count: 0,
            next_token: None,
        },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: UsersResponse = serde_json::from_str(&json).unwrap();
    assert!(back.data.is_empty());
}
