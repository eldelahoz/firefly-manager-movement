use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use chrono::NaiveDate;
use ffm_application::dtos::Ff3AccountDto;
use ffm_domain::value_objects::AmountCents;
use ffm_presentation::{app, AppState};
use serde_json::json;
use std::str::FromStr;
use tower::ServiceExt;

use crate::common::{FakeFf3AccountRepository, FakeFireflyClient};

mod common;

#[tokio::test]
async fn get_firefly_accounts_returns_accounts_and_syncs() {
    let accounts = vec![
        Ff3AccountDto {
            ff3_id: 1,
            name: "Main".into(),
            account_type: "asset".into(),
            currency: "EUR".into(),
        },
        Ff3AccountDto {
            ff3_id: 2,
            name: "Savings".into(),
            account_type: "asset".into(),
            currency: "USD".into(),
        },
    ];
    let repo = Arc::new(FakeFf3AccountRepository::new());
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::with_accounts(accounts)),
        ff3_accounts: repo.clone(),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        ff3_health_probe: false,
    };
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/firefly/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 2);
    assert_eq!(value[0]["name"], "Main");
    assert_eq!(value[0]["id"], 1);
    assert_eq!(value[1]["currency"], "USD");
    assert_eq!(repo.upserted_count(), 2);
}

#[tokio::test]
async fn get_firefly_accounts_ff3_error_returns_502() {
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::failing()),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        ff3_health_probe: false,
    };
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/firefly/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn post_account_transactions_ok() {
    let txs = vec![ffm_application::dtos::Ff3TransactionDto {
        ff3_id: 42,
        date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
        amount_cents: AmountCents::from_str("-123.45").unwrap(),
        description: "lunch".into(),
    }];
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::with_transactions(txs)),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        ff3_health_probe: false,
    };
    let router = app(state);

    let body = json!({ "start": "2026-01-01", "end": "2026-01-31" });
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/firefly/accounts/1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["id"], 42);
    assert_eq!(value[0]["date"], "2026-01-05");
    assert_eq!(value[0]["amount_cents"], -12345);
    assert_eq!(value[0]["description"], "lunch");
}

#[tokio::test]
async fn query_account_transactions_ok() {
    let txs = vec![ffm_application::dtos::Ff3TransactionDto {
        ff3_id: 99,
        date: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        amount_cents: AmountCents::from_str("500.00").unwrap(),
        description: "deposit".into(),
    }];
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::with_transactions(txs)),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        ff3_health_probe: false,
    };
    let router = app(state);

    let body = json!({ "start": "2026-03-01", "end": "2026-03-31" });
    let response = router
        .oneshot(
            Request::builder()
                .method("QUERY")
                .uri("/api/firefly/accounts/5/transactions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["id"], 99);
    assert_eq!(value[0]["description"], "deposit");
}

#[tokio::test]
async fn get_on_transactions_returns_405() {
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        ff3_health_probe: false,
    };
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/firefly/accounts/1/transactions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn transactions_start_after_end_returns_400() {
    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::ok()),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: Arc::new(common::FakeReconciliationStore::new()),
        matches: Arc::new(common::FakeReconciliationStore::new()),
        ff3_transactions: Arc::new(common::FakeReconciliationStore::new()),
        ff3_health_probe: false,
    };
    let router = app(state);

    let body = json!({ "start": "2026-12-01", "end": "2026-01-01" });
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/firefly/accounts/1/transactions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
