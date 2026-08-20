use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use ffm_presentation::app;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok_json() {
    let router = app();

    let response = router
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert_eq!(body, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn openapi_json_is_served() {
    let router = app();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["openapi"], "3.1.0");
    assert!(value["paths"]["/health"].is_object());
}