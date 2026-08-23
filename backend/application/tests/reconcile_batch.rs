use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewBatch, NewFf3Account, NewFf3TransactionCreation,
    NewMovement, ReconciliationRun,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, Ff3AccountRepository, FireflyClient, MovementRepository,
    ReconciliationRepository,
};
use ffm_application::use_cases::reconcile_batch::{ReconcileBatch, ReconcileBatchInput};

use ffm_domain::entities::{Batch, Ff3Account, Movement, Reconciliation};
use ffm_domain::ids::{BatchId, Ff3AccountId, MovementId, ReconciliationId};
use ffm_domain::value_objects::{AmountCents, BatchStatus, MovementStatus};

// ── Fakes ───────────────────────────────────────────────────────────────

fn epoch() -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp(0, 0)
        .expect("epoch must be valid")
        .naive_utc()
}

struct FakeFireflyClient {
    accounts: Vec<Ff3AccountDto>,
    transactions: Vec<Ff3TransactionDto>,
}

#[async_trait]
impl FireflyClient for FakeFireflyClient {
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
        Ok(self.transactions.clone())
    }
    async fn create_transaction(
        &self,
        _creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        Err(ApplicationError::InvalidState("not used in F4".into()))
    }
    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        Err(ApplicationError::InvalidState("not used in F4".into()))
    }
}

struct FakeFf3AccountRepo {
    accounts: Mutex<Vec<Ff3Account>>,
}

impl FakeFf3AccountRepo {
    fn new() -> Self {
        Self {
            accounts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Ff3AccountRepository for FakeFf3AccountRepo {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        let mut store = self.accounts.lock().unwrap();
        for a in accounts {
            if let Some(existing) = store.iter_mut().find(|e| e.ff3_id == a.ff3_id) {
                existing.name = a.name;
                existing.account_type = a.account_type;
                existing.currency = a.currency;
                existing.synced_at = epoch();
            } else {
                let id = Ff3AccountId::new(store.len() as i64 + 1);
                store.push(Ff3Account {
                    id,
                    ff3_id: a.ff3_id,
                    name: a.name,
                    account_type: a.account_type,
                    currency: a.currency,
                    synced_at: epoch(),
                });
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

struct FakeBatchRepo {
    batches: Mutex<Vec<Batch>>,
    next_id: Mutex<i64>,
}

impl FakeBatchRepo {
    fn new() -> Self {
        Self {
            batches: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    fn seed(self, batch: Batch) -> Self {
        self.batches.lock().unwrap().push(batch);
        self
    }
}

#[async_trait]
impl BatchRepository for FakeBatchRepo {
    async fn insert(&self, batch: NewBatch) -> Result<BatchId, ApplicationError> {
        let mut id = self.next_id.lock().unwrap();
        let batch_id = BatchId::new(*id);
        *id += 1;
        self.batches.lock().unwrap().push(Batch {
            id: batch_id,
            filename: batch.filename,
            file_hash: batch.file_hash,
            format: batch.format,
            currency: batch.currency,
            account_id: batch.account_id,
            status: batch.status,
            created_at: epoch(),
        });
        Ok(batch_id)
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
        account_id: Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        if let Some(b) = self.batches.lock().unwrap().iter_mut().find(|b| b.id == id) {
            b.account_id = Some(account_id);
        }
        Ok(())
    }
}

struct FakeMovementRepo {
    movements: Mutex<Vec<Movement>>,
}

impl FakeMovementRepo {
    fn new() -> Self {
        Self {
            movements: Mutex::new(Vec::new()),
        }
    }

    fn seed(self, movements: Vec<Movement>) -> Self {
        self.movements.lock().unwrap().extend(movements);
        self
    }
}

#[async_trait]
impl MovementRepository for FakeMovementRepo {
    async fn insert_batch(&self, movements: Vec<NewMovement>) -> Result<(), ApplicationError> {
        let mut store = self.movements.lock().unwrap();
        for m in movements {
            let next_id = store.len() as i64 + 1;
            store.push(Movement {
                id: MovementId::new(next_id),
                batch_id: m.batch_id,
                line_no: m.line_no,
                date: m.date,
                amount_cents: m.amount_cents,
                description: m.description,
                raw_json: m.raw_json,
                status: m.status,
                created_at: epoch(),
            });
        }
        Ok(())
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
struct FakeReconciliationRepo {
    runs: Mutex<Vec<(ReconciliationId, ReconciliationRun)>>,
    next_id: Mutex<i64>,
    /// Mirrors the real `save_run`: movement status updates are applied
    /// as part of the run (single tx in the MySQL adapter).
    movements: Option<Arc<FakeMovementRepo>>,
}

impl FakeReconciliationRepo {
    fn with_movements(movements: Arc<FakeMovementRepo>) -> Self {
        Self {
            runs: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
            movements: Some(movements),
        }
    }

    fn saved_runs(&self) -> Vec<ReconciliationRun> {
        self.runs
            .lock()
            .unwrap()
            .iter()
            .map(|(_, r)| r.clone())
            .collect()
    }

    fn run_count(&self) -> usize {
        self.runs.lock().unwrap().len()
    }
}

#[async_trait]
impl ReconciliationRepository for FakeReconciliationRepo {
    async fn insert(
        &self,
        _reconciliation: ffm_application::dtos::NewReconciliation,
    ) -> Result<ReconciliationId, ApplicationError> {
        unimplemented!("F4 uses save_run")
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
            .rfind(|(_, r)| r.reconciliation.batch_id == batch_id)
            .map(|(id, r)| Reconciliation {
                id: *id,
                batch_id: r.reconciliation.batch_id,
                tolerance_days: r.reconciliation.tolerance_days,
                window_start: r.reconciliation.window_start,
                window_end: r.reconciliation.window_end,
                matched_count: r.reconciliation.matched_count,
                missing_count: r.reconciliation.missing_count,
                extra_count: r.reconciliation.extra_count,
                ran_at: epoch(),
            }))
    }
    async fn save_run(&self, run: ReconciliationRun) -> Result<ReconciliationId, ApplicationError> {
        let recon_id = {
            let mut id = self.next_id.lock().unwrap();
            let recon_id = ReconciliationId::new(*id);
            *id += 1;
            recon_id
        };

        if let Some(movements) = &self.movements {
            for (movement_id, status) in &run.movement_status_updates {
                movements.update_status(*movement_id, *status).await?;
            }
        }

        self.runs.lock().unwrap().push((recon_id, run));
        Ok(recon_id)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn movement(
    id: i64,
    batch: BatchId,
    line: i32,
    date: NaiveDate,
    cents: i64,
    status: MovementStatus,
) -> Movement {
    Movement {
        id: MovementId::new(id),
        batch_id: batch,
        line_no: line,
        date,
        amount_cents: AmountCents::new(cents),
        description: format!("m{id}"),
        raw_json: "{}".into(),
        status,
        created_at: epoch(),
    }
}

fn tx(ff3_id: i64, date: NaiveDate, cents: i64) -> Ff3TransactionDto {
    Ff3TransactionDto {
        ff3_id,
        date,
        amount_cents: AmountCents::new(cents),
        description: format!("tx{ff3_id}"),
    }
}

struct Fixture {
    batch_id: BatchId,
    batches: Arc<FakeBatchRepo>,
    movements: Arc<FakeMovementRepo>,
    ff3_accounts: Arc<FakeFf3AccountRepo>,
    reconciliations: Arc<FakeReconciliationRepo>,
    firefly_transactions: Arc<Mutex<Vec<Ff3TransactionDto>>>,
}

type ReconcileUc = ReconcileBatch<
    Arc<dyn FireflyClient>,
    Arc<dyn Ff3AccountRepository>,
    Arc<FakeBatchRepo>,
    Arc<FakeMovementRepo>,
    Arc<FakeReconciliationRepo>,
>;

impl Fixture {
    fn uc(&self) -> ReconcileUc {
        ReconcileBatch::new(
            self.firefly_client(),
            self.ff3_accounts.clone(),
            self.batches.clone(),
            self.movements.clone(),
            self.reconciliations.clone(),
        )
    }

    fn firefly_client(&self) -> Arc<FakeFireflyClient> {
        Arc::new(FakeFireflyClient {
            accounts: vec![Ff3AccountDto {
                ff3_id: 7,
                name: "Checking".into(),
                account_type: "asset".into(),
                currency: "EUR".into(),
            }],
            transactions: self.firefly_transactions.lock().unwrap().clone(),
        })
    }

    fn input(&self) -> ReconcileBatchInput {
        ReconcileBatchInput {
            batch_id: self.batch_id,
            ff3_account_id: 7,
            window_start: d(2026, 1, 1),
            window_end: d(2026, 1, 31),
            tolerance_days: 0,
        }
    }
}

fn fixture_eur() -> Fixture {
    let batches = Arc::new(FakeBatchRepo::new().seed(Batch {
        id: BatchId::new(1),
        filename: "stmt.csv".into(),
        file_hash: "h".into(),
        format: "csv".into(),
        currency: "EUR".into(),
        account_id: None,
        status: BatchStatus::Parsed,
        created_at: epoch(),
    }));
    let movements = Arc::new(FakeMovementRepo::new().seed(vec![
        movement(
            1,
            BatchId::new(1),
            1,
            d(2026, 1, 5),
            12340,
            MovementStatus::Imported,
        ),
        movement(
            2,
            BatchId::new(1),
            2,
            d(2026, 1, 6),
            -5000,
            MovementStatus::Imported,
        ),
        movement(
            3,
            BatchId::new(1),
            3,
            d(2026, 1, 7),
            100,
            MovementStatus::Ignored,
        ),
        movement(
            4,
            BatchId::new(1),
            4,
            d(2026, 1, 8),
            777,
            MovementStatus::Imported,
        ),
    ]));
    let ff3_accounts = Arc::new(FakeFf3AccountRepo::new());
    let reconciliations = Arc::new(FakeReconciliationRepo::with_movements(movements.clone()));
    let firefly_transactions = Arc::new(Mutex::new(vec![
        tx(101, d(2026, 1, 5), 12340),
        tx(102, d(2026, 1, 6), -5000),
        tx(103, d(2026, 1, 6), -5000),
        tx(104, d(2026, 1, 20), 9900),
    ]));
    Fixture {
        batch_id: BatchId::new(1),
        batches,
        movements,
        ff3_accounts,
        reconciliations,
        firefly_transactions,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn reconcile_persists_run_and_updates_statuses() {
    let f = fixture_eur();
    let result = f.uc().execute(f.input()).await.expect("reconcile ok");

    // Counts: M1+M2 matched, M4 missing (M3 ignored → excluded), extra group = extra + duplicates.
    assert_eq!(result.matched_count, 2);
    assert_eq!(result.missing_count, 1);
    assert_eq!(result.extra_count, 2);
    assert_eq!(result.duplicate_count, 1);

    // Run persisted with snapshot counts.
    assert_eq!(f.reconciliations.run_count(), 1);
    let runs = f.reconciliations.saved_runs();
    let run = &runs[0];
    assert_eq!(run.reconciliation.batch_id, BatchId::new(1));
    assert_eq!(run.reconciliation.matched_count, 2);
    assert_eq!(run.reconciliation.missing_count, 1);
    assert_eq!(run.reconciliation.extra_count, 2);

    // All fetched FF3 transactions cached for the run; 103 flagged duplicate (RF-15).
    assert_eq!(run.ff3_transactions.len(), 4);
    let by_ff3: HashSet<i64> = run.ff3_transactions.iter().map(|t| t.ff3_id).collect();
    assert_eq!(by_ff3, HashSet::from([101, 102, 103, 104]));
    assert!(
        run.ff3_transactions
            .iter()
            .find(|t| t.ff3_id == 103)
            .expect("tx 103 cached")
            .is_duplicate
    );
    assert!(
        !run.ff3_transactions
            .iter()
            .find(|t| t.ff3_id == 104)
            .expect("tx 104 cached")
            .is_duplicate
    );

    // Matches: 1↔101, 2↔102.
    assert_eq!(run.matches.len(), 2);
    let pairs: HashSet<(i64, i64)> = run
        .matches
        .iter()
        .map(|m| (m.movement_id.as_i64(), m.ff3_id))
        .collect();
    assert_eq!(pairs, HashSet::from([(1, 101), (2, 102)]));

    // Statuses updated: 1,2 → Matched; 4 → Missing; 3 (ignored) untouched.
    let statuses = f.movements.movements.lock().unwrap().clone();
    let status_of = |id: i64| {
        statuses
            .iter()
            .find(|m| m.id.as_i64() == id)
            .expect("movement")
            .status
    };
    assert_eq!(status_of(1), MovementStatus::Matched);
    assert_eq!(status_of(2), MovementStatus::Matched);
    assert_eq!(status_of(4), MovementStatus::Missing);
    assert_eq!(status_of(3), MovementStatus::Ignored);

    // Batch updated: account linked + status Reconciled.
    let batch = f.batches.get(BatchId::new(1)).await.expect("batch");
    assert!(batch.account_id.is_some());
    assert_eq!(batch.status, BatchStatus::Reconciled);
}

#[tokio::test]
async fn reconcile_currency_mismatch_is_blocked() {
    let f = fixture_eur();
    {
        let mut batches = f.batches.batches.lock().unwrap();
        batches[0].currency = "USD".into();
    }

    let err = f.uc().execute(f.input()).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::InvalidState(_)));
    assert!(err.to_string().to_lowercase().contains("currency"));
    assert_eq!(f.reconciliations.run_count(), 0);
}

#[tokio::test]
async fn reconcile_unknown_batch_is_not_found() {
    let f = fixture_eur();
    let mut input = f.input();
    input.batch_id = BatchId::new(999);
    let err = f.uc().execute(input).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn reconcile_unknown_account_is_not_found() {
    let f = fixture_eur();
    let mut input = f.input();
    input.ff3_account_id = 4242;
    let err = f.uc().execute(input).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn reconcile_invalid_window_is_rejected() {
    let f = fixture_eur();
    let mut input = f.input();
    input.window_start = d(2026, 2, 1);
    input.window_end = d(2026, 1, 1);
    let err = f.uc().execute(input).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::InvalidState(_)));
}

#[tokio::test]
async fn reconcile_rerun_creates_new_reconciliation() {
    let f = fixture_eur();

    // First run: M4 (+777) is missing.
    let first = f.uc().execute(f.input()).await.expect("first run");
    assert_eq!(first.missing_count, 1);
    assert_eq!(f.reconciliations.run_count(), 1);

    // FF3 gains a matching transaction for M4 (RF-13 re-rapprochement).
    f.firefly_transactions
        .lock()
        .unwrap()
        .push(tx(105, d(2026, 1, 8), 777));

    let second = f.uc().execute(f.input()).await.expect("second run");
    assert_eq!(second.matched_count, 3);
    assert_eq!(second.missing_count, 0);
    assert_eq!(f.reconciliations.run_count(), 2);

    // Latest reconciliation for the batch is the second one.
    let latest = f
        .reconciliations
        .latest_by_batch(BatchId::new(1))
        .await
        .expect("latest")
        .expect("some");
    assert_eq!(latest.id, second.reconciliation_id);

    // M4 confirmed matched by the re-run.
    let movements = f.movements.movements.lock().unwrap().clone();
    assert_eq!(
        movements
            .iter()
            .find(|m| m.id.as_i64() == 4)
            .expect("m4")
            .status,
        MovementStatus::Matched
    );
}

#[tokio::test]
async fn reconcile_syncs_ff3_account_cache() {
    let f = fixture_eur();
    f.uc().execute(f.input()).await.expect("reconcile ok");
    let cached = f
        .ff3_accounts
        .by_ff3_id(7)
        .await
        .expect("account cached after sync");
    assert!(cached.is_some());
}
