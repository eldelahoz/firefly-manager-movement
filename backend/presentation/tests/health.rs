use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use ffm_presentation::{app, AppState};
use tower::ServiceExt;

use crate::common::FakeFireflyClient;

mod common;

#[tokio::test]
async fn health_returns_ok_json() {
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(common::FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
    };
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert_eq!(body, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn health_with_probe_ok() {
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(common::FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: true,
    };
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"reachable\":true"));
    assert!(body.contains("\"version\":\"6.1.36\""));
}

#[tokio::test]
async fn health_with_probe_degraded() {
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::failing()),
        ff3_accounts: Arc::new(common::FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: true,
    };
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("\"status\":\"degraded\""));
    assert!(body.contains("\"reachable\":false"));
}

#[tokio::test]
async fn openapi_json_is_served() {
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(common::FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
    };
    let router = app(state);

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
    assert!(value["paths"]["/api/firefly/accounts"].is_object());
}
