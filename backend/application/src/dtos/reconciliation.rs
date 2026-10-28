//! Creation types for the reconciliation workflow
//! (inputs to `*Repository::insert*`).
//! No `id` or timestamps (assigned by the DB / Clock in infra).

use chrono::NaiveDate;

use ffm_domain::ids::{BatchId, Ff3TransactionId, MovementId, ReconciliationId};
use ffm_domain::value_objects::{AmountCents, MovementStatus};

#[derive(Debug, Clone)]
pub struct NewReconciliation {
    pub batch_id: BatchId,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: i32,
    pub missing_count: i32,
    pub extra_count: i32,
}

#[derive(Debug, Clone)]
pub struct NewFf3Transaction {
    pub reconciliation_id: ReconciliationId,
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub is_duplicate: bool,
}

#[derive(Debug, Clone)]
pub struct NewMatch {
    pub reconciliation_id: ReconciliationId,
    pub movement_id: MovementId,
    pub ff3_transaction_id: Ff3TransactionId,
}

#[derive(Debug, Clone)]
pub struct NewCreation {
    pub movement_id: MovementId,
    pub ff3_id: i64,
}

/// FF3 transaction examined during a run, without the local
/// `reconciliation_id` (assigned by persistence inside the run's
/// transaction).
#[derive(Debug, Clone)]
pub struct RunFf3Transaction {
    pub ff3_id: i64,
    pub ff3_journal_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub is_duplicate: bool,
}

/// Pairing persisted for the run; references the FF3 transaction by
/// its external `ff3_id` (the local row id does not exist yet).
#[derive(Debug, Clone)]
pub struct RunMatch {
    pub movement_id: MovementId,
    pub ff3_id: i64,
    pub ff3_journal_id: i64,
}

/// Whole rapprochement run to persist atomically (R8): reconciliation
/// snapshot + per-run FF3 transaction cache + 1-to-1 matches +
/// movement status updates.
#[derive(Debug, Clone)]
pub struct ReconciliationRun {
    pub reconciliation: NewReconciliation,
    pub ff3_transactions: Vec<RunFf3Transaction>,
    pub matches: Vec<RunMatch>,
    pub movement_status_updates: Vec<(MovementId, MovementStatus)>,
}
