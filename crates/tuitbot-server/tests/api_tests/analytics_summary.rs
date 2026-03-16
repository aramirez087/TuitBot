use super::*;

// ============================================================

#[tokio::test]
async fn analytics_summary_returns_expected_shape() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/summary").await;
    assert_eq!(status, StatusCode::OK);
    // Shape check: must have followers, actions_today, engagement, top_topics.
    assert!(body["followers"].is_object(), "missing followers object");
    assert!(
        body["actions_today"].is_object(),
        "missing actions_today object"
    );
    assert!(body["engagement"].is_object(), "missing engagement object");
    assert!(body["top_topics"].is_array(), "missing top_topics array");
    // Numeric follower fields.
    assert!(body["followers"]["current"].is_number());
    assert!(body["followers"]["change_7d"].is_number());
    assert!(body["followers"]["change_30d"].is_number());
}

#[tokio::test]
async fn analytics_summary_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/analytics/summary")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn analytics_recent_performance_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/recent-performance").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "expected JSON array");
}

#[tokio::test]
async fn analytics_recent_performance_honours_limit_param() {
    let router = test_router().await;
    // With no data in the DB the result should be empty regardless of limit.
    let (status, body) = get_json(router, "/api/analytics/recent-performance?limit=5").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() <= 5);
}

#[tokio::test]
async fn analytics_followers_honours_days_param() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/followers?days=7").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn analytics_topics_honours_limit_param() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/topics?limit=5").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() <= 5);
}
