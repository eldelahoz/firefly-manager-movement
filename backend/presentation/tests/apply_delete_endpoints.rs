//! E2E tests for apply + delete (F5).

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
    BatchRepository, CreationRepository, Ff3AccountRepository, FireflyClient, MovementRepository,
};
use ffm_domain::entities::{Batch, Creation, Ff3Account, Movement};
use ffm_domain::ids::{BatchId, CreationId, MovementId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};
use ffm_presentation::{app, AppState};
use tower::ServiceExt;

mod common;
use common::{
    FakeCsvDecoder, FakeCsvMappingRepository, FakeDeletedAuditRepository, FakeReconciliationStore,
};

// ── Fakes ───────────────────────────────────────────────────────────────

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Firefly fake: fixed account list, records creations, can delete,
/// optionally fails creations of specific amounts.
struct FakeFirefly {
    created: Mutex<Vec<NewFf3TransactionCreation>>,
    deleted: Mutex<Vec<i64>>,
    fail_amounts: Vec<i64>,
    next_id: Mutex<i64>,
}

#[async_trait]
impl FireflyClient for FakeFirefly {
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
        _start: Option<NaiveDate>,
        _end: Option<NaiveDate>,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        Ok(vec![])
    }
    async fn create_transaction(
        &self,
        creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        if self.fail_amounts.contains(&creation.amount.as_i64()) {
            return Err(ApplicationError::FireflyClient("FF3 500".into()));
        }
        let mut id = self.next_id.lock().unwrap();
        *id += 1;
        let ff3_id = 800 + *id;
        self.created.lock().unwrap().push(creation);
        Ok(ff3_id)
    }
    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError> {
        self.deleted.lock().unwrap().push(ff3_id);
        Ok(())
    }
}

struct FakeAccountRepo {
    accounts: Mutex<Vec<Ff3Account>>,
}

#[async_trait]
impl Ff3AccountRepository for FakeAccountRepo {
    async fn upsert_batch(&self, _accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        unimplemented!("seeded")
    }
    async fn by_ff3_id(&self, _ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        unimplemented!()
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
        unimplemented!()
    }
}

struct FakeBatchRepo {
    batches: Mutex<Vec<Batch>>,
}

#[async_trait]
impl BatchRepository for FakeBatchRepo {
    async fn insert(&self, _batch: NewBatch) -> Result<BatchId, ApplicationError> {
        unimplemented!()
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
        unimplemented!()
    }
    async fn update_status(
        &self,
        _id: BatchId,
        _status: BatchStatus,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
    async fn update_account(
        &self,
        _id: BatchId,
        _account_id: ffm_domain::ids::Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}

struct FakeMovementRepo {
    rows: Mutex<Vec<Movement>>,
}

impl FakeMovementRepo {
    fn status_of(&self, id: i64) -> MovementStatus {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.id.as_i64() == id)
            .expect("movement")
            .status
    }
}

#[async_trait]
impl MovementRepository for FakeMovementRepo {
    async fn insert_batch(&self, _movements: Vec<NewMovement>) -> Result<(), ApplicationError> {
        unimplemented!()
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

/// Stateful creations repo (idempotency rows).
#[derive(Default)]
struct FakeCreationRepo {
    rows: Mutex<Vec<Creation>>,
    next_id: Mutex<i64>,
}

#[async_trait]
impl CreationRepository for FakeCreationRepo {
    async fn insert(
        &self,
        creation: ffm_application::dtos::NewCreation,
    ) -> Result<CreationId, ApplicationError> {
        let mut id = self.next_id.lock().unwrap();
        *id += 1;
        self.rows.lock().unwrap().push(Creation {
            id: CreationId::new(*id),
            movement_id: creation.movement_id,
            ff3_id: creation.ff3_id,
            created_at: epoch(),
        });
        Ok(CreationId::new(*id))
    }
    async fn by_movement(
        &self,
        movement_id: MovementId,
    ) -> Result<Option<Creation>, ApplicationError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.movement_id == movement_id)
            .cloned())
    }
    async fn by_batch(&self, _batch_id: BatchId) -> Result<Vec<Creation>, ApplicationError> {
        unimplemented!()
    }
}

// ── Fixture ─────────────────────────────────────────────────────────────

fn movement(id: i64, cents: i64, status: MovementStatus) -> Movement {
    Movement {
        id: MovementId::new(id),
        batch_id: BatchId::new(1),
        line_no: id as i32,
        date: d(2026, 1, 10),
        amount_cents: AmountCents::new(cents),
        description: format!("m{id}"),
        raw_json: "{}".into(),
        status,
        created_at: epoch(),
    }
}

async fn seeded_run_store(movements: Arc<FakeMovementRepo>) -> Arc<FakeReconciliationStore> {
    // Seed a previous reconciliation via save_run: one cached extra tx
    // ff3_id 999 (duplicate), no matches, all movements missing.
    use ffm_application::ports::ReconciliationRepository;
    let store = Arc::new(FakeReconciliationStore::with_movements(
        movements.clone() as Arc<dyn MovementRepository>
    ));
    let run = ffm_application::dtos::ReconciliationRun {
        reconciliation: ffm_application::dtos::NewReconciliation {
            batch_id: BatchId::new(1),
            tolerance_days: 0,
            window_start: d(2026, 1, 1),
            window_end: d(2026, 1, 31),
            matched_count: 0,
            missing_count: 2,
            extra_count: 1,
        },
        ff3_transactions: vec![ffm_application::dtos::RunFf3Transaction {
            ff3_id: 999,
            date: d(2026, 1, 12),
            amount_cents: AmountCents::new(-4200),
            description: "dupe".into(),
            is_duplicate: true,
        }],
        matches: vec![],
        movement_status_updates: vec![
            (MovementId::new(1), MovementStatus::Missing),
            (MovementId::new(2), MovementStatus::Missing),
        ],
    };
    store.save_run(run).await.expect("seed run");
    store
}

struct TestApp {
    state: AppState,
    pub firefly: Arc<FakeFirefly>,
    pub movements: Arc<FakeMovementRepo>,
    pub audits: Arc<FakeDeletedAuditRepository>,
}

async fn test_app(fail_amounts: Vec<i64>) -> TestApp {
    let firefly = Arc::new(FakeFirefly {
        created: Mutex::new(vec![]),
        deleted: Mutex::new(vec![]),
        fail_amounts,
        next_id: Mutex::new(0),
    });
    let movements = Arc::new(FakeMovementRepo {
        rows: Mutex::new(vec![
            movement(1, -5000, MovementStatus::Missing),
            movement(2, 3000, MovementStatus::Missing),
        ]),
    });
    let batches = Arc::new(FakeBatchRepo {
        batches: Mutex::new(vec![Batch {
            id: BatchId::new(1),
            filename: "stmt.csv".into(),
            file_hash: "h".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: Some(ffm_domain::ids::Ff3AccountId::new(1)),
            status: BatchStatus::Reconciled,
            headers: vec![],
            detected_delimiter: ",".into(),
            created_at: epoch(),
        }]),
    });
    let creations = Arc::new(FakeCreationRepo::default());
    let audits = Arc::new(FakeDeletedAuditRepository::default());
    let store = seeded_run_store(movements.clone()).await;

    let state = AppState {
        firefly: firefly.clone() as Arc<dyn FireflyClient>,
        ff3_accounts: Arc::new(FakeAccountRepo {
            accounts: Mutex::new(vec![Ff3Account {
                id: ffm_domain::ids::Ff3AccountId::new(1),
                ff3_id: 7,
                name: "Checking".into(),
                account_type: "asset".into(),
                currency: "EUR".into(),
                iban: None,
                account_number: None,
                synced_at: epoch(),
            }]),
        }),
        csv_decoder: Arc::new(FakeCsvDecoder),
        batches,
        movements: movements.clone(),
        csv_mappings: Arc::new(FakeCsvMappingRepository),
        reconciliations: store.clone(),
        matches: store.clone(),
        ff3_transactions: store.clone(),
        creations: creations.clone(),
        deleted_audits: audits.clone(),
        ff3_health_probe: false,
        app_settings: Arc::new(common::FakeAppSettingsRepository::default()),
        env_defaults: ffm_application::use_cases::app_settings::EnvDefaults::default(),
        firefly_manager: Arc::new(common::FakeFireflyConnectionManager::default()),
        firefly_factory: Arc::new(common::FakeFireflyClientFactory::new(
            common::FakeFireflyClient::ok(),
        )),
    };

    TestApp {
        state,
        firefly,
        movements,
        audits,
    }
}

impl TestApp {
    fn app(&self) -> axum::Router {
        app(self.state.clone())
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
async fn apply_endpoint_creates_and_reports() {
    let t = test_app(vec![]).await;
    let body = json!({
        "movements": [
            { "movement_id": 1, "counterparty_ff3_id": 42, "category": "Groceries" },
            { "movement_id": 2, "counterparty_ff3_id": 42, "category": "Groceries" }
        ]
    });

    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/apply", body))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["created_count"], 2);
    assert_eq!(json["failed_count"], 0);
    assert!(json["reconciliation_id"].is_u64());

    // FF3 called with origin note + directions.
    let calls = t.firefly.created.lock().unwrap().clone();
    assert_eq!(calls.len(), 2);
    for c in &calls {
        assert_eq!(c.tags, Vec::<String>::new()); // No tags (origin trace is note)
        assert_eq!(c.notes.as_deref(), Some("ffm:stmt.csv:1"));
    }

    // Movements confirmed matched via the incremental run.
    assert_eq!(t.movements.status_of(1), MovementStatus::Matched);
    assert_eq!(t.movements.status_of(2), MovementStatus::Matched);
}

#[tokio::test]
async fn apply_retry_endpoint_is_idempotent() {
    let t = test_app(vec![]).await;
    let body = json!({
        "movements": [
            { "movement_id": 1, "counterparty_ff3_id": 42 },
            { "movement_id": 2, "counterparty_ff3_id": 42 }
        ]
    });

    let r1 = t
        .app()
        .oneshot(post_json("/api/imports/1/apply", body.clone()))
        .await
        .expect("first");
    assert_eq!(r1.status(), StatusCode::OK);

    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/apply", body))
        .await
        .expect("retry");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["created_count"], 0);
    assert_eq!(json["already_created_count"], 2);
    // NFR-05: no double creations.
    assert_eq!(t.firefly.created.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn apply_partial_failure_continues_and_reports_reason() {
    let t = test_app(vec![3000]).await; // credit (+3000) fails
    let body = json!({
        "movements": [
            { "movement_id": 1, "counterparty_ff3_id": 42 },
            { "movement_id": 2, "counterparty_ff3_id": 42 }
        ]
    });

    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/apply", body))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["created_count"], 1);
    assert_eq!(json["failed_count"], 1);

    let failed = json["reports"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["outcome"] == "failed")
        .expect("failed report");
    assert_eq!(failed["movement_id"], 2);
    assert!(failed["reason"].as_str().unwrap().contains("FF3"));

    assert_eq!(t.movements.status_of(1), MovementStatus::Matched);
    assert_eq!(t.movements.status_of(2), MovementStatus::Missing);
}

#[tokio::test]
async fn apply_not_missing_is_400() {
    let t = test_app(vec![]).await;
    let body = json!({
        "movements": [
            { "movement_id": 999, "counterparty_ff3_id": 42 }
        ]
    });
    let response = t
        .app()
        .oneshot(post_json("/api/imports/1/apply", body))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_endpoint_snapshots_and_audits() {
    let t = test_app(vec![]).await;
    let response = t
        .app()
        .oneshot(
            Request::delete("/api/firefly/transactions/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["ff3_id"], 999);
    assert_eq!(json["status"], "deleted");

    // FF3 delete called + audit snapshot stored.
    assert_eq!(*t.firefly.deleted.lock().unwrap(), vec![999]);
    let rows = t.audits.rows.lock().unwrap();
    assert_eq!(rows.len(), 1);
    let snapshot: serde_json::Value = serde_json::from_str(&rows[0].snapshot_json).unwrap();
    assert_eq!(snapshot["ff3_id"], 999);
    assert_eq!(snapshot["is_duplicate"], true);
}

#[tokio::test]
async fn delete_unknown_tx_is_404() {
    let t = test_app(vec![]).await;
    let response = t
        .app()
        .oneshot(
            Request::delete("/api/firefly/transactions/424242")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(t.audits.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn apply_endpoint_creates_without_counterparty() {
    let t = test_app(vec![]).await;

    let response = t
        .app()
        .oneshot(post_json(
            "/api/imports/1/apply",
            serde_json::json!({
                "movements": [
                    { "movement_id": 1 },
                    { "movement_id": 2 }
                ]
            }),
        ))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["created_count"], 2);

    let created = t.firefly.created.lock().unwrap();
    let debit = created
        .iter()
        .find(|c| c.amount.as_i64() == -5000)
        .expect("debit");
    assert_eq!(debit.source_ff3_id, Some(7));
    assert_eq!(debit.destination_ff3_id, None);
    let credit = created
        .iter()
        .find(|c| c.amount.as_i64() == 3000)
        .expect("credit");
    assert_eq!(credit.source_ff3_id, None);
    assert_eq!(credit.destination_ff3_id, Some(7));
}
