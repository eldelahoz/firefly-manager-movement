use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use ffm_application::dtos::{
    About, ApplyMissingInput, AttachmentPayload, ExternalIdHit, Ff3AccountDto, Ff3TransactionDto,
    MovementApply, NewCreation, NewFf3Account, NewFf3TransactionCreation, NewMatch,
    NewReconciliation, ReconciliationRun,
};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, CounterpartySuggestionRepository, CreationRepository, Ff3AccountRepository,
    Ff3TransactionRepository, FireflyClient, InMemoryCounterpartySuggestions, MatchRepository,
    MovementRepository, ReconciliationRepository,
};
use ffm_application::use_cases::app_settings::FakeSettings;
use ffm_application::use_cases::apply_missing::{ApplyMissing, ApplyOutcome, RunPorts};

use ffm_application::dtos::CounterpartyRef;
use ffm_domain::entities::{
    Batch, Creation, Ff3Account, Ff3Transaction, Match, Movement, Reconciliation,
};
use ffm_domain::ids::{
    BatchId, CreationId, Ff3AccountId, Ff3TransactionId, MatchId, MovementId, ReconciliationId,
};
use ffm_domain::transaction_kind::Direction;
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
/// Also records attachments uploaded via `attach_to_transaction`.
struct FakeFirefly {
    accounts: Vec<Ff3AccountDto>,
    created: Mutex<Vec<NewFf3TransactionCreation>>,
    next_id: Mutex<i64>,
    fail_amounts: Vec<i64>,
    attachments: Mutex<Vec<(i64, String, String)>>,
    fail_attachments: Mutex<bool>,
    /// `(account ff3_id, hit)` stored in Firefly III, searchable by prefix.
    stored: Mutex<Vec<(i64, ExternalIdHit)>>,
    search_fails: Mutex<bool>,
    searches: Mutex<Vec<(i64, String)>>,
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
    async fn accounts_by_type(
        &self,
        _account_type: &str,
    ) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        unimplemented!("not used by this fake")
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
    ) -> Result<ffm_application::dtos::CreatedFf3Transaction, ApplicationError> {
        if self.fail_amounts.contains(&creation.amount.as_i64()) {
            return Err(ApplicationError::FireflyClient("FF3 500".into()));
        }
        let mut id = self.next_id.lock().unwrap();
        *id += 1;
        if let Some(ext) = &creation.external_id {
            self.stored.lock().unwrap().push((7, hit(ext, 500 + *id)));
        }
        self.created.lock().unwrap().push(creation);
        Ok(ffm_application::dtos::CreatedFf3Transaction {
            ff3_id: 500 + *id,
            source_id: Some(70),
            destination_id: Some(80),
            source_name: Some("S".into()),
            source_type: Some("Loan".into()),
            destination_name: Some("D".into()),
            destination_type: Some("Expense account".into()),
        })
    }
    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        unimplemented!("F5.3")
    }
    async fn transactions_by_external_id_prefix(
        &self,
        account_ff3_id: i64,
        prefix: &str,
    ) -> Result<Vec<ExternalIdHit>, ApplicationError> {
        self.searches
            .lock()
            .unwrap()
            .push((account_ff3_id, prefix.to_string()));
        if *self.search_fails.lock().unwrap() {
            return Err(ApplicationError::FireflyClient("FF3 unreachable".into()));
        }
        Ok(self
            .stored
            .lock()
            .unwrap()
            .iter()
            .filter(|(acc, h)| *acc == account_ff3_id && h.external_id.starts_with(prefix))
            .map(|(_, h)| h.clone())
            .collect())
    }
    async fn attach_to_transaction(
        &self,
        journal_ff3_id: i64,
        filename: &str,
        content_base64: &str,
    ) -> Result<(), ApplicationError> {
        if *self.fail_attachments.lock().unwrap() {
            return Err(ApplicationError::FireflyClient("attach 500".into()));
        }
        self.attachments.lock().unwrap().push((
            journal_ff3_id,
            filename.to_string(),
            content_base64.to_string(),
        ));
        Ok(())
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
    fail_inserts: Mutex<bool>,
}

#[async_trait]
impl CreationRepository for FakeCreationRepo {
    async fn insert(&self, creation: NewCreation) -> Result<CreationId, ApplicationError> {
        if *self.fail_inserts.lock().unwrap() {
            return Err(ApplicationError::Database("disk I/O error".into()));
        }
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
                ff3_journal_id: 0,
                id: Ff3TransactionId::new(30),
                reconciliation_id: ReconciliationId::new(10),
                ff3_id: 300,
                date: d(2026, 1, 3),
                amount_cents: AmountCents::new(900),
                description: "tx300".into(),
                is_duplicate: false,
                created_at: epoch(),
                counterparty_name: Some("Shop".into()),
                counterparty_type: Some("Expense account".into()),
            },
            Ff3Transaction {
                ff3_journal_id: 0,
                id: Ff3TransactionId::new(31),
                reconciliation_id: ReconciliationId::new(10),
                ff3_id: 310,
                date: d(2026, 1, 25),
                amount_cents: AmountCents::new(111),
                description: "tx310".into(),
                is_duplicate: false,
                created_at: epoch(),
                counterparty_name: None,
                counterparty_type: None,
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
        // Mirror the DB: UNIQUE(reconciliation_id, movement_id) and
        // UNIQUE(reconciliation_id, ff3_transaction_id).
        let movements: std::collections::HashSet<_> =
            run.matches.iter().map(|m| m.movement_id).collect();
        let ff3_ids: std::collections::HashSet<_> = run.matches.iter().map(|m| m.ff3_id).collect();
        if movements.len() != run.matches.len() || ff3_ids.len() != run.matches.len() {
            return Err(ApplicationError::Database(
                "UNIQUE constraint failed: matches".into(),
            ));
        }
        let recon_id = ReconciliationId::new(20 + self.saved_runs.lock().unwrap().len() as i64);

        let mut ff3_local = std::collections::HashMap::new();
        {
            let mut next_tx = self.next_tx_id.lock().unwrap();
            for t in &run.ff3_transactions {
                *next_tx += 1;
                ff3_local.insert(t.ff3_id, *next_tx);
                self.tx_rows.lock().unwrap().push(Ff3Transaction {
                    ff3_journal_id: 0,
                    id: Ff3TransactionId::new(*next_tx),
                    reconciliation_id: recon_id,
                    ff3_id: t.ff3_id,
                    date: t.date,
                    amount_cents: t.amount_cents,
                    description: t.description.clone(),
                    is_duplicate: t.is_duplicate,
                    created_at: epoch(),
                    counterparty_name: t.counterparty_name.clone(),
                    counterparty_type: t.counterparty_type.clone(),
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

fn hit(external_id: &str, ff3_id: i64) -> ExternalIdHit {
    ExternalIdHit {
        external_id: external_id.into(),
        ff3_id,
        counterparty_name: Some("Shop".into()),
        counterparty_type: Some("Expense account".into()),
    }
}

const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const PREFIX: &str = "ffm-0123456789abcdef-";

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
    settings: Arc<FakeSettings>,
    memory: Arc<InMemoryCounterpartySuggestions>,
}

fn fx(fail_amounts: Vec<i64>) -> Fx {
    let movements = Arc::new(FakeMovementRepo {
        movements: Mutex::new(vec![
            movement(1, -5000, MovementStatus::Missing), // debit
            movement(2, 3000, MovementStatus::Missing),  // credit
            movement(3, 900, MovementStatus::Matched),   // already matched (prev run)
            movement(4, 0, MovementStatus::Missing),     // zero amount
            movement(5, 111, MovementStatus::Ignored),
        ]),
    });
    Fx {
        memory: Arc::new(InMemoryCounterpartySuggestions::default()),
        firefly: Arc::new(FakeFirefly {
            accounts: vec![],
            created: Mutex::new(vec![]),
            next_id: Mutex::new(0),
            fail_amounts,
            attachments: Mutex::new(vec![]),
            fail_attachments: Mutex::new(false),
            stored: Mutex::new(vec![]),
            search_fails: Mutex::new(false),
            searches: Mutex::new(vec![]),
        }),
        movements: movements.clone(),
        creations: Arc::new(FakeCreationRepo::default()),
        runs: Arc::new(FakeRunStore::seeded(movements)),
        batches: Arc::new(FakeBatchRepo {
            batches: vec![Batch {
                id: BatchId::new(1),
                filename: "stmt.csv".into(),
                file_hash: HASH.into(),
                format: "csv".into(),
                currency: "EUR".into(),
                account_id: Some(Ff3AccountId::new(1)),
                status: BatchStatus::Reconciled,
                headers: vec![],
                detected_delimiter: ",".into(),
                created_at: epoch(),
            }],
        }),
        settings: Arc::new(FakeSettings(std::sync::Mutex::new(Some(
            ffm_domain::entities::AppSettings {
                ff3_url: "http://example.com".into(),
                ff3_token: "token".into(),
                default_creation_tag: None,
                enable_origin_trace: true,
                default_date_format: None,
                updated_at: epoch(),
            },
        )))),
    }
}

struct StaticAccounts;

#[async_trait]
impl Ff3AccountRepository for StaticAccounts {
    async fn upsert_batch(&self, _accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    /// Cached counterparties: 7 = the batch account, 10 expense, 20 revenue,
    /// 42 liability, 30 own asset (EUR), 31 own asset in USD.
    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        let (account_type, currency) = match ff3_id {
            7 | 30 => ("asset", "EUR"),
            10 => ("expense", "EUR"),
            20 => ("revenue", "EUR"),
            42 => ("liabilities", "EUR"),
            31 => ("asset", "USD"),
            _ => return Ok(None),
        };
        Ok(Some(Ff3Account {
            id: Ff3AccountId::new(ff3_id + 100),
            ff3_id,
            name: format!("acc {ff3_id}"),
            account_type: account_type.into(),
            currency: currency.into(),
            iban: None,
            account_number: None,
            synced_at: epoch(),
        }))
    }
    async fn by_id(&self, id: Ff3AccountId) -> Result<Option<Ff3Account>, ApplicationError> {
        if id == Ff3AccountId::new(1) {
            Ok(Some(Ff3Account {
                id,
                ff3_id: 7,
                name: "Checking".into(),
                account_type: "asset".into(),
                currency: "EUR".into(),
                iban: None,
                account_number: None,
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

type ApplyUc<Mem = Arc<InMemoryCounterpartySuggestions>> = ApplyMissing<
    Arc<dyn FireflyClient>,
    Arc<dyn Ff3AccountRepository>,
    Arc<FakeBatchRepo>,
    Arc<FakeMovementRepo>,
    Arc<FakeCreationRepo>,
    Arc<FakeRunStore>,
    Arc<FakeRunStore>,
    Arc<FakeRunStore>,
    Arc<FakeSettings>,
    Mem,
>;

impl Fx {
    fn uc(&self) -> ApplyUc {
        self.uc_with(self.memory.clone())
    }

    fn uc_with<Mem>(&self, memory: Mem) -> ApplyUc<Mem> {
        ApplyMissing::new(
            self.firefly.clone() as Arc<dyn FireflyClient>,
            Arc::new(StaticAccounts) as Arc<dyn Ff3AccountRepository>,
            self.batches.clone(),
            self.movements.clone(),
            self.creations.clone(),
            RunPorts {
                reconciliations: self.runs.clone(),
                matches: self.runs.clone(),
                ff3_transactions: self.runs.clone(),
            },
            self.settings.clone(),
            memory,
        )
    }

    fn input(&self, movements: Vec<MovementApply>) -> ApplyMissingInput {
        ApplyMissingInput {
            batch_id: BatchId::new(1),
            movements,
        }
    }
}

fn ma(movement_id: i64, counterparty_ff3_id: i64) -> MovementApply {
    ma_opt(movement_id, Some(counterparty_ff3_id))
}

fn ma_opt(movement_id: i64, counterparty_ff3_id: Option<i64>) -> MovementApply {
    MovementApply {
        movement_id: MovementId::new(movement_id),
        counterparty_ff3_id,
        counterparty_name: None,
        counterparty_cash: false,
        description: None,
        category: None,
        budget: None,
        tags: None,
        notes: None,
        attachment: None,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn applies_per_movement_customization() {
    let f = fx(vec![]);
    let input = f.input(vec![
        MovementApply {
            movement_id: MovementId::new(1),
            counterparty_ff3_id: Some(10),
            counterparty_name: None,
            counterparty_cash: false,
            description: Some("Custom debit".into()),
            category: Some("Rent".into()),
            budget: Some("Housing".into()),
            tags: Some(vec!["tag-a".into()]),
            notes: Some("note-1".into()),
            attachment: None,
        },
        MovementApply {
            movement_id: MovementId::new(2),
            counterparty_ff3_id: Some(20),
            counterparty_name: None,
            counterparty_cash: false,
            description: None, // falls back to statement description
            category: Some("Income".into()),
            budget: None,
            tags: None,
            notes: None,
            attachment: None,
        },
    ]);
    let result = f.uc().execute(input).await.expect("apply ok");

    assert_eq!(result.reports.len(), 2);
    assert!(result
        .reports
        .iter()
        .all(|r| matches!(r.outcome, ApplyOutcome::Created)));

    let calls = f.firefly.calls();
    assert_eq!(calls.len(), 2);

    let debit = calls
        .iter()
        .find(|c| c.amount.as_i64() == -5000)
        .expect("debit");
    assert_eq!(debit.transaction_type, "withdrawal");
    assert_eq!(debit.source_ff3_id, Some(7)); // batch account
    assert_eq!(debit.destination_ff3_id, Some(10)); // per-movement counterparty
    assert_eq!(debit.description, "Custom debit");
    assert_eq!(debit.category.as_deref(), Some("Rent"));
    assert_eq!(debit.budget.as_deref(), Some("Housing"));
    assert_eq!(debit.notes.as_deref(), Some("note-1\n\nffm:stmt.csv:1"));

    let credit = calls
        .iter()
        .find(|c| c.amount.as_i64() == 3000)
        .expect("credit");
    assert_eq!(credit.transaction_type, "deposit");
    assert_eq!(credit.source_ff3_id, Some(20)); // per-movement counterparty
    assert_eq!(credit.destination_ff3_id, Some(7)); // batch account
    assert_eq!(credit.description, "m2"); // statement description fallback
    assert_eq!(credit.category.as_deref(), Some("Income"));
    assert!(credit.budget.is_none());
}

#[tokio::test]
async fn origin_tag_appended_in_addition_to_user_tags() {
    let f = fx(vec![]);
    let input = f.input(vec![MovementApply {
        movement_id: MovementId::new(1),
        counterparty_ff3_id: Some(42),
        counterparty_name: None,
        counterparty_cash: false,
        description: None,
        category: None,
        budget: None,
        tags: Some(vec!["user-tag".into()]),
        notes: None,
        attachment: None,
    }]);
    let result = f.uc().execute(input).await.expect("apply ok");
    assert_eq!(result.created_count, 1);

    let calls = f.firefly.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tags, vec!["user-tag".to_string()]);
    assert_eq!(calls[0].notes.as_deref(), Some("ffm:stmt.csv:1"));
}

#[tokio::test]
async fn origin_trace_note_appended_when_user_tags_empty() {
    let f = fx(vec![]);
    let input = f.input(vec![ma(1, 42)]);
    let result = f.uc().execute(input).await.expect("apply ok");
    assert_eq!(result.created_count, 1);

    let calls = f.firefly.calls();
    assert_eq!(calls[0].tags, Vec::<String>::new()); // No tags (origin trace is note, not tag)
    assert_eq!(calls[0].notes.as_deref(), Some("ffm:stmt.csv:1"));
}

#[tokio::test]
async fn origin_trace_appended_below_user_notes() {
    let f = fx(vec![]);
    let input = f.input(vec![MovementApply {
        movement_id: MovementId::new(1),
        counterparty_ff3_id: Some(42),
        counterparty_name: None,
        counterparty_cash: false,
        description: None,
        category: None,
        budget: None,
        tags: None,
        notes: Some("User note here".into()),
        attachment: None,
    }]);
    let result = f.uc().execute(input).await.expect("apply ok");
    assert_eq!(result.created_count, 1);

    let calls = f.firefly.calls();
    assert_eq!(calls[0].tags, Vec::<String>::new()); // No tags (origin trace is note, not tag)
    assert_eq!(
        calls[0].notes.as_deref(),
        Some("User note here\n\nffm:stmt.csv:1")
    );
}

#[tokio::test]
async fn zero_amount_movement_fails_without_ff3_call() {
    let f = fx(vec![]);
    // m4 has amount_cents = 0
    let input = f.input(vec![MovementApply {
        movement_id: MovementId::new(4),
        counterparty_ff3_id: Some(42),
        counterparty_name: None,
        counterparty_cash: false,
        description: None,
        category: None,
        budget: None,
        tags: None,
        notes: None,
        attachment: None,
    }]);
    let result = f.uc().execute(input).await.expect("apply ok");

    assert_eq!(result.created_count, 0);
    assert_eq!(result.failed_count, 1);
    let failed = result
        .reports
        .iter()
        .find(|r| r.outcome == ApplyOutcome::Failed)
        .unwrap();
    assert!(failed.reason.as_deref().unwrap().contains("zero"));
    assert_eq!(f.firefly.calls().len(), 0);
}

#[tokio::test]
async fn attachment_uploaded_after_creation() {
    let f = fx(vec![]);
    let input = f.input(vec![MovementApply {
        movement_id: MovementId::new(1),
        counterparty_ff3_id: Some(42),
        counterparty_name: None,
        counterparty_cash: false,
        description: None,
        category: None,
        budget: None,
        tags: None,
        notes: None,
        attachment: Some(AttachmentPayload {
            filename: "receipt.pdf".into(),
            content_base64: "base64data".into(),
        }),
    }]);
    let result = f.uc().execute(input).await.expect("apply ok");
    assert_eq!(result.created_count, 1);

    // Transaction created first.
    assert_eq!(f.firefly.calls().len(), 1);
    let ff3_id = result.reports[0].ff3_id.unwrap();

    // Attachment uploaded after creation.
    let attachments = f.firefly.attachments.lock().unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].0, ff3_id);
    assert_eq!(attachments[0].1, "receipt.pdf");
    assert_eq!(attachments[0].2, "base64data");
}

#[tokio::test]
async fn attachment_failure_does_not_fail_creation() {
    let f = fx(vec![]);
    *f.firefly.fail_attachments.lock().unwrap() = true;
    let input = f.input(vec![MovementApply {
        movement_id: MovementId::new(1),
        counterparty_ff3_id: Some(42),
        counterparty_name: None,
        counterparty_cash: false,
        description: None,
        category: None,
        budget: None,
        tags: None,
        notes: None,
        attachment: Some(AttachmentPayload {
            filename: "receipt.pdf".into(),
            content_base64: "base64data".into(),
        }),
    }]);
    let result = f.uc().execute(input).await.expect("apply ok");
    assert_eq!(result.created_count, 1);
    let report = &result.reports[0];
    assert!(report.reason.as_deref().unwrap().contains("attachment"));
}

#[tokio::test]
async fn apply_creates_with_origin_tag_and_confirms_matched() {
    let f = fx(vec![]);
    let input = f.input(vec![ma(1, 42), ma(2, 42)]);
    let result = f.uc().execute(input).await.expect("apply ok");

    assert_eq!(result.reports.len(), 2);
    assert!(result
        .reports
        .iter()
        .all(|r| matches!(r.outcome, ApplyOutcome::Created)));
    assert_eq!(result.created_count, 2);

    let calls = f.firefly.calls();
    assert_eq!(calls.len(), 2);
    for c in &calls {
        assert_eq!(c.tags, Vec::<String>::new()); // No tags (origin trace is note)
        assert_eq!(c.notes.as_deref(), Some("ffm:stmt.csv:1"));
    }

    assert_eq!(f.creations.rows.lock().unwrap().len(), 2);

    // Incremental confirmation (D7).
    assert_eq!(f.runs.saved_runs().len(), 1);
    let run = &f.runs.saved_runs()[0];
    assert_eq!(run.reconciliation.matched_count, 3); // m3 + m1 + m2
    assert_eq!(run.reconciliation.missing_count, 0);
    assert_eq!(run.reconciliation.extra_count, 1); // carried over
                                                   // The new run must carry over the previous run's match rows too, otherwise
                                                   // get_results shows the old pairs as `extra` (they'd be in the tx cache
                                                   // with no match row pointing at them).
    assert_eq!(run.matches.len(), 3); // carried (m3→300) + new (m1→501, m2→502)
    let pairs: Vec<(i64, i64)> = run
        .matches
        .iter()
        .map(|m| (m.movement_id.as_i64(), m.ff3_id))
        .collect();
    assert!(
        pairs.contains(&(3, 300)),
        "carried-over match missing: {pairs:?}"
    );
    assert!(pairs.contains(&(1, 501)), "new m1 match missing: {pairs:?}");
    assert!(pairs.contains(&(2, 502)), "new m2 match missing: {pairs:?}");

    assert_eq!(f.movements.status_of(1), MovementStatus::Matched);
    assert_eq!(f.movements.status_of(2), MovementStatus::Matched);
}

#[tokio::test]
async fn apply_retry_creates_nothing_nfr05() {
    let f = fx(vec![]);
    let input = f.input(vec![ma(1, 42), ma(2, 42)]);
    f.uc().execute(input.clone()).await.expect("first apply");
    let calls_after_first = f.firefly.calls().len();
    assert_eq!(calls_after_first, 2);

    let result = f.uc().execute(input).await.expect("retry ok");
    assert_eq!(result.created_count, 0);
    assert_eq!(result.already_created_count, 2);
    assert_eq!(result.failed_count, 0);
    assert_eq!(f.firefly.calls().len(), 2);
    assert_eq!(f.runs.saved_runs().len(), 1);
}

#[tokio::test]
async fn apply_partial_failure_continues_batch() {
    let f = fx(vec![3000]); // the credit (+3000) fails in FF3
    let input = f.input(vec![ma(1, 42), ma(2, 42)]);
    let result = f.uc().execute(input).await.expect("apply ok");

    assert_eq!(result.created_count, 1);
    assert_eq!(result.failed_count, 1);
    let failed = result
        .reports
        .iter()
        .find(|r| matches!(r.outcome, ApplyOutcome::Failed))
        .expect("failed report");
    assert_eq!(failed.movement_id.as_i64(), 2);
    assert!(failed.reason.as_deref().unwrap_or("").contains("FF3"));

    assert_eq!(f.creations.rows.lock().unwrap().len(), 1);
    assert_eq!(f.movements.status_of(1), MovementStatus::Matched);
    assert_eq!(f.movements.status_of(2), MovementStatus::Missing);
}

#[tokio::test]
async fn apply_requires_reconciled_batch() {
    let f = fx(vec![]);
    f.runs.reconciliations.lock().unwrap().clear();
    let input = f.input(vec![ma(1, 42)]);
    let err = f.uc().execute(input).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::InvalidState(_)));
}

#[tokio::test]
async fn apply_unknown_batch_or_movement_is_not_found() {
    let f = fx(vec![]);
    let mut input = f.input(vec![ma(1, 42)]);
    input.batch_id = BatchId::new(999);
    let err = f.uc().execute(input).await.expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));

    let err = f
        .uc()
        .execute(f.input(vec![MovementApply {
            movement_id: MovementId::new(777),
            counterparty_ff3_id: Some(42),
            counterparty_name: None,
            counterparty_cash: false,
            description: None,
            category: None,
            budget: None,
            tags: None,
            notes: None,
            attachment: None,
        }]))
        .await
        .expect_err("must fail");
    assert!(matches!(err, ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn apply_rejects_movement_not_in_missing_state() {
    let f = fx(vec![]);
    // m3 is Matched (no creation row) → hard reject.
    let err = f
        .uc()
        .execute(f.input(vec![ma(3, 42)]))
        .await
        .expect_err("must fail");
    assert!(matches!(err, ApplicationError::InvalidState(_)));
    assert_eq!(f.firefly.calls().len(), 0);
}

#[tokio::test]
async fn applies_without_counterparty_leaves_side_unset() {
    let f = fx(vec![]);
    let input = f.input(vec![ma_opt(1, None), ma_opt(2, None)]);
    let result = f.uc().execute(input).await.expect("apply ok");
    assert_eq!(result.created_count, 2);

    let calls = f.firefly.calls();
    let debit = calls
        .iter()
        .find(|c| c.amount.as_i64() == -5000)
        .expect("debit");
    assert_eq!(debit.transaction_type, "withdrawal");
    assert_eq!(debit.source_ff3_id, Some(7)); // batch account is always the known side
    assert_eq!(debit.destination_ff3_id, None); // counterparty omitted

    let credit = calls
        .iter()
        .find(|c| c.amount.as_i64() == 3000)
        .expect("credit");
    assert_eq!(credit.transaction_type, "deposit");
    assert_eq!(credit.source_ff3_id, None);
    assert_eq!(credit.destination_ff3_id, Some(7));
}

fn with_default_tag(f: &Fx, tag: &str) {
    f.settings
        .0
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .default_creation_tag = Some(tag.into());
}

#[tokio::test]
async fn absent_tags_use_default_creation_tag() {
    let f = fx(vec![]);
    with_default_tag(&f, "ffm-import");
    f.uc()
        .execute(f.input(vec![ma(1, 42)]))
        .await
        .expect("apply ok");
    assert_eq!(f.firefly.calls()[0].tags, vec!["ffm-import".to_string()]);
}

#[tokio::test]
async fn explicit_empty_tags_do_not_add_default_tag() {
    let f = fx(vec![]);
    with_default_tag(&f, "ffm-import");
    let mut m = ma(1, 42);
    m.tags = Some(vec![]);
    f.uc().execute(f.input(vec![m])).await.expect("apply ok");
    assert_eq!(f.firefly.calls()[0].tags, Vec::<String>::new());
}

// ── Counterparty types (RF-08 v0.10) ──────────────────────────────────────

fn named(movement_id: i64, name: &str) -> MovementApply {
    MovementApply {
        counterparty_name: Some(name.into()),
        ..ma_opt(movement_id, None)
    }
}

fn report_type(
    result: &ffm_application::use_cases::apply_missing::ApplyOutput,
    id: i64,
) -> Option<&'static str> {
    result
        .reports
        .iter()
        .find(|r| r.movement_id == MovementId::new(id))
        .and_then(|r| r.transaction_type)
        .map(|k| k.as_str())
}

#[tokio::test]
async fn own_asset_counterparty_creates_transfer_and_liability_follows_direction() {
    let f = fx(vec![]);
    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 30), ma(2, 42)]))
        .await
        .expect("apply ok");
    assert_eq!(result.created_count, 2);
    assert_eq!(report_type(&result, 1), Some("transfer"));
    assert_eq!(report_type(&result, 2), Some("deposit"));

    let calls = f.firefly.calls();
    let transfer = calls.iter().find(|c| c.amount.as_i64() == -5000).unwrap();
    assert_eq!(transfer.transaction_type, "transfer");
    assert_eq!(transfer.source_ff3_id, Some(7));
    assert_eq!(transfer.destination_ff3_id, Some(30));
    let deposit = calls.iter().find(|c| c.amount.as_i64() == 3000).unwrap();
    assert_eq!(deposit.transaction_type, "deposit");
    assert_eq!(deposit.source_ff3_id, Some(42));
    assert_eq!(deposit.destination_ff3_id, Some(7));
}

#[tokio::test]
async fn expense_and_revenue_counterparties_report_their_type() {
    let f = fx(vec![]);
    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 10), ma(2, 20)]))
        .await
        .expect("apply ok");
    assert_eq!(report_type(&result, 1), Some("withdrawal"));
    assert_eq!(report_type(&result, 2), Some("deposit"));
}

#[tokio::test]
async fn new_name_sends_destination_or_source_name_without_id() {
    let f = fx(vec![]);
    let result = f
        .uc()
        .execute(f.input(vec![named(1, " Corner Shop "), named(2, "Employer")]))
        .await
        .expect("apply ok");
    assert_eq!(result.created_count, 2);
    assert_eq!(report_type(&result, 1), Some("withdrawal"));
    assert_eq!(report_type(&result, 2), Some("deposit"));

    let calls = f.firefly.calls();
    let debit = calls.iter().find(|c| c.amount.as_i64() == -5000).unwrap();
    assert_eq!(debit.transaction_type, "withdrawal");
    assert_eq!(debit.source_ff3_id, Some(7));
    assert_eq!(debit.destination_ff3_id, None);
    assert_eq!(debit.destination_name.as_deref(), Some("Corner Shop"));
    assert_eq!(debit.source_name, None);
    let credit = calls.iter().find(|c| c.amount.as_i64() == 3000).unwrap();
    assert_eq!(credit.transaction_type, "deposit");
    assert_eq!(credit.source_ff3_id, None);
    assert_eq!(credit.source_name.as_deref(), Some("Employer"));
    assert_eq!(credit.destination_ff3_id, Some(7));
    assert_eq!(credit.destination_name, None);
}

#[tokio::test]
async fn cash_reports_withdrawal_and_deposit() {
    let f = fx(vec![]);
    let result = f
        .uc()
        .execute(f.input(vec![ma_opt(1, None), ma_opt(2, None)]))
        .await
        .expect("apply ok");
    assert_eq!(report_type(&result, 1), Some("withdrawal"));
    assert_eq!(report_type(&result, 2), Some("deposit"));
}

#[tokio::test]
async fn rejected_counterparty_fails_that_movement_only_and_retry_creates_it() {
    let f = fx(vec![]);
    // Money out to a revenue account is rejected; the credit still goes through.
    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 20), ma(2, 20)]))
        .await
        .expect("apply ok");
    assert_eq!(result.created_count, 1);
    assert_eq!(result.failed_count, 1);
    let failed = result
        .reports
        .iter()
        .find(|r| r.movement_id == MovementId::new(1))
        .unwrap();
    assert_eq!(failed.outcome, ApplyOutcome::Failed);
    assert!(failed.reason.as_deref().unwrap().contains("revenue"));
    assert_eq!(failed.transaction_type, None);
    assert_eq!(f.firefly.calls().len(), 1);
    assert_eq!(f.movements.status_of(1), MovementStatus::Missing);
    assert!(f
        .creations
        .by_movement(MovementId::new(1))
        .await
        .unwrap()
        .is_none());

    // Retry with a valid counterparty creates it; the created one is not duplicated.
    let retry = f
        .uc()
        .execute(f.input(vec![ma(1, 10), ma(2, 20)]))
        .await
        .expect("retry ok");
    assert_eq!(retry.created_count, 1);
    assert_eq!(retry.already_created_count, 1);
    assert_eq!(report_type(&retry, 1), Some("withdrawal"));
    assert_eq!(f.firefly.calls().len(), 2);
}

#[tokio::test]
async fn invalid_counterparties_are_rejected_before_firefly() {
    let both = MovementApply {
        counterparty_name: Some("Corner Shop".into()),
        ..ma(1, 10)
    };
    let cases = [
        (ma(1, 7), "batch account"),
        (ma(1, 31), "currencies"),
        (ma(1, 99), "not a known account"),
        (both, "not both"),
    ];
    for (apply, expected) in cases {
        let f = fx(vec![]);
        let result = f
            .uc()
            .execute(f.input(vec![apply]))
            .await
            .expect("apply ok");
        let report = &result.reports[0];
        assert_eq!(report.outcome, ApplyOutcome::Failed, "{expected}");
        let reason = report.reason.as_deref().unwrap();
        assert!(
            reason.contains(expected),
            "{reason} should contain {expected}"
        );
        assert!(f.firefly.calls().is_empty());
    }
    // Money in from an expense account.
    let f = fx(vec![]);
    let result = f
        .uc()
        .execute(f.input(vec![ma(2, 10)]))
        .await
        .expect("apply ok");
    assert!(result.reports[0]
        .reason
        .as_deref()
        .unwrap()
        .contains("expense"));
    assert!(f.firefly.calls().is_empty());
}

#[tokio::test]
async fn confirmation_run_keeps_previous_snapshot_counterparty() {
    let f = fx(vec![]);
    f.uc()
        .execute(f.input(vec![ma(1, 10)]))
        .await
        .expect("apply ok");
    let run = &f.runs.saved_runs()[0];
    let carried = run
        .ff3_transactions
        .iter()
        .find(|t| t.ff3_id == 300)
        .unwrap();
    assert_eq!(carried.counterparty_name.as_deref(), Some("Shop"));
    assert_eq!(
        carried.counterparty_type.as_deref(),
        Some("Expense account")
    );
}

#[tokio::test]
async fn confirmation_run_stores_counterparty_from_firefly_response() {
    let f = fx(vec![]);
    f.uc()
        .execute(f.input(vec![ma(1, 10), ma(2, 42)]))
        .await
        .expect("apply ok");
    let run = &f.runs.saved_runs()[0];
    let row = |cents: i64| {
        run.ff3_transactions
            .iter()
            .find(|t| t.amount_cents.as_i64() == cents)
            .unwrap()
    };
    // Money out: the batch account is the source, so the counterparty is the destination.
    assert_eq!(row(-5000).counterparty_name.as_deref(), Some("D"));
    assert_eq!(
        row(-5000).counterparty_type.as_deref(),
        Some("Expense account")
    );
    // Money in: the batch account is the destination, so the counterparty is the source.
    assert_eq!(row(3000).counterparty_name.as_deref(), Some("S"));
    assert_eq!(row(3000).counterparty_type.as_deref(), Some("Loan"));
}

// ── NFR-05: external_id recovery ────────────────────────────────────────

#[tokio::test]
async fn each_creation_carries_a_content_based_external_id() {
    let f = fx(vec![]);
    f.uc()
        .execute(f.input(vec![ma(1, 42), ma(2, 42)]))
        .await
        .expect("apply ok");

    let ids: Vec<_> = f
        .firefly
        .calls()
        .into_iter()
        .map(|c| c.external_id)
        .collect();
    assert_eq!(
        ids,
        vec![Some(format!("{PREFIX}1")), Some(format!("{PREFIX}2"))]
    );
    assert_eq!(
        *f.firefly.searches.lock().unwrap(),
        vec![(7, PREFIX.to_string())]
    );
}

#[tokio::test]
async fn retry_after_record_failure_does_not_post_again() {
    let f = fx(vec![]);
    *f.creations.fail_inserts.lock().unwrap() = true;
    let first = f
        .uc()
        .execute(f.input(vec![ma(1, 42), ma(2, 42)]))
        .await
        .expect("a record failure must not abort the batch");
    assert_eq!(first.failed_count, 2);
    assert_eq!(f.firefly.calls().len(), 2, "both reached Firefly III");
    let reason = first.reports[0].reason.as_deref().unwrap();
    assert!(reason.contains("not recorded"), "{reason}");
    assert_eq!(f.movements.status_of(1), MovementStatus::Missing);

    *f.creations.fail_inserts.lock().unwrap() = false;
    let retry = f
        .uc()
        .execute(f.input(vec![ma(1, 42), ma(2, 42)]))
        .await
        .expect("retry ok");
    assert_eq!(f.firefly.calls().len(), 2, "no second POST");
    assert_eq!(retry.already_created_count, 2);
    assert_eq!(retry.created_count, 0);
    let ff3_ids: Vec<_> = f
        .creations
        .rows
        .lock()
        .unwrap()
        .iter()
        .map(|c| (c.movement_id.as_i64(), c.ff3_id))
        .collect();
    assert_eq!(ff3_ids, vec![(1, 501), (2, 502)]);
    assert_eq!(f.movements.status_of(1), MovementStatus::Matched);
}

#[tokio::test]
async fn line_already_in_firefly_is_recorded_without_post() {
    let f = fx(vec![]);
    f.firefly.stored.lock().unwrap().extend([
        (7, hit(&format!("{PREFIX}2"), 777)),
        (7, hit("ffm-other-1", 9)),
    ]);

    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 42), ma(2, 42)]))
        .await
        .expect("apply ok");

    let calls = f.firefly.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].external_id, Some(format!("{PREFIX}1")));
    let m2 = result
        .reports
        .iter()
        .find(|r| r.movement_id.as_i64() == 2)
        .unwrap();
    assert_eq!(m2.outcome, ApplyOutcome::AlreadyCreated);
    assert_eq!(m2.ff3_id, Some(777));
    assert_eq!(
        f.creations
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.movement_id.as_i64() == 2)
            .map(|c| c.ff3_id),
        Some(777)
    );
    assert_eq!(f.movements.status_of(2), MovementStatus::Matched);
    let run = &f.runs.saved_runs()[0];
    assert!(run
        .matches
        .iter()
        .any(|m| m.movement_id.as_i64() == 2 && m.ff3_id == 777));
}

#[tokio::test]
async fn search_failure_creates_nothing() {
    let f = fx(vec![]);
    *f.firefly.search_fails.lock().unwrap() = true;

    let err = f
        .uc()
        .execute(f.input(vec![ma(1, 42), ma(2, 42)]))
        .await
        .expect_err("search failure must stop the apply");
    assert!(matches!(err, ApplicationError::FireflyClient(_)), "{err:?}");
    assert!(f.firefly.calls().is_empty());
    assert!(f.creations.rows.lock().unwrap().is_empty());
    assert!(f.runs.saved_runs().is_empty());
}

#[tokio::test]
async fn recovery_with_match_already_in_previous_run_does_not_duplicate_it() {
    // Live case: the previous confirmation run already pairs m1 with tx 777,
    // but the creations row is gone and m1 is back to missing.
    let f = fx(vec![]);
    {
        let mut recs = f.runs.reconciliations.lock().unwrap();
        recs[0].matched_count = 2;
        recs[0].missing_count = 1;
    }
    f.runs.tx_rows.lock().unwrap().push(Ff3Transaction {
        ff3_journal_id: 0,
        id: Ff3TransactionId::new(32),
        reconciliation_id: ReconciliationId::new(10),
        ff3_id: 777,
        date: d(2026, 1, 1),
        amount_cents: AmountCents::new(-5000),
        description: "m1".into(),
        is_duplicate: false,
        created_at: epoch(),
        counterparty_name: Some("Shop".into()),
        counterparty_type: Some("Expense account".into()),
    });
    f.runs.match_rows.lock().unwrap().push(Match {
        id: MatchId::new(2),
        reconciliation_id: ReconciliationId::new(10),
        movement_id: MovementId::new(1),
        ff3_transaction_id: Ff3TransactionId::new(32),
        matched_at: epoch(),
    });
    f.firefly
        .stored
        .lock()
        .unwrap()
        .push((7, hit(&format!("{PREFIX}1"), 777)));

    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 42)]))
        .await
        .expect("an existing match must not abort the apply");

    assert!(f.firefly.calls().is_empty(), "no POST");
    assert_eq!(result.already_created_count, 1);
    assert_eq!(result.failed_count, 0);
    assert_eq!(f.movements.status_of(1), MovementStatus::Matched);
    let run = &f.runs.saved_runs()[0];
    let pairs: Vec<_> = run
        .matches
        .iter()
        .map(|m| (m.movement_id.as_i64(), m.ff3_id))
        .collect();
    assert_eq!(
        pairs.iter().filter(|p| **p == (1, 777)).count(),
        1,
        "{pairs:?}"
    );
    assert!(pairs.contains(&(3, 300)), "{pairs:?}");
    assert_eq!(run.reconciliation.matched_count, 2);
    assert_eq!(run.reconciliation.missing_count, 1);
    let cached: Vec<_> = run
        .ff3_transactions
        .iter()
        .filter(|t| t.ff3_id == 777)
        .collect();
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].counterparty_name.as_deref(), Some("Shop"));
}

fn store(f: &Fx, account: i64, line: i32, ff3_id: i64) {
    f.firefly
        .stored
        .lock()
        .unwrap()
        .push((account, hit(&format!("{PREFIX}{line}"), ff3_id)));
}

#[tokio::test]
async fn line_stored_on_another_account_is_not_recovered() {
    let f = fx(vec![]);
    store(&f, 99, 1, 777);

    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 42)]))
        .await
        .expect("apply ok");

    assert_eq!(
        *f.firefly.searches.lock().unwrap(),
        vec![(7, PREFIX.to_string())]
    );
    assert_eq!(result.created_count, 1);
    assert_eq!(result.already_created_count, 0);
    assert_eq!(f.firefly.calls().len(), 1, "created on the batch account");
}

#[tokio::test]
async fn recovered_tx_matched_to_another_movement_fails_without_recording() {
    let f = fx(vec![]);
    store(&f, 7, 1, 300); // tx 300 is paired with m3 in the previous run

    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 42)]))
        .await
        .expect("apply ok");

    assert!(f.firefly.calls().is_empty(), "no POST");
    assert_eq!(result.failed_count, 1);
    assert_eq!(
        result.reports[0].reason.as_deref(),
        Some("Firefly III tx 300 already matched to movement 3; re-run reconciliation")
    );
    assert!(f.creations.rows.lock().unwrap().is_empty());
    assert_eq!(f.movements.status_of(1), MovementStatus::Missing);
    assert!(f.runs.saved_runs().is_empty());
}

#[tokio::test]
async fn movement_already_paired_with_another_tx_fails_before_post() {
    let f = fx(vec![]);
    f.runs.match_rows.lock().unwrap().push(Match {
        id: MatchId::new(2),
        reconciliation_id: ReconciliationId::new(10),
        movement_id: MovementId::new(1),
        ff3_transaction_id: Ff3TransactionId::new(31), // tx 310
        matched_at: epoch(),
    });

    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 42)]))
        .await
        .expect("apply ok");

    assert!(f.firefly.calls().is_empty(), "no POST");
    assert_eq!(
        result.reports[0].reason.as_deref(),
        Some("movement 1 already matched to Firefly III tx 310; re-run reconciliation")
    );
    assert!(f.runs.saved_runs().is_empty());
}

#[tokio::test]
async fn recovery_record_failure_is_failed_without_post() {
    let f = fx(vec![]);
    store(&f, 7, 1, 777);
    *f.creations.fail_inserts.lock().unwrap() = true;

    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 42), ma(2, 42)]))
        .await
        .expect("a record failure must not abort the batch");

    let m1 = &result.reports[0];
    assert_eq!(m1.outcome, ApplyOutcome::Failed);
    assert_eq!(m1.ff3_id, Some(777));
    assert!(m1.reason.as_deref().unwrap().contains("not recorded"));
    assert_eq!(f.firefly.calls().len(), 1, "only m2 is posted");
    assert_eq!(f.movements.status_of(1), MovementStatus::Missing);
}

#[tokio::test]
async fn no_search_when_every_movement_is_already_created() {
    let f = fx(vec![]);
    let input = f.input(vec![ma(1, 42), ma(2, 42)]);
    f.uc().execute(input.clone()).await.expect("first apply");
    f.firefly.searches.lock().unwrap().clear();
    *f.firefly.search_fails.lock().unwrap() = true;

    let retry = f.uc().execute(input).await.expect("no Firefly III needed");
    assert_eq!(retry.already_created_count, 2);
    assert!(f.firefly.searches.lock().unwrap().is_empty());
}

#[tokio::test]
async fn recovery_reports_that_edits_and_attachment_were_not_applied() {
    let f = fx(vec![]);
    store(&f, 7, 1, 777);
    store(&f, 7, 2, 778);
    let mut edited = ma(1, 42);
    edited.notes = Some("note".into());
    edited.attachment = Some(AttachmentPayload {
        filename: "r.pdf".into(),
        content_base64: "AA==".into(),
    });

    let result = f
        .uc()
        .execute(f.input(vec![edited, ma(2, 42)]))
        .await
        .expect("apply ok");

    assert_eq!(result.already_created_count, 2);
    let reason = result.reports[0].reason.as_deref().unwrap_or("");
    assert!(reason.contains("not applied"), "{reason}");
    assert_eq!(result.reports[1].reason, None, "nothing was sent for m2");
    assert!(f.firefly.attachments.lock().unwrap().is_empty());
}

#[tokio::test]
async fn recovering_a_previous_extra_lowers_extra_count_and_keeps_its_counterparty() {
    let f = fx(vec![]);
    store(&f, 7, 1, 310); // tx 310 is the previous run's unmatched extra

    let result = f
        .uc()
        .execute(f.input(vec![ma(1, 42)]))
        .await
        .expect("apply ok");

    assert_eq!(result.already_created_count, 1);
    let run = &f.runs.saved_runs()[0];
    assert_eq!(run.reconciliation.matched_count, 2);
    assert_eq!(run.reconciliation.missing_count, 1);
    assert_eq!(run.reconciliation.extra_count, 0);
    let tx: Vec<_> = run
        .ff3_transactions
        .iter()
        .filter(|t| t.ff3_id == 310)
        .collect();
    assert_eq!(tx.len(), 1);
    assert_eq!(tx[0].counterparty_name.as_deref(), Some("Shop"));
}

#[tokio::test]
async fn recovered_tx_not_in_cache_gets_counterparty_from_the_search() {
    let f = fx(vec![]);
    store(&f, 7, 1, 777);

    f.uc()
        .execute(f.input(vec![ma(1, 42)]))
        .await
        .expect("apply ok");

    let run = &f.runs.saved_runs()[0];
    let tx = run
        .ff3_transactions
        .iter()
        .find(|t| t.ff3_id == 777)
        .expect("cached");
    assert_eq!(tx.counterparty_name.as_deref(), Some("Shop"));
    assert_eq!(tx.counterparty_type.as_deref(), Some("Expense account"));
}

// ── Counterparty memory (RF-08 v0.11) ───────────────────────────────────

fn learned(f: &Fx) -> Vec<(ffm_application::dtos::CounterpartyMemoryEntry, i64)> {
    f.memory.learned.lock().unwrap().clone()
}

fn cp(ff3_id: Option<i64>, name: Option<&str>, cash: bool) -> CounterpartyRef {
    CounterpartyRef {
        ff3_id,
        name: name.map(Into::into),
        cash,
    }
}

#[tokio::test]
async fn creation_with_an_existing_account_is_remembered_by_id_and_name() {
    let f = fx(vec![]);
    f.uc()
        .execute(f.input(vec![ma(1, 10)]))
        .await
        .expect("apply ok");
    let entries = learned(&f);
    assert_eq!(entries.len(), 1);
    let (e, uses) = &entries[0];
    assert_eq!(e.account_ff3_id, 7);
    assert_eq!(e.direction, Direction::Out);
    assert_eq!(e.desc_key, "m1");
    assert_eq!(e.counterparty, cp(Some(10), Some("acc 10"), false));
    assert_eq!(*uses, 1);
}

#[tokio::test]
async fn creation_by_new_name_is_remembered_with_the_id_firefly_returned() {
    let f = fx(vec![]);
    // Money out: Firefly III created the destination (id 80 in the fake).
    f.uc()
        .execute(f.input(vec![named(1, " Corner Shop ")]))
        .await
        .expect("apply ok");
    assert_eq!(
        learned(&f)[0].0.counterparty,
        cp(Some(80), Some("Corner Shop"), false)
    );
}

#[tokio::test]
async fn creation_with_cash_is_remembered_as_cash_for_money_in() {
    let f = fx(vec![]);
    let explicit_cash = MovementApply {
        counterparty_cash: true,
        ..ma_opt(2, None)
    };
    f.uc()
        .execute(f.input(vec![explicit_cash]))
        .await
        .expect("apply ok");
    let (e, _) = &learned(&f)[0];
    assert_eq!(e.direction, Direction::In);
    assert_eq!(e.counterparty, cp(None, None, true));
}

#[tokio::test]
async fn failed_already_created_or_recovered_movements_teach_nothing() {
    let f = fx(vec![-5000]);
    // m1 fails in Firefly III.
    f.uc()
        .execute(f.input(vec![ma(1, 10)]))
        .await
        .expect("apply ok");
    assert!(learned(&f).is_empty());

    // m2 is created once (learns), then retried (already created: no new use).
    f.uc()
        .execute(f.input(vec![ma(2, 20)]))
        .await
        .expect("apply ok");
    f.uc()
        .execute(f.input(vec![ma(2, 20)]))
        .await
        .expect("apply ok");
    assert_eq!(learned(&f).len(), 1);
    assert_eq!(learned(&f)[0].1, 1);

    // A line recovered by external_id teaches nothing either.
    let g = fx(vec![]);
    g.firefly
        .stored
        .lock()
        .unwrap()
        .push((7, hit(&format!("{PREFIX}1"), 900)));
    g.uc()
        .execute(g.input(vec![ma(1, 10)]))
        .await
        .expect("apply ok");
    assert!(learned(&g).is_empty());
}

#[tokio::test]
async fn no_counterparty_is_created_as_cash_but_teaches_nothing() {
    let f = fx(vec![]);
    let result = f
        .uc()
        .execute(f.input(vec![ma_opt(1, None)]))
        .await
        .expect("apply ok");
    assert_eq!(result.reports[0].outcome, ApplyOutcome::Created);
    let call = &f.firefly.calls()[0];
    assert_eq!(call.transaction_type, "withdrawal");
    assert_eq!(
        (call.destination_ff3_id, call.destination_name.as_deref()),
        (None, None)
    );
    assert!(learned(&f).is_empty());
}

#[tokio::test]
async fn explicit_cash_with_an_id_or_a_name_fails_that_movement() {
    let f = fx(vec![]);
    let with_id = MovementApply {
        counterparty_cash: true,
        ..ma(1, 10)
    };
    let with_name = MovementApply {
        counterparty_cash: true,
        ..named(2, "Employer")
    };
    let result = f
        .uc()
        .execute(f.input(vec![with_id, with_name]))
        .await
        .expect("apply ok");
    assert!(result
        .reports
        .iter()
        .all(|r| r.outcome == ApplyOutcome::Failed
            && r.reason.as_deref().unwrap().contains("counterparty_cash")));
    assert!(f.firefly.calls().is_empty());
    assert!(learned(&f).is_empty());
}

struct FailingMemory;

#[async_trait]
impl CounterpartySuggestionRepository for FailingMemory {
    async fn remember(
        &self,
        _entry: ffm_application::dtos::CounterpartyMemoryEntry,
    ) -> Result<(), ApplicationError> {
        Err(ApplicationError::Database("disk full".into()))
    }
    async fn learned(
        &self,
        _account_ff3_id: i64,
    ) -> Result<Vec<ffm_application::dtos::CounterpartyMemoryEntry>, ApplicationError> {
        unimplemented!()
    }
    async fn save_history(
        &self,
        _batch_id: BatchId,
        _suggestions: Vec<(MovementId, CounterpartyRef)>,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn history_by_batch(
        &self,
        _batch_id: BatchId,
    ) -> Result<std::collections::HashMap<MovementId, CounterpartyRef>, ApplicationError> {
        unimplemented!()
    }
}

#[tokio::test]
async fn a_failing_memory_still_creates_and_reports_the_warning() {
    let f = fx(vec![]);
    let result = f
        .uc_with(FailingMemory)
        .execute(f.input(vec![ma(1, 10)]))
        .await
        .expect("apply ok");
    let report = &result.reports[0];
    assert_eq!(report.outcome, ApplyOutcome::Created);
    assert!(report
        .reason
        .as_deref()
        .unwrap()
        .contains("counterparty not remembered: "));
    assert_eq!(result.created_count, 1);
}
