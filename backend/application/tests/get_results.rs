use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use ffm_application::dtos::{NewFf3Transaction, NewMatch, NewReconciliation, ReconciliationRun};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    DeletedAuditRepository, Ff3TransactionRepository, MatchRepository, MovementRepository,
    ReconciliationRepository,
};
use ffm_application::use_cases::get_results::GetResults;

use ffm_domain::entities::{Ff3Transaction, Match, Movement, Reconciliation};
use ffm_domain::ids::{
    BatchId, DeletedAuditId, Ff3TransactionId, MatchId, MovementId, ReconciliationId,
};
use ffm_domain::value_objects::{AmountCents, MovementStatus};

// ── Fakes ───────────────────────────────────────────────────────────────

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

struct FakeReconciliationRepo {
    by_batch: Mutex<Vec<Reconciliation>>,
}

#[async_trait]
impl ReconciliationRepository for FakeReconciliationRepo {
    async fn insert(
        &self,
        _reconciliation: NewReconciliation,
    ) -> Result<ReconciliationId, ApplicationError> {
        unimplemented!()
    }
    async fn latest_by_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<Option<Reconciliation>, ApplicationError> {
        Ok(self
            .by_batch
            .lock()
            .unwrap()
            .iter()
            .rfind(|r| r.batch_id == batch_id)
            .cloned())
    }
    async fn save_run(
        &self,
        _run: ReconciliationRun,
    ) -> Result<ReconciliationId, ApplicationError> {
        unimplemented!()
    }
}

struct FakeFf3TxRepo {
    rows: Mutex<Vec<Ff3Transaction>>,
}

#[async_trait]
impl Ff3TransactionRepository for FakeFf3TxRepo {
    async fn insert_batch(
        &self,
        _transactions: Vec<NewFf3Transaction>,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.reconciliation_id == reconciliation_id)
            .cloned()
            .collect())
    }
    async fn latest_by_ff3_id(
        &self,
        ff3_id: i64,
    ) -> Result<Option<Ff3Transaction>, ApplicationError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .rfind(|t| t.ff3_id == ff3_id)
            .cloned())
    }
    async fn latest_by_ff3_ids(
        &self,
        _ff3_ids: &[i64],
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        unimplemented!()
    }
}

struct FakeMatchRepo {
    rows: Mutex<Vec<Match>>,
}

#[async_trait]
impl MatchRepository for FakeMatchRepo {
    async fn insert_batch(&self, _matches: Vec<NewMatch>) -> Result<(), ApplicationError> {
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

struct FakeMovementRepo {
    rows: Mutex<Vec<Movement>>,
}

#[async_trait]
impl MovementRepository for FakeMovementRepo {
    async fn insert_batch(
        &self,
        _movements: Vec<ffm_application::dtos::NewMovement>,
    ) -> Result<(), ApplicationError> {
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

struct FakeDeletedAuditRepo {
    ff3_ids: Mutex<Vec<i64>>,
}

#[async_trait]
impl DeletedAuditRepository for FakeDeletedAuditRepo {
    async fn insert(
        &self,
        _audit: ffm_application::dtos::NewDeletedAudit,
    ) -> Result<DeletedAuditId, ApplicationError> {
        unimplemented!()
    }
    async fn deleted_ff3_ids(&self) -> Result<std::collections::HashSet<i64>, ApplicationError> {
        Ok(self.ff3_ids.lock().unwrap().iter().copied().collect())
    }
}

// ── Fixture ─────────────────────────────────────────────────────────────

struct Fx {
    rec: FakeReconciliationRepo,
    txs: FakeFf3TxRepo,
    matches: FakeMatchRepo,
    movements: FakeMovementRepo,
    audits: FakeDeletedAuditRepo,
}

fn movement(id: i64, status: MovementStatus) -> Movement {
    Movement {
        id: MovementId::new(id),
        batch_id: BatchId::new(1),
        line_no: id as i32,
        date: d(2026, 1, 10),
        amount_cents: AmountCents::new(1000 * id),
        description: format!("m{id}"),
        raw_json: "{}".into(),
        status,
        created_at: epoch(),
    }
}

fn ff3_tx(local_id: i64, ff3_id: i64, dup: bool) -> Ff3Transaction {
    Ff3Transaction {
        id: Ff3TransactionId::new(local_id),
        reconciliation_id: ReconciliationId::new(10),
        ff3_id,
        date: d(2026, 1, 10),
        amount_cents: AmountCents::new(1000 * ff3_id),
        description: format!("tx{ff3_id}"),
        is_duplicate: dup,
        created_at: epoch(),
    }
}

fn run_fixture() -> Fx {
    let rec = FakeReconciliationRepo {
        by_batch: Mutex::new(vec![Reconciliation {
            id: ReconciliationId::new(10),
            batch_id: BatchId::new(1),
            tolerance_days: 0,
            window_start: d(2026, 1, 1),
            window_end: d(2026, 1, 31),
            matched_count: 1,
            missing_count: 1,
            extra_count: 2,
            ran_at: epoch(),
        }]),
    };
    let txs = FakeFf3TxRepo {
        rows: Mutex::new(vec![
            ff3_tx(1, 101, false), // matched (in a match row)
            ff3_tx(2, 102, false), // extra (unmatched)
            ff3_tx(3, 103, true),  // duplicate (unmatched, flagged)
        ]),
    };
    let matches = FakeMatchRepo {
        rows: Mutex::new(vec![Match {
            id: MatchId::new(1),
            reconciliation_id: ReconciliationId::new(10),
            movement_id: MovementId::new(1),
            ff3_transaction_id: Ff3TransactionId::new(1),
            matched_at: epoch(),
        }]),
    };
    let movements = FakeMovementRepo {
        rows: Mutex::new(vec![
            movement(1, MovementStatus::Matched),
            movement(2, MovementStatus::Missing),
            movement(3, MovementStatus::Ignored),
        ]),
    };
    Fx {
        rec,
        txs,
        matches,
        movements,
        audits: FakeDeletedAuditRepo {
            ff3_ids: Mutex::new(vec![]),
        },
    }
}

impl Fx {
    fn uc(
        self,
    ) -> GetResults<
        FakeReconciliationRepo,
        FakeMatchRepo,
        FakeFf3TxRepo,
        FakeMovementRepo,
        FakeDeletedAuditRepo,
    > {
        GetResults::new(
            self.rec,
            self.matches,
            self.txs,
            self.movements,
            self.audits,
        )
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn results_returns_three_groups() {
    let r = run_fixture()
        .uc()
        .execute(BatchId::new(1))
        .await
        .expect("results ok");

    assert_eq!(r.reconciliation_id, ReconciliationId::new(10));

    // Matched: movement 1 ↔ ff3 tx 101.
    assert_eq!(r.matched.len(), 1);
    assert_eq!(r.matched[0].movement.id, MovementId::new(1));
    assert_eq!(r.matched[0].ff3.ff3_id, 101);

    // Missing: only status == Missing (movement 2).
    assert_eq!(r.missing.len(), 1);
    assert_eq!(r.missing[0].id, MovementId::new(2));

    // Extra: unmatched ff3 txs = 102 (plain) + 103 (duplicate flag).
    assert_eq!(r.extra.len(), 2);
    let extra_ids: Vec<i64> = r.extra.iter().map(|t| t.ff3_id).collect();
    assert_eq!(extra_ids, vec![102, 103]);
    let dup = r.extra.iter().find(|t| t.ff3_id == 103).expect("103");
    assert!(dup.is_duplicate);
    let plain = r.extra.iter().find(|t| t.ff3_id == 102).expect("102");
    assert!(!plain.is_duplicate);
}

#[tokio::test]
async fn results_without_reconciliation_is_not_found() {
    let f = run_fixture();
    f.rec.by_batch.lock().unwrap().clear();
    let err = f
        .uc()
        .execute(BatchId::new(1))
        .await
        .expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn results_include_ignored_movements() {
    let r = run_fixture()
        .uc()
        .execute(BatchId::new(1))
        .await
        .expect("results ok");

    assert_eq!(r.ignored.len(), 1);
    assert_eq!(r.ignored[0].id, MovementId::new(3));
    assert_eq!(r.ignored[0].status, MovementStatus::Ignored);
}

#[tokio::test]
async fn results_exclude_deleted_ff3_transactions_from_extra() {
    let f = run_fixture();
    // Duplicate 103 was deleted in FF3 after the run (audit exists, RF-15).
    f.audits.ff3_ids.lock().unwrap().push(103);

    let r = f.uc().execute(BatchId::new(1)).await.expect("results ok");

    let extra_ids: Vec<i64> = r.extra.iter().map(|t| t.ff3_id).collect();
    assert_eq!(extra_ids, vec![102]);
}
