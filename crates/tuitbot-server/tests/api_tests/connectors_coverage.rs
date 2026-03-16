//! Coverage tests for connector routes (Google Drive link, status, disconnect).

use super::*;

// ============================================================
// GET /api/connectors/google-drive/status
// ============================================================

#[tokio::test]
async fn connectors_status_empty_connections() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/connectors/google-drive/status").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["connections"].is_array());
    assert!(body["connections"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn connectors_status_requires_auth() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/status")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================
// POST /api/connectors/google-drive/link
// ============================================================

/// link_google_drive uses Query params, not JSON body. The default
/// test connector_config has no client_id/secret, so we expect a
/// validation error from GoogleDriveConnector::new.
#[tokio::test]
async fn connectors_link_no_config_returns_error() {
    let router = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/connectors/google-drive/link")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let status = resp.status();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "expected 400 without connector config"
    );
}

#[tokio::test]
async fn connectors_link_requires_auth() {
    let router = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/connectors/google-drive/link")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn connectors_link_with_force_param() {
    let router = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/connectors/google-drive/link?force=true")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let status = resp.status();
    // Still fails because connector config is empty, but exercises the force param path.
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "expected 400 even with force=true (no config)"
    );
}

// ============================================================
// GET /api/connectors/google-drive/callback
// ============================================================

#[tokio::test]
async fn connectors_callback_missing_code() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback?state=abc")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn connectors_callback_missing_state() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback?code=abc")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn connectors_callback_empty_params() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn connectors_callback_invalid_state() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback?code=testcode&state=nonexistent")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "invalid state should return 400"
    );
}

#[tokio::test]
async fn connectors_callback_empty_code() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback?code=&state=abc")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "empty code should return 400"
    );
}

#[tokio::test]
async fn connectors_callback_empty_state() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback?code=abc&state=")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "empty state should return 400"
    );
}

// ============================================================
// DELETE /api/connectors/google-drive/{id}
// ============================================================

#[tokio::test]
async fn connectors_disconnect_nonexistent() {
    let router = test_router().await;
    let (status, body) = delete_json(router, "/api/connectors/google-drive/99999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("connection not found"));
}

#[tokio::test]
async fn connectors_disconnect_requires_auth() {
    let router = test_router().await;
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/connectors/google-drive/1")
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn connectors_disconnect_zero_id() {
    let router = test_router().await;
    let (status, body) = delete_json(router, "/api/connectors/google-drive/0").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("connection not found"));
}
