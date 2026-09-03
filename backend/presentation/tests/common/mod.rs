#![allow(dead_code)]
//! Shared test fakes for presentation E2E tests.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};

use ffm_application::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewFf3Account, NewFf3TransactionCreation,
};
use ffm_application::dtos::{NewFf3Transaction, NewMatch, NewReconciliation, ReconciliationRun};
use ffm_application::error::ApplicationError;
use ffm_application::ports::{
    AppSettingsRepository, BatchRepository, CsvDecoder, CsvMappingRepository, Ff3AccountRepository,
    Ff3TransactionRepository, FireflyClient, FireflyClientFactory, FireflyConnectionManager,
    MatchRepository, MovementRepository, ReconciliationRepository,
};
use ffm_application::dtos::NewAppSettings;
use ffm_domain::entities::{AppSettings, Ff3Account};
use ffm_domain::entities::{Batch, CsvMapping, Ff3Transaction, Match, Movement, Reconciliation};
use ffm_domain::ids::{BatchId, CsvMappingId, MovementId, ReconciliationId};
use ffm_domain::value_objects::{BatchStatus, HeaderSignature, MovementStatus};

use ffm_application::dtos::{NewBatch, NewCsvMapping, NewMovement};

// ── FakeFireflyClient ──────────────────────────────────────────────────

#[derive(Clone)]
pub struct FakeFireflyClient {
    pub fail: bool,
    pub accounts: Vec<Ff3AccountDto>,
    pub transactions: Vec<Ff3TransactionDto>,
}

impl FakeFireflyClient {
    #[allow(dead_code)]
    pub fn ok() -> Self {
        Self {
            fail: false,
            accounts: vec![],
            transactions: vec![],
        }
    }

    pub fn failing() -> Self {
        Self {
            fail: true,
            accounts: vec![],
            transactions: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn with_accounts(accounts: Vec<Ff3AccountDto>) -> Self {
        Self {
            fail: false,
            accounts,
            transactions: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn with_transactions(transactions: Vec<Ff3TransactionDto>) -> Self {
        Self {
            fail: false,
            accounts: vec![],
            transactions,
        }
    }
}

#[async_trait]
impl FireflyClient for FakeFireflyClient {
    async fn about(&self) -> Result<About, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(About {
                version: "6.1.36".into(),
            })
        }
    }

    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(self.accounts.clone())
        }
    }

    async fn account_transactions(
        &self,
        _account_ff3_id: i64,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(self.transactions.clone())
        }
    }

    async fn create_transaction(
        &self,
        _creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(100)
        }
    }

    async fn delete_transaction(&self, _ff3_id: i64) -> Result<(), ApplicationError> {
        if self.fail {
            Err(ApplicationError::FireflyClient("FF3 unreachable".into()))
        } else {
            Ok(())
        }
    }
}

// ── FakeFf3AccountRepository ───────────────────────────────────────────

pub struct FakeFf3AccountRepository {
    upserted: std::sync::Mutex<Vec<NewFf3Account>>,
}

impl FakeFf3AccountRepository {
    pub fn new() -> Self {
        Self {
            upserted: std::sync::Mutex::new(vec![]),
        }
    }

    #[allow(dead_code)]
    pub fn upserted_count(&self) -> usize {
        self.upserted.lock().unwrap().len()
    }
}

impl Default for FakeFf3AccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Ff3AccountRepository for FakeFf3AccountRepository {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        self.upserted.lock().unwrap().extend(accounts);
        Ok(())
    }

    async fn by_ff3_id(&self, _ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        Ok(None)
    }
    async fn by_id(
        &self,
        _id: ffm_domain::ids::Ff3AccountId,
    ) -> Result<Option<Ff3Account>, ApplicationError> {
        Ok(None)
    }

    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        Ok(vec![])
    }
}

// ── No-op fakes for imports (used by tests that don't touch import endpoints) ─

pub struct FakeCsvDecoder;

#[async_trait]
impl CsvDecoder for FakeCsvDecoder {
    async fn decode(&self, bytes: Vec<u8>) -> Result<String, ApplicationError> {
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

pub struct FakeBatchRepository;

#[async_trait]
impl BatchRepository for FakeBatchRepository {
    async fn insert(&self, _batch: NewBatch) -> Result<BatchId, ApplicationError> {
        Ok(BatchId::new(1))
    }
    async fn get(&self, _id: BatchId) -> Result<Batch, ApplicationError> {
        Err(ApplicationError::NotFound("batch".into()))
    }
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError> {
        Ok(vec![])
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

pub struct FakeMovementRepository;

#[async_trait]
impl MovementRepository for FakeMovementRepository {
    async fn insert_batch(&self, _movements: Vec<NewMovement>) -> Result<(), ApplicationError> {
        Ok(())
    }
    async fn by_batch(&self, _batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError> {
        Ok(vec![])
    }
    async fn update_status(
        &self,
        _id: MovementId,
        _status: MovementStatus,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}

pub struct FakeCsvMappingRepository;

#[async_trait]
impl CsvMappingRepository for FakeCsvMappingRepository {
    async fn by_signature(
        &self,
        _signature: HeaderSignature,
    ) -> Result<Option<CsvMapping>, ApplicationError> {
        Ok(None)
    }
    async fn upsert(&self, _mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError> {
        Ok(CsvMappingId::new(1))
    }
}

// ── No-op fakes for apply/delete (tests that don't exercise them) ──────

pub struct FakeCreationRepository;

#[async_trait]
impl ffm_application::ports::CreationRepository for FakeCreationRepository {
    async fn insert(
        &self,
        _creation: ffm_application::dtos::NewCreation,
    ) -> Result<ffm_domain::ids::CreationId, ApplicationError> {
        Ok(ffm_domain::ids::CreationId::new(1))
    }
    async fn by_movement(
        &self,
        _movement_id: MovementId,
    ) -> Result<Option<ffm_domain::entities::Creation>, ApplicationError> {
        Ok(None)
    }
    async fn by_batch(
        &self,
        _batch_id: BatchId,
    ) -> Result<Vec<ffm_domain::entities::Creation>, ApplicationError> {
        Ok(vec![])
    }
}

#[derive(Default)]
pub struct FakeDeletedAuditRepository {
    pub rows: std::sync::Mutex<Vec<ffm_application::dtos::NewDeletedAudit>>,
}

#[async_trait]
impl ffm_application::ports::DeletedAuditRepository for FakeDeletedAuditRepository {
    async fn insert(
        &self,
        audit: ffm_application::dtos::NewDeletedAudit,
    ) -> Result<ffm_domain::ids::DeletedAuditId, ApplicationError> {
        self.rows.lock().unwrap().push(audit);
        Ok(ffm_domain::ids::DeletedAuditId::new(1))
    }
}

// ── FakeReconciliationStore (F4: reconcile + results) ──────────────────

fn test_epoch() -> NaiveDateTime {
    chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0)
        .expect("epoch")
        .naive_utc()
}

/// In-memory store implementing the three run-related ports. Mirrors the
/// MySQL adapter: `save_run` splits the run into `ff3_transactions` rows
/// (assigning local ids) + `matches` rows, so `GetResults` can read them.
#[derive(Default)]
pub struct FakeReconciliationStore {
    reconciliations: std::sync::Mutex<Vec<Reconciliation>>,
    tx_rows: std::sync::Mutex<Vec<Ff3Transaction>>,
    match_rows: std::sync::Mutex<Vec<Match>>,
    next_recon_id: std::sync::Mutex<i64>,
    next_tx_id: std::sync::Mutex<i64>,
    next_match_id: std::sync::Mutex<i64>,
    /// When set, movement status updates are applied (atomic run).
    pub movements: Option<std::sync::Arc<dyn MovementRepository>>,
}

impl FakeReconciliationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_movements(movements: std::sync::Arc<dyn MovementRepository>) -> Self {
        Self {
            movements: Some(movements),
            ..Self::default()
        }
    }

    pub fn run_count(&self) -> usize {
        self.reconciliations.lock().unwrap().len()
    }
}

#[async_trait]
impl ReconciliationRepository for FakeReconciliationStore {
    async fn insert(
        &self,
        _reconciliation: NewReconciliation,
    ) -> Result<ReconciliationId, ApplicationError> {
        unimplemented!("F4 uses save_run")
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
        let recon_id = {
            let mut id = self.next_recon_id.lock().unwrap();
            *id += 1;
            ReconciliationId::new(*id)
        };

        self.reconciliations.lock().unwrap().push(Reconciliation {
            id: recon_id,
            batch_id: run.reconciliation.batch_id,
            tolerance_days: run.reconciliation.tolerance_days,
            window_start: run.reconciliation.window_start,
            window_end: run.reconciliation.window_end,
            matched_count: run.reconciliation.matched_count,
            missing_count: run.reconciliation.missing_count,
            extra_count: run.reconciliation.extra_count,
            ran_at: test_epoch(),
        });

        let mut ff3_local: HashMap<i64, i64> = HashMap::new();
        {
            let mut next_tx = self.next_tx_id.lock().unwrap();
            for t in &run.ff3_transactions {
                *next_tx += 1;
                ff3_local.insert(t.ff3_id, *next_tx);
                self.tx_rows.lock().unwrap().push(Ff3Transaction {
                    id: ffm_domain::ids::Ff3TransactionId::new(*next_tx),
                    reconciliation_id: recon_id,
                    ff3_id: t.ff3_id,
                    date: t.date,
                    amount_cents: t.amount_cents,
                    description: t.description.clone(),
                    is_duplicate: t.is_duplicate,
                    created_at: test_epoch(),
                });
            }
        }

        {
            let mut next_match = self.next_match_id.lock().unwrap();
            for m in &run.matches {
                *next_match += 1;
                self.match_rows.lock().unwrap().push(Match {
                    id: ffm_domain::ids::MatchId::new(*next_match),
                    reconciliation_id: recon_id,
                    movement_id: m.movement_id,
                    ff3_transaction_id: ffm_domain::ids::Ff3TransactionId::new(
                        ff3_local[&m.ff3_id],
                    ),
                    matched_at: test_epoch(),
                });
            }
        }

        if let Some(movements) = &self.movements {
            for (movement_id, status) in &run.movement_status_updates {
                movements.update_status(*movement_id, *status).await?;
            }
        }

        Ok(recon_id)
    }
}

#[async_trait]
impl Ff3TransactionRepository for FakeReconciliationStore {
    async fn insert_batch(
        &self,
        _transactions: Vec<NewFf3Transaction>,
    ) -> Result<(), ApplicationError> {
        unimplemented!("F4 uses save_run")
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
        ff3_ids: &[i64],
    ) -> Result<Vec<Ff3Transaction>, ApplicationError> {
        let rows = self.tx_rows.lock().unwrap();
        let mut latest = std::collections::HashMap::new();
        for t in rows.iter() {
            if ff3_ids.contains(&t.ff3_id) {
                latest.insert(t.ff3_id, t.clone());
            }
        }
        Ok(latest.into_values().collect())
    }
}

#[async_trait]
impl MatchRepository for FakeReconciliationStore {
    async fn insert_batch(&self, _matches: Vec<NewMatch>) -> Result<(), ApplicationError> {
        unimplemented!("F4 uses save_run")
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
    async fn matched_ff3_ids(&self, ff3_ids: &[i64]) -> Result<HashSet<i64>, ApplicationError> {
        let rows = self.tx_rows.lock().unwrap();
        let local_to_ff3: HashMap<i64, i64> =
            rows.iter().map(|t| (t.id.as_i64(), t.ff3_id)).collect();
        let wanted: HashSet<i64> = ff3_ids.iter().copied().collect();
        Ok(self
            .match_rows
            .lock()
            .unwrap()
            .iter()
            .filter_map(|m| local_to_ff3.get(&m.ff3_transaction_id.as_i64()).copied())
            .filter(|ff3_id| wanted.contains(ff3_id))
            .collect())
    }
}

// ── FakeAppSettingsRepository ──────────────────────────────────────────

pub struct FakeAppSettingsRepository {
    settings: std::sync::Mutex<Option<AppSettings>>,
}

impl FakeAppSettingsRepository {
    pub fn new() -> Self {
        Self {
            settings: std::sync::Mutex::new(None),
        }
    }

    pub fn with_settings(s: AppSettings) -> Self {
        Self {
            settings: std::sync::Mutex::new(Some(s)),
        }
    }
}

impl Default for FakeAppSettingsRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AppSettingsRepository for FakeAppSettingsRepository {
    async fn get(&self) -> Result<Option<AppSettings>, ApplicationError> {
        Ok(self.settings.lock().unwrap().clone())
    }

    async fn save(&self, s: NewAppSettings) -> Result<AppSettings, ApplicationError> {
        let row = AppSettings {
            ff3_url: s.ff3_url,
            ff3_token: s.ff3_token,
            default_creation_tag: s.default_creation_tag,
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        };
        *self.settings.lock().unwrap() = Some(row.clone());
        Ok(row)
    }
}

// ── FakeFireflyConnectionManager ───────────────────────────────────────

#[derive(Default, Clone)]
pub struct FakeFireflyConnectionManager {
    pub calls: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>,
}

#[async_trait]
impl FireflyConnectionManager for FakeFireflyConnectionManager {
    async fn reconfigure(&self, ff3_url: &str, ff3_token: &str) -> Result<(), ApplicationError> {
        self.calls.lock().unwrap().push((ff3_url.to_string(), ff3_token.to_string()));
        Ok(())
    }
}

// ── FakeFireflyClientFactory ───────────────────────────────────────────

pub struct FakeFireflyClientFactory {
    pub client: FakeFireflyClient,
}

impl FakeFireflyClientFactory {
    pub fn new(client: FakeFireflyClient) -> Self {
        Self { client }
    }
}

impl FireflyClientFactory for FakeFireflyClientFactory {
    fn build(&self, _ff3_url: &str, _ff3_token: &str) -> Arc<dyn FireflyClient> {
        Arc::new(FakeFireflyClient {
            fail: self.client.fail,
            accounts: self.client.accounts.clone(),
            transactions: self.client.transactions.clone(),
        })
    }
}
