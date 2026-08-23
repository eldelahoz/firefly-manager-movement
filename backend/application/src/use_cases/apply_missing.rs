//! ApplyMissing use case (RF-08, RF-11, NFR-05).
//!
//! Creates the selected `missing` movements in Firefly III with an
//! origin tag (filename + batch id), records `creations` for
//! idempotency, and confirms `created` → `matched` through an
//! incremental re-rapprochement (D7): a cumulative run snapshot built
//! from the previous run plus the new pairings, so `GetResults` stays
//! consistent.

use ffm_domain::ids::{BatchId, MovementId, ReconciliationId};
use ffm_domain::value_objects::MovementStatus;

use crate::dtos::{
    NewCreation, NewFf3TransactionCreation, ReconciliationRun, RunFf3Transaction, RunMatch,
};
use crate::error::ApplicationError;
use crate::ports::{
    BatchRepository, CreationRepository, Ff3AccountRepository, Ff3TransactionRepository,
    FireflyClient, MatchRepository, MovementRepository, ReconciliationRepository,
};

/// Input for the apply use case.
#[derive(Debug)]
pub struct ApplyMissingInput {
    pub batch_id: BatchId,
    pub movement_ids: Vec<MovementId>,
    /// Counterparty account (external FF3 id): expense/revenue side.
    pub counterparty_ff3_id: i64,
    pub category: Option<String>,
    pub budget: Option<String>,
    pub notes: Option<String>,
}

/// Per-movement outcome (TRD §4: batch continues on FF3 errors, final
/// report per movement succès/échec/raison).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    Created,
    AlreadyCreated,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ApplyMovementReport {
    pub movement_id: MovementId,
    pub outcome: ApplyOutcome,
    /// FF3 id of the created transaction (when created).
    pub ff3_id: Option<i64>,
    /// Failure reason (when failed).
    pub reason: Option<String>,
}

/// Result of an apply run.
#[derive(Debug)]
pub struct ApplyMissingResult {
    pub reports: Vec<ApplyMovementReport>,
    pub created_count: usize,
    pub already_created_count: usize,
    pub failed_count: usize,
    /// Reconciliation of the incremental confirmation run (None when
    /// nothing was created).
    pub reconciliation_id: Option<ReconciliationId>,
}

/// The run-related read/persistence ports, grouped so the constructor
/// stays within argument limits.
pub struct RunPorts<R, MT, TX> {
    pub reconciliations: R,
    pub matches: MT,
    pub ff3_transactions: TX,
}

pub struct ApplyMissing<F, A, B, M, C, R, MT, TX> {
    firefly: F,
    ff3_accounts: A,
    batches: B,
    movements: M,
    creations: C,
    runs: RunPorts<R, MT, TX>,
}

impl<F, A, B, M, C, R, MT, TX> ApplyMissing<F, A, B, M, C, R, MT, TX> {
    pub fn new(
        firefly: F,
        ff3_accounts: A,
        batches: B,
        movements: M,
        creations: C,
        runs: RunPorts<R, MT, TX>,
    ) -> Self {
        Self {
            firefly,
            ff3_accounts,
            batches,
            movements,
            creations,
            runs,
        }
    }
}

impl<F, A, B, M, C, R, MT, TX> ApplyMissing<F, A, B, M, C, R, MT, TX>
where
    F: FireflyClient,
    A: Ff3AccountRepository,
    B: BatchRepository,
    M: MovementRepository,
    C: CreationRepository,
    R: ReconciliationRepository,
    MT: MatchRepository,
    TX: Ff3TransactionRepository,
{
    pub async fn execute(
        &self,
        input: ApplyMissingInput,
    ) -> Result<ApplyMissingResult, ApplicationError> {
        let batch = self.batches.get(input.batch_id).await?;

        let account_id = batch.account_id.ok_or_else(|| {
            ApplicationError::InvalidState(format!(
                "batch {} has no Firefly III account (reconcile first)",
                input.batch_id
            ))
        })?;
        let account =
            self.ff3_accounts.by_id(account_id).await?.ok_or_else(|| {
                ApplicationError::NotFound(format!("account {account_id} not found"))
            })?;

        // Guard: apply is the incremental step after a reconcile (D7).
        let prev = self
            .runs
            .reconciliations
            .latest_by_batch(input.batch_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::InvalidState(format!(
                    "batch {} has no reconciliation (reconcile first)",
                    input.batch_id
                ))
            })?;

        // Validate the selection: every id must belong to the batch, and be
        // either `missing` or already applied (idempotent retry, NFR-05).
        let batch_movements = self.movements.by_batch(input.batch_id).await?;
        let mut selected = Vec::with_capacity(input.movement_ids.len());
        for id in &input.movement_ids {
            let m = batch_movements
                .iter()
                .find(|m| m.id == *id)
                .ok_or_else(|| {
                    ApplicationError::NotFound(format!(
                        "movement {id} not found in batch {}",
                        input.batch_id
                    ))
                })?;
            let applied = self.creations.by_movement(m.id).await?.is_some();
            if !applied && m.status != MovementStatus::Missing {
                return Err(ApplicationError::InvalidState(format!(
                    "movement {} is {} (only missing movements can be applied)",
                    m.id,
                    m.status.as_str()
                )));
            }
            selected.push(m.clone());
        }

        let origin_tag = format!("ffm:{}:{}", batch.filename, input.batch_id.as_i64());

        let mut reports = Vec::with_capacity(selected.len());
        let mut new_txs: Vec<RunFf3Transaction> = Vec::new();
        let mut new_matches: Vec<RunMatch> = Vec::new();
        let mut status_updates = Vec::new();

        for m in selected {
            // Idempotency (NFR-05): skip movements with an existing creation.
            if self.creations.by_movement(m.id).await?.is_some() {
                reports.push(ApplyMovementReport {
                    movement_id: m.id,
                    outcome: ApplyOutcome::AlreadyCreated,
                    ff3_id: None,
                    reason: None,
                });
                continue;
            }

            // Signed amount → FF3 type + direction: debit (negative)
            // leaves the asset account; credit (positive) enters it.
            let (transaction_type, source, destination) = if m.amount_cents.as_i64() < 0 {
                ("withdrawal", account.ff3_id, input.counterparty_ff3_id)
            } else {
                ("deposit", input.counterparty_ff3_id, account.ff3_id)
            };

            let creation = NewFf3TransactionCreation {
                transaction_type: transaction_type.to_string(),
                amount: m.amount_cents,
                date: m.date,
                description: m.description.clone(),
                source_ff3_id: source,
                destination_ff3_id: destination,
                category: input.category.clone(),
                budget: input.budget.clone(),
                notes: input.notes.clone(),
                tags: vec![origin_tag.clone()],
            };

            match self.firefly.create_transaction(creation).await {
                Ok(ff3_id) => {
                    self.creations
                        .insert(NewCreation {
                            movement_id: m.id,
                            ff3_id,
                        })
                        .await?;
                    self.movements
                        .update_status(m.id, MovementStatus::Created)
                        .await?;

                    new_txs.push(RunFf3Transaction {
                        ff3_id,
                        date: m.date,
                        amount_cents: m.amount_cents,
                        description: m.description.clone(),
                        is_duplicate: false,
                    });
                    new_matches.push(RunMatch {
                        movement_id: m.id,
                        ff3_id,
                    });
                    status_updates.push((m.id, MovementStatus::Matched));

                    reports.push(ApplyMovementReport {
                        movement_id: m.id,
                        outcome: ApplyOutcome::Created,
                        ff3_id: Some(ff3_id),
                        reason: None,
                    });
                }
                Err(e) => {
                    // Per-movement treatment: the batch continues (TRD §4).
                    reports.push(ApplyMovementReport {
                        movement_id: m.id,
                        outcome: ApplyOutcome::Failed,
                        ff3_id: None,
                        reason: Some(e.to_string()),
                    });
                }
            }
        }

        let created_count = reports
            .iter()
            .filter(|r| r.outcome == ApplyOutcome::Created)
            .count();
        let already_created_count = reports
            .iter()
            .filter(|r| r.outcome == ApplyOutcome::AlreadyCreated)
            .count();
        let failed_count = reports
            .iter()
            .filter(|r| r.outcome == ApplyOutcome::Failed)
            .count();

        // Incremental confirmation (D7): cumulative run = previous run
        // data + the new pairings; created movements become matched.
        let reconciliation_id = if created_count > 0 {
            let prev_matches = self.runs.matches.by_reconciliation(prev.id).await?;
            let prev_txs = self
                .runs
                .ff3_transactions
                .by_reconciliation(prev.id)
                .await?;

            let run = ReconciliationRun {
                reconciliation: crate::dtos::NewReconciliation {
                    batch_id: input.batch_id,
                    tolerance_days: prev.tolerance_days,
                    window_start: prev.window_start,
                    window_end: prev.window_end,
                    matched_count: prev.matched_count + created_count as i32,
                    missing_count: (prev.missing_count - created_count as i32).max(0),
                    extra_count: prev.extra_count,
                },
                ff3_transactions: prev_txs
                    .iter()
                    .map(|t| RunFf3Transaction {
                        ff3_id: t.ff3_id,
                        date: t.date,
                        amount_cents: t.amount_cents,
                        description: t.description.clone(),
                        is_duplicate: t.is_duplicate,
                    })
                    .chain(new_txs)
                    .collect(),
                matches: {
                    let mut all: Vec<RunMatch> = prev_matches
                        .iter()
                        .map(|m| RunMatch {
                            movement_id: m.movement_id,
                            ff3_id: 0, // resolved below
                        })
                        .collect();
                    for (i, m) in prev_matches.iter().enumerate() {
                        let tx = prev_txs.iter().find(|t| t.id == m.ff3_transaction_id);
                        match tx {
                            Some(t) => all[i].ff3_id = t.ff3_id,
                            None => {
                                return Err(ApplicationError::Database(format!(
                                    "match references unknown ff3 transaction {}",
                                    m.ff3_transaction_id
                                )))
                            }
                        }
                    }
                    all.extend(new_matches);
                    all
                },
                movement_status_updates: status_updates,
            };
            Some(self.runs.reconciliations.save_run(run).await?)
        } else {
            None
        };

        Ok(ApplyMissingResult {
            reports,
            created_count,
            already_created_count,
            failed_count,
            reconciliation_id,
        })
    }
}
