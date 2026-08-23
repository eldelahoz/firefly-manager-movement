//! DeleteFireflyTransaction use case (RF-15).
//!
//! Builds a point-in-time JSON snapshot from the latest cached run row
//! of the FF3 transaction, deletes it in Firefly III, then records the
//! audit row. Snapshot before delete, audit only after a successful
//! delete.

use serde_json::json;

use crate::dtos::NewDeletedAudit;
use crate::error::ApplicationError;
use crate::ports::{DeletedAuditRepository, Ff3TransactionRepository, FireflyClient};

pub struct DeleteFireflyTransaction<F, T, A> {
    firefly: F,
    ff3_transactions: T,
    deleted_audits: A,
}

impl<F, T, A> DeleteFireflyTransaction<F, T, A> {
    pub fn new(firefly: F, ff3_transactions: T, deleted_audits: A) -> Self {
        Self {
            firefly,
            ff3_transactions,
            deleted_audits,
        }
    }
}

impl<F, T, A> DeleteFireflyTransaction<F, T, A>
where
    F: FireflyClient,
    T: Ff3TransactionRepository,
    A: DeletedAuditRepository,
{
    pub async fn execute(&self, ff3_id: i64) -> Result<(), ApplicationError> {
        let tx = self
            .ff3_transactions
            .latest_by_ff3_id(ff3_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::NotFound(format!(
                    "FF3 transaction {ff3_id} not found in any run cache (reconcile first)"
                ))
            })?;

        // Point-in-time snapshot of what is about to be deleted
        // (denormalized on purpose, TRD §2).
        let snapshot_json = json!({
            "ff3_id": tx.ff3_id,
            "reconciliation_id": tx.reconciliation_id.as_i64(),
            "date": tx.date.format("%Y-%m-%d").to_string(),
            "amount_cents": tx.amount_cents.as_i64(),
            "description": tx.description,
            "is_duplicate": tx.is_duplicate,
        })
        .to_string();

        self.firefly.delete_transaction(ff3_id).await?;

        self.deleted_audits
            .insert(NewDeletedAudit {
                ff3_id,
                snapshot_json,
            })
            .await?;

        Ok(())
    }
}
