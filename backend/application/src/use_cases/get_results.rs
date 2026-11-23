//! GetResults use case (RF-07).
//!
//! Reads the latest reconciliation of the batch and returns the three
//! result groups: matched (pairs), missing (movements), extra
//! (unmatched FF3 transactions, duplicates flagged with `is_duplicate`),
//! plus the counterparty suggestion of each missing movement (RF-08 v0.11).

use std::collections::{HashMap, HashSet};

use ffm_domain::description_key::description_key;
use ffm_domain::entities::{Ff3Transaction, Movement};
use ffm_domain::ids::{BatchId, MovementId, ReconciliationId};
use ffm_domain::transaction_kind::Direction;
use ffm_domain::value_objects::MovementStatus;

use crate::dtos::{CounterpartyRef, SuggestedCounterparty, SuggestionSource};
use crate::error::ApplicationError;
use crate::ports::{
    BatchRepository, CounterpartySuggestionRepository, DeletedAuditRepository,
    Ff3AccountRepository, Ff3TransactionRepository, MatchRepository, MovementRepository,
    ReconciliationRepository,
};

/// Ports needed to resolve counterparty suggestions.
pub struct SuggestionPorts<B, C, S> {
    pub batches: B,
    pub accounts: C,
    pub suggestions: S,
}

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
    /// Counterparty suggestion per missing movement; absent = none.
    pub suggestions: HashMap<MovementId, SuggestedCounterparty>,
}

pub struct GetResults<R, M, T, V, A, B, C, S> {
    reconciliations: R,
    matches: M,
    ff3_transactions: T,
    movements: V,
    deleted_audits: A,
    suggestion_ports: SuggestionPorts<B, C, S>,
}

impl<R, M, T, V, A, B, C, S> GetResults<R, M, T, V, A, B, C, S> {
    pub fn new(
        reconciliations: R,
        matches: M,
        ff3_transactions: T,
        movements: V,
        deleted_audits: A,
        suggestion_ports: SuggestionPorts<B, C, S>,
    ) -> Self {
        Self {
            reconciliations,
            matches,
            ff3_transactions,
            movements,
            deleted_audits,
            suggestion_ports,
        }
    }
}

impl<R, M, T, V, A, B, C, S> GetResults<R, M, T, V, A, B, C, S>
where
    R: ReconciliationRepository,
    M: MatchRepository,
    T: Ff3TransactionRepository,
    V: MovementRepository,
    A: DeletedAuditRepository,
    B: BatchRepository,
    C: Ff3AccountRepository,
    S: CounterpartySuggestionRepository,
{
    /// Learned (batch account + direction + description key) wins over the
    /// history stored at reconcile time. Choices are returned as stored: an id
    /// is never downgraded to a name, so an account Firefly III no longer has
    /// is rejected by apply instead of being recreated as an expense account.
    async fn suggestions(
        &self,
        batch_id: BatchId,
        missing: &[Movement],
    ) -> Result<HashMap<MovementId, SuggestedCounterparty>, ApplicationError> {
        let ports = &self.suggestion_ports;
        let mut out = HashMap::new();
        let Some(account_id) = ports.batches.get(batch_id).await?.account_id else {
            return Ok(out);
        };
        let Some(account) = ports.accounts.by_id(account_id).await? else {
            return Ok(out);
        };
        let learned: HashMap<(Direction, String), CounterpartyRef> = ports
            .suggestions
            .learned(account.ff3_id)
            .await?
            .into_iter()
            .map(|e| ((e.direction, e.desc_key), e.counterparty))
            .collect();
        let history = ports.suggestions.history_by_batch(batch_id).await?;
        for m in missing {
            let key = description_key(&m.description);
            if key.is_empty() {
                continue;
            }
            let direction = Direction::from_cents(m.amount_cents.as_i64());
            let found = match learned.get(&(direction, key)) {
                Some(c) => Some((c, SuggestionSource::Learned)),
                None => history.get(&m.id).map(|c| (c, SuggestionSource::History)),
            };
            if let Some((choice, source)) = found {
                out.insert(
                    m.id,
                    SuggestedCounterparty {
                        counterparty: choice.clone(),
                        source,
                    },
                );
            }
        }
        Ok(out)
    }

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

        let suggestions = self.suggestions(batch_id, &missing).await?;

        Ok(GetResultsResult {
            reconciliation_id: reconciliation.id,
            matched,
            ignored,
            missing,
            extra,
            suggestions,
        })
    }
}
