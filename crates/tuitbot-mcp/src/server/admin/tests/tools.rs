//! Coverage tests for admin/tools.rs handler methods.

use rmcp::handler::server::wrapper::Parameters;

use crate::requests::*;

use super::super::AdminMcpServer;
use super::make_state;

#[tokio::test]
async fn admin_get_stats() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_stats(Parameters(GetStatsRequest { days: None }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_follower_trend() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_follower_trend(Parameters(GetFollowerTrendRequest { days: None }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_action_log() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_action_log(Parameters(GetActionLogRequest {
            limit: None,
            offset: None,
            action_type: None
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_action_counts() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_action_counts(Parameters(SinceHoursRequest { since_hours: None }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_recent_mutations() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_recent_mutations(Parameters(GetRecentMutationsRequest {
            limit: None,
            since_hours: None
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_mutation_detail() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_mutation_detail(Parameters(GetMutationDetailRequest {
            mutation_id: "nonexistent".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_rate_limits() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.get_rate_limits().await.is_ok());
}

#[tokio::test]
async fn admin_get_recent_replies() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_recent_replies(Parameters(SinceHoursRequest { since_hours: None }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_reply_count_today() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.get_reply_count_today().await.is_ok());
}

#[tokio::test]
async fn admin_list_target_accounts() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.list_target_accounts().await.is_ok());
}

#[tokio::test]
async fn admin_list_unreplied_tweets() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .list_unreplied_tweets(Parameters(ListUnrepliedTweetsRequest { limit: None }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_score_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .score_tweet(Parameters(ScoreTweetRequest {
            text: "hello world".to_string(),
            author_followers: None,
            author_following: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_list_pending_approvals() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.list_pending_approvals().await.is_ok());
}

#[tokio::test]
async fn admin_get_pending_count() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.get_pending_count().await.is_ok());
}

#[tokio::test]
async fn admin_approve_item() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .approve_item(Parameters(ApprovalIdRequest { id: 99999 }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_reject_item() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .reject_item(Parameters(ApprovalIdRequest { id: 99999 }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_approve_all() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.approve_all().await.is_ok());
}

#[tokio::test]
async fn admin_get_config() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.get_config().await.is_ok());
}

#[tokio::test]
async fn admin_validate_config() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.validate_config().await.is_ok());
}

#[tokio::test]
async fn admin_health_check() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.health_check().await.is_ok());
}

#[tokio::test]
async fn admin_get_policy_status() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.get_policy_status().await.is_ok());
}

#[tokio::test]
async fn admin_get_discovery_feed() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_discovery_feed(Parameters(DiscoveryFeedRequest {
            limit: None,
            min_score: None
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_suggest_topics() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s.suggest_topics().await.is_ok());
}

#[tokio::test]
async fn admin_get_tweet_by_id() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .get_tweet_by_id(Parameters(TweetIdRequest {
            tweet_id: "123".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_user_by_username() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_user_by_username(Parameters(UsernameRequest {
            username: "tuitbot".to_string()
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_search_tweets() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_search_tweets(Parameters(SearchTweetsRequest {
            query: "rust".to_string(),
            max_results: None,
            since_id: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_user_mentions() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_user_mentions(Parameters(GetUserMentionsRequest {
            since_id: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_get_user_tweets() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_get_user_tweets(Parameters(GetUserTweetsRequest {
            user_id: "u1".to_string(),
            max_results: None,
            pagination_token: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_post_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_post_tweet(Parameters(PostTweetTextRequest {
            text: "test tweet".to_string(),
            media_ids: None,
        }))
        .await
        .is_ok());
}

#[tokio::test]
async fn admin_reply_to_tweet() {
    let s = AdminMcpServer::new(make_state().await);
    assert!(s
        .x_reply_to_tweet(Parameters(ReplyToTweetRequest {
            text: "reply".to_string(),
            in_reply_to_id: "123".to_string(),
            media_ids: None,
        }))
        .await
        .is_ok());
}
