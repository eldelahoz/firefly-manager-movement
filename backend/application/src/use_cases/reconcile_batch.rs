//! ReconcileBatch use case (RF-05, RF-06, RF-12, RF-13).
//!
//! Syncs the FF3 account cache, checks the currency (RF-12), fetches
//! the FF3 transactions of the window, runs the `MatchingEngine` and
//! persists the whole run atomically (R8).

use std::collections::HashSet;

use chrono::NaiveDate;

use ffm_domain::entities::Ff3Transaction;
use ffm_domain::ids::{BatchId, Ff3TransactionId, ReconciliationId};
use ffm_domain::matching_engine::run_matching;
use ffm_domain::value_objects::{BatchStatus, MovementStatus};

use crate::dtos::{
    NewFf3Account, NewReconciliation, ReconciliationRun, RunFf3Transaction, RunMatch,
};
use crate::error::ApplicationError;
use crate::ports::{
    BatchRepository, Ff3AccountRepository, FireflyClient, MovementRepository,
    ReconciliationRepository,
};

/// Input for the reconcile use case.
#[derive(Debug)]
pub struct ReconcileBatchInput {
    pub batch_id: BatchId,
    /// Target FF3 asset account (external id).
    pub ff3_account_id: i64,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub tolerance_days: i32,
}

/// Result summary of a rapprochement run.
#[derive(Debug)]
pub struct ReconcileBatchResult {
    pub reconciliation_id: ReconciliationId,
    pub batch_id: BatchId,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: usize,
    pub missing_count: usize,
    /// Unmatched FF3 transactions: `extra` + `duplicates` (RF-07 third group).
    pub extra_count: usize,
    pub duplicate_count: usize,
}

pub struct ReconcileBatch<F, A, B, M, R> {
    firefly: F,
    ff3_accounts: A,
    batches: B,
    movements: M,
    reconciliations: R,
}

impl<F, A, B, M, R> ReconcileBatch<F, A, B, M, R> {
    pub fn new(firefly: F, ff3_accounts: A, batches: B, movements: M, reconciliations: R) -> Self {
        Self {
            firefly,
            ff3_accounts,
            batches,
            movements,
            reconciliations,
        }
    }
}

impl<F, A, B, M, R> ReconcileBatch<F, A, B, M, R>
where
    F: FireflyClient,
    A: Ff3AccountRepository,
    B: BatchRepository,
    M: MovementRepository,
    R: ReconciliationRepository,
{
    pub async fn execute(
        &self,
        input: ReconcileBatchInput,
    ) -> Result<ReconcileBatchResult, ApplicationError> {
        let batch = self.batches.get(input.batch_id).await?;

        if input.window_start > input.window_end {
            return Err(ApplicationError::InvalidState(format!(
                "window_start ({}) must be <= window_end ({})",
                input.window_start, input.window_end
            )));
        }

        // Sync the FF3 account cache (design §4.1).
        let accounts = self.firefly.asset_accounts().await?;
        self.ff3_accounts
            .upsert_batch(
                accounts
                    .iter()
                    .map(|a| NewFf3Account {
                        ff3_id: a.ff3_id,
                        name: a.name.clone(),
                        account_type: a.account_type.clone(),
                        currency: a.currency.clone(),
                    })
                    .collect(),
            )
            .await?;

        let account = self
            .ff3_accounts
            .by_ff3_id(input.ff3_account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::NotFound(format!(
                    "Firefly III account {} not found",
                    input.ff3_account_id
                ))
            })?;

        // Currency check (RF-12) — blocking.
        if !batch
            .currency
            .trim()
            .eq_ignore_ascii_case(account.currency.trim())
        {
            return Err(ApplicationError::InvalidState(format!(
                "currency mismatch: batch is {} but account '{}' is {}",
                batch.currency, account.name, account.currency
            )));
        }

        // Fetch the FF3 window and the batch movements (ignored excluded).
        let ff3_txs = self
            .firefly
            .account_transactions(input.ff3_account_id, input.window_start, input.window_end)
            .await?;

        let movements: Vec<_> = self
            .movements
            .by_batch(input.batch_id)
            .await?
            .into_iter()
            .filter(|m| m.status != MovementStatus::Ignored)
            .collect();

        // Engine input: synthetic entities; the FF3 external id is used
        // as local id so `MatchResult` references it.
        let engine_ff3: Vec<Ff3Transaction> = ff3_txs
            .iter()
            .map(|t| Ff3Transaction {
                id: Ff3TransactionId::new(t.ff3_id),
                reconciliation_id: ReconciliationId::new(0),
                ff3_id: t.ff3_id,
                date: t.date,
                amount_cents: t.amount_cents,
                description: t.description.clone(),
                is_duplicate: false,
                created_at: Default::default(),
            })
            .collect();

        let result = run_matching(&movements, &engine_ff3, input.tolerance_days);

        let duplicates: HashSet<i64> = result.duplicates.iter().map(|id| id.as_i64()).collect();
        let extra_count = result.extra.len() + result.duplicates.len();

        let mut status_updates = Vec::with_capacity(result.matches.len() + result.missing.len());
        for pair in &result.matches {
            status_updates.push((pair.movement_id, MovementStatus::Matched));
        }
        for id in &result.missing {
            status_updates.push((*id, MovementStatus::Missing));
        }

        let run = ReconciliationRun {
            reconciliation: NewReconciliation {
                batch_id: input.batch_id,
                tolerance_days: input.tolerance_days,
                window_start: input.window_start,
                window_end: input.window_end,
                matched_count: result.matches.len() as i32,
                missing_count: result.missing.len() as i32,
                extra_count: extra_count as i32,
            },
            ff3_transactions: ff3_txs
                .iter()
                .map(|t| RunFf3Transaction {
                    ff3_id: t.ff3_id,
                    date: t.date,
                    amount_cents: t.amount_cents,
                    description: t.description.clone(),
                    is_duplicate: duplicates.contains(&t.ff3_id),
                })
                .collect(),
            matches: result
                .matches
                .iter()
                .map(|p| RunMatch {
                    movement_id: p.movement_id,
                    ff3_id: p.ff3_transaction_id.as_i64(),
                })
                .collect(),
            movement_status_updates: status_updates,
        };

        let reconciliation_id = self.reconciliations.save_run(run).await?;

        // Batch metadata (outside the run snapshot).
        self.batches
            .update_account(input.batch_id, account.id)
            .await?;
        self.batches
            .update_status(input.batch_id, BatchStatus::Reconciled)
            .await?;

        Ok(ReconcileBatchResult {
            reconciliation_id,
            batch_id: input.batch_id,
            tolerance_days: input.tolerance_days,
            window_start: input.window_start,
            window_end: input.window_end,
            matched_count: result.matches.len(),
            missing_count: result.missing.len(),
            extra_count,
            duplicate_count: result.duplicates.len(),
        })
    }
}
