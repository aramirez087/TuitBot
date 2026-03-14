//! Coverage tests for admin/handlers.rs handler methods.

use rmcp::handler::server::wrapper::Parameters;

use crate::requests::*;

use super::super::AdminMcpServer;
use super::make_state;

#[tokio::test]
async fn admin_quote_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_quote_tweet(Parameters(QuoteTweetRequest {
            text: "quoting this".to_string(),
            quoted_tweet_id: "123".to_string(),
            media_ids: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_like_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_like_tweet(Parameters(LikeTweetMcpRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_follow_user() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_follow_user(Parameters(FollowUserMcpRequest {
            target_user_id: "u2".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_unfollow_user() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_unfollow_user(Parameters(UnfollowUserMcpRequest {
            target_user_id: "u2".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_retweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_retweet(Parameters(RetweetMcpRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_unretweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_unretweet(Parameters(UnretweetMcpRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_delete_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_delete_tweet(Parameters(DeleteTweetMcpRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_post_thread() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_post_thread(Parameters(PostThreadMcpRequest {
            tweets: vec!["tweet 1".to_string(), "tweet 2".to_string()],
            media_ids: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_upload_media() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_upload_media(Parameters(UploadMediaMcpRequest {
            file_path: "/tmp/nonexistent.jpg".to_string(),
            alt_text: None,
            dry_run: true,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_post_tweet_dry_run() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_post_tweet_dry_run(Parameters(PostTweetDryRunRequest {
            text: "dry run tweet".to_string(),
            media_ids: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_post_thread_dry_run() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_post_thread_dry_run(Parameters(PostThreadDryRunRequest {
            tweets: vec!["part 1".to_string()],
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_home_timeline() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_home_timeline(Parameters(GetHomeTimelineRequest {
            max_results: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_followers() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_followers(Parameters(GetFollowersRequest {
            user_id: "u1".to_string(),
            max_results: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_following() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_following(Parameters(GetFollowingRequest {
            user_id: "u1".to_string(),
            max_results: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_user_by_id() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_user_by_id(Parameters(GetUserByIdRequest {
            user_id: "u1".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_liked_tweets() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_liked_tweets(Parameters(GetLikedTweetsRequest {
            user_id: "u1".to_string(),
            max_results: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_bookmarks() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_bookmarks(Parameters(GetBookmarksRequest {
            max_results: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_users_by_ids() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_users_by_ids(Parameters(GetUsersByIdsRequest {
            user_ids: vec!["u1".to_string()],
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_tweet_liking_users() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_tweet_liking_users(Parameters(GetTweetLikingUsersRequest {
            tweet_id: "123".to_string(),
            max_results: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_unlike_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_unlike_tweet(Parameters(UnlikeTweetMcpRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_bookmark_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_bookmark_tweet(Parameters(BookmarkTweetMcpRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_unbookmark_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_unbookmark_tweet(Parameters(UnbookmarkTweetMcpRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_x_usage() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_x_usage(Parameters(GetXUsageRequest { since_days: None }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_author_context() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_author_context(Parameters(GetAuthorContextRequest {
            tweet_id: "123".to_string(),
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_recommend_engagement_action() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .recommend_engagement_action(Parameters(RecommendEngagementRequest {
            tweet_id: "123".to_string(),
            context: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_topic_performance_snapshot() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .topic_performance_snapshot(Parameters(TopicPerformanceSnapshotRequest {
            topic: "rust".to_string(),
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_mcp_tool_metrics() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_mcp_tool_metrics(Parameters(GetMcpToolMetricsRequest { since_hours: None }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_mcp_error_breakdown() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_mcp_error_breakdown(Parameters(GetMcpErrorBreakdownRequest {
            since_hours: None
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_find_reply_opportunities() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .find_reply_opportunities(Parameters(FindReplyOpportunitiesRequest {
            query: None,
            min_score: None,
            limit: None,
            since_id: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_draft_replies_for_candidates() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .draft_replies_for_candidates(Parameters(DraftRepliesRequest {
            candidate_ids: vec![],
            archetype: None,
            mention_product: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_propose_and_queue_replies() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .propose_and_queue_replies(Parameters(ProposeAndQueueRepliesRequest {
            items: vec![],
            mention_product: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_generate_thread_plan() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .generate_thread_plan(Parameters(GenerateThreadPlanRequest {
            topic: "Rust async".to_string(),
            objective: None,
            target_audience: None,
            structure: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_x_get() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get(Parameters(XGetRequest {
            path: "/2/tweets".to_string(),
            host: None,
            query: None,
            headers: None,
            auto_paginate: false,
            max_pages: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_x_post() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_post(Parameters(XPostRequest {
            path: "/2/tweets".to_string(),
            host: None,
            query: None,
            body: Some(r#"{"text":"test"}"#.to_string()),
            headers: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_x_put() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_put(Parameters(XPutRequest {
            path: "/2/tweets/123/hidden".to_string(),
            host: None,
            query: None,
            body: Some(r#"{"hidden":true}"#.to_string()),
            headers: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_x_delete() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_delete(Parameters(XDeleteRequest {
            path: "/2/tweets/123".to_string(),
            host: None,
            query: None,
            headers: None,
        }))
        .await
        .is_ok());
}
