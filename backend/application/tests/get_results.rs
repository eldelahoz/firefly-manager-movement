use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use ffm_application::dtos::{
    CounterpartyMemoryEntry, CounterpartyRef, NewBatch, NewFf3Account, SuggestedCounterparty,
    SuggestionSource,
};
use ffm_application::dtos::{NewFf3Transaction, NewMatch, NewReconciliation, ReconciliationRun};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    BatchRepository, CounterpartySuggestionRepository, DeletedAuditRepository,
    Ff3AccountRepository, Ff3TransactionRepository, InMemoryCounterpartySuggestions,
    MatchRepository, MovementRepository, ReconciliationRepository,
};
use ffm_application::use_cases::get_results::{GetResults, SuggestionPorts};
use ffm_domain::entities::{Batch, Ff3Account};
use ffm_domain::ids::Ff3AccountId;
use ffm_domain::transaction_kind::Direction;
use ffm_domain::value_objects::BatchStatus;

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

/// Batch 1 on local account 1 (Firefly III account 7), unless `account_id` is cleared.
struct FakeBatchRepo {
    account_id: Mutex<Option<Ff3AccountId>>,
}

#[async_trait]
impl BatchRepository for FakeBatchRepo {
    async fn insert(&self, _batch: NewBatch) -> Result<BatchId, ApplicationError> {
        unimplemented!()
    }
    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError> {
        Ok(Batch {
            id,
            filename: "stmt.csv".into(),
            file_hash: "h".into(),
            format: "csv".into(),
            currency: "EUR".into(),
            account_id: *self.account_id.lock().unwrap(),
            status: BatchStatus::Reconciled,
            headers: vec![],
            detected_delimiter: ",".into(),
            created_at: epoch(),
        })
    }
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        unimplemented!()
    }
    async fn update_status(&self, _id: BatchId, _s: BatchStatus) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn update_account(
        &self,
        _id: BatchId,
        _account_id: Ff3AccountId,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }
}

/// Account cache: local 1 = Firefly III 7 (the batch account); 10 and 20 known.
struct FakeAccounts;

fn account(ff3_id: i64) -> Ff3Account {
    Ff3Account {
        id: Ff3AccountId::new(if ff3_id == 7 { 1 } else { ff3_id + 100 }),
        ff3_id,
        name: format!("current {ff3_id}"),
        account_type: "expense".into(),
        currency: "EUR".into(),
        iban: None,
        account_number: None,
        synced_at: epoch(),
    }
}

#[async_trait]
impl Ff3AccountRepository for FakeAccounts {
    async fn upsert_batch(&self, _a: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        unimplemented!()
    }
    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        Ok([7, 10, 20].contains(&ff3_id).then(|| account(ff3_id)))
    }
    async fn by_id(&self, id: Ff3AccountId) -> Result<Option<Ff3Account>, ApplicationError> {
        Ok((id == Ff3AccountId::new(1)).then(|| account(7)))
    }
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        unimplemented!()
    }
}

// ── Fixture ─────────────────────────────────────────────────────────────

struct Fx {
    rec: FakeReconciliationRepo,
    txs: FakeFf3TxRepo,
    matches: FakeMatchRepo,
    movements: FakeMovementRepo,
    audits: FakeDeletedAuditRepo,
    batches: FakeBatchRepo,
    suggestions: std::sync::Arc<InMemoryCounterpartySuggestions>,
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
        ff3_journal_id: 0,
        id: Ff3TransactionId::new(local_id),
        reconciliation_id: ReconciliationId::new(10),
        ff3_id,
        date: d(2026, 1, 10),
        amount_cents: AmountCents::new(1000 * ff3_id),
        description: format!("tx{ff3_id}"),
        is_duplicate: dup,
        created_at: epoch(),
        counterparty_name: None,
        counterparty_type: None,
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
        batches: FakeBatchRepo {
            account_id: Mutex::new(Some(Ff3AccountId::new(1))),
        },
        suggestions: Default::default(),
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
        FakeBatchRepo,
        FakeAccounts,
        std::sync::Arc<InMemoryCounterpartySuggestions>,
    > {
        GetResults::new(
            self.rec,
            self.matches,
            self.txs,
            self.movements,
            self.audits,
            SuggestionPorts {
                batches: self.batches,
                accounts: FakeAccounts,
                suggestions: self.suggestions,
            },
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

// ── Counterparty suggestions (RF-08 v0.11) ──────────────────────────────

fn cp(ff3_id: Option<i64>, name: Option<&str>, cash: bool) -> CounterpartyRef {
    CounterpartyRef {
        ff3_id,
        name: name.map(Into::into),
        cash,
    }
}

impl Fx {
    async fn learn(&self, account: i64, direction: Direction, key: &str, c: CounterpartyRef) {
        self.suggestions
            .remember(CounterpartyMemoryEntry {
                account_ff3_id: account,
                direction,
                desc_key: key.into(),
                counterparty: c,
            })
            .await
            .unwrap();
    }
    async fn history(&self, movement: i64, c: CounterpartyRef) {
        let mut all = self
            .suggestions
            .history_by_batch(BatchId::new(1))
            .await
            .unwrap();
        all.insert(MovementId::new(movement), c);
        self.suggestions
            .save_history(BatchId::new(1), all.into_iter().collect())
            .await
            .unwrap();
    }
}

fn suggested(c: CounterpartyRef, source: SuggestionSource) -> SuggestedCounterparty {
    SuggestedCounterparty {
        counterparty: c,
        source,
    }
}

// Movement 2 is the missing one: money in, description "m2".

#[tokio::test]
async fn learned_suggestion_wins_over_history() {
    let f = run_fixture();
    f.learn(
        7,
        Direction::In,
        "m2",
        cp(Some(10), Some("Acme Motors"), false),
    )
    .await;
    f.history(2, cp(Some(20), Some("Employer"), false)).await;
    let r = f.uc().execute(BatchId::new(1)).await.unwrap();
    assert_eq!(
        r.suggestions[&MovementId::new(2)],
        suggested(
            cp(Some(10), Some("Acme Motors"), false),
            SuggestionSource::Learned
        )
    );
}

#[tokio::test]
async fn history_is_used_when_nothing_is_learned_for_that_account_and_direction() {
    let f = run_fixture();
    f.learn(7, Direction::Out, "m2", cp(Some(10), Some("x"), false))
        .await;
    f.learn(8, Direction::In, "m2", cp(Some(10), Some("x"), false))
        .await;
    f.history(2, cp(Some(20), Some("Employer"), false)).await;
    let r = f.uc().execute(BatchId::new(1)).await.unwrap();
    assert_eq!(
        r.suggestions[&MovementId::new(2)],
        suggested(
            cp(Some(20), Some("Employer"), false),
            SuggestionSource::History
        )
    );
}

/// Never downgraded to "create by name": an id not in the account cache is
/// returned as stored, and apply rejects it if the account is really gone.
#[tokio::test]
async fn ids_missing_from_the_account_cache_are_kept_with_their_name() {
    let f = run_fixture();
    f.movements
        .rows
        .lock()
        .unwrap()
        .push(movement(4, MovementStatus::Missing));
    f.learn(
        7,
        Direction::In,
        "m2",
        cp(Some(99), Some("Car Loan"), false),
    )
    .await;
    f.history(4, cp(Some(98), Some("Old Loan"), false)).await;
    let r = f.uc().execute(BatchId::new(1)).await.unwrap();
    assert_eq!(
        r.suggestions[&MovementId::new(2)],
        suggested(
            cp(Some(99), Some("Car Loan"), false),
            SuggestionSource::Learned
        )
    );
    assert_eq!(
        r.suggestions[&MovementId::new(4)],
        suggested(
            cp(Some(98), Some("Old Loan"), false),
            SuggestionSource::History
        )
    );
}

#[tokio::test]
async fn learned_cash_and_name_only_are_suggested_as_is() {
    let f = run_fixture();
    f.movements
        .rows
        .lock()
        .unwrap()
        .push(movement(4, MovementStatus::Missing));
    f.learn(7, Direction::In, "m2", cp(None, None, true)).await;
    f.learn(7, Direction::In, "m4", cp(None, Some("Corner Shop"), false))
        .await;
    let r = f.uc().execute(BatchId::new(1)).await.unwrap();
    assert_eq!(
        r.suggestions[&MovementId::new(2)],
        suggested(cp(None, None, true), SuggestionSource::Learned)
    );
    assert_eq!(
        r.suggestions[&MovementId::new(4)],
        suggested(
            cp(None, Some("Corner Shop"), false),
            SuggestionSource::Learned
        )
    );
}

#[tokio::test]
async fn no_learned_entry_and_no_history_gives_no_suggestion() {
    let f = run_fixture();
    // Learned for the matched movement's description only.
    f.learn(7, Direction::In, "m1", cp(Some(10), None, false))
        .await;
    let r = f.uc().execute(BatchId::new(1)).await.unwrap();
    assert!(r.suggestions.is_empty(), "{:?}", r.suggestions);
}

#[tokio::test]
async fn batch_without_account_gives_no_suggestion() {
    let f = run_fixture();
    *f.batches.account_id.lock().unwrap() = None;
    f.learn(7, Direction::In, "m2", cp(Some(10), None, false))
        .await;
    let r = f.uc().execute(BatchId::new(1)).await.unwrap();
    assert!(r.suggestions.is_empty());
}
