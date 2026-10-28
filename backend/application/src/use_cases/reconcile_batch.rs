//! ReconcileBatch use case (RF-05, RF-06, RF-12, RF-13).
//!
//! Syncs the FF3 account cache, checks the currency (RF-12), fetches
//! the FF3 transactions of the window, runs the `MatchingEngine` and
//! persists the whole run atomically (R8).

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;

use ffm_domain::entities::Ff3Transaction;
use ffm_domain::ids::{BatchId, Ff3TransactionId, MovementId, ReconciliationId};
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
                        iban: a.iban.clone(),
                        account_number: a.account_number.clone(),
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

        // Currency check (RF-12) - blocking.
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
            .account_transactions(
                input.ff3_account_id,
                Some(input.window_start),
                Some(input.window_end),
            )
            .await?;

        let movements: Vec<_> = self
            .movements
            .by_batch(input.batch_id)
            .await?
            .into_iter()
            .filter(|m| m.status != MovementStatus::Ignored)
            .collect();

        // Engine input: synthetic entities; the FF3 external id (group, or
        // journal in pass 2) is used as local id so `MatchResult` references it.
        let engine_row = |id: i64, date, amount_cents, description: &str| Ff3Transaction {
            id: Ff3TransactionId::new(id),
            reconciliation_id: ReconciliationId::new(0),
            ff3_id: id,
            ff3_journal_id: 0,
            date,
            amount_cents,
            description: description.to_string(),
            is_duplicate: false,
            created_at: Default::default(),
        };

        // Pass 1: each FF3 transaction as a whole (split groups by their total).
        let engine_ff3: Vec<Ff3Transaction> = ff3_txs
            .iter()
            .map(|t| engine_row(t.ff3_id, t.date, t.amount_cents, &t.description))
            .collect();
        let result = run_matching(&movements, &engine_ff3, input.tolerance_days);

        // Pass 2: the statement may list each split of an unmatched group as
        // its own line; try the splits against the movements still missing.
        let unmatched: HashSet<i64> = result.extra.iter().map(|id| id.as_i64()).collect();
        let journal_group: HashMap<i64, i64> = ff3_txs
            .iter()
            .filter(|t| unmatched.contains(&t.ff3_id))
            .flat_map(|t| t.splits.iter().map(move |s| (s.journal_id, t.ff3_id)))
            .collect();
        let split_rows: Vec<Ff3Transaction> = ff3_txs
            .iter()
            .filter(|t| unmatched.contains(&t.ff3_id))
            .flat_map(|t| &t.splits)
            .map(|s| engine_row(s.journal_id, s.date, s.amount_cents, &s.description))
            .collect();
        let still_missing: Vec<_> = movements
            .iter()
            .filter(|m| result.missing.contains(&m.id))
            .cloned()
            .collect();
        let by_split = run_matching(&still_missing, &split_rows, input.tolerance_days);

        // A group with at least one matched split is cached split by split.
        let split_groups: HashSet<i64> = by_split
            .matches
            .iter()
            .map(|p| journal_group[&p.ff3_transaction_id.as_i64()])
            .collect();
        let split_matched: HashSet<MovementId> =
            by_split.matches.iter().map(|p| p.movement_id).collect();
        let duplicates: HashSet<i64> = result.duplicates.iter().map(|id| id.as_i64()).collect();
        let split_duplicates: HashSet<i64> =
            by_split.duplicates.iter().map(|id| id.as_i64()).collect();

        let mut ff3_rows = Vec::with_capacity(ff3_txs.len());
        for t in &ff3_txs {
            if split_groups.contains(&t.ff3_id) {
                ff3_rows.extend(t.splits.iter().map(|s| RunFf3Transaction {
                    ff3_id: t.ff3_id,
                    ff3_journal_id: s.journal_id,
                    date: s.date,
                    amount_cents: s.amount_cents,
                    description: s.description.clone(),
                    is_duplicate: split_duplicates.contains(&s.journal_id),
                }));
            } else {
                ff3_rows.push(RunFf3Transaction {
                    ff3_id: t.ff3_id,
                    ff3_journal_id: 0,
                    date: t.date,
                    amount_cents: t.amount_cents,
                    description: t.description.clone(),
                    is_duplicate: duplicates.contains(&t.ff3_id),
                });
            }
        }

        let mut matches: Vec<RunMatch> = result
            .matches
            .iter()
            .map(|p| RunMatch {
                movement_id: p.movement_id,
                ff3_id: p.ff3_transaction_id.as_i64(),
                ff3_journal_id: 0,
            })
            .collect();
        matches.extend(by_split.matches.iter().map(|p| {
            let journal = p.ff3_transaction_id.as_i64();
            RunMatch {
                movement_id: p.movement_id,
                ff3_id: journal_group[&journal],
                ff3_journal_id: journal,
            }
        }));
        let missing: Vec<MovementId> = result
            .missing
            .iter()
            .filter(|id| !split_matched.contains(id))
            .copied()
            .collect();
        let extra_count = ff3_rows.len() - matches.len();
        let duplicate_count = ff3_rows.iter().filter(|t| t.is_duplicate).count();
        let (matched_count, missing_count) = (matches.len(), missing.len());

        let mut status_updates = Vec::with_capacity(matched_count + missing_count);
        for m in &matches {
            status_updates.push((m.movement_id, MovementStatus::Matched));
        }
        for id in &missing {
            status_updates.push((*id, MovementStatus::Missing));
        }

        let run = ReconciliationRun {
            reconciliation: NewReconciliation {
                batch_id: input.batch_id,
                tolerance_days: input.tolerance_days,
                window_start: input.window_start,
                window_end: input.window_end,
                matched_count: matched_count as i32,
                missing_count: missing_count as i32,
                extra_count: extra_count as i32,
            },
            ff3_transactions: ff3_rows,
            matches,
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
            matched_count,
            missing_count,
            extra_count,
            duplicate_count,
        })
    }
}
