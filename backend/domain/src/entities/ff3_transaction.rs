use chrono::{NaiveDate, NaiveDateTime};

use crate::ids::{Ff3TransactionId, ReconciliationId};
use crate::value_objects::AmountCents;

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
