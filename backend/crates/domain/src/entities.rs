//! Domain entities (POD structs).
//! No serde or diesel derives — those are infrastructure details.

use chrono::{NaiveDate, NaiveDateTime};

use crate::ids::{
    BatchId, CreationId, CsvMappingId, DeletedAuditId, Ff3AccountId, Ff3TransactionId, MatchId,
    MovementId, ReconciliationId,
};
use crate::value_objects::{AmountCents, BatchStatus, HeaderSignature, MovementStatus};

/// Local cache of a Firefly III asset account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff3Account {
    pub id: Ff3AccountId,
    pub ff3_id: i64,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub synced_at: NaiveDateTime,
}

/// Batch of movements imported from a file (immutable, RF-09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub id: BatchId,
    pub filename: String,
    pub file_hash: String,
    pub format: String,
    pub currency: String,
    pub account_id: Option<Ff3AccountId>,
    pub status: BatchStatus,
    pub created_at: NaiveDateTime,
}

/// Movement imported from a bank statement.
/// `amount_cents` is signed (negative = debit); no `direction` column (TRD D2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    pub id: MovementId,
    pub batch_id: BatchId,
    pub line_no: i32,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
    pub status: MovementStatus,
    pub created_at: NaiveDateTime,
}

/// Frozen result of a rapprochement run (audit snapshot).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    pub id: ReconciliationId,
    pub batch_id: BatchId,
    pub tolerance_days: i32,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub matched_count: i32,
    pub missing_count: i32,
    pub extra_count: i32,
    pub ran_at: NaiveDateTime,
}

/// Firefly III transaction examined in a run (per-run cache).
/// `is_duplicate` flags surplus FF3 txs facing the same movement (RF-15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ff3Transaction {
    pub id: Ff3TransactionId,
    pub reconciliation_id: ReconciliationId,
    pub ff3_id: i64,
    pub date: NaiveDate,
    pub amount_cents: AmountCents,
    pub description: String,
    pub is_duplicate: bool,
    pub created_at: NaiveDateTime,
}

/// One-to-one pairing historized per run (the 1-to-1 is guaranteed by the schema).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub id: MatchId,
    pub reconciliation_id: ReconciliationId,
    pub movement_id: MovementId,
    pub ff3_transaction_id: Ff3TransactionId,
    pub matched_at: NaiveDateTime,
}

/// Traceability of a transaction we created in Firefly III.
/// `UNIQUE(movement_id)` → idempotency (NFR-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Creation {
    pub id: CreationId,
    pub movement_id: MovementId,
    pub ff3_id: i64,
    pub created_at: NaiveDateTime,
}

/// CSV mapping memorized by header signature (RF-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvMapping {
    pub id: CsvMappingId,
    pub header_signature: HeaderSignature,
    pub mapping_json: String,
    pub created_at: NaiveDateTime,
}

/// Audit snapshot of a deleted FF3 transaction (point-in-time, denormalized).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedAudit {
    pub id: DeletedAuditId,
    pub ff3_id: i64,
    pub snapshot_json: String,
    pub deleted_at: NaiveDateTime,
}
