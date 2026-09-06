use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};

use ffm_application::dtos::{NewBatch, NewMovement};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, MatchRepository, MovementRepository, ReconciliationRepository,
};
use ffm_application::use_cases::ignore_movement::IgnoreMovement;

use ffm_domain::entities::{Batch, Match, Movement, Reconciliation};
use ffm_domain::ids::{BatchId, Ff3TransactionId, MatchId, MovementId, ReconciliationId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};

// ── Fakes ───────────────────────────────────────────────────────────────

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
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
            .ok_or_else(|| ApplicationError::NotFound("batch not found".into()))
    }
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        unimplemented!()
    }
    async fn update_status(
        &self,
        _id: BatchId,
        _status: BatchStatus,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn update_account(
        &self,
        _id: BatchId,
        _account_id: ffm_domain::ids::Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}

struct FakeMovementRepo {
    rows: Mutex<Vec<Movement>>,
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

#[derive(Default)]
struct FakeReconciliationRepo {
    runs: Mutex<Vec<Reconciliation>>,
}

#[async_trait]
impl ReconciliationRepository for FakeReconciliationRepo {
    async fn insert(
        &self,
        _reconciliation: ffm_application::dtos::NewReconciliation,
    ) -> Result<ReconciliationId, ApplicationError> {
        unimplemented!()
    }
    async fn latest_by_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<Option<Reconciliation>, ApplicationError> {
        Ok(self
            .runs
            .lock()
            .unwrap()
            .iter()
            .rfind(|r| r.batch_id == batch_id)
            .cloned())
    }
    async fn save_run(
        &self,
        _run: ffm_application::dtos::ReconciliationRun,
    ) -> Result<ReconciliationId, ApplicationError> {
        unimplemented!()
    }
}

#[derive(Default)]
struct FakeMatchRepo {
    rows: Mutex<Vec<Match>>,
}

#[async_trait]
impl MatchRepository for FakeMatchRepo {
    async fn insert_batch(
        &self,
        _matches: Vec<ffm_application::dtos::NewMatch>,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Match>, ApplicationError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.reconciliation_id == reconciliation_id)
            .cloned()
            .collect())
    }
    async fn matched_ff3_ids(
        &self,
        _ff3_ids: &[i64],
    ) -> Result<std::collections::HashSet<i64>, ApplicationError> {
        unimplemented!()
    }
}

// ── Fixture ─────────────────────────────────────────────────────────────

struct Fx {
    batches: FakeBatchRepo,
    movements: Arc<FakeMovementRepo>,
    reconciliations: FakeReconciliationRepo,
    matches: FakeMatchRepo,
}

fn fx() -> Fx {
    Fx {
        batches: FakeBatchRepo {
            batches: Mutex::new(vec![Batch {
                id: BatchId::new(1),
                filename: "stmt.csv".into(),
                file_hash: "h".into(),
                format: "csv".into(),
                currency: "EUR".into(),
                account_id: None,
                status: BatchStatus::Reconciled,
                created_at: epoch(),
            }]),
        },
        movements: Arc::new(FakeMovementRepo {
            rows: Mutex::new(vec![Movement {
                id: MovementId::new(5),
                batch_id: BatchId::new(1),
                line_no: 5,
                date: chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                amount_cents: AmountCents::new(-100),
                description: "m5".into(),
                raw_json: "{}".into(),
                status: MovementStatus::Missing,
                created_at: epoch(),
            }]),
        }),
        reconciliations: FakeReconciliationRepo {
            runs: Mutex::new(vec![Reconciliation {
                id: ReconciliationId::new(10),
                batch_id: BatchId::new(1),
                tolerance_days: 0,
                window_start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                window_end: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                matched_count: 0,
                missing_count: 1,
                extra_count: 0,
                ran_at: epoch(),
            }]),
        },
        matches: FakeMatchRepo::default(),
    }
}

impl Fx {
    fn uc(
        self,
    ) -> IgnoreMovement<Arc<FakeMovementRepo>, FakeBatchRepo, FakeReconciliationRepo, FakeMatchRepo>
    {
        IgnoreMovement::new(
            self.movements.clone(),
            self.batches,
            self.reconciliations,
            self.matches,
        )
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn ignore_sets_movement_ignored() {
    let f = fx();
    let movements = f.movements.clone();
    f.uc()
        .execute(BatchId::new(1), MovementId::new(5), true)
        .await
        .expect("ignore ok");

    let rows = movements.rows.lock().unwrap().clone();
    assert_eq!(rows[0].status, MovementStatus::Ignored);
}

#[tokio::test]
async fn ignore_is_idempotent() {
    let f = fx();
    f.movements.rows.lock().unwrap()[0].status = MovementStatus::Ignored;
    f.uc()
        .execute(BatchId::new(1), MovementId::new(5), true)
        .await
        .expect("already ignored → ok");
}

#[tokio::test]
async fn ignore_unknown_batch_is_not_found() {
    let f = fx();
    let err = f
        .uc()
        .execute(BatchId::new(999), MovementId::new(5), true)
        .await
        .expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn ignore_movement_of_other_batch_is_not_found() {
    let f = fx();
    let err = f
        .uc()
        .execute(BatchId::new(1), MovementId::new(777), true)
        .await
        .expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn un_ignore_restores_missing_when_not_in_latest_run_matches() {
    let f = fx();
    f.movements.rows.lock().unwrap()[0].status = MovementStatus::Ignored;
    let movements = f.movements.clone();

    let status = f
        .uc()
        .execute(BatchId::new(1), MovementId::new(5), false)
        .await
        .expect("un-ignore ok");

    assert_eq!(status, MovementStatus::Missing);
    let rows = movements.rows.lock().unwrap().clone();
    assert_eq!(rows[0].status, MovementStatus::Missing);
}

#[tokio::test]
async fn un_ignore_restores_matched_when_in_latest_run_matches() {
    let f = fx();
    f.movements.rows.lock().unwrap()[0].status = MovementStatus::Ignored;
    f.matches.rows.lock().unwrap().push(Match {
        id: MatchId::new(1),
        reconciliation_id: ReconciliationId::new(10),
        movement_id: MovementId::new(5),
        ff3_transaction_id: Ff3TransactionId::new(1),
        matched_at: epoch(),
    });

    let status = f
        .uc()
        .execute(BatchId::new(1), MovementId::new(5), false)
        .await
        .expect("un-ignore ok");

    assert_eq!(status, MovementStatus::Matched);
}

#[tokio::test]
async fn un_ignore_without_any_run_restores_imported() {
    let f = fx();
    f.movements.rows.lock().unwrap()[0].status = MovementStatus::Ignored;
    f.reconciliations.runs.lock().unwrap().clear();

    let status = f
        .uc()
        .execute(BatchId::new(1), MovementId::new(5), false)
        .await
        .expect("un-ignore ok");

    assert_eq!(status, MovementStatus::Imported);
}
