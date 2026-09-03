//! E2E tests for the rapprochement endpoints (F4).

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde_json::json;

use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewBatch, NewFf3Account, NewFf3TransactionCreation,
    NewMovement,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, Ff3AccountRepository, FireflyClient, MovementRepository,
};
use ffm_domain::entities::{Batch, Ff3Account, Movement};
use ffm_domain::ids::{BatchId, MovementId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};
use ffm_presentation::{app, AppState};
use tower::ServiceExt;

mod common;
use common::{FakeCsvDecoder, FakeCsvMappingRepository, FakeReconciliationStore};

// ── Stateful fakes ──────────────────────────────────────────────────────

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Firefly fake reading the transaction list at request time, so a test
/// can mutate it between two reconcile calls (RF-13 re-run).
struct FireflyAtRequest {
    transactions: Arc<Mutex<Vec<Ff3TransactionDto>>>,
}

#[async_trait]
impl FireflyClient for FireflyAtRequest {
    async fn about(&self) -> Result<About, ApplicationError> {
        Ok(About {
            version: "6.1.36".into(),
        })
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        Ok(vec![Ff3AccountDto {
            ff3_id: 7,
            name: "Checking".into(),
            account_type: "asset".into(),
            currency: "EUR".into(),
            iban: None,
            account_number: None,
        }])
    }
    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        Ok(self.transactions.lock().unwrap().clone())
    }
    async fn create_transaction(
        &self,
        _creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        unimplemented!("F5")
    }
    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        unimplemented!("F5")
    }
}

struct FakeFf3AccountRepo {
    accounts: Mutex<Vec<Ff3Account>>,
}

#[async_trait]
impl Ff3AccountRepository for FakeFf3AccountRepo {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        let mut store = self.accounts.lock().unwrap();
        for a in accounts {
            match store.iter_mut().find(|e| e.ff3_id == a.ff3_id) {
                Some(existing) => {
                    existing.name = a.name;
                    existing.account_type = a.account_type;
                    existing.currency = a.currency;
                    existing.synced_at = epoch();
                }
                None => {
                    let id = ffm_domain::ids::Ff3AccountId::new(store.len() as i64 + 1);
                    store.push(Ff3Account {
                        id,
                        ff3_id: a.ff3_id,
                        name: a.name,
                        account_type: a.account_type,
                        currency: a.currency,
                        iban: None,
                        account_number: None,
                        synced_at: epoch(),
                    });
                }
            }
        }
        Ok(())
    }
    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        Ok(self
            .accounts
            .lock()
            .unwrap()
            .iter()
            .find(|a| a.ff3_id == ff3_id)
            .cloned())
    }
    async fn by_id(
        &self,
        id: ffm_domain::ids::Ff3AccountId,
    ) -> Result<Option<Ff3Account>, ApplicationError> {
        Ok(self
            .accounts
            .lock()
            .unwrap()
            .iter()
            .find(|a| a.id == id)
            .cloned())
    }
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        Ok(self.accounts.lock().unwrap().clone())
    }
}

struct StatefulBatchRepo {
    batches: Mutex<Vec<Batch>>,
}

#[async_trait]
impl BatchRepository for StatefulBatchRepo {
    async fn insert(&self, _batch: NewBatch) -> Result<BatchId, ApplicationError> {
        unimplemented!("seeded in fixture")
    }
    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError> {
        self.batches
            .lock()
            .unwrap()
            .iter()
            .find(|b| b.id == id)
            .cloned()
            .ok_or_else(|| ApplicationError::NotFound("batch".into()))
    }
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        Ok(self.batches.lock().unwrap().clone())
    }
    async fn update_status(
        &self,
        id: BatchId,
        status: BatchStatus,
    ) -> Result<(), ApplicationError> {
        if let Some(b) = self.batches.lock().unwrap().iter_mut().find(|b| b.id == id) {
            b.status = status;
        }
        Ok(())
    }
    async fn update_account(
        &self,
        id: BatchId,
        account_id: ffm_domain::ids::Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        if let Some(b) = self.batches.lock().unwrap().iter_mut().find(|b| b.id == id) {
            b.account_id = Some(account_id);
        }
        Ok(())
    }
}

struct StatefulMovementRepo {
    rows: Mutex<Vec<Movement>>,
}

#[async_trait]
impl MovementRepository for StatefulMovementRepo {
    async fn insert_batch(&self, _movements: Vec<NewMovement>) -> Result<(), ApplicationError> {
        unimplemented!("seeded in fixture")
    }
    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.batch_id == batch_id)
            .cloned()
            .collect())
    }
    async fn update_status(
        &self,
        id: MovementId,
        status: MovementStatus,
    ) -> Result<(), ApplicationError> {
        if let Some(m) = self.rows.lock().unwrap().iter_mut().find(|m| m.id == id) {
            m.status = status;
        }
        Ok(())
    }
}

// ── Fixture ─────────────────────────────────────────────────────────────

fn movement(id: i64, date: NaiveDate, cents: i64) -> Movement {
    Movement {
        id: MovementId::new(id),
        batch_id: BatchId::new(1),
        line_no: id as i32,
        date,
        amount_cents: AmountCents::new(cents),
        description: format!("m{id}"),
        raw_json: "{}".into(),
        status: MovementStatus::Imported,
        created_at: epoch(),
    }
}

fn default_txs() -> Vec<Ff3TransactionDto> {
    vec![
        Ff3TransactionDto {
            ff3_id: 101,
            date: d(2026, 1, 5),
            amount_cents: AmountCents::new(12340),
            description: "tx101".into(),
        },
        Ff3TransactionDto {
            ff3_id: 102,
            date: d(2026, 1, 6),
            amount_cents: AmountCents::new(-5000),
            description: "tx102".into(),
        },
        Ff3TransactionDto {
            ff3_id: 103,
            date: d(2026, 1, 6),
            amount_cents: AmountCents::new(-5000),
            description: "tx103".into(),
        },
        Ff3TransactionDto {
            ff3_id: 104,
            date: d(2026, 1, 20),
            amount_cents: AmountCents::new(9900),
            description: "tx104".into(),
        },
    ]
}

fn reconcile_body() -> serde_json::Value {
    json!({
        "ff3_account_id": 7,
        "window_start": "2026-01-01",
        "window_end": "2026-01-31",
        "tolerance_days": 0
    })
}

struct TestApp {
    state: AppState,
    pub movements: Arc<StatefulMovementRepo>,
    pub batches: Arc<StatefulBatchRepo>,
    pub store: Arc<FakeReconciliationStore>,
    pub firefly_transactions: Arc<Mutex<Vec<Ff3TransactionDto>>>,
}

impl TestApp {
    fn app(&self) -> axum::Router {
        app(self.state.clone())
    }
}

fn test_app(transactions: Vec<Ff3TransactionDto>) -> TestApp {
    let firefly_transactions = Arc::new(Mutex::new(transactions));
    let movements = Arc::new(StatefulMovementRepo {
        rows: Mutex::new(vec![
            movement(1, d(2026, 1, 5), 12340),
            movement(2, d(2026, 1, 6), -5000),
            movement(4, d(2026, 1, 8), 777),
        ]),
    });
    let batches = Arc::new(StatefulBatchRepo {
        batches: Mutex::new(vec![Batch {
            id: BatchId::new(1),
            filename: "stmt.csv".into(),
            file_hash: "h".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: None,
            status: BatchStatus::Parsed,
            created_at: epoch(),
        }]),
    });
    let store = Arc::new(FakeReconciliationStore::with_movements(movements.clone()));

    let state = AppState {
        firefly: Arc::new(FireflyAtRequest {
            transactions: firefly_transactions.clone(),
        }),
        ff3_accounts: Arc::new(FakeFf3AccountRepo {
            accounts: Mutex::new(vec![]),
        }),
        csv_decoder: Arc::new(FakeCsvDecoder),
        batches: batches.clone(),
        movements: movements.clone(),
        csv_mappings: Arc::new(FakeCsvMappingRepository),
        reconciliations: store.clone(),
        matches: store.clone(),
        ff3_transactions: store.clone(),
        creations: Arc::new(common::FakeCreationRepository),
        deleted_audits: Arc::new(common::FakeDeletedAuditRepository::default()),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(common::FakeFireflyClient::ok())),
    };

    TestApp {
        state,
        movements,
        batches,
        store,
        firefly_transactions,
    }
}

fn post_json(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::post(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn read_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json")
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn reconcile_endpoint_returns_counts_and_results_three_groups() {
    let t = test_app(default_txs());

    // POST reconcile.
    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/reconcile", reconcile_body()))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["matched_count"], 2);
    assert_eq!(json["missing_count"], 1);
    assert_eq!(json["extra_count"], 2);
    assert_eq!(json["duplicate_count"], 1);

    // Batch metadata updated.
    let batch = t
        .batches
        .batches
        .lock()
        .unwrap()
        .iter()
        .find(|b| b.id.as_i64() == 1)
        .cloned()
        .expect("batch");
    assert_eq!(batch.status, BatchStatus::Reconciled);
    assert!(batch.account_id.is_some());

    // GET results: 3 groups.
    let response = t
        .app()
        .oneshot(
            Request::get("/api/imports/1/results")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;

    let matched = json["matched"].as_array().expect("matched");
    assert_eq!(matched.len(), 2);
    let pair_movements: Vec<i64> = matched
        .iter()
        .map(|p| p["movement"]["id"].as_i64().expect("id"))
        .collect();
    assert!(pair_movements.contains(&1) && pair_movements.contains(&2));

    let missing = json["missing"].as_array().expect("missing");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0]["id"], 4);

    let extra = json["extra"].as_array().expect("extra");
    assert_eq!(extra.len(), 2);
    let dup: Vec<&serde_json::Value> = extra.iter().filter(|t| t["is_duplicate"] == true).collect();
    assert_eq!(dup.len(), 1);
    assert_eq!(dup[0]["ff3_id"], 103);
}

#[tokio::test]
async fn reconcile_rerun_updates_results() {
    let t = test_app(default_txs());

    // First run: m4 missing.
    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/reconcile", reconcile_body()))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);

    // FF3 now has the missing movement's transaction (RF-13).
    t.firefly_transactions
        .lock()
        .unwrap()
        .push(Ff3TransactionDto {
            ff3_id: 105,
            date: d(2026, 1, 8),
            amount_cents: AmountCents::new(777),
            description: "tx105".into(),
        });

    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/reconcile", reconcile_body()))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["matched_count"], 3);
    assert_eq!(json["missing_count"], 0);
    assert_eq!(t.store.run_count(), 2);

    // Results reflect the latest run.
    let response = t
        .app()
        .oneshot(
            Request::get("/api/imports/1/results")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    let json = read_json(response).await;
    assert_eq!(json["matched"].as_array().expect("matched").len(), 3);
}

#[tokio::test]
async fn reconcile_unknown_batch_is_404() {
    let t = test_app(default_txs());
    let response = t
        .app()
        .oneshot(post_json("/api/imports/999/reconcile", reconcile_body()))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reconcile_currency_mismatch_is_400() {
    let t = test_app(default_txs());
    t.batches.batches.lock().unwrap()[0].currency = "USD".into();
    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/reconcile", reconcile_body()))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn results_before_reconcile_is_404() {
    let t = test_app(default_txs());
    let response = t
        .app()
        .oneshot(
            Request::get("/api/imports/1/results")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ignore_endpoint_marks_movement_ignored() {
    let t = test_app(default_txs());

    // Reconcile first so the movement has a status.
    let _ = t
        .app()
        .oneshot(post_json("/api/imports/1/reconcile", reconcile_body()))
        .await
        .expect("reconcile");

    let response = t
        .app()
        .oneshot(
            Request::post("/api/imports/1/movements/4/ignore")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["status"], "ignored");

    let rows = t.movements.rows.lock().unwrap().clone();
    assert_eq!(
        rows.iter().find(|m| m.id.as_i64() == 4).expect("m4").status,
        MovementStatus::Ignored
    );
}

#[tokio::test]
async fn ignore_unknown_movement_is_404() {
    let t = test_app(default_txs());
    let response = t
        .app()
        .oneshot(
            Request::post("/api/imports/1/movements/999/ignore")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
