use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewCreation, NewFf3Account, NewFf3TransactionCreation,
    NewMatch, NewReconciliation, ReconciliationRun,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, CreationRepository, Ff3AccountRepository, Ff3TransactionRepository,
    FireflyClient, MatchRepository, MovementRepository, ReconciliationRepository,
};
use ffm_application::use_cases::apply_missing::{ApplyMissing, ApplyMissingInput, ApplyOutcome};

use ffm_domain::entities::{
    Batch, Creation, Ff3Account, Ff3Transaction, Match, Movement, Reconciliation,
};
use ffm_domain::ids::{
    BatchId, CreationId, Ff3AccountId, Ff3TransactionId, MatchId, MovementId, ReconciliationId,
};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};

// ── Fakes ───────────────────────────────────────────────────────────────

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Firefly fake that records every `create_transaction` call and can be
/// told to fail for specific amounts (per-movement failure, TRD §4).
struct FakeFirefly {
    accounts: Vec<Ff3AccountDto>,
    created: Mutex<Vec<NewFf3TransactionCreation>>,
    next_id: Mutex<i64>,
    fail_amounts: Vec<i64>,
}

impl FakeFirefly {
    fn calls(&self) -> Vec<NewFf3TransactionCreation> {
        self.created.lock().unwrap().clone()
    }
}

#[async_trait]
impl FireflyClient for FakeFirefly {
    async fn about(&self) -> Result<About, ApplicationError> {
        Ok(About {
            version: "6.1.36".into(),
        })
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        Ok(self.accounts.clone())
    }
    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: NaiveDate,
        _end: NaiveDate,
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
        self.created.lock().unwrap().push(creation);
        Ok(500 + *id)
    }
    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        unimplemented!("F5.3")
    }
}

struct FakeBatchRepo {
    batches: Vec<Batch>,
}

#[async_trait]
impl BatchRepository for FakeBatchRepo {
    async fn insert(
        &self,
        _batch: ffm_application::dtos::NewBatch,
    ) -> Result<BatchId, ApplicationError> {
        unimplemented!()
    }
    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError> {
        self.batches
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
        Ok(())
    }
    async fn update_account(
        &self,
        _id: BatchId,
        _account_id: Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}

struct FakeMovementRepo {
    movements: Mutex<Vec<Movement>>,
}

impl FakeMovementRepo {
    fn status_of(&self, id: i64) -> MovementStatus {
        self.movements
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
    async fn insert_batch(
        &self,
        _movements: Vec<ffm_application::dtos::NewMovement>,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError> {
        Ok(self
            .movements
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
        if let Some(m) = self
            .movements
            .lock()
            .unwrap()
            .iter_mut()
            .find(|m| m.id == id)
        {
            m.status = status;
        }
        Ok(())
    }
}

#[derive(Default)]
struct FakeCreationRepo {
    rows: Mutex<Vec<Creation>>,
    next_id: Mutex<i64>,
}

#[async_trait]
impl CreationRepository for FakeCreationRepo {
    async fn insert(&self, creation: NewCreation) -> Result<CreationId, ApplicationError> {
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

/// Shared store for the run-related ports, with a seeded previous run
/// (mirrors what a real reconcile left behind) and a recording `save_run`.
#[derive(Default)]
struct FakeRunStore {
    reconciliations: Mutex<Vec<Reconciliation>>,
    tx_rows: Mutex<Vec<Ff3Transaction>>,
    match_rows: Mutex<Vec<Match>>,
    saved_runs: Mutex<Vec<ReconciliationRun>>,
    next_tx_id: Mutex<i64>,
    next_match_id: Mutex<i64>,
    movements: Option<Arc<FakeMovementRepo>>,
}

impl FakeRunStore {
    fn seeded(movements: Arc<FakeMovementRepo>) -> Self {
        let store = Self {
            movements: Some(movements),
            ..Self::default()
        };
        // Previous run (id 10): m3 matched tx 30, one extra tx 31.
        store.reconciliations.lock().unwrap().push(Reconciliation {
            id: ReconciliationId::new(10),
            batch_id: BatchId::new(1),
            tolerance_days: 0,
            window_start: d(2026, 1, 1),
            window_end: d(2026, 1, 31),
            matched_count: 1,
            missing_count: 2,
            extra_count: 1,
            ran_at: epoch(),
        });
        store.tx_rows.lock().unwrap().extend([
            Ff3Transaction {
                id: Ff3TransactionId::new(30),
                reconciliation_id: ReconciliationId::new(10),
                ff3_id: 300,
                date: d(2026, 1, 3),
                amount_cents: AmountCents::new(900),
                description: "tx300".into(),
                is_duplicate: false,
                created_at: epoch(),
            },
            Ff3Transaction {
                id: Ff3TransactionId::new(31),
                reconciliation_id: ReconciliationId::new(10),
                ff3_id: 310,
                date: d(2026, 1, 25),
                amount_cents: AmountCents::new(111),
                description: "tx310".into(),
                is_duplicate: false,
                created_at: epoch(),
            },
        ]);
        store.match_rows.lock().unwrap().push(Match {
            id: MatchId::new(1),
            reconciliation_id: ReconciliationId::new(10),
            movement_id: MovementId::new(3),
            ff3_transaction_id: Ff3TransactionId::new(30),
            matched_at: epoch(),
        });
        store
    }

    fn saved_runs(&self) -> Vec<ReconciliationRun> {
        self.saved_runs.lock().unwrap().clone()
    }
}

#[async_trait]
impl ReconciliationRepository for FakeRunStore {
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
            .reconciliations
            .lock()
            .unwrap()
            .iter()
            .rfind(|r| r.batch_id == batch_id)
            .cloned())
    }
    async fn save_run(&self, run: ReconciliationRun) -> Result<ReconciliationId, ApplicationError> {
        let recon_id = ReconciliationId::new(20 + self.saved_runs.lock().unwrap().len() as i64);

        let mut ff3_local = std::collections::HashMap::new();
        {
            let mut next_tx = self.next_tx_id.lock().unwrap();
            for t in &run.ff3_transactions {
                *next_tx += 1;
                ff3_local.insert(t.ff3_id, *next_tx);
                self.tx_rows.lock().unwrap().push(Ff3Transaction {
                    id: Ff3TransactionId::new(*next_tx),
                    reconciliation_id: recon_id,
                    ff3_id: t.ff3_id,
                    date: t.date,
                    amount_cents: t.amount_cents,
                    description: t.description.clone(),
                    is_duplicate: t.is_duplicate,
                    created_at: epoch(),
                });
            }
        }
        {
            let mut next_match = self.next_match_id.lock().unwrap();
            for m in &run.matches {
                *next_match += 1;
                self.match_rows.lock().unwrap().push(Match {
                    id: MatchId::new(*next_match),
                    reconciliation_id: recon_id,
                    movement_id: m.movement_id,
                    ff3_transaction_id: Ff3TransactionId::new(ff3_local[&m.ff3_id]),
                    matched_at: epoch(),
                });
            }
        }
        self.reconciliations.lock().unwrap().push(Reconciliation {
            id: recon_id,
            batch_id: run.reconciliation.batch_id,
            tolerance_days: run.reconciliation.tolerance_days,
            window_start: run.reconciliation.window_start,
            window_end: run.reconciliation.window_end,
            matched_count: run.reconciliation.matched_count,
            missing_count: run.reconciliation.missing_count,
            extra_count: run.reconciliation.extra_count,
            ran_at: epoch(),
        });
        self.saved_runs.lock().unwrap().push(run.clone());

        if let Some(movements) = &self.movements {
            for (movement_id, status) in &run.movement_status_updates {
                movements.update_status(*movement_id, *status).await?;
            }
        }
        Ok(recon_id)
    }
}

#[async_trait]
impl Ff3TransactionRepository for FakeRunStore {
    async fn insert_batch(
        &self,
        _transactions: Vec<ffm_application::dtos::NewFf3Transaction>,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        Ok(self
            .tx_rows
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
            .tx_rows
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

#[async_trait]
impl MatchRepository for FakeRunStore {
    async fn insert_batch(&self, _matches: Vec<NewMatch>) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Match>, ApplicationError> {
        Ok(self
            .match_rows
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

fn movement(id: i64, cents: i64, status: MovementStatus) -> Movement {
    Movement {
        id: MovementId::new(id),
        batch_id: BatchId::new(1),
        line_no: id as i32,
        date: d(2026, 1, id.clamp(1, 28) as u32),
        amount_cents: AmountCents::new(cents),
        description: format!("m{id}"),
        raw_json: "{}".into(),
        status,
        created_at: epoch(),
    }
}

struct Fx {
    firefly: Arc<FakeFirefly>,
    movements: Arc<FakeMovementRepo>,
    creations: Arc<FakeCreationRepo>,
    runs: Arc<FakeRunStore>,
    batches: Arc<FakeBatchRepo>,
}

fn fx(fail_amounts: Vec<i64>) -> Fx {
    let movements = Arc::new(FakeMovementRepo {
        movements: Mutex::new(vec![
            movement(1, -5000, MovementStatus::Missing), // debit
            movement(2, 3000, MovementStatus::Missing),  // credit
            movement(3, 900, MovementStatus::Matched),   // already matched (prev run)
            movement(4, 111, MovementStatus::Ignored),
        ]),
    });
    Fx {
        firefly: Arc::new(FakeFirefly {
            accounts: vec![],
            created: Mutex::new(vec![]),
            next_id: Mutex::new(0),
            fail_amounts,
        }),
        movements: movements.clone(),
        creations: Arc::new(FakeCreationRepo::default()),
        runs: Arc::new(FakeRunStore::seeded(movements)),
        batches: Arc::new(FakeBatchRepo {
            batches: vec![Batch {
                id: BatchId::new(1),
                filename: "stmt.csv".into(),
                file_hash: "h".into(),
                format: "csv".into(),
                currency: "EUR".into(),
                account_id: Some(Ff3AccountId::new(1)),
                status: BatchStatus::Reconciled,
                created_at: epoch(),
            }],
        }),
    }
}

struct StaticAccounts;

#[async_trait]
impl Ff3AccountRepository for StaticAccounts {
    async fn upsert_batch(&self, _accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_ff3_id(&self, _ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        unimplemented!()
    }
    async fn by_id(&self, id: Ff3AccountId) -> Result<Option<Ff3Account>, ApplicationError> {
        if id == Ff3AccountId::new(1) {
            Ok(Some(Ff3Account {
                id,
                ff3_id: 7,
                name: "Checking".into(),
                account_type: "asset".into(),
                currency: "EUR".into(),
                synced_at: epoch(),
            }))
        } else {
            Ok(None)
        }
    }
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        unimplemented!()
    }
}

type ApplyUc = ApplyMissing<
    Arc<dyn FireflyClient>,
    Arc<dyn Ff3AccountRepository>,
    Arc<FakeBatchRepo>,
    Arc<FakeMovementRepo>,
    Arc<FakeCreationRepo>,
    Arc<FakeRunStore>,
    Arc<FakeRunStore>,
    Arc<FakeRunStore>,
>;

impl Fx {
    fn uc(&self) -> ApplyUc {
        ApplyMissing::new(
            self.firefly.clone() as Arc<dyn FireflyClient>,
            Arc::new(StaticAccounts) as Arc<dyn Ff3AccountRepository>,
            self.batches.clone(),
            self.movements.clone(),
            self.creations.clone(),
            ffm_application::use_cases::apply_missing::RunPorts {
                reconciliations: self.runs.clone(),
                matches: self.runs.clone(),
                ff3_transactions: self.runs.clone(),
            },
        )
    }

    fn input(&self, ids: &[i64]) -> ApplyMissingInput {
        ApplyMissingInput {
            batch_id: BatchId::new(1),
            movement_ids: ids.iter().map(|i| MovementId::new(*i)).collect(),
            counterparty_ff3_id: 42,
            category: Some("Groceries".into()),
            budget: None,
            notes: None,
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn apply_creates_missing_with_origin_tag_and_confirms_matched() {
    let f = fx(vec![]);
    let result = f.uc().execute(f.input(&[1, 2])).await.expect("apply ok");

    // Per-movement report.
    assert_eq!(result.reports.len(), 2);
    assert!(result
        .reports
        .iter()
        .all(|r| matches!(r.outcome, ApplyOutcome::Created)));
    assert_eq!(result.created_count, 2);
    assert_eq!(result.already_created_count, 0);
    assert_eq!(result.failed_count, 0);

    // FF3 calls: signs → withdrawal (source=batch account 7, dest=42) for
    // the debit; deposit (source=42, dest=7) for the credit. Origin tag
    // filename + batch id (RF-11). Category carried over.
    let calls = f.firefly.calls();
    assert_eq!(calls.len(), 2);
    let debit = calls
        .iter()
        .find(|c| c.amount.as_i64() == -5000)
        .expect("debit call");
    assert_eq!(debit.transaction_type, "withdrawal");
    assert_eq!(debit.source_ff3_id, 7);
    assert_eq!(debit.destination_ff3_id, 42);
    assert_eq!(debit.category.as_deref(), Some("Groceries"));
    let credit = calls
        .iter()
        .find(|c| c.amount.as_i64() == 3000)
        .expect("credit call");
    assert_eq!(credit.transaction_type, "deposit");
    assert_eq!(credit.source_ff3_id, 42);
    assert_eq!(credit.destination_ff3_id, 7);
    for c in &calls {
        assert_eq!(c.tags, vec!["ffm:stmt.csv:1".to_string()]);
    }

    // Creations recorded (idempotency rows).
    assert_eq!(f.creations.rows.lock().unwrap().len(), 2);

    // Incremental confirmation (D7): cumulative mini-run saved, statuses
    // created → matched.
    assert_eq!(f.runs.saved_runs().len(), 1);
    let run = &f.runs.saved_runs()[0];
    assert_eq!(run.reconciliation.matched_count, 3); // m3 + m1 + m2
    assert_eq!(run.reconciliation.missing_count, 0);
    assert_eq!(run.reconciliation.extra_count, 1); // carried over
                                                   // Matches: prev (m3↔300) + the two new ones.
    assert_eq!(run.matches.len(), 3);
    let pairs: Vec<(i64, i64)> = run
        .matches
        .iter()
        .map(|m| (m.movement_id.as_i64(), m.ff3_id))
        .collect();
    assert!(pairs.contains(&(3, 300)));
    assert!(pairs.contains(&(1, 501)) || pairs.contains(&(1, 502)));
    assert!(pairs.contains(&(2, 501)) || pairs.contains(&(2, 502)));
    // Tx cache: prev 2 + new 2.
    assert_eq!(run.ff3_transactions.len(), 4);

    assert_eq!(f.movements.status_of(1), MovementStatus::Matched);
    assert_eq!(f.movements.status_of(2), MovementStatus::Matched);
}

#[tokio::test]
async fn apply_retry_creates_nothing_nfr05() {
    let f = fx(vec![]);
    f.uc().execute(f.input(&[1, 2])).await.expect("first apply");
    let calls_after_first = f.firefly.calls().len();
    assert_eq!(calls_after_first, 2);

    let result = f.uc().execute(f.input(&[1, 2])).await.expect("retry ok");
    assert_eq!(result.created_count, 0);
    assert_eq!(result.already_created_count, 2);
    assert_eq!(result.failed_count, 0);
    // 0 double creations - same FF3 call count (NFR-05).
    assert_eq!(f.firefly.calls().len(), 2);
    // No extra run persisted for a pure no-op retry.
    assert_eq!(f.runs.saved_runs().len(), 1);
}

#[tokio::test]
async fn apply_partial_failure_continues_batch() {
    let f = fx(vec![3000]); // the credit (+3000) fails in FF3
    let result = f.uc().execute(f.input(&[1, 2])).await.expect("apply ok");

    assert_eq!(result.created_count, 1);
    assert_eq!(result.failed_count, 1);
    let failed = result
        .reports
        .iter()
        .find(|r| matches!(r.outcome, ApplyOutcome::Failed))
        .expect("failed report");
    assert_eq!(failed.movement_id.as_i64(), 2);
    assert!(failed.reason.as_deref().unwrap_or("").contains("FF3"));

    // Batch continued: m1 created; m2 stays missing; no creation row for m2.
    assert_eq!(f.creations.rows.lock().unwrap().len(), 1);
    assert_eq!(f.movements.status_of(1), MovementStatus::Matched);
    assert_eq!(f.movements.status_of(2), MovementStatus::Missing);

    // Mini-run covers only the success: matched 2 (m3+m1), missing 1.
    let run = &f.runs.saved_runs()[0];
    assert_eq!(run.reconciliation.matched_count, 2);
    assert_eq!(run.reconciliation.missing_count, 1);
    assert_eq!(run.matches.len(), 2);
}

#[tokio::test]
async fn apply_requires_reconciled_batch() {
    let f = fx(vec![]);
    // No previous run → nothing to confirm incrementally.
    f.runs.reconciliations.lock().unwrap().clear();
    let err = f.uc().execute(f.input(&[1])).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::InvalidState(_)));

    // Batch without account (never reconciled against one).
    let f2 = fx(vec![]);
    {
        let mut batches = f2.batches.batches.clone();
        batches[0].account_id = None;
        f2.runs.reconciliations.lock().unwrap().clear();
    }
    let err = f2
        .uc()
        .execute(f2.input(&[1]))
        .await
        .expect_err("must fail");
    assert!(matches!(err, ApplicationError::InvalidState(_)));
}

#[tokio::test]
async fn apply_unknown_batch_or_movement_is_not_found() {
    let f = fx(vec![]);
    let mut input = f.input(&[1]);
    input.batch_id = BatchId::new(999);
    let err = f.uc().execute(input).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));

    let err = f
        .uc()
        .execute(f.input(&[777]))
        .await
        .expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn apply_rejects_movement_not_in_missing_state() {
    let f = fx(vec![]);
    // m3 is Matched (no creation row) → hard reject.
    let err = f.uc().execute(f.input(&[3])).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::InvalidState(_)));
    assert_eq!(f.firefly.calls().len(), 0);
}
