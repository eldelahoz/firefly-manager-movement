//! GetResults use case (RF-07).
//!
//! Reads the latest reconciliation of the batch and returns the three
//! result groups: matched (pairs), missing (movements), extra
//! (unmatched FF3 transactions, duplicates flagged with `is_duplicate`).

use std::collections::HashSet;

use ffm_domain::entities::{Ff3Transaction, Movement};
use ffm_domain::ids::{BatchId, ReconciliationId};
use ffm_domain::value_objects::MovementStatus;

use crate::error::ApplicationError;
use crate::ports::{
    DeletedAuditRepository, Ff3TransactionRepository, MatchRepository, MovementRepository,
    ReconciliationRepository,
};

/// A matched pair: the batch movement and the FF3 transaction it
/// paired with in the run.
#[derive(Debug)]
pub struct MatchedPair {
    pub movement: Movement,
    pub ff3: Ff3Transaction,
}

/// Result of the latest run: three groups plus the ignored movements.
#[derive(Debug)]
pub struct GetResultsResult {
    pub reconciliation_id: ReconciliationId,
    pub matched: Vec<MatchedPair>,
    pub missing: Vec<Movement>,
    /// Movements manually marked ignored (RF-10); listed in the
    /// dedicated table below the results.
    pub ignored: Vec<Movement>,
    /// Unmatched FF3 transactions of the run: plain `extra` plus
    /// `duplicates` (distinguished by `is_duplicate`, RF-15).
    pub extra: Vec<Ff3Transaction>,
}

pub struct GetResults<R, M, T, V, A> {
    reconciliations: R,
    matches: M,
    ff3_transactions: T,
    movements: V,
    deleted_audits: A,
}

impl<R, M, T, V, A> GetResults<R, M, T, V, A> {
    pub fn new(
        reconciliations: R,
        matches: M,
        ff3_transactions: T,
        movements: V,
        deleted_audits: A,
    ) -> Self {
        Self {
            reconciliations,
            matches,
            ff3_transactions,
            movements,
            deleted_audits,
        }
    }
}

impl<R, M, T, V, A> GetResults<R, M, T, V, A>
where
    R: ReconciliationRepository,
    M: MatchRepository,
    T: Ff3TransactionRepository,
    V: MovementRepository,
    A: DeletedAuditRepository,
{
    pub async fn execute(&self, batch_id: BatchId) -> Result<GetResultsResult, ApplicationError> {
        let reconciliation = self
            .reconciliations
            .latest_by_batch(batch_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::NotFound(format!("no reconciliation for batch {batch_id}"))
            })?;

        let run_matches = self.matches.by_reconciliation(reconciliation.id).await?;
        let run_txs = self
            .ff3_transactions
            .by_reconciliation(reconciliation.id)
            .await?;
        let movements = self.movements.by_batch(batch_id).await?;

        let matched_tx_ids: HashSet<i64> = run_matches
            .iter()
            .map(|m| m.ff3_transaction_id.as_i64())
            .collect();

        // Matched group: pair each match row with its movement + tx data.
        let mut matched = Vec::with_capacity(run_matches.len());
        for m in &run_matches {
            let movement = movements
                .iter()
                .find(|mv| mv.id == m.movement_id)
                .cloned()
                .ok_or_else(|| {
                    ApplicationError::Database(format!(
                        "match references unknown movement {}",
                        m.movement_id
                    ))
                })?;
            let ff3 = run_txs
                .iter()
                .find(|t| t.id == m.ff3_transaction_id)
                .cloned()
                .ok_or_else(|| {
                    ApplicationError::Database(format!(
                        "match references unknown ff3 transaction {}",
                        m.ff3_transaction_id
                    ))
                })?;
            matched.push(MatchedPair { movement, ff3 });
        }

        // Ignored group: status-only state (RF-10) — never removed from the batch.
        let ignored: Vec<Movement> = movements
            .iter()
            .filter(|m| m.status == MovementStatus::Ignored)
            .cloned()
            .collect();

        // Missing group: current movements still in `missing`.
        let missing: Vec<Movement> = movements
            .into_iter()
            .filter(|m| m.status == MovementStatus::Missing)
            .collect();

        // Extra group: run transactions not paired by any match row.
        // FF3 transactions deleted after the run (RF-15 audit) are
        // removed from the list — the row no longer exists in FF3.
        let deleted_ids = self.deleted_audits.deleted_ff3_ids().await?;
        let extra: Vec<Ff3Transaction> = run_txs
            .into_iter()
            .filter(|t| !matched_tx_ids.contains(&t.id.as_i64()))
            .filter(|t| !deleted_ids.contains(&t.ff3_id))
            .collect();

        Ok(GetResultsResult {
            reconciliation_id: reconciliation.id,
            matched,
            ignored,
            missing,
            extra,
        })
    }
}
