use chrono::{NaiveDate, NaiveDateTime};

use crate::ids::{BatchId, MovementId};
use crate::value_objects::{AmountCents, MovementStatus};

/// A movement imported from a bank statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    pub id: MovementId,
    pub batch_id: BatchId,
    pub line_no: i32,
    pub date: NaiveDate,
    /// Signed amount in cents (negative = debit, positive = credit);
    /// no `direction` column — the sense is the sign. (TRD D2)
    pub amount_cents: AmountCents,
    pub description: String,
    pub raw_json: String,
    pub status: MovementStatus,
    pub created_at: NaiveDateTime,
}
