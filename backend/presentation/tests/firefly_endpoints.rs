use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use chrono::NaiveDate;
use ffm_application::dtos::{
    Ff3AccountDto, NewReconciliation, ReconciliationRun, RunFf3Transaction, RunMatch,
};
use ffm_application::ports::ReconciliationRepository;
use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::{AmountCents, MovementStatus};
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
            iban: None,
            account_number: None,
            role: None,
            liability_type: None,
        },
        Ff3AccountDto {
            ff3_id: 2,
            name: "Savings".into(),
            account_type: "asset".into(),
            currency: "USD".into(),
            iban: None,
            account_number: None,
            role: None,
            liability_type: None,
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
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
async fn get_firefly_accounts_by_type_returns_expense_accounts() {
    let account = |ff3_id: i64, name: &str, account_type: &str| Ff3AccountDto {
        ff3_id,
        name: name.into(),
        account_type: account_type.into(),
        currency: "EUR".into(),
        iban: None,
        account_number: None,
        role: None,
        liability_type: None,
    };
    let accounts = vec![
        account(1, "Main", "asset"),
        account(7, "Groceries", "expense"),
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
    };
    let router = app(state);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/firefly/accounts?type=expense")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["id"], 7);
    assert_eq!(value[0]["name"], "Groceries");
    assert_eq!(repo.upserted_count(), 1);
}

#[tokio::test]
async fn get_firefly_tags_categories_and_budgets_return_names() {
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
    };
    let router = app(state);

    for (uri, expected) in [
        ("/api/firefly/tags", json!(["groceries", "rent"])),
        ("/api/firefly/categories", json!(["Food"])),
        ("/api/firefly/budgets", json!(["Monthly"])),
    ] {
        let response = router
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value, expected, "{uri}");
    }
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
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
        splits: vec![],
        counterparty_name: Some("Restaurant".into()),
        counterparty_type: Some("Expense account".into()),
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
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
    assert_eq!(value[0]["counterparty_name"], "Restaurant");
    assert_eq!(value[0]["counterparty_type"], "Expense account");
}

#[tokio::test]
async fn query_account_transactions_ok() {
    let txs = vec![ffm_application::dtos::Ff3TransactionDto {
        ff3_id: 99,
        date: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        amount_cents: AmountCents::from_str("500.00").unwrap(),
        description: "deposit".into(),
        splits: vec![],
        counterparty_name: None,
        counterparty_type: None,
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
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
async fn post_account_transactions_enriches_status_from_cache() {
    fn live(ff3_id: i64) -> ffm_application::dtos::Ff3TransactionDto {
        ffm_application::dtos::Ff3TransactionDto {
            ff3_id,
            date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            amount_cents: AmountCents::new(-100),
            description: format!("tx{ff3_id}"),
            splits: vec![],
            counterparty_name: None,
            counterparty_type: None,
        }
    }
    fn cached(ff3_id: i64, dup: bool) -> RunFf3Transaction {
        RunFf3Transaction {
            ff3_journal_id: 0,
            ff3_id,
            date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            amount_cents: AmountCents::new(-100),
            description: format!("tx{ff3_id}"),
            is_duplicate: dup,
            counterparty_name: None,
            counterparty_type: None,
        }
    }

    // One shared store for matches + ff3_transactions (the real adapters
    // sit on the same tables).
    let store = Arc::new(common::FakeReconciliationStore::new());
    store
        .save_run(ReconciliationRun {
            reconciliation: NewReconciliation {
                batch_id: BatchId::new(1),
                tolerance_days: 0,
                window_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                window_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                matched_count: 1,
                missing_count: 0,
                extra_count: 2,
            },
            ff3_transactions: vec![cached(1, false), cached(2, true), cached(3, false)],
            matches: vec![RunMatch {
                ff3_journal_id: 0,
                movement_id: MovementId::new(1),
                ff3_id: 1,
            }],
            movement_status_updates: vec![(MovementId::new(1), MovementStatus::Matched)],
        })
        .await
        .unwrap();

    let state = AppState {
        firefly: Arc::new(FakeFireflyClient::with_transactions(vec![
            live(1),
            live(2),
            live(3),
            live(4),
        ])),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
        csv_decoder: Arc::new(common::FakeCsvDecoder),
        batches: Arc::new(common::FakeBatchRepository),
        movements: Arc::new(common::FakeMovementRepository),
        csv_mappings: Arc::new(common::FakeCsvMappingRepository),
        reconciliations: store.clone(),
        matches: store.clone(),
        ff3_transactions: store,
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
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
    let by_id = |id: i64| {
        value
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == id)
            .cloned()
            .unwrap()
    };
    assert_eq!(by_id(1)["status"], "matched");
    assert_eq!(by_id(2)["status"], "duplicate");
    assert_eq!(by_id(3)["status"], "extra");
    assert!(by_id(4)["status"].is_null());
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
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
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
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

fn accounts_state(accounts: Vec<Ff3AccountDto>) -> AppState {
    AppState {
        firefly: Arc::new(FakeFireflyClient::with_accounts(accounts)),
        ff3_accounts: Arc::new(FakeFf3AccountRepository::new()),
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
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        settings_lock: Arc::new(tokio::sync::Mutex::new(())),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
    }
}

async fn get_json(router: axum::Router, uri: &str) -> serde_json::Value {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn get_firefly_accounts_exposes_role_and_liability_type() {
    let account =
        |ff3_id: i64, account_type: &str, role: Option<&str>, lt: Option<&str>| Ff3AccountDto {
            ff3_id,
            name: format!("acc {ff3_id}"),
            account_type: account_type.into(),
            currency: "COP".into(),
            iban: None,
            account_number: None,
            role: role.map(Into::into),
            liability_type: lt.map(Into::into),
        };
    let accounts = vec![
        account(4, "asset", Some("ccAsset"), None),
        account(7, "liability", None, Some("loan")),
    ];

    let assets = get_json(
        app(accounts_state(accounts.clone())),
        "/api/firefly/accounts",
    )
    .await;
    assert_eq!(assets[0]["role"], "ccAsset");
    assert!(assets[0]["liability_type"].is_null());

    let liabilities = get_json(
        app(accounts_state(accounts)),
        "/api/firefly/accounts?type=liability",
    )
    .await;
    assert_eq!(liabilities[0]["liability_type"], "loan");
    assert!(liabilities[0]["role"].is_null());
}
