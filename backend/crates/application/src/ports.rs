//! Application layer ports (traits).
//!
//! Infrastructure adapters implement these traits. Use cases depend
//! on the interfaces, not on concrete implementations
//! (Dependency Inversion).

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};

use ffm_domain::entities::{
    Batch, Creation, CsvMapping, Ff3Account, Ff3Transaction, Match, Movement, Reconciliation,
};
use ffm_domain::ids::{
    BatchId, CreationId, CsvMappingId, DeletedAuditId, Ff3AccountId, MovementId, ReconciliationId,
};
use ffm_domain::value_objects::{BatchStatus, HeaderSignature, MovementStatus};

use crate::dtos::{
    About, Ff3AccountDto, Ff3TransactionDto, NewBatch, NewCreation, NewCsvMapping, NewDeletedAudit,
    NewFf3Account, NewFf3Transaction, NewFf3TransactionCreation, NewMatch, NewMovement,
    NewReconciliation,
};
use crate::error::ApplicationError;

#[async_trait]
pub trait BatchRepository: Send + Sync {
    async fn insert(&self, batch: NewBatch) -> Result<BatchId, ApplicationError>;
    async fn get(&self, id: BatchId) -> Result<Batch, ApplicationError>;
    async fn list(&self) -> Result<Vec<Batch>, ApplicationError>;
    async fn update_status(&self, id: BatchId, status: BatchStatus)
        -> Result<(), ApplicationError>;
    async fn update_account(
        &self,
        id: BatchId,
        account_id: Ff3AccountId,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait MovementRepository: Send + Sync {
    async fn insert_batch(&self, movements: Vec<NewMovement>) -> Result<(), ApplicationError>;
    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Movement>, ApplicationError>;
    async fn update_status(
        &self,
        id: MovementId,
        status: MovementStatus,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait ReconciliationRepository: Send + Sync {
    async fn insert(
        &self,
        reconciliation: NewReconciliation,
    ) -> Result<ReconciliationId, ApplicationError>;
    async fn latest_by_batch(
        &self,
        batch_id: BatchId,
    ) -> Result<Option<Reconciliation>, ApplicationError>;
}

#[async_trait]
pub trait Ff3TransactionRepository: Send + Sync {
    async fn insert_batch(
        &self,
        transactions: Vec<NewFf3Transaction>,
    ) -> Result<(), ApplicationError>;
    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Ff3Transaction>, ApplicationError>;
}

#[async_trait]
pub trait MatchRepository: Send + Sync {
    async fn insert_batch(&self, matches: Vec<NewMatch>) -> Result<(), ApplicationError>;
    async fn by_reconciliation(
        &self,
        reconciliation_id: ReconciliationId,
    ) -> Result<Vec<Match>, ApplicationError>;
}

#[async_trait]
pub trait CreationRepository: Send + Sync {
    async fn insert(&self, creation: NewCreation) -> Result<CreationId, ApplicationError>;
    async fn by_movement(
        &self,
        movement_id: MovementId,
    ) -> Result<Option<Creation>, ApplicationError>;
    async fn by_batch(&self, batch_id: BatchId) -> Result<Vec<Creation>, ApplicationError>;
}

#[async_trait]
pub trait Ff3AccountRepository: Send + Sync {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError>;
    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError>;
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError>;
}

#[async_trait]
pub trait CsvMappingRepository: Send + Sync {
    async fn by_signature(
        &self,
        signature: HeaderSignature,
    ) -> Result<Option<CsvMapping>, ApplicationError>;
    async fn upsert(&self, mapping: NewCsvMapping) -> Result<CsvMappingId, ApplicationError>;
}

#[async_trait]
pub trait DeletedAuditRepository: Send + Sync {
    async fn insert(&self, audit: NewDeletedAudit) -> Result<DeletedAuditId, ApplicationError>;
}

/// Port for the Firefly III HTTP client (API v1).
#[async_trait]
pub trait FireflyClient: Send + Sync {
    async fn about(&self) -> Result<About, ApplicationError>;
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError>;
    async fn account_transactions(
        &self,
        account_ff3_id: i64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError>;
    async fn create_transaction(
        &self,
        creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError>;
    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError>;
}

/// Clock for testable time logic.
pub trait Clock: Send + Sync {
    fn now(&self) -> NaiveDateTime;
}

/// File decoder (encoding + delimiter) — infra detail.
#[async_trait]
pub trait CsvDecoder: Send + Sync {
    async fn decode(&self, bytes: Vec<u8>) -> Result<String, ApplicationError>;
}

// ── Blanket impls so Arc<dyn Trait> can be used in use cases ─────────────

use std::sync::Arc;

#[async_trait]
impl FireflyClient for Arc<dyn FireflyClient> {
    async fn about(&self) -> Result<About, ApplicationError> {
        self.as_ref().about().await
    }
    async fn asset_accounts(&self) -> Result<Vec<Ff3AccountDto>, ApplicationError> {
        self.as_ref().asset_accounts().await
    }
    async fn account_transactions(
        &self,
        account_ff3_id: i64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Ff3TransactionDto>, ApplicationError> {
        self.as_ref()
            .account_transactions(account_ff3_id, start, end)
            .await
    }
    async fn create_transaction(
        &self,
        creation: NewFf3TransactionCreation,
    ) -> Result<i64, ApplicationError> {
        self.as_ref().create_transaction(creation).await
    }
    async fn delete_transaction(&self, ff3_id: i64) -> Result<(), ApplicationError> {
        self.as_ref().delete_transaction(ff3_id).await
    }
}

#[async_trait]
impl Ff3AccountRepository for Arc<dyn Ff3AccountRepository> {
    async fn upsert_batch(&self, accounts: Vec<NewFf3Account>) -> Result<(), ApplicationError> {
        self.as_ref().upsert_batch(accounts).await
    }
    async fn by_ff3_id(&self, ff3_id: i64) -> Result<Option<Ff3Account>, ApplicationError> {
        self.as_ref().by_ff3_id(ff3_id).await
    }
    async fn all_asset(&self) -> Result<Vec<Ff3Account>, ApplicationError> {
        self.as_ref().all_asset().await
    }
}
