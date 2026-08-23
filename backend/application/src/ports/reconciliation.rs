use async_trait::async_trait;

use ffm_domain::entities::{Creation, Ff3Transaction, Match, Reconciliation};
use ffm_domain::ids::{BatchId, CreationId, MovementId, ReconciliationId};

use crate::dtos::{NewCreation, NewFf3Transaction, NewMatch, NewReconciliation, ReconciliationRun};
use crate::error::ApplicationError;

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
    /// Persists a whole rapprochement run atomically (R8): the
    /// reconciliation snapshot, the per-run FF3 transaction cache, the
    /// 1-to-1 matches and the movement status updates.
    async fn save_run(&self, run: ReconciliationRun) -> Result<ReconciliationId, ApplicationError>;
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
    /// Most recent cached row for an external FF3 transaction id
    /// (point-in-time snapshot source for the delete audit, RF-15).
    async fn latest_by_ff3_id(
        &self,
        ff3_id: i64,
    ) -> Result<Option<Ff3Transaction>, ApplicationError>;
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
